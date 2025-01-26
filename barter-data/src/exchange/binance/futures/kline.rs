use barter_integration::subscription::SubscriptionId;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{binance::channel::BinanceChannel, ExchangeId, ExchangeSub},
    Identifier,
};
use crate::exchange::binance::book::l1::BinanceOrderBookL1;
use crate::subscription::ticker::Ticker;

/// Binance real-time candle message.
///
/// Note:
/// For [`BinanceFuturesUsd`](super::futures::BinanceFuturesUsd) this real-time stream is
/// undocumented.
///
/// See discord: <https://discord.com/channels/910237311332151317/923160222711812126/975712874582388757>
///
/// ### Raw Payload Examples
/// See docs: <https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#klinecandlestick-streams-for-utc>
/// ```json
///{
///   "e": "kline",         // Event type
///   "E": 1672515782136,   // Event time
///   "s": "BNBBTC",        // Symbol
///   "k": {
///     "t": 1672515780000, // Kline start time
///     "T": 1672515839999, // Kline close time
///     "s": "BNBBTC",      // Symbol
///     "i": "1m",          // Interval
///     "f": 100,           // First trade ID
///     "L": 200,           // Last trade ID
///     "o": "0.0010",      // Open price
///     "c": "0.0020",      // Close price
///     "h": "0.0025",      // High price
///     "l": "0.0015",      // Low price
///     "v": "1000",        // Base asset volume
///     "n": 100,           // Number of trades
///     "x": false,         // Is this kline closed?
///     "q": "1.0000",      // Quote asset volume
///     "V": "500",         // Taker buy base asset volume
///     "Q": "0.500",       // Taker buy quote asset volume
///     "B": "123456"       // Ignore
///   }
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceKline {
    #[serde(alias = "k")]
    pub kline: BinanceKlineData,
}
/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) Liquidation order.
///
/// ### Raw Payload Examples
/// ```json
/// {
///     "t": 1672515780000, // Kline start time
///     "T": 1672515839999, // Kline close time
///     "s": "BNBBTC",      // Symbol
///     "i": "1m",          // Interval
///     "f": 100,           // First trade ID
///     "L": 200,           // Last trade ID
///     "o": "0.0010",      // Open price
///     "c": "0.0020",      // Close price
///     "h": "0.0025",      // High price
///     "l": "0.0015",      // Low price
///     "v": "1000",        // Base asset volume
///     "n": 100,           // Number of trades
///     "x": false,         // Is this kline closed?
///     "q": "1.0000",      // Quote asset volume
///     "V": "500",         // Taker buy base asset volume
///     "Q": "0.500",       // Taker buy quote asset volume
///     "B": "123456"       // Ignore
///   }
/// ```
///
/// See docs: <https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#klinecandlestick-streams-for-utc>
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceKlineData {
    #[serde(alias = "s", deserialize_with = "de_kline_subscription_id")]
    pub subscription_id: SubscriptionId,

    #[serde(alias = "o", with = "rust_decimal::serde::str")]
    pub open: Decimal,
    #[serde(alias = "c", with = "rust_decimal::serde::str")]
    pub close: Decimal,
    #[serde(alias = "h", with = "rust_decimal::serde::str")]
    pub high: Decimal,
    #[serde(alias = "l", with = "rust_decimal::serde::str")]
    pub low: Decimal,

    #[serde(alias = "v", with = "rust_decimal::serde::str")]
    pub base_asset_volume: Decimal,
    #[serde(alias = "n", with = "rust_decimal::serde::str")]
    pub trade_nums: Decimal,
    #[serde(alias = "x")]
    pub is_kline_closed: bool,
    #[serde(alias = "q", with = "rust_decimal::serde::str")]
    pub quote_asset_volume: Decimal,
    #[serde(alias = "V", with = "rust_decimal::serde::str")]
    pub buy_base_asset_volume: Decimal,
    #[serde(alias = "Q", with = "rust_decimal::serde::str")]
    pub buy_quote_asset_volume: Decimal,

    #[serde(
        alias = "t",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub start_time: DateTime<Utc>,

    #[serde(
        alias = "T",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub close_time: DateTime<Utc>,
}

