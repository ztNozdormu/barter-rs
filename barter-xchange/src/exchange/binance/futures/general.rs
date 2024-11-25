use barter_integration::protocol::http::rest::client::RestClient;

use crate::{config::Config, exchange::{binance::{api::{BinanceParser, Futures, RequestUnsinger, API}, model::{ExchangeInformation, FetchCommonRequest}}, errors::{ErrorKind, Result}}};

#[derive(Clone)]
pub struct FuturesGeneral {
}

impl FuturesGeneral {
    // Test connectivity
    // pub fn ping(&self) -> Result<String> {
    //     self.client
    //         .get::<Empty>(API::Futures(Futures::Ping), None)?;
    //     Ok("pong".into())
    // }

    // // Check server time
    // pub fn get_server_time(&self) -> Result<ServerTime> {
    //     self.client.get(API::Futures(Futures::Time), None)
    // }

    // Obtain exchange information
    // - Current exchange trading rules and symbol information
    pub fn exchange_info(&self) -> Result<ExchangeInformation> {
         // 参数处理
        let config = Config::default();
         
        let rug = RequestUnsinger {};
      
        let url: String = API::Futures(Futures::ExchangeInfo).into();
        let url = Box::leak(url.into_boxed_str());
        
        let fetch_candles_request = FetchCommonRequest(url);
        // // Build RestClient with Ftx configuration
        let rest_client = RestClient::new(config.futures_rest_api_endpoint, rug, BinanceParser);
     
     
        todo!()
    }

    // Get Symbol information
    // pub fn get_symbol_info<S>(&self, symbol: S) -> Result<Symbol>
    // where
    //     S: Into<String>,
    // {
    //     let upper_symbol = symbol.into().to_uppercase();
    //     match self.exchange_info() {
    //         Ok(info) => {
    //             for item in info.symbols {
    //                 if item.symbol == upper_symbol {
    //                     return Ok(item);
    //                 }
    //             }
    //             bail!("Symbol not found")
    //         }
    //         Err(e) => Err(e),
    //     }
    // }

    // pub fn get_symbol_infos<S>() -> Result<KlineSummaries>{
    //     let rug = RequestUnsinger {};
    //     // // Build RestClient with Ftx configuration
    //     let rest_client = RestClient::new(config.futures_rest_api_endpoint, rug, BinanceParser);
    //     let response: std::result::Result<
    //         (FetchCandlesResponse, barter_integration::metric::Metric),
    //         crate::exchange::errors::ExecutionError,
    //     > = rest_client.execute(fetch_candles_request).await;
    //     match response {
    //         Ok((data, metric)) => {
    //             // println!("Success: {:?}", data.0);
    //             // println!("Metric: {:?}", metric);
    //             let klines: KlineSummaries = KlineSummaries::AllKlineSummaries(
    //                 data.0
    //                     .iter()
    //                     .map(|row| row.try_into())
    //                     .collect::<Result<Vec<KlineSummary>>>()?,
    //             );
    //             Ok(klines)
    //         }
    //         Err(e) => {
    //             println!("Errorm: {}", e);
    //             Err(ErrorKind::MarketError(e).into())
    //         }
    // }
}
