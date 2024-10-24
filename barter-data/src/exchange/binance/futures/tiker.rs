use barter_integration::model::{Exchange, Side, SubscriptionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{binance::channel::BinanceChannel, ExchangeId, ExchangeSub},
    subscription::{tiker::{self, Tiker}, trade::PublicTrade},
    Identifier,
};

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
/// #### Spot Side::Buy Trade
/// ```json
/// {
///     "e":"trade",
///     "E":1649324825173,
///     "s":"ETHUSDT",
///     "t":1000000000,
///     "p":"10000.19",
///     "q":"0.239000",
///     "b":10108767791,
///     "a":10108764858,
///     "T":1749354825200,
///     "m":false,
///     "M":true
/// }
/// ```
///
/// #### FuturePerpetual Side::Sell Trade
/// ```json
/// {
///     "e": "trade",
///     "E": 1649839266194,
///     "T": 1749354825200,
///     "s": "ETHUSDT",
///     "t": 1000000000,
///     "p":"10000.19",
///     "q":"0.239000",
///     "X": "MARKET",
///     "m": true
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceTiker {
    #[serde(alias = "s", deserialize_with = "de_tiker_subscription_id")]
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
    pub  first_id: u64,
    #[serde(alias = "L")]
    pub  last_id: u64,
    #[serde(alias = "n")]
    pub  count: u64,
}

impl Identifier<Option<SubscriptionId>> for BinanceTiker {
    fn id(&self) -> Option<SubscriptionId> {
        Some(self.subscription_id.clone())
    }
}

impl<InstrumentId> From<(ExchangeId, InstrumentId, BinanceTiker)>
    for MarketIter<InstrumentId, Tiker>
{
    fn from((exchange_id, instrument, tiker): (ExchangeId, InstrumentId, BinanceTiker)) -> Self {
        Self(vec![Ok(MarketEvent {
            exchange_time: tiker.close_time,
            received_time: Utc::now(),
            exchange: Exchange::from(exchange_id),
            instrument,
            kind: Tiker {
                price_change: tiker.price_change,
                price_change_percent: tiker.price_change_percent,
                weighted_avg_price: tiker.weighted_avg_price,
                // prev_close_price: tiker.prev_close_price,
                last_qty: tiker.last_qty,
                // bid_price: tiker.bid_price,
                // bid_qty: tiker.bid_qty,
                // ask_price: tiker.ask_price,
                // ask_qty: tiker.ask_qty,
                open: tiker.open_price,
                high: tiker.high_price,
                low: tiker.low_price,
                last_price: tiker.last_price,
                volume: tiker.volume,
                quote_volume: tiker.quote_volume,
                open_time: tiker.open_time,
                close_time: tiker.close_time,
                first_id: tiker.first_id,
                last_id: tiker.last_id,
                count: tiker.count,
            },
        })])
    }
}

/// Deserialize a [`BinanceTiker`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`]
/// (eg/ "BTCUSDT@tiker").
pub fn de_tiker_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
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
//                 expected: Result<BinanceTiker, SocketError>,
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
//                     expected: Ok(BinanceTiker {
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
//                     expected: Ok(BinanceTiker {
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
//                     expected: Ok(BinanceTiker {
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
//                     expected: Ok(BinanceTiker {
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
//                 let actual = serde_json::from_str::<BinanceTiker>(test.input);
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
