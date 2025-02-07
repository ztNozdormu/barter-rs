// use crate::{
//     event::{MarketEvent, MarketIter},
//     exchange::{binance::channel::BinanceChannel, ExchangeId, ExchangeSub},
//     Identifier,
// };
use super::super::BinanceChannel;
use crate::{
    event::{MarketEvent, MarketIter},
    exchange::subscription::ExchangeSub,
    subscription::kline::KLine,
    Identifier,
};
use barter_instrument::exchange::ExchangeId;
use barter_integration::subscription::SubscriptionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) kline.
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

    #[serde(alias = "o", deserialize_with = "barter_integration::de::de_str")]
    pub open: f64,
    #[serde(alias = "c", deserialize_with = "barter_integration::de::de_str")]
    pub close: f64,
    #[serde(alias = "h", deserialize_with = "barter_integration::de::de_str")]
    pub high: f64,
    #[serde(alias = "l", deserialize_with = "barter_integration::de::de_str")]
    pub low: f64,

    #[serde(alias = "v", deserialize_with = "barter_integration::de::de_str")]
    pub base_asset_volume: f64,

    #[serde(alias = "n")]
    pub trade_nums: u64,
    #[serde(alias = "x")]
    pub is_kline_closed: bool,
    #[serde(alias = "q", deserialize_with = "barter_integration::de::de_str")]
    pub quote_asset_volume: f64,
    #[serde(alias = "V", deserialize_with = "barter_integration::de::de_str")]
    pub buy_base_asset_volume: f64,
    #[serde(alias = "Q", deserialize_with = "barter_integration::de::de_str")]
    pub buy_quote_asset_volume: f64,

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

impl Identifier<Option<SubscriptionId>> for BinanceKline {
    fn id(&self) -> Option<SubscriptionId> {
        Some(self.kline.subscription_id.clone())
    }
}

impl<InstrumentKey> From<(ExchangeId, InstrumentKey, BinanceKline)>
    for MarketIter<InstrumentKey, KLine>
{
    fn from((exchange_id, instrument, kline): (ExchangeId, InstrumentKey, BinanceKline)) -> Self {
        Self(vec![Ok(MarketEvent {
            time_exchange: kline.kline.close_time,
            time_received: Utc::now(),
            exchange: exchange_id,
            instrument,
            kind: KLine {
                close_time: kline.kline.close_time,
                open: kline.kline.open,
                high: kline.kline.high,
                low: kline.kline.low,
                close: kline.kline.close,
                volume: kline.kline.base_asset_volume,
                trade_count: kline.kline.trade_nums,
                closed: kline.kline.is_kline_closed,
            },
        })])
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    mod de {
        use super::*;
        use barter_integration::de::datetime_utc_from_epoch_duration;
        use std::time::Duration;

        #[test]
        fn test_binance_kline() {
            let input = r#"
            {
              "e": "kline",
              "E": 1672515782136,
              "s": "BNBBTC",
              "k": {
                "t": 1672515780000,
                "T": 1672515839999,
                "s": "BNBBTC",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "0.0010",
                "c": "0.0020",
                "h": "0.0025",
                "l": "0.0015",
                "v": "1000",
                "n": 100,
                "x": false,
                "q": "1.0000",
                "V": "500",
                "Q": "0.500",
                "B": "123456"
              }
            }
            "#;
            assert_eq!(
                serde_json::from_str::<BinanceKline>(input).unwrap(),
                BinanceKline {
                    kline: BinanceKlineData {
                        subscription_id: SubscriptionId::from("@kline|BNBBTC"),
                        open: 0.0010,
                        close: 0.0020,
                        high: 0.0025,
                        low: 0.0015,
                        base_asset_volume: 1000f64,
                        trade_nums: 100,
                        is_kline_closed: false,
                        quote_asset_volume: 1.0000,
                        buy_base_asset_volume: 500f64,
                        buy_quote_asset_volume: 0.500,
                        start_time: datetime_utc_from_epoch_duration(Duration::from_millis(
                            1672515780000
                        )),
                        close_time: datetime_utc_from_epoch_duration(Duration::from_millis(
                            1672515839999
                        )),
                    },
                }
            );
        }
    }
}
