use std::{borrow::Cow, collections::BTreeMap};

use barter_integration::{
    error::SocketError,
    protocol::http::{rest::RestRequest, BuildStrategy, HttpParser},
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::error;
use crate::exchange::errors::ExecutionError;

use serde_json::Value;

use super::{config::Config, futures::market::FuturesMarket, model::{KlineSummaries, KlineSummary}};

#[allow(clippy::all)]
pub enum API {
    Futures(Futures),
}

pub enum Futures {
    Ping,
    Time,
    ExchangeInfo,
    Depth,
    Trades,
    HistoricalTrades,
    AggTrades,
    Klines,
    ContinuousKlines,
    IndexPriceKlines,
    MarkPriceKlines,
    PremiumIndex,
    FundingRate,
    Ticker24hr,
    TickerPrice,
    BookTicker,
    AllForceOrders,
    AllOpenOrders,
    AllOrders,
    UserTrades,
    Order,
    PositionRisk,
    Balance,
    PositionSide,
    OpenInterest,
    OpenInterestHist,
    TopLongShortAccountRatio,
    TopLongShortPositionRatio,
    GlobalLongShortAccountRatio,
    TakerlongshortRatio,
    LvtKlines,
    IndexInfo,
    ChangeInitialLeverage,
    MarginType,
    PositionMargin,
    Account,
    OpenOrders,
    UserDataStream,
    Income,
}

impl From<API> for String {
    fn from(item: API) -> Self {
        String::from(match item {
            API::Futures(route) => match route {
                Futures::Ping => "/fapi/v1/ping",
                Futures::Time => "/fapi/v1/time",
                Futures::ExchangeInfo => "/fapi/v1/exchangeInfo",
                Futures::Depth => "/fapi/v1/depth",
                Futures::Trades => "/fapi/v1/trades",
                Futures::HistoricalTrades => "/fapi/v1/historicalTrades",
                Futures::AggTrades => "/fapi/v1/aggTrades",
                Futures::Klines => "/fapi/v1/klines",
                Futures::ContinuousKlines => "/fapi/v1/continuousKlines",
                Futures::IndexPriceKlines => "/fapi/v1/indexPriceKlines",
                Futures::MarkPriceKlines => "/fapi/v1/markPriceKlines",
                Futures::PremiumIndex => "/fapi/v1/premiumIndex",
                Futures::FundingRate => "/fapi/v1/fundingRate",
                Futures::Ticker24hr => "/fapi/v1/ticker/24hr",
                Futures::TickerPrice => "/fapi/v1/ticker/price",
                Futures::BookTicker => "/fapi/v1/ticker/bookTicker",
                Futures::AllForceOrders => "/fapi/v1/allForceOrders",
                Futures::AllOpenOrders => "/fapi/v1/allOpenOrders",
                Futures::AllOrders => "/fapi/v1/allOrders",
                Futures::UserTrades => "/fapi/v1/userTrades",
                Futures::PositionSide => "/fapi/v1/positionSide/dual",
                Futures::Order => "/fapi/v1/order",
                Futures::PositionRisk => "/fapi/v2/positionRisk",
                Futures::Balance => "/fapi/v2/balance",
                Futures::OpenInterest => "/fapi/v1/openInterest",
                Futures::OpenInterestHist => "/futures/data/openInterestHist",
                Futures::TopLongShortAccountRatio => "/futures/data/topLongShortAccountRatio",
                Futures::TopLongShortPositionRatio => "/futures/data/topLongShortPositionRatio",
                Futures::GlobalLongShortAccountRatio => "/futures/data/globalLongShortAccountRatio",
                Futures::TakerlongshortRatio => "/futures/data/takerlongshortRatio",
                Futures::LvtKlines => "/fapi/v1/lvtKlines",
                Futures::IndexInfo => "/fapi/v1/indexInfo",
                Futures::ChangeInitialLeverage => "/fapi/v1/leverage",
                Futures::MarginType => "/fapi/v1/marginType",
                Futures::PositionMargin => "/fapi/v1/positionMargin",
                Futures::Account => "/fapi/v2/account",
                Futures::OpenOrders => "/fapi/v1/openOrders",
                Futures::UserDataStream => "/fapi/v1/listenKey",
                Futures::Income => "/fapi/v1/income",
            },
        })
    }
}

// *****************************************************
//              Binance Futures Data Fetch
// *****************************************************

pub struct BinanceSigner {
    pub api_key: String,
    pub secret: String,
}

// Configuration required to sign every Ftx `RestRequest`
struct BinanceSignConfig<'a> {
    api_key: &'a str,
    time: DateTime<Utc>,
    method: reqwest::Method,
    path: Cow<'static, str>,
}

