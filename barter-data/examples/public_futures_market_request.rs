fn main() {
    market_data();
}

fn market_data() {
    let market: FuturesMarket = Binance::new(None, None);

    match market.get_klines("btcusdt", "1m", 10, None, None) {
        Ok(KlineSummaries::AllKlineSummaries(answer)) => println!("First kline: {:?}", answer[0]),
        Err(e) => println!("Error: {}", e),
    }

}
