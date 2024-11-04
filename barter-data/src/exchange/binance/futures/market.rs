/*!
## Implemented functionality
- [x] `Order Book`
- [x] `Recent Trades List`
- [ ] `Old Trades Lookup (MARKET_DATA)`
- [x] `Compressed/Aggregate Trades List`
- [x] `Kline/Candlestick Data`
- [x] `Mark Price`
- [ ] `Get Funding Rate History (MARKET_DATA)`
- [x] `24hr Ticker Price Change Statistics`
- [x] `Symbol Price Ticker`
- [x] `Symbol Order Book Ticker`
- [x] `Get all Liquidation Orders`
- [x] `Open Interest`
- [ ] `Notional and Leverage Brackets (MARKET_DATA)`
- [ ] `Open Interest Statistics (MARKET_DATA)`
- [ ] `Top Trader Long/Short Ratio (Accounts) (MARKET_DATA)`
- [ ] `Top Trader Long/Short Ratio (Positions) (MARKET_DATA)`
- [ ] `Long/Short Ratio (MARKET_DATA)`
- [ ] `Taker Buy/Sell Volume (MARKET_DATA)`
*/

use crate::exchange::binance::api::{BinanceParser, FetchCandlesRequest, RequestUnsinger};
use crate::exchange::binance::config::{self, Config};
use crate::exchange::binance::model::KlineSummaries;
use crate::exchange::binance::util::build_request;
use crate::subscription::candle;
use std::collections::BTreeMap;
use barter_integration::protocol::http::rest::client::RestClient;
use barter_integration::protocol::http::rest::RestRequest;
use crate::exchange::errors::{ErrorKind, Result};
// TODO
// Make enums for Strings
// Add limit parameters to functions
// Implement all functions

#[derive(Clone)]
pub struct FuturesMarket {
    // pub client: Client,
    pub recv_window: u64,
}

impl FuturesMarket {
  
    // Returns up to 'limit' klines for given symbol and interval ("1m", "5m", ...)
    // https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data
    pub async fn  get_klines<S1, S2, S3, S4, S5>(
        &self, symbol: S1, interval: S2, limit: S3, start_time: S4, end_time: S5,
    ) -> Result<KlineSummaries>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("interval".into(), interval.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{}", st));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{}", et));
        }
       // 参数处理 
       let config = Config::default();
        let request = build_request(parameters);
        let fetch_candles_request = FetchCandlesRequest{
            query_params: request
        };
        let rug =RequestUnsinger {};
        // // Build RestClient with Ftx configuration
        let rest_client = RestClient::new(config.futures_rest_api_endpoint, rug, BinanceParser);
        let response = rest_client.execute(fetch_candles_request).await;
        // Fetch Result<FetchBalancesResponse, ExecutionError>
        match response {
            Ok((data, metric)) => {
                println!("Success: {:?}", data);
                println!("Metric: {:?}", metric);
                // let klines: KlineSummaries = KlineSummaries::AllKlineSummaries(
                //     data.result.iter()
                //         .map(|row| row.try_into())
                //         .collect::<Result<Vec<KlineSummary>>>()?,
                // );
            Ok(KlineSummaries::AllKlineSummaries(data.result))
            }
            Err(e) => Err(ErrorKind::MarketError(e).into()),
            // Err(e) => Err(),
        }
  }
}