pub struct BinanceParser;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FetchCandlesResponse
{
    pub result: Vec<Vec<Value>>,
}

impl HttpParser for BinanceParser {
    type ApiError = serde_json::Value;
    type OutputError = ExecutionError;

    fn parse<KlineSummaries>(
            &self,
            status: StatusCode,
            payload: &[u8],
        ) -> Result<KlineSummaries, Self::OutputError>
        where
        KlineSummaries: DeserializeOwned, {

           // Attempt to deserialise reqwest::Response bytes into Ok(Response)
           let data = serde_json::from_slice::<Vec<Vec<Value>>>(payload).expect("数据转转错误!");
        //    println!("data: {:?}", data);
        let parse_ok_error = match serde_json::from_slice::<Vec<Vec<Value>>>(payload) {
            
        //    let parse_ok_error = match serde_json::from_value::<FetchCandlesResponse>(data.into()) {
        // let parse_ok_error = match serde_json::from_slice::<FetchCandlesResponse>(payload) {  
        // let parse_ok_error = match convert(payload) {  
            Ok(response) =>{
                // let fetch_candles_response: FetchCandlesResponse = FetchCandlesResponse{result: data};
                let fetch_candles_response: KlineSummaries = convert_to_structs(response).expect("转换失败"); 
                return Ok(fetch_candles_response);
            },
            Err(serde_error) => serde_error,
        };
        // println!("parse_ok_error: {}", parse_ok_error);
        // println!("payload: {:?}", payload.to_vec());
        // Attempt to deserialise API Error if Ok(Response) deserialisation failed
        let parse_api_error_error = match serde_json::from_slice::<Self::ApiError>(payload) {
            Ok(api_error) => return Err(self.parse_api_error(status, api_error)),
            Err(serde_error) => serde_error,
        };
   
        // Log errors if failed to deserialise reqwest::Response into Response or API Self::Error
        error!(
            status_code = ?status,
            ?parse_ok_error,
            ?parse_api_error_error,
            response_body = %String::from_utf8_lossy(payload),
            "error deserializing HTTP response"
        );

        Err(Self::OutputError::from(SocketError::DeserialiseBinary {
            error: parse_ok_error,
            payload: payload.to_vec(),
        }))
    }

    fn parse_api_error(&self, status: StatusCode, api_error: Self::ApiError) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        let error = api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            }
            _ => ExecutionError::Socket(SocketError::HttpResponse(status, error)),
        }
    }
}

// Function to convert Vec<Vec<Value>> to Vec<MyStruct>
fn convert_to_structs(data: Vec<Vec<Value>>) -> crate::exchange::errors::Result<KlineSummaries> {
  
    let klines: KlineSummaries = KlineSummaries::AllKlineSummaries(
        data.iter()
            .map(|row| row.try_into())
            .collect::<crate::exchange::errors::Result<Vec<KlineSummary>>>()?,
    );
    Ok(klines)
}

pub struct RequestUnsinger {}
impl BuildStrategy for RequestUnsinger {
    fn build<Request>(
        &self,
        request: Request,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        // Add Ftx required Headers & build reqwest::Request
        builder.build().map_err(SocketError::from)
    }
}

// 市场数据对应模型定义
pub struct FetchCandlesRequest {
    pub(crate) query_params: BTreeMap<String, String>,
}

impl RestRequest for FetchCandlesRequest {
    type Response = KlineSummaries; // Define Response type
    type QueryParams = BTreeMap<String, String>; // FetchBalances does not require any QueryParams
    type Body = (); // FetchBalances does not require any Body

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/fapi/v1/klines")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }

    fn query_params(&self) -> Option<&Self::QueryParams> {
        Some(&self.query_params)
    }
}



pub trait Binance {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
    fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> Self;
}

impl Binance for FuturesMarket {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesMarket {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> FuturesMarket {
        FuturesMarket {
            // client: Client::new(
            //     api_key,
            //     secret_key,
            //     config.futures_rest_api_endpoint.clone(),
            // ),
            recv_window: config.recv_window,
        }
    }
}
