use crate::engine::audit::context::EngineContext;
use crate::engine::audit::AuditTick;
use crate::engine::execution_tx::{ExecutionTxMap, MultiExchangeTxMap};
use crate::engine::{run, EngineOutput, Processor};
use crate::risk::RiskManager;
use crate::strategy::algo::AlgoStrategy;
use crate::strategy::close_positions::ClosePositionsStrategy;
use crate::strategy::on_disconnect::OnDisconnectStrategy;
use crate::strategy::on_trading_disabled::OnTradingDisabled;
use crate::{engine::{
    audit::EngineAudit,
    clock::{EngineClock, LiveClock}

    ,
    state::{
        instrument::market_data::FeedMarketData,
        trading::TradingState,
        EngineState,
    },
    Engine,
}, execution::builder::ExecutionBuilder, logging::init_logging, risk::{DefaultRiskManager, DefaultRiskManagerState}, strategy::trend_strategy::{TrendStrategy, TrendStrategyState}, EngineEvent};
use barter_data::event::{DataKind, MarketEvent};
use barter_data::{
    streams::{
        builder::dynamic::indexed::init_indexed_multi_exchange_market_stream,
        reconnect::stream::ReconnectingStream,
    },
    subscription::SubKind,
};
use barter_execution::{balance::Balance, client::mock::MockExecutionConfig, AccountEvent};
use barter_instrument::exchange::ExchangeIndex;
use barter_instrument::instrument::InstrumentIndex;
use barter_instrument::{
    asset::Asset,
    exchange::ExchangeId,
    index::IndexedInstruments,
    instrument::{
        kind::InstrumentKind,
        spec::{
            InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity,
            OrderQuantityUnits,
        },
        Instrument,
    },
    Underlying,
};
use barter_integration::channel::{mpsc_unbounded, ChannelTxDroppable, Tx, UnboundedRx, UnboundedTx};
use fnv::FnvHashMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use futures::StreamExt;
use tracing::info;
use crate::engine::state::instrument::market_data::MarketDataState;
use crate::execution::request::ExecutionRequest;

const EXCHANGE: ExchangeId = ExchangeId::BinanceFuturesUsd;
const RISK_FREE_RETURN: Decimal = dec!(0.05);
const MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS: u64 = 100;
const MOCK_EXCHANGE_FEES_PERCENT: Decimal = dec!(0.05);
const STARTING_BALANCE_USDT: Balance = Balance {
    total: dec!(10_000.0),
    free: dec!(10_000.0),
};
const STARTING_BALANCE_BTC: Balance = Balance {
    total: dec!(0.1),
    free: dec!(0.1),
};
const STARTING_BALANCE_ETH: Balance = Balance {
    total: dec!(1.0),
    free: dec!(1.0),
};
const STARTING_BALANCE_SOL: Balance = Balance {
    total: dec!(10.0),
    free: dec!(10.0),
};


// #[derive(Debug)]
// pub struct TradingRobot<Clock, MarketState, StrategyState, RiskState, ExecutionTxs, Strategy, Risk> {
//     // engine: Engine<Clock, State, ExecutionTxs, Strategy, Risk>,
//     engine: Engine<Clock,EngineState<MarketState, StrategyState, RiskState>,ExecutionTxs,Strategy,Risk>,
//     feed_tx:  UnboundedTx<EngineEvent<DataKind>>,
//     feed_rx:  UnboundedRx<EngineEvent<DataKind>>,
//     audit_tx: UnboundedTx<AuditTick<EngineAudit<EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
//     audit_rx: UnboundedRx<AuditTick<EngineAudit<EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
//     instruments: IndexedInstruments,
//     state: EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
// }

// impl<Clock, MarketState, StrategyState, RiskState, ExecutionTxs, Strategy, Risk> TradingRobot<Clock, MarketState, StrategyState, RiskState, ExecutionTxs, Strategy, Risk>
// where
//     Clock: EngineClock,// + for<'a> Processor<&'a EngineEvent<MarketState::EventKind>>,
//     MarketState: MarketDataState,
//     StrategyState: for<'a> Processor<&'a AccountEvent>
//     + for<'a> Processor<&'a MarketEvent<InstrumentIndex, MarketState::EventKind>>,
//     RiskState: for<'a> Processor<&'a AccountEvent>
//     + for<'a> Processor<&'a MarketEvent<InstrumentIndex, MarketState::EventKind>>,
//     ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,
//     Strategy: OnTradingDisabled<
//         Clock,
//         EngineState<MarketState, StrategyState, RiskState>,
//         ExecutionTxs,
//         Risk,
//     > + OnDisconnectStrategy<
//         Clock,
//         EngineState<MarketState, StrategyState, RiskState>,
//         ExecutionTxs,
//         Risk,
//     > + AlgoStrategy<State= EngineState<MarketState, StrategyState, RiskState>>
//     + ClosePositionsStrategy<State= EngineState<MarketState, StrategyState, RiskState>>,
//     Risk: RiskManager<State= EngineState<MarketState, StrategyState, RiskState>>,
// {
//

/// 使用类型别名简化结构
type MyEngineState = EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>;

/// Engine 类型别名
// type MyEngine<Clock, ExecutionTxs, Strategy, Risk> = Engine<Clock, MyEngineState, ExecutionTxs, Strategy, Risk>;

/// TradingRobot 类型定义
pub struct TradingRobot {
    // engine: Engine<LiveClock, MyEngineState, ExecutionTxs, Strategy, Risk>,
    // engine: Engine<LiveClock, MyEngineState, MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>, TrendStrategy.default(), DefaultRiskManager>,
    feed_tx: UnboundedTx<EngineEvent<DataKind>>,
    feed_rx: UnboundedRx<EngineEvent<DataKind>>,
    // audit_tx: UnboundedTx<AuditTick<EngineAudit<MyEngineState, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
    // audit_rx: UnboundedRx<AuditTick<EngineAudit<MyEngineState, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
    instruments: IndexedInstruments,
    // state: MyEngineState,
}

impl TradingRobot {


    // Initialize the TradingRobot with necessary components and configurations
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>>
    {
        // Initialise Tracing
        init_logging();

        // Initialise Channels
        let (feed_tx,mut feed_rx) = mpsc_unbounded();
        // let (audit_tx, audit_rx) = mpsc_unbounded();

        // Construct IndexedInstruments
        let instruments = Self::indexed_instruments();

        // Initialise MarketData Stream & forward to Engine feed
        let market_stream =
            init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await?;
        tokio::spawn(market_stream.forward_to(feed_tx.clone()));

        // Construct Engine clock
        let clock = LiveClock;
//
//         // Construct EngineState from IndexedInstruments and hard-coded exchange asset Balances
        let state = EngineState::<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>::builder(&instruments)
            .time_engine_start(clock.time())
            .trading_state(TradingState::Enabled)
            .balances([
              (EXCHANGE, "btc", STARTING_BALANCE_BTC),
            // (EXCHANGE, "eth", STARTING_BALANCE_ETH),
            // (EXCHANGE, "sol", STARTING_BALANCE_SOL),
            // You can add more balances if needed.
            ])
            .build();

        // Generate initial AccountSnapshot from EngineState for BinanceSpot MockExchange
        // Note: for live-trading this would be automatically fetched via the AccountStream init
        let mut initial_account = FnvHashMap::from(&state);
        assert_eq!(initial_account.len(), 1);
//
        // Initialise ExecutionManager & forward Account Streams to Engine feed
        let (execution_txs, account_stream) = ExecutionBuilder::new(&instruments)
            .add_mock(MockExecutionConfig::new(
                EXCHANGE,
                initial_account.remove(&EXCHANGE).unwrap(),
                MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS,
                MOCK_EXCHANGE_FEES_PERCENT,
            ))?
            .init()
            .await?;
        tokio::spawn(account_stream.forward_to(feed_tx.clone()));

        // Construct Engine
        // let mut engine = Engine::new(
        //     clock,
        //     state.clone(),
        //     execution_txs,
        //     TrendStrategy::default(),
        //     DefaultRiskManager::default(),
        // );

        Ok(TradingRobot {
            // engine,
            feed_tx,
            feed_rx,
            // audit_tx,
            // audit_rx,
            instruments,
            // state,
        })
    }

    // Start the robot, running its core engine
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize channels for communication
        // let (feed_tx, mut feed_rx) = mpsc_unbounded();
        let (audit_tx, audit_rx) = mpsc_unbounded();


        // Initialise MarketData Stream & forward to Engine feed
        let market_stream =
            init_indexed_multi_exchange_market_stream(&self.instruments, &[SubKind::KLines(1)]).await?;
        tokio::spawn(market_stream.forward_to(self.feed_tx.clone()));

        // Construct Engine clock
        let clock = LiveClock;

        // Construct EngineState from IndexedInstruments and hard-coded exchange asset Balances
        let state = EngineState::<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>::builder(&self.instruments)
            .time_engine_start(clock.time())
            .trading_state(TradingState::Enabled)
            .balances([
                (EXCHANGE, "btc", STARTING_BALANCE_BTC),
                // You can add more balances if needed.
            ])
            .build();

        // Generate initial AccountSnapshot from EngineState for BinanceSpot MockExchange
        // Note: for live-trading this would be automatically fetched via the AccountStream init
        let mut initial_account = FnvHashMap::from(&state);
        assert_eq!(initial_account.len(), 1);

        // Initialise ExecutionManager & forward Account Streams to Engine feed
        let (execution_txs, account_stream) = ExecutionBuilder::new(&self.instruments)
            .add_mock(MockExecutionConfig::new(
                EXCHANGE,
                initial_account.remove(&EXCHANGE).unwrap(),
                MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS,
                MOCK_EXCHANGE_FEES_PERCENT,
            ))?
            .init()
            .await?;
        tokio::spawn(account_stream.forward_to(self.feed_tx.clone()));

        // Initialize Engine
        let mut engine = Engine::new(
            LiveClock,
            state,
            execution_txs,
            TrendStrategy::default(),
            DefaultRiskManager::default(),
        );

        // Start Engine and handle events
        let shutdown_audit = run(&mut self.feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx));
        let engine_task = (engine, shutdown_audit);

        // Run dummy asynchronous AuditStream consumer
        let audit_task = tokio::spawn(async move {
            let mut audit_stream = audit_rx.into_stream();
            while let Some(audit) = audit_stream.next().await {
                info!(?audit, "AuditStream consumed AuditTick");
                if let EngineAudit::Shutdown(_) = audit.event {
                    break;
                }
            }
        });

        // Wait for the engine to perform tasks (e.g., sleep for a while)
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;

        // Wait for the tasks to finish gracefully
        let (engine, _shutdown_audit) = engine_task; //engine_task.await?;
        let _audit_stream = audit_task.await?;

        Ok(())
    }



    // Send a general command to the engine (cancel orders, close positions, etc.)
    // pub fn send_command(&mut self, command: T) -> Result<(), Box<dyn std::error::Error>> {
    //     self.feed_tx.send(command)?;
    //     Ok(())
    // }

    // Shutdown the engine gracefully
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // self.send_command(EngineEvent::Shutdown)?;
        Ok(())
    }

    // Run dummy asynchronous AuditStream consumer
    // Note: you probably want to use this Stream to replicate EngineState, or persist events, etc.
    //  --> eg/ see examples/engine_with_replica_engine_state.rs
    // pub fn start_audit_stream(&mut self) {
    //     let mut audit_stream = self.audit_rx.into_stream();
    //     tokio::spawn(async move {
    //         while let Some(audit) = audit_stream.next().await {
    //             info!(?audit, "AuditStream consumed AuditTick");
    //             if let EngineAudit::Shutdown(_) = audit.event {
    //                 info!(?audit, "AuditStream consumed AuditTick shutdown");
    //                 break;
    //             }
    //         }
    //     });
    // }

    // Get indexed instruments
    fn indexed_instruments() -> IndexedInstruments {
        IndexedInstruments::builder()
            .add_instrument(Instrument::new(
                ExchangeId::BinanceFuturesUsd,
                "binance_perpetual_btc_usdt",
                "BTCUSDT",
                Underlying::new("btc", "usdt"),
                InstrumentKind::Perpetual {
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
