use std::collections::VecDeque;
use crate::{engine::Processor, Timed};
use barter_data::{
    event::{DataKind, MarketEvent},
    subscription::{
        book::OrderBookL1,
        kline::{KLine, KLines},
    },
};
use barter_instrument::instrument::InstrumentIndex;
use barter_xchange::exchange::binance::model::Kline;
use derive_more::Constructor;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Defines a state object for tracking and managing the market data state of an instrument.
///
/// Implementations must handle market event processing and logic for providing the latest
/// instrument price.
///
/// The trait enables users to provide their own instrument market data state, and the type of
/// [`MarketEvent`] that is required to update it.
///
/// For an example, see the [`DefaultMarketData`] implementation.
pub trait MarketDataState<InstrumentKey = InstrumentIndex>
where
    Self: Debug + Clone + Send + for<'a> Processor<&'a MarketEvent<InstrumentKey, Self::EventKind>>,
{
    /// [`MarketEvent<_, EventKind>`](MarketEvent) expected by this market data state.
    type EventKind: Debug + Clone + Send;

    /// Latest price for an instrument, if available.
    ///
    /// Return the latest market price for an instrument, if available.
    ///
    /// An instrument price could be derived in many ways, but some common examples include:
    /// - Most recent `PublicTrade` price.
    /// - Volume-weighted mid-price from an `OrderBookL1`.
    /// - Volume-weighted mid-price from an `OrderBookL2`.
    fn price(&self) -> Option<Decimal>;
}

/// Basic [`MarketDataState`] that tracks the [`OrderBookL1`] and last traded price for an
/// instrument.
///
/// Trading strategies may wish to maintain more data here, such as candles, indicators,
/// L2 book, etc.
#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize, Constructor,
)]
pub struct DefaultMarketData {
    pub l1: OrderBookL1,
    pub last_traded_price: Option<Timed<Decimal>>,
}

impl MarketDataState for DefaultMarketData {
    type EventKind = DataKind;

    fn price(&self) -> Option<Decimal> {
        self.l1
            .volume_weighed_mid_price()
            .or(self.last_traded_price.as_ref().map(|timed| timed.value))
    }
}

impl<InstrumentKey> Processor<&MarketEvent<InstrumentKey, DataKind>> for DefaultMarketData {
    type Audit = ();

    fn process(&mut self, event: &MarketEvent<InstrumentKey, DataKind>) -> Self::Audit {
        match &event.kind {
            DataKind::Trade(trade) => {
                if self
                    .last_traded_price
                    .as_ref()
                    .map_or(true, |price| price.time < event.time_exchange)
                {
                    if let Some(price) = Decimal::from_f64(trade.price) {
                        self.last_traded_price
                            .replace(Timed::new(price, event.time_exchange));
                    }
                }
            }
            DataKind::OrderBookL1(l1) => {
                if self.l1.last_update_time < event.time_exchange {
                    self.l1 = l1.clone()
                }
            }
            _ => {}
        }
    }
}

/// Basic [`MarketDataState`] that tracks the [`Kline`] and sets traded kline for an
/// instrument.
///
/// Trading strategies may wish to maintain more data here, such as candles, indicators,
/// KLines , etc.
#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deserialize, Serialize, Constructor)]
pub struct FeedMarketData {
    pub closed: bool,
    pub last_kline: KLine,
    pub klines: VecDeque<KLine>,
}

impl FeedMarketData {
    // Method to add a KLine to klines, ensuring a max of 500 elements
    pub fn update(&mut self, kline: KLine) {
        self.closed = kline.closed;
        self.last_kline = kline.clone();
        // If closed is true
        if self.closed {
            // If closed is true & there are already 500 elements, remove the oldest one (front of the deque)
            if self.klines.len() == 500 {
                // Remove elements from the back (bottom)
                self.klines.pop_back();
            }
            // Add the new kline to the deque
            self.klines.push_front(kline);
        }
    }
}

impl MarketDataState for FeedMarketData {
    type EventKind = DataKind;
    fn price(&self) -> Option<Decimal> {
        Some(Decimal::from_f64(self.last_kline.close).unwrap_or(dec!(0.0)))
    }
}

impl<InstrumentKey> Processor<&MarketEvent<InstrumentKey, DataKind>> for FeedMarketData {
    type Audit = ();

    fn process(&mut self, event: &MarketEvent<InstrumentKey, DataKind>) -> Self::Audit {
        match &event.kind {
            DataKind::KLine(kline) => {
                // update FeedMarketData
                self.update(kline.clone());
            }
            _ => {}
        }
    }
}
