use barter_xchange::exchange::binance::{
    api::Binance, futures::general::FuturesGeneral, model::Symbol,
};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    general_data().await;
}

async fn general_data() {
    let general: FuturesGeneral = Binance::new(None, None);

    match general.exchange_info().await {
        Ok(exchange_info) => println!("exchange_info: {0:?}", exchange_info.timezone),
        Err(e) => println!("Error: {}", e),
    }

    match general.get_symbol_info("btcusdt").await {
        Ok(symbol) => println!("Symbol: {symbol:?}"),
        Err(e) => println!("Error: {}", e),
    }

    match general.get_symbol_infos().await {
        Ok(symbols) => println!("Symbols: {symbols:?};Count : {0}", symbols.len()),
        Err(e) => println!("Error: {}", e),
    }
}
