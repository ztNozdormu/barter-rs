use super::SubscriptionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Candle`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Tikers;

impl SubscriptionKind for Tikers {
    type Event = Tiker;
}

/// Normalised Barter OHLCV [`Tiker`] model.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Tiker {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
}
