use crate::engine::clock::LiveClock;
use crate::engine::execution_tx::MultiExchangeTxMap;
use crate::engine::state::instrument::filter::InstrumentFilter;
use crate::engine::state::instrument::market_data::{FeedMarketData, MarketDataState};
use crate::engine::state::EngineState;
use crate::engine::Engine;
use crate::execution::request::ExecutionRequest;
use crate::risk::{DefaultRiskManager, DefaultRiskManagerState};
use crate::strategy::algo::AlgoStrategy;
use crate::strategy::close_positions::{close_open_positions_with_market_orders, ClosePositionsStrategy};
use crate::strategy::on_disconnect::OnDisconnectStrategy;
use crate::strategy::on_trading_disabled::OnTradingDisabled;
use crate::strategy::DefaultStrategyState;
use barter_execution::balance::Balance;
use barter_execution::order::id::{ClientOrderId, OrderId, StrategyId};
use barter_execution::order::request::{OrderRequestCancel, OrderRequestOpen, RequestOpen};
use barter_execution::order::{OrderKey, OrderKind, TimeInForce};
use barter_execution::trade::TradeId;
use barter_instrument::asset::{Asset, AssetIndex};
use barter_instrument::exchange::{ExchangeId, ExchangeIndex};
use barter_instrument::index::IndexedInstruments;
use barter_instrument::instrument::kind::InstrumentKind;
use barter_instrument::instrument::quote::InstrumentQuoteAsset;
use barter_instrument::instrument::spec::{InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity, OrderQuantityUnits};
use barter_instrument::instrument::{Instrument, InstrumentIndex};
use barter_instrument::{Side, Underlying};
use barter_integration::channel::UnboundedTx;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// const info
pub const STARTING_TIMESTAMP: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;
const RISK_FREE_RETURN: Decimal = dec!(0.05);
const STARTING_BALANCE_USDT: Balance = Balance {
    total: dec!(40_000.0),
    free: dec!(40_000.0),
};
const STARTING_BALANCE_BTC: Balance = Balance {
    total: dec!(1.0),
    free: dec!(1.0),
};
const STARTING_BALANCE_ETH: Balance = Balance {
    total: dec!(10.0),
    free: dec!(10.0),
};
const QUOTE_FEES_PERCENT: f64 = 0.1; // 10%


pub struct MartinStrategy {
    pub(crate) id: StrategyId,
}

impl MartinStrategy {
    pub fn strategy_id() -> StrategyId {
        StrategyId::new("MartinStrategy")
    }

    pub fn gen_cid(instrument: usize) -> ClientOrderId {
        ClientOrderId::new(InstrumentIndex(instrument).to_string())
    }

    pub fn gen_trade_id(instrument: usize) -> TradeId {
        TradeId::new(InstrumentIndex(instrument).to_string())
    }

    pub fn gen_order_id(instrument: usize) -> OrderId {
        OrderId::new(InstrumentIndex(instrument).to_string())
    }

    // Get indexed instruments
    pub fn indexed_instruments() -> IndexedInstruments {
        IndexedInstruments::builder()
            .add_instrument(Instrument::new(
                ExchangeId::BinanceFuturesUsd,
                "binance_perpetual_btc_usdt",
                "BTCUSDT",
                Underlying::new("btc", "usdt"),
                InstrumentQuoteAsset::UnderlyingQuote,
                InstrumentKind::Perpetual {
                    contract_size: Default::default(),
                    settlement_asset: Asset::from("btc"),
                },
                Some(InstrumentSpec::new(
                    InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
                    InstrumentSpecQuantity::new(
                        OrderQuantityUnits::Quote,
                        dec!(0.00001),
                        dec!(0.00001),
                    ),
                    InstrumentSpecNotional::new(dec!(5.0)),
                )),
            ))
            .build()
    }

}
impl AlgoStrategy for MartinStrategy {
    type State = EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>;

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
                let price = state.market.price()?;

                // Generate Market order to buy the minimum allowed quantity
                Some(OrderRequestOpen {
                    key: OrderKey {
                        exchange: state.instrument.exchange,
                        instrument: state.key,
                        strategy: self.id.clone(),
                        cid: Self::gen_cid(state.key.index()),
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

impl ClosePositionsStrategy for MartinStrategy {
    type State = EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>;

    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a InstrumentFilter<ExchangeIndex, AssetIndex, InstrumentIndex>,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        close_open_positions_with_market_orders(&self.id, state, filter)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OnDisconnectOutput;
impl
OnDisconnectStrategy<
    LiveClock,
    EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
    MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
    DefaultRiskManager<
        EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
    >,
> for MartinStrategy
{
    type OnDisconnect = OnDisconnectOutput;

    fn on_disconnect(
        _: &mut Engine<
            LiveClock,
            EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
            MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
            Self,
            DefaultRiskManager<
                EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
            >,
        >,
        _: ExchangeId,
    ) -> Self::OnDisconnect {
        OnDisconnectOutput
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OnTradingDisabledOutput;
impl
OnTradingDisabled<
    LiveClock,
    EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
    MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
    DefaultRiskManager<
        EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
    >,
> for MartinStrategy
{
    type OnTradingDisabled = OnTradingDisabledOutput;

    fn on_trading_disabled(
        _: &mut Engine<
            LiveClock,
            EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
            MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
            Self,
            DefaultRiskManager<
                EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
            >,
        >,
    ) -> Self::OnTradingDisabled {
        OnTradingDisabledOutput
    }
}

