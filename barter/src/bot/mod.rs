use std::fmt::Debug;
use derive_more::Constructor;
use crate::{
    engine::{
        audit::EngineAudit,
        clock::{EngineClock, LiveClock},
        command::Command,
        run,
        state::{
            instrument::{filter::InstrumentFilter, market_data::FeedMarketData},
            trading::TradingState,
            EngineState,
        },
        Engine,
    },
    execution::builder::ExecutionBuilder,
    logging::init_logging,
    risk::{DefaultRiskManager, DefaultRiskManagerState},
    statistic::time::Daily,
    strategy::trend_strategy::{TrendStrategy, TrendStrategyState},
    EngineEvent,
};
use barter_data::{
    streams::{
        builder::dynamic::indexed::init_indexed_multi_exchange_market_stream,
        reconnect::stream::ReconnectingStream,
    },
    subscription::SubKind,
};
use barter_execution::{balance::Balance, client::mock::MockExecutionConfig};
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
use barter_integration::channel::{mpsc_unbounded, Channel, ChannelTxDroppable, Tx, UnboundedRx, UnboundedTx};
use fnv::FnvHashMap;
use futures::{StreamExt};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use barter_data::event::DataKind;
use crate::engine::audit::AuditTick;
use crate::engine::audit::context::EngineContext;
use crate::engine::EngineOutput;

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


#[derive(Debug)]
pub struct TradingRobot<Clock, State, ExecutionTxs, Strategy, Risk>  where Clock: Clone, ExecutionTxs: Clone, Risk: Clone, State: Clone, Strategy: Clone {
    engine: Engine<Clock, State, ExecutionTxs, Strategy, Risk>,
    feed_tx:  UnboundedTx<EngineEvent<DataKind>>,
    feed_rx:  UnboundedRx<EngineEvent<DataKind>>,
    audit_tx: UnboundedTx<AuditTick<EngineAudit<EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
    audit_rx: UnboundedRx<AuditTick<EngineAudit<EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>, EngineEvent<DataKind>, EngineOutput<(), ()>>, EngineContext>>,
    instruments: IndexedInstruments,
    state: EngineState<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>,
}

impl<Clock, State, ExecutionTxs, Strategy, Risk> TradingRobot<Clock, State, ExecutionTxs, Strategy, Risk> where
{
    // impl<T,Clock, State, ExecutionTxs, Strategy, Risk> TradingRobot<Clock, State, ExecutionTxs, Strategy, Risk> where
    //     T: Debug + Clone + Send{
    // Initialize the TradingRobot with necessary components and configurations
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialise Tracing
        init_logging();

        // Initialise Channels
        let (feed_tx,mut feed_rx) = mpsc_unbounded();
        let (audit_tx, audit_rx) = mpsc_unbounded();

        // Construct IndexedInstruments
        let instruments = Self::indexed_instruments();

        // Initialise MarketData Stream & forward to Engine feed
        let market_stream =
            init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await?;
        tokio::spawn(market_stream.forward_to(feed_tx.clone()));

        // Construct Engine clock
        let clock = LiveClock;

        // Construct EngineState from IndexedInstruments and hard-coded exchange asset Balances
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
        let engine = Engine::new(
            clock,
            state.clone(),
            execution_txs,
            TrendStrategy::default(),
            DefaultRiskManager::default(),
        );

        Ok(TradingRobot {
            engine,
            feed_tx,
            feed_rx,
            audit_tx,
            audit_rx,
            instruments,
            state,
        })
    }

    // Start the robot, running its core engine
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Run synchronous Engine on blocking task
        // let feed_tx = self.feed_tx.clone();
        let mut engine = self.engine.clone();

        tokio::task::spawn_blocking(move || {
            let shutdown_audit = run(
                &mut self.feed_rx,
                &mut self.engine,
                &mut ChannelTxDroppable::new(self.audit_tx.clone()),
            );
            (engine, shutdown_audit)
        });

        Ok(())
    }

    // Send a general command to the engine (cancel orders, close positions, etc.)
    // pub fn send_command(&mut self, command: T) -> Result<(), Box<dyn std::error::Error>> {
    //     self.feed_tx.send(command)?;
    //     Ok(())
    // }

    // Shutdown the engine gracefully
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // self.send_command(EngineEvent::Shutdown)?;
        Ok(())
    }

    // Run dummy asynchronous AuditStream consumer
    // Note: you probably want to use this Stream to replicate EngineState, or persist events, etc.
    //  --> eg/ see examples/engine_with_replica_engine_state.rs
    pub fn start_audit_stream(&mut self) {
        let mut audit_stream = self.audit_rx.into_stream();
        tokio::spawn(async move {
            while let Some(audit) = audit_stream.next().await {
                info!(?audit, "AuditStream consumed AuditTick");
                if let EngineAudit::Shutdown(_) = audit.event {
                    info!(?audit, "AuditStream consumed AuditTick shutdown");
                    break;
                }
            }
        });
    }

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
