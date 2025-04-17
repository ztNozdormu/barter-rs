use std::fmt::Display;
use super::SubscriptionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`Kline`]
/// market events.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize, Serialize)]
pub struct KLines(pub u32);

impl SubscriptionKind for KLines {
    type Event = KLine;
    fn as_str(&self) -> &'static str {
        const STATIC_PREFIX: &str = "KLines"; // 定义一个静态常量
                                              // 把数字转换为字符串并拼接
        let combined = format!("{}{}", STATIC_PREFIX, self.0);

        // 这会引发问题，因为 combined 是一个临时的 String
        // 不能直接返回 &'static str
        Box::leak(combined.into_boxed_str())
        // "klines"
    }
}

impl std::fmt::Display for KLines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Normalised Barter [`Kline`] model.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize, Serialize, Default)]
pub struct KLine {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
    pub closed: bool,
}
