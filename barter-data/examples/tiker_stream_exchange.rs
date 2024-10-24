use barter_data::{
    exchange::
        binance::futures::BinanceFuturesUsd
    ,
    streams::{reconnect::stream::ReconnectingStream, Streams},
    subscription::
        tiker::Tikers
    ,
};
use barter_instrument::instrument::kind::InstrumentKind;
use futures::StreamExt;
use tracing::{info, warn};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Notes:
    // - MarketEvent<DataKind> could use a custom enumeration if more flexibility is required.
    // - Each call to StreamBuilder::subscribe() creates a separate WebSocket connection for those
    //   Subscriptions passed.

     // Initialise PublicTrades Streams for various exchanges
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let streams = Streams::<Tikers>::builder()
        .subscribe([
            (BinanceFuturesUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, Tikers),
            (BinanceFuturesUsd::default(), "eth", "usdt", InstrumentKind::Perpetual, Tikers),
        ])
        .init()
        .await
        .unwrap();

    // Join all exchange Streams into a single tokio_stream::StreamMap
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    // let mut joined_stream = streams.join_map().await;

    // while let Some((exchange, data)) = joined_stream.next().await {
    //     info!("Exchange: {exchange}, MarketEvent<DataKind>: {data:?}");
    // }

    // Select and merge every exchange Stream using futures_util::stream::select_all
    // Note: use `Streams.select(ExchangeId)` to interact with individual exchange streams!
    let mut joined_stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    while let Some(event) = joined_stream.next().await {
        info!("{event:?}");
    }
}

// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}
