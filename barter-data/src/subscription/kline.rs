use super::SubscriptionKind;
use barter_macro::{DeSubKind, SerSubKind};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};
use crate::subscription::liquidation::Liquidation;
use crate::subscription::ticker::Tikers;

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Kline`]
/// market events.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DeSubKind, SerSubKind,
)]
pub struct Klines;

impl SubscriptionKind for Klines {
    type Event = Kline;
    fn as_str(&self) -> &'static str {
        "klines"
    }
}

impl Display for Kline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Normalised Barter [`Kline`] model.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Kline {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
    pub closed: bool,
}