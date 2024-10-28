use super::{tiker::Tiker, SubscriptionKind};
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Candle`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Deserialize,
    Serialize,
    Display,
)]
pub struct Candles;

impl SubscriptionKind for Candles {
    type Event = Candle;

    fn as_str(&self) -> &'static str {
        "candles"
    }
}

/// Normalised Barter OHLCV [`Candle`] model.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Candle {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
}

impl Candle {

    pub fn new(tiker: Tiker) -> Self {
        Self {
            close_time: tiker.close_time,
            open : tiker.open,
            high : tiker.high,
            low : tiker.low,
            close : tiker.last_price,
            volume : tiker.volume,
            trade_count : 0u64,
        }
    }
    pub fn update(&mut self, tiker: Tiker) {
        self.close_time = tiker.close_time;
        self.open = tiker.open;
        self.high = tiker.high;
        self.low = tiker.low;
        self.close = tiker.last_price;
        self.volume = tiker.volume;
        self.trade_count = 0u64;
    }
}

