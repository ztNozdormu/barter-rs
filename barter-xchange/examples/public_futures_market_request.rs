use barter_xchange::exchange::binance::{
    api::Binance, futures::market::FuturesMarket, model::KlineSummaries,
};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    market_data().await;
}

async fn market_data() {
    let market: FuturesMarket = Binance::new(None, None);

    match market.get_klines("btcusdt", "5m", None, None, None).await {
        Ok(KlineSummaries::AllKlineSummaries(answer)) => println!(
            "First kline: {:?} kline count : {}",
            answer[0],
            answer.len()
        ),
        Err(e) => println!("Error: {}", e),
    }
}
