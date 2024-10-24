use super::SubscriptionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Tiker`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Tikers;

impl SubscriptionKind for Tikers {
    type Event = Tiker;

    fn as_str(&self) -> &'static str {
        "tikers"
    }
}

/// Normalised Barter OHLCV [`Tiker`] model.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Tiker {
    
    pub price_change: f64,
    pub price_change_percent: f64,
    pub weighted_avg_price: f64,
    // pub prev_close_price: f64,

    pub last_qty: f64,
    // pub bid_price: f64,
    // pub bid_qty: f64,
    // pub ask_price: f64,
    // pub ask_qty: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub last_price: f64,

    pub volume: f64,
    pub quote_volume: f64,

    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub  first_id: u64,
    pub  last_id: u64,
    pub  count: u64,
}
