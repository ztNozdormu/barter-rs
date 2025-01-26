use super::SubscriptionKind;
use barter_macro::{DeSubKind, SerSubKind};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Ticker`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DeSubKind, SerSubKind,
)]
pub struct Tikers;

impl SubscriptionKind for Tikers {
    type Event = Ticker;

    fn as_str(&self) -> &'static str {
        "tikers"
    }
}
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
/// Normalised Barter OHLCV [`Ticker`] model.
// #[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize, Serialize, Eq)]
// pub struct Ticker {
//     pub price_change: f64,
//     pub price_change_percent: f64,
//     pub weighted_avg_price: f64,
//     // pub prev_close_price: f64,
//     pub last_qty: f64,
//     // pub bid_price: f64,
//     // pub bid_qty: f64,
//     // pub ask_price: f64,
//     // pub ask_qty: f64,
//     pub open: f64,
//     pub high: f64,
//     pub low: f64,
//     pub last_price: f64,
//
//     pub volume: f64,
//     pub quote_volume: f64,
//
//     pub open_time: DateTime<Utc>,
//     pub close_time: DateTime<Utc>,
//     pub first_id: u64,
//     pub last_id: u64,
//     pub count: u64,
// }

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
/// Normalised Barter OHLCV [`Ticker`] model.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize, Serialize)]
pub struct Ticker {
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
    pub first_id: u64,
    pub last_id: u64,
    pub count: u64,
}

impl std::fmt::Display for Tikers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
