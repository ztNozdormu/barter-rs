use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use barter::engine::Processor;
use barter::engine::state::instrument::data::InstrumentDataState;
use barter::engine::state::order::in_flight_recorder::InFlightRequestRecorder;
use barter::engine::state::position::PositionManager;
use barter::statistic::summary::instrument::TearSheetGenerator;
use barter_data::event::{DataKind, MarketEvent};
use barter_execution::{AccountEvent, AccountEventKind};
use barter_execution::order::request::{OrderRequestCancel, OrderRequestOpen};
use barter_instrument::exchange::ExchangeIndex;
use barter_instrument::instrument::InstrumentIndex;
use crate::bot::data::FeedMarketData;

//====== 策略自定义数据对象=====
#[derive(Debug, Clone)]
struct TrendInstrumentData {
    tear: TearSheetGenerator,
    position: PositionManager,
}

impl TrendInstrumentData {
    pub fn init(time_engine_start: DateTime<Utc>) -> Self {
        Self {
            tear: TearSheetGenerator::init(time_engine_start),
            position: PositionManager::default(),
        }
    }
}

impl Default for TrendInstrumentData {
    fn default() -> Self {
        Self {
            tear: TearSheetGenerator::init(DateTime::<Utc>::MIN_UTC),
            position: Default::default(),
        }
    }
}

//====== 策略市场数据对象和自定元数据对象集合=====
#[derive(Debug, Clone, Default)]
pub(crate) struct TrendStrategyInstrumentData {
    pub(crate) market_data: FeedMarketData,
    pub(crate) trend_instrument_data: TrendInstrumentData,
}


impl TrendStrategyInstrumentData {
    pub fn init(time_engine_start: DateTime<Utc>) -> Self {
        Self {
            market_data: FeedMarketData::default(),
            trend_instrument_data: TrendInstrumentData::init(time_engine_start),
        }
    }
}

impl InstrumentDataState for TrendStrategyInstrumentData {
    type MarketEventKind = DataKind;

    fn price(&self) -> Option<Decimal> {
        self.market_data.price()
    }
}

impl<InstrumentKey> Processor<&MarketEvent<InstrumentKey, DataKind>>
for TrendStrategyInstrumentData
{
    type Audit = ();

    fn process(&mut self, event: &MarketEvent<InstrumentKey, DataKind>) -> Self::Audit {
        self.market_data.process(event)
    }
}

impl Processor<&AccountEvent> for TrendStrategyInstrumentData {
    type Audit = ();

    fn process(&mut self, event: &AccountEvent) -> Self::Audit {

        let AccountEventKind::Trade(trade) = &event.kind else {
            return;
        };

        self.trend_instrument_data
            .position
            .update_from_trade(trade)
            .inspect(|closed| self.trend_instrument_data.tear.update_from_position(closed));

    }
}

impl InFlightRequestRecorder for TrendStrategyInstrumentData {
    fn record_in_flight_cancel(&mut self, _: &OrderRequestCancel<ExchangeIndex, InstrumentIndex>) {}

    fn record_in_flight_open(&mut self, _: &OrderRequestOpen<ExchangeIndex, InstrumentIndex>) {}
}