use crate::{
    engine::{
        state::{
            instrument::{
                filter::InstrumentFilter,
                market_data::{FeedMarketData, MarketDataState},
            },
            EngineState,
        },
        Engine, Processor,
    },
    risk::DefaultRiskManagerState,
    strategy::{
        algo::AlgoStrategy,
        close_positions::{close_open_positions_with_market_orders, ClosePositionsStrategy},
        on_disconnect::OnDisconnectStrategy,
        on_trading_disabled::OnTradingDisabled,
    },
};
use barter_data::event::MarketEvent;
use barter_execution::{
    order::{
        id::{ClientOrderId, StrategyId},
        Order, OrderKind, TimeInForce,
    },
    AccountEvent,
};
use barter_instrument::{
    asset::AssetIndex,
    exchange::{ExchangeId, ExchangeIndex},
    instrument::InstrumentIndex,
    Side,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tracing::info;
use barter_execution::order::id::OrderId;
use barter_execution::order::OrderKey;
use barter_execution::order::request::{OrderRequestCancel, OrderRequestOpen, RequestCancel, RequestOpen};
use barter_execution::trade::TradeId;
use barter_integration::channel::UnboundedTx;
use crate::engine::clock::HistoricalClock;
use crate::engine::execution_tx::MultiExchangeTxMap;
use crate::engine::state::instrument::market_data::DefaultMarketData;
use crate::execution::request::ExecutionRequest;
use crate::risk::DefaultRiskManager;
use crate::strategy::DefaultStrategyState;

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

impl<State> AlgoStrategy for TrendStrategy<State> {
    type State = EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>;

    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        let opens = state
            .instruments
            .instruments(&InstrumentFilter::None)
            .filter_map(|state| {
                // Don't open more if we have a Position already
                if state.position.current.is_some() {
                    return None;
                }

                // Don't open more orders if there are already some InFlight
                if !state.orders.0.is_empty() {
                    return None;
                }

                // Don't open if there is no market data price available
                // let price = state.market.price()?;

                // info!("{price:?}");
                let last_kline = state.market.last_kline;
                // info!("{last_kline:?}");
                if state.market.closed {
                    let klines = &state.market.klines;
                    info!("{klines:?}");
                }

                // Generate Market order to buy the minimum allowed quantity
                Some(OrderRequestOpen {
                    key: OrderKey {
                        exchange: state.instrument.exchange,
                        instrument: state.key,
                        strategy: self.id.clone(),
                        cid: gen_cid(state.key.index()),
                    },
                    state: RequestOpen {
                        side: Side::Buy,
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

fn strategy_id() -> StrategyId {
    StrategyId::new("TrendStrategy")
}

fn gen_cid(instrument: usize) -> ClientOrderId {
    ClientOrderId::new(InstrumentIndex(instrument).to_string())
}

fn gen_trade_id(instrument: usize) -> TradeId {
    TradeId::new(InstrumentIndex(instrument).to_string())
}

fn gen_order_id(instrument: usize) -> OrderId {
    OrderId::new(InstrumentIndex(instrument).to_string())
}

impl<MarketState, StrategyState, RiskState> ClosePositionsStrategy for TrendStrategy<EngineState<MarketState, StrategyState, RiskState>> {
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

#[derive(Debug, PartialEq)]
struct OnDisconnectOutput;
impl<MarketState, StrategyState, RiskState>
OnDisconnectStrategy<
    HistoricalClock,
    EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
    MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
    DefaultRiskManager<
        EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
    >,
> for TrendStrategy<EngineState<MarketState, StrategyState, RiskState>>
{
    type OnDisconnect = OnDisconnectOutput;

    fn on_disconnect(
        _: &mut Engine<
            HistoricalClock,
            EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
            MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
            Self,
            DefaultRiskManager<
                EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
            >,
        >,
        _: ExchangeId,
    ) -> Self::OnDisconnect {
        OnDisconnectOutput
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
