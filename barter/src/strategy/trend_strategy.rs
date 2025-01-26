use crate::engine::state::instrument::filter::InstrumentFilter;
use crate::engine::state::instrument::market_data::{DefaultMarketData, MarketDataState};
use crate::engine::state::EngineState;
use crate::engine::{Engine, Processor};
use crate::risk::DefaultRiskManagerState;
use crate::strategy::algo::AlgoStrategy;
use crate::strategy::close_positions::{close_open_positions_with_market_orders, ClosePositionsStrategy};
use crate::strategy::on_disconnect::OnDisconnectStrategy;
use crate::strategy::on_trading_disabled::OnTradingDisabled;
use crate::strategy::DefaultStrategyState;
use barter_data::event::MarketEvent;
use barter_execution::order::id::{ClientOrderId, StrategyId};
use barter_execution::order::{Order, OrderKind, RequestCancel, RequestOpen, TimeInForce};
use barter_execution::AccountEvent;
use barter_instrument::{
    asset::AssetIndex,
    exchange::{ExchangeId, ExchangeIndex},
    instrument::InstrumentIndex,
    Side,
};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Naive implementation of all strategy interfaces.
///
/// *THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION*.
///
/// This strategy:
/// - Generates no algorithmic orders (AlgoStrategy).
/// - Closes positions via the naive [`close_open_positions_with_market_orders`] logic (ClosePositionsStrategy).
/// - Does nothing when an exchange disconnects (OnDisconnectStrategy).
/// - Does nothing when trading state is set to disabled (OnDisconnectStrategy).
#[derive(Debug, Clone)]
pub struct TrendStrategy<State> {
    pub id: StrategyId,
    phantom: PhantomData<State>,
}

impl<State> Default for TrendStrategy<State> {
    fn default() -> Self {
        Self {
            id: StrategyId::new("default"),
            phantom: PhantomData,
        }
    }
}
// impl<State, ExchangeKey, InstrumentKey> AlgoStrategy<ExchangeKey, InstrumentKey>
// for TrendStrategy<State>
// {}
impl<State> AlgoStrategy for TrendStrategy<State> {
    type State = EngineState<DefaultMarketData, DefaultStrategyState, DefaultRiskManagerState>;

    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeIndex, InstrumentIndex, RequestCancel>>,
        impl IntoIterator<Item = Order<ExchangeIndex, InstrumentIndex, RequestOpen>>,
    ) {

        let opens = state.instruments.instruments().filter_map(|state| {
            // Don't open more if we have a Position already
            if state.position.is_some() {
                return None;
            }

            // Don't open more orders if there are already some InFlight
            if !state.orders.0.is_empty() {
                return None;
            }

            // Don't open if there is no market data price available
            let price = state.market.price()?;

            // Generate Market order to buy the minimum allowed quantity
            Some(Order {
                exchange: state.instrument.exchange,
                instrument: state.key,
                strategy: self.id.clone(),
                cid: gen_cid(state.key.index()),
                side: Side::Buy,
                state: RequestOpen {
                    kind: OrderKind::Market,
                    time_in_force: TimeInForce::ImmediateOrCancel,
                    price,
                    quantity: dec!(1),
                },
            })
        });
        (std::iter::empty(), opens)
    }
}

impl<MarketState, StrategyState, RiskState> ClosePositionsStrategy
for TrendStrategy<EngineState<MarketState, StrategyState, RiskState>>
where
    MarketState: MarketDataState,
{
    type State = EngineState<MarketState, StrategyState, RiskState>;

    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a InstrumentFilter,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeIndex, InstrumentIndex, RequestCancel>> + 'a,
        impl IntoIterator<Item = Order<ExchangeIndex, InstrumentIndex, RequestOpen>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        close_open_positions_with_market_orders(&self.id, state, filter)
    }
}

impl<Clock, State, ExecutionTxs, Risk> OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
for TrendStrategy<State>
{
    type OnDisconnect = ();

    fn on_disconnect(
        _: &mut Engine<Clock, State, ExecutionTxs, Self, Risk>,
        _: ExchangeId,
    ) -> Self::OnDisconnect {
    }
}

impl<Clock, State, ExecutionTxs, Risk> OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
for TrendStrategy<State>
{
    type OnTradingDisabled = ();

    fn on_trading_disabled(
        _: &mut Engine<Clock, State, ExecutionTxs, Self, Risk>,
    ) -> Self::OnTradingDisabled {
    }
}

/// Empty strategy state that can be used for strategies that require no specific global state.
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize,
)]
pub struct TrendStrategyState;

impl<ExchangeKey, AssetKey, InstrumentKey>
Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for TrendStrategyState
{
    type Audit = ();
    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for TrendStrategyState {
    type Audit = ();
    fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
}


fn gen_cid(instrument: usize) -> ClientOrderId {
    ClientOrderId::new(InstrumentIndex(instrument).to_string())
}
