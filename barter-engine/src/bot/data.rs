pub(crate) mod trend_data;

use std::collections::VecDeque;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use barter::engine::Processor;
use barter::engine::state::instrument::data::InstrumentDataState;
use barter::engine::state::order::in_flight_recorder::InFlightRequestRecorder;
use barter_data::derive_more::Constructor;
use barter_data::event::{DataKind, MarketEvent};
use barter_data::subscription::kline::KLine;
use barter_execution::AccountEvent;
use barter_execution::order::request::{OrderRequestCancel, OrderRequestOpen};
use serde::{Deserialize, Serialize};




/// Basic [`InstrumentDataState`] that tracks the [`Kline`] and sets traded kline for an
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

impl InstrumentDataState for FeedMarketData {
    type MarketEventKind = DataKind;

    fn price(&self) -> Option<Decimal> {
        Some(Decimal::from_f64(self.last_kline.close).unwrap_or(dec!(0.0)))
    }
}

impl<InstrumentKey> Processor<&MarketEvent<InstrumentKey, DataKind>>
for FeedMarketData
{
    type Audit = ();

    fn process(&mut self, event: &MarketEvent<InstrumentKey, DataKind>) -> Self::Audit {
        match &event.kind {
            DataKind::KLine(kline) => {
                self.update(kline.clone());
            }
            _ => {}
        }
    }
}

impl<ExchangeKey, AssetKey, InstrumentKey>
Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for FeedMarketData
{
    type Audit = ();

    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

impl<ExchangeKey, InstrumentKey> InFlightRequestRecorder<ExchangeKey, InstrumentKey>
for FeedMarketData
{
    fn record_in_flight_cancel(&mut self, _: &OrderRequestCancel<ExchangeKey, InstrumentKey>) {}

    fn record_in_flight_open(&mut self, _: &OrderRequestOpen<ExchangeKey, InstrumentKey>) {}
}
