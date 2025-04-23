use std::any::Any;
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
use std::marker::PhantomData;
use chrono::{DateTime, Utc};
use smol_str::SmolStr;
use tracing::{info, Instrument};
use barter::engine::clock::HistoricalClock;
use barter::engine::Engine;
use barter::engine::execution_tx::MultiExchangeTxMap;
use barter::engine::state::EngineState;
use barter::engine::state::global::DefaultGlobalData;
use barter::engine::state::instrument::data::InstrumentDataState;
use barter::engine::state::instrument::filter::InstrumentFilter;
use barter::risk::DefaultRiskManager;
use barter::strategy::algo::AlgoStrategy;
use barter::strategy::close_positions::{close_open_positions_with_market_orders, ClosePositionsStrategy};
use barter::strategy::on_disconnect::OnDisconnectStrategy;
use barter::strategy::on_trading_disabled::OnTradingDisabled;
use barter_execution::order::OrderKey;
use barter_execution::order::request::{OrderRequestCancel, OrderRequestOpen};
use crate::bot::data::trend_data::TrendStrategyInstrumentData;
use crate::bot::{MultiStrategy, StrategyA};

pub(crate) struct TrendStrategy{
    pub id: StrategyId,
}

impl Default for TrendStrategy {
    fn default() -> Self {
        Self {
            id: StrategyId::new("TrendStrategy"),
        }
    }
}


impl AlgoStrategy for TrendStrategy {
    type State = EngineState<DefaultGlobalData, TrendStrategyInstrumentData>;

    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        let opens = state.instruments.instruments(&InstrumentFilter::None).filter_map(|state| {
            // Don't open more if we have a Position already
            if state.position.current.is_some() {
                return None;
            }

            // Don't open more orders if there are already some InFlight
            if !state.orders.0.is_empty() {
                return None;
            }

            // Don't open if there is no market data price available
            // let price = state.data.market_data.price()?;
            // info!("{price:?}");
            // let last_kline = state.data.market_data.last_kline;
            // info!("{last_kline:?}");

            let klines = &state.data.market_data.klines;
            if(klines.len() > 1){
                info!("{klines:?}");
            }
            return None;
            // TODO Generate Market order to buy the minimum allowed quantity
            // Some(OrderRequestOpen {
            //     key: OrderKey {
            //         exchange: state.instrument.exchange,
            //         instrument: state.key,
            //         strategy: self.id.clone(),
            //         cid: ClientOrderId::random(),
            //     },
            //     state: RequestOpen {
            //         side: Side::Buy,
            //         price: Decimal::from_f64(trade_not_sent_as_order_open.price).unwrap(),
            //         quantity: Decimal::from_f64(trade_not_sent_as_order_open.amount).unwrap(),
            //         kind: OrderKind::Market,
            //         time_in_force: TimeInForce::ImmediateOrCancel,
            //     },
            // })

        });
        (std::iter::empty(), opens)
    }
}


impl ClosePositionsStrategy for TrendStrategy {
    type State = EngineState<DefaultGlobalData, TrendStrategyInstrumentData>;

    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a InstrumentFilter,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        close_open_positions_with_market_orders(&self.id, state, filter, |_| {
            ClientOrderId::random()
        })
    }
}

// impl
// OnDisconnectStrategy<
//     HistoricalClock,
//     EngineState<DefaultGlobalData, TrendStrategyInstrumentData>,
//     MultiExchangeTxMap,
//     DefaultRiskManager<EngineState<DefaultGlobalData, TrendStrategyInstrumentData>>,
// > for TrendStrategy
// {
//     type OnDisconnect = ();
//
//     fn on_disconnect(
//         _: &mut Engine<
//             HistoricalClock,
//             EngineState<DefaultGlobalData, TrendStrategyInstrumentData>,
//             MultiExchangeTxMap,
//             Self,
//             DefaultRiskManager<EngineState<DefaultGlobalData, TrendStrategyInstrumentData>>,
//         >,
//         _: ExchangeId,
//     ) -> Self::OnDisconnect {
//     }
// }
//
// impl OnTradingDisabled<
//     HistoricalClock,
//     EngineState<DefaultGlobalData, TrendStrategyInstrumentData>,
//     MultiExchangeTxMap,
//     DefaultRiskManager<EngineState<DefaultGlobalData, TrendStrategyInstrumentData>>,
// > for TrendStrategy
// {
//     type OnTradingDisabled = ();
//
//     fn on_trading_disabled(
//         _: &mut Engine<
//             HistoricalClock,
//             EngineState<DefaultGlobalData, TrendStrategyInstrumentData>,
//             MultiExchangeTxMap,
//             Self,
//             DefaultRiskManager<EngineState<DefaultGlobalData, TrendStrategyInstrumentData>>,
//         >,
//     ) -> Self::OnTradingDisabled {
//     }
// }

impl<Clock, State, ExecutionTxs, Risk> OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
for TrendStrategy
{
    type OnDisconnect = ();

    fn on_disconnect(
        _: &mut Engine<Clock, State, ExecutionTxs, Self, Risk>,
        _: ExchangeId,
    ) -> Self::OnDisconnect {
    }
}

impl<Clock, State, ExecutionTxs, Risk> OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
for TrendStrategy
{
    type OnTradingDisabled = ();

    fn on_trading_disabled(
        _: &mut Engine<Clock, State, ExecutionTxs, Self, Risk>,
    ) -> Self::OnTradingDisabled {
    }
}