/// Deserialize a [`BinanceKliner`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`].
///
/// eg/ "@kline|BTCUSDT"
pub fn de_kline_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|market| ExchangeSub::from((BinanceChannel::KLINES, market)).id())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     mod de {
//         use std::time::Duration;

//         use barter_integration::{de::datetime_utc_from_epoch_duration, error::SocketError};
//         use serde::de::Error;

//         use super::*;

//         #[test]
//         fn test_binance_trade() {
//             struct TestCase {
//                 input: &'static str,
//                 expected: Result<BinanceTicker, SocketError>,
//             }

//             let tests = vec![
//                 TestCase {
//                     // TC0: Spot trade valid
//                     input: r#"
//                     {
//                         "e":"trade","E":1649324825173,"s":"ETHUSDT","t":1000000000,
//                         "p":"10000.19","q":"0.239000","b":10108767791,"a":10108764858,
//                         "T":1749354825200,"m":false,"M":true
//                     }
//                     "#,
//                     expected: Ok(BinanceTicker {
//                         subscription_id: SubscriptionId::from("@trade|ETHUSDT"),
//                         time: datetime_utc_from_epoch_duration(Duration::from_millis(
//                             1749354825200,
//                         )),
//                         id: 1000000000,
//                         price: 10000.19,
//                         amount: 0.239000,
//                         side: Side::Buy,
//                     }),
//                 },
//                 TestCase {
//                     // TC1: Spot trade malformed w/ "yes" is_buyer_maker field
//                     input: r#"{
//                         "e":"trade","E":1649324825173,"s":"ETHUSDT","t":1000000000,
//                         "p":"10000.19000000","q":"0.239000","b":10108767791,"a":10108764858,
//                         "T":1649324825173,"m":"yes","M":true
//                     }"#,
//                     expected: Err(SocketError::Deserialise {
//                         error: serde_json::Error::custom(""),
//                         payload: "".to_owned(),
//                     }),
//                 },
//                 TestCase {
//                     // TC2: FuturePerpetual trade w/ type MARKET
//                     input: r#"
//                     {
//                         "e": "trade","E": 1649839266194,"T": 1749354825200,"s": "ETHUSDT",
//                         "t": 1000000000,"p":"10000.19","q":"0.239000","X": "MARKET","m": true
//                     }
//                     "#,
//                     expected: Ok(BinanceTicker {
//                         subscription_id: SubscriptionId::from("@trade|ETHUSDT"),
//                         time: datetime_utc_from_epoch_duration(Duration::from_millis(
//                             1749354825200,
//                         )),
//                         id: 1000000000,
//                         price: 10000.19,
//                         amount: 0.239000,
//                         side: Side::Sell,
//                     }),
//                 },
//                 TestCase {
//                     // TC3: FuturePerpetual trade w/ type LIQUIDATION
//                     input: r#"
//                     {
//                         "e": "trade","E": 1649839266194,"T": 1749354825200,"s": "ETHUSDT",
//                         "t": 1000000000,"p":"10000.19","q":"0.239000","X": "LIQUIDATION","m": false
//                     }
//                     "#,
//                     expected: Ok(BinanceTicker {
//                         subscription_id: SubscriptionId::from("@trade|ETHUSDT"),
//                         time: datetime_utc_from_epoch_duration(Duration::from_millis(
//                             1749354825200,
//                         )),
//                         id: 1000000000,
//                         price: 10000.19,
//                         amount: 0.239000,
//                         side: Side::Buy,
//                     }),
//                 },
//                 TestCase {
//                     // TC4: FuturePerpetual trade w/ type LIQUIDATION
//                     input: r#"{
//                         "e": "trade","E": 1649839266194,"T": 1749354825200,"s": "ETHUSDT",
//                         "t": 1000000000,"p":"10000.19","q":"0.239000","X": "INSURANCE_FUND","m": false
//                     }"#,
//                     expected: Ok(BinanceTicker {
//                         subscription_id: SubscriptionId::from("@trade|ETHUSDT"),
//                         time: datetime_utc_from_epoch_duration(Duration::from_millis(
//                             1749354825200,
//                         )),
//                         id: 1000000000,
//                         price: 10000.19,
//                         amount: 0.239000,
//                         side: Side::Buy,
//                     }),
//                 },
//             ];

//             for (index, test) in tests.into_iter().enumerate() {
//                 let actual = serde_json::from_str::<BinanceTicker>(test.input);
//                 match (actual, test.expected) {
//                     (Ok(actual), Ok(expected)) => {
//                         assert_eq!(actual, expected, "TC{} failed", index)
//                     }
//                     (Err(_), Err(_)) => {
//                         // Test passed
//                     }
//                     (actual, expected) => {
//                         // Test failed
//                         panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
//                     }
//                 }
//             }
//         }
//     }
// }
