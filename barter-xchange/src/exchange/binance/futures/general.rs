use error_chain::bail;


use crate::exchange::{binance::{api::{Futures, API}, model::ExchangeInformation}, errors::{ErrorKind, Result}};
use barter_integration::{metric, protocol::http::rest::client::RestClient};
use std::collections::BTreeMap;

use crate::{config::Config, exchange::binance::{api::{BinanceParser, RequestUnsinger}, model::{FetchCandlesRequest, FetchCandlesResponse, KlineSummaries, KlineSummary}}};


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
        self.client.get(API::Futures(Futures::ExchangeInfo), None)
    }

    // Get Symbol information
    pub fn get_symbol_info<S>(&self, symbol: S) -> Result<Symbol>
    where
        S: Into<String>,
    {
        let upper_symbol = symbol.into().to_uppercase();
        match self.exchange_info() {
            Ok(info) => {
                for item in info.symbols {
                    if item.symbol == upper_symbol {
                        return Ok(item);
                    }
                }
                bail!("Symbol not found")
            }
            Err(e) => Err(e),
        }
    }

    // Get All Symbol information
    pub fn get_symbols(&self) -> Result<Vec<Symbol>>
    {
        match self.exchange_info() {
            Ok(info) => Ok(info.symbols),
            Err(e) => Err(e),
        }
    }
}
