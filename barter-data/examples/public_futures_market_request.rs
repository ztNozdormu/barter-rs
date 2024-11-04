use barter_data::exchange::binance::{
    api::Binance, futures::market::FuturesMarket, model::KlineSummaries,
};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    market_data().await;
}

async fn market_data() {
    let market: FuturesMarket = Binance::new(None, None);

    match market.get_klines("btcusdt", "1m", 10, None, None).await {
        Ok(KlineSummaries::AllKlineSummaries(answer)) => println!("First kline: {:?}", answer[0]),
        Err(e) => println!("Error: {}", e),
    }
}
