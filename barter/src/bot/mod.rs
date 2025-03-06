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
}, execution::builder::ExecutionBuilder, logging::init_logging, risk::{DefaultRiskManager, DefaultRiskManagerState}, EngineEvent};
use barter_data::{event::{DataKind, MarketEvent}, streams::{
    builder::dynamic::indexed::init_indexed_multi_exchange_market_stream,
    reconnect::stream::ReconnectingStream,
}, subscription::SubKind};
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
use std::sync::{Arc, Mutex};
use futures::StreamExt;
use tracing::info;
use crate::engine::audit::ProcessAudit::ProcessWithOutput;
use crate::engine::command::Command;
use crate::engine::state::instrument::filter::InstrumentFilter;
use crate::engine::state::instrument::market_data::MarketDataState;
use crate::execution::request::ExecutionRequest;
use std::borrow::BorrowMut;
use barter_instrument::instrument::quote::InstrumentQuoteAsset;
use crate::strategy::DefaultStrategyState;
use crate::strategy::martin_strategy::MartinStrategy;

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

/// A basic structure representing a trading robot.
#[derive(Debug)]
pub struct TradingRobot {
    feed_tx: UnboundedTx<EngineEvent<DataKind>>,
    instruments: IndexedInstruments,
}

impl TradingRobot {

    // Start the robot, running its core engine
    pub async fn launch() -> Result<Self, Box<dyn std::error::Error>> {

        // Initialise Tracing
        init_logging();
        let (execution_tx, mut execution_rx) = mpsc_unbounded();
        // Initialise data Channels
        let (feed_tx,mut feed_rx) = mpsc_unbounded();

        // Initialize event channels for communication
        let (audit_tx, audit_rx) = mpsc_unbounded();

        // Construct IndexedInstruments
        let instruments = MartinStrategy::indexed_instruments();

        // Initialise MarketData Stream & forward to Engine feed
        let market_stream =
            init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await.expect("init_indexed_multi_exchange_market_stream error");
        tokio::spawn(market_stream.forward_to(feed_tx.clone()));

        // build engine
        let mut engine = Self::build_engine(TradingState::Disabled, instruments.clone(), execution_tx);

        // Start Engine and handle events feed_tx->feed_tx->run->（engine->audit_tx）->audit_rx->处理具体事件
        let _engine_task = tokio::task::spawn_blocking(move || {
            let shutdown_audit = run(
                &mut feed_rx,
                &mut engine,
                &mut  ChannelTxDroppable::new(audit_tx),
            );
            (engine, shutdown_audit)
        });

        // Run dummy asynchronous AuditStream consumer TODO
        let _audit_task = tokio::spawn(async move {
            let mut audit_stream = audit_rx.into_stream();
            while let Some(audit) = audit_stream.next().await {
                info!(?audit, "AuditStream consumed AuditTick");
                if let EngineAudit::Shutdown(_) = audit.event {
                    // info!(?audit, "AuditStream consumed AuditTick shutdown");
                    info!("AuditStream consumed AuditTick shutdown");
                    break;
                }

                if let EngineAudit::Process(_) = audit.event {
                    // info!(?audit,"AuditStream consumed AuditTick ClosePositions");
                    info!("AuditStream consumed AuditTick ClosePositions");
                }

                if let EngineAudit::Snapshot(state) = audit.event {
                    // info!(?state,"AuditStream consumed AuditTick Snapshot State");
                    info!("AuditStream consumed AuditTick Snapshot State");
                }
            }
            audit_stream
        });

        // Wait for the engine to perform tasks (e.g., sleep for a while)
        // tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        // let (feed_rx,_engine, _shutdown_audit) = engine_task.await?;

        Ok(TradingRobot {
            feed_tx,
            instruments,
        })
    }

    /// build engine
    fn build_engine(
        trading_state: TradingState,
        instruments: IndexedInstruments,
        execution_tx: UnboundedTx<ExecutionRequest>,
    ) -> Engine<
        LiveClock,
        EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
        MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
        MartinStrategy,
        DefaultRiskManager<
            EngineState<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>,
        >,
    > {

        let clock = LiveClock;

        let state =
            EngineState::<FeedMarketData, DefaultStrategyState, DefaultRiskManagerState>::builder(
                &instruments,
            )
                .time_engine_start(crate::strategy::martin_strategy::STARTING_TIMESTAMP)
                .trading_state(trading_state)
                .balances([
                    (ExchangeId::BinanceFuturesUsd, "usdt", STARTING_BALANCE_USDT),
                ])
                .build();

        let initial_account = FnvHashMap::from(&state);
        assert_eq!(initial_account.len(), 1);

        let execution_txs =
            MultiExchangeTxMap::from_iter([(ExchangeId::BinanceFuturesUsd, Some(execution_tx))]);

        Engine::new(
            clock,
            state,
            execution_txs,
            MartinStrategy { id: MartinStrategy::strategy_id() },
            DefaultRiskManager::default(),
        )
    }

    // Send a general command to the engine (cancel orders, close positions, etc.)
    pub async fn send_command<T>(&self, command: T) -> Result<(), Box<dyn std::error::Error>> where T: Debug + Clone + Send + Into<EngineEvent<DataKind>> {
        self.feed_tx.send(command)?;
        Ok(())
    }

    // Shutdown the engine gracefully
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(EngineEvent::Shutdown).await?;
        Ok(())
    }

    /// Disable Strategy order generation (still continues to update EngineState)
    pub async fn disable(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(TradingState::Disabled).await?;
        Ok(())
    }
    ///  Cancel all open orders
    pub async fn cancel_orders(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Command::CancelOrders(InstrumentFilter::None)).await?;
        Ok(())
    }
    /// Send orders to close current positions
    pub async fn close_position(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(Command::ClosePositions(InstrumentFilter::None)).await?;
        Ok(())
    }

}
