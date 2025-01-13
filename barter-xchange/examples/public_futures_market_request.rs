use barter_xchange::exchange::binance::{
    api::Binance, futures::market::FuturesMarket, model::KlineSummaries,
};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    klines().await;
    last_kline().await;
}

async fn klines() {
    let market: FuturesMarket = Binance::new(None, None);

    match market.klines("btcusdt", "5m", None, None, None).await {
        Ok(KlineSummaries::AllKlineSummaries(answer)) => println!(
            "First kline: {:?} kline count : {}",
            answer[0],
            answer.len()
        ),
        Err(e) => println!("Error: {}", e),
    }
}

async fn last_kline() {
    let market: FuturesMarket = Binance::new(None, None);

    match market.last_kline("btcusdt", "5m").await {
        Ok(last_kline) => println!("last kline: {:?}", last_kline),
        Err(e) => println!("Error: {}", e),
    }
}
