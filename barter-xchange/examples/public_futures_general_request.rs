use barter_xchange::exchange::binance::{
    api::Binance, futures::general::FuturesGeneral,
};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    general_data().await;
}

async fn general_data() {
    let general: FuturesGeneral = Binance::new(None, None);

    match general.exchange_info().await {
        Ok(exchange_info) =>  println!(
            "exchange_info: {0:?}", exchange_info.timezone
        ),
        Err(e) => println!("Error: {}", e),
    }
}
