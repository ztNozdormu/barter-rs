use barter_integration::subscription::SubscriptionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{binance::channel::BinanceChannel, ExchangeId, ExchangeSub},
    Identifier,
};
use crate::subscription::ticker::Ticker;

/// Binance real-time trade message.
///
/// Note:
/// For [`BinanceFuturesUsd`](super::futures::BinanceFuturesUsd) this real-time stream is
/// undocumented.
///
/// See discord: <https://discord.com/channels/910237311332151317/923160222711812126/975712874582388757>
///
/// ### Raw Payload Examples
/// See docs: <https://binance-docs.github.io/apidocs/spot/en/#trade-streams>
/// ```json
/// {
///   "e": "24hrTicker",  // Event type
///   "E": 123456789,     // Event time
///   "s": "BTCUSDT",     // Symbol
///   "p": "0.0015",      // Price change
///   "P": "250.00",      // Price change percent
///   "w": "0.0018",      // Weighted average price
///   "c": "0.0025",      // Last price
///   "Q": "10",          // Last quantity
///   "o": "0.0010",      // Open price
///   "h": "0.0025",      // High price
///   "l": "0.0010",      // Low price
///   "v": "10000",       // Total traded base asset volume
///   "q": "18",          // Total traded quote asset volume
///   "O": 0,             // Statistics open time
///   "C": 86400000,      // Statistics close time
///   "F": 0,             // First trade ID
///   "L": 18150,         // Last trade Id
///   "n": 18151          // Total number of trades
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceTicker {
    #[serde(alias = "s", deserialize_with = "de_ticker_subscription_id")]
    pub subscription_id: SubscriptionId,
    #[serde(alias = "p", deserialize_with = "barter_integration::de::de_str")]
    pub price_change: f64,
    #[serde(alias = "P", deserialize_with = "barter_integration::de::de_str")]
    pub price_change_percent: f64,
    #[serde(alias = "w", deserialize_with = "barter_integration::de::de_str")]
    pub weighted_avg_price: f64,
    // #[serde(alias = "x", deserialize_with = "barter_integration::de::de_str")]
    // pub prev_close_price: f64,
    #[serde(alias = "c", deserialize_with = "barter_integration::de::de_str")]
    pub last_price: f64,
    #[serde(alias = "Q", deserialize_with = "barter_integration::de::de_str")]
    pub last_qty: f64,
    // #[serde(alias = "b", deserialize_with = "barter_integration::de::de_str")]
    // pub bid_price: f64,
    // #[serde(alias = "B", deserialize_with = "barter_integration::de::de_str")]
    // pub bid_qty: f64,
    // #[serde(alias = "a", deserialize_with = "barter_integration::de::de_str")]
    // pub ask_price: f64,
    // #[serde(alias = "A", deserialize_with = "barter_integration::de::de_str")]
    // pub ask_qty: f64,
    #[serde(alias = "o", deserialize_with = "barter_integration::de::de_str")]
    pub open_price: f64,
    #[serde(alias = "h", deserialize_with = "barter_integration::de::de_str")]
    pub high_price: f64,
    #[serde(alias = "l", deserialize_with = "barter_integration::de::de_str")]
    pub low_price: f64,
    #[serde(alias = "v", deserialize_with = "barter_integration::de::de_str")]
    pub volume: f64,
    #[serde(alias = "q", deserialize_with = "barter_integration::de::de_str")]
    pub quote_volume: f64,
    #[serde(
        alias = "O",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub open_time: DateTime<Utc>,
    #[serde(
        alias = "C",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub close_time: DateTime<Utc>,
    #[serde(alias = "F")]
    pub first_id: u64,
    #[serde(alias = "L")]
    pub last_id: u64,
    #[serde(alias = "n")]
    pub count: u64,
}

impl Identifier<Option<SubscriptionId>> for BinanceTicker {
    fn id(&self) -> Option<SubscriptionId> {
        Some(self.subscription_id.clone())
    }
}

impl<InstrumentKey> From<(ExchangeId, InstrumentKey, BinanceTicker)>
    for MarketIter<InstrumentKey, Ticker>
{
    fn from((exchange_id, instrument, Ticker): (ExchangeId, InstrumentKey, BinanceTicker)) -> Self {
        Self(vec![Ok(MarketEvent {
            time_exchange: Ticker.close_time,
            time_received: Utc::now(),
            exchange: exchange_id,
            instrument,
            kind: Ticker {
                price_change: Ticker.price_change,
                price_change_percent: Ticker.price_change_percent,
                weighted_avg_price: Ticker.weighted_avg_price,
                // prev_close_price: Ticker.prev_close_price,
                last_qty: Ticker.last_qty,
                // bid_price: Ticker.bid_price,
                // bid_qty: Ticker.bid_qty,
                // ask_price: Ticker.ask_price,
                // ask_qty: Ticker.ask_qty,
                open: Ticker.open_price,
                high: Ticker.high_price,
                low: Ticker.low_price,
                last_price: Ticker.last_price,
                volume: Ticker.volume,
                quote_volume: Ticker.quote_volume,
                open_time: Ticker.open_time,
                close_time: Ticker.close_time,
                first_id: Ticker.first_id,
                last_id: Ticker.last_id,
                count: Ticker.count,
            },
        })])
    }
}

/// Deserialize a [`BinanceTicker`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`]
/// (eg/ "BTCUSDT@Ticker").
pub fn de_ticker_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|market| ExchangeSub::from((BinanceChannel::TICKERS, market)).id())
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
