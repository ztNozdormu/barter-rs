use std::sync::{Arc, Mutex};
use crate::engine::state::{EngineState, trading::TradingState};
use crate::strategy::trend_strategy::{TrendStrategy, TrendStrategyState};
use crate::risk::{DefaultRiskManager, DefaultRiskManagerState, RiskManager};
use crate::execution::builder::ExecutionBuilder;
use fnv::FnvHashMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use futures::StreamExt;
use tracing::{info};
use barter_data::event::DataKind;
use barter_data::streams::builder::dynamic::indexed::init_indexed_multi_exchange_market_stream;
use barter_data::streams::reconnect::stream::ReconnectingStream;
use barter_data::subscription::SubKind;
use barter_execution::balance::Balance;
use barter_execution::client::mock::MockExecutionConfig;
use barter_instrument::asset::Asset;
use barter_instrument::exchange::{ExchangeId, ExchangeIndex};
use barter_instrument::index::IndexedInstruments;
use barter_instrument::instrument::{Instrument, InstrumentIndex};
use barter_instrument::instrument::kind::InstrumentKind;
use barter_instrument::instrument::spec::{InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity, OrderQuantityUnits};
use barter_instrument::Underlying;
use barter_integration::channel::{mpsc_unbounded, ChannelTxDroppable, Tx, UnboundedRx, UnboundedTx};
use crate::engine::clock::{EngineClock, LiveClock};
use crate::engine::command::Command;
use crate::engine::{run, Engine};
use crate::engine::audit::EngineAudit;
use crate::engine::execution_tx::ExecutionTxMap;
use crate::engine::state::instrument::filter::InstrumentFilter;
use crate::engine::state::instrument::market_data::FeedMarketData;
use crate::EngineEvent;
use crate::logging::init_logging;

const EXCHANGE: ExchangeId = ExchangeId::BinanceFuturesUsd;
const STARTING_BALANCE_BTC: Balance = Balance {
    total: dec!(0.1),
    free: dec!(0.1),
};

const MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS: u64 = 100;

const MOCK_EXCHANGE_FEES_PERCENT: Decimal = dec!(0.05);

/// Generic TradingRobot struct
pub struct TradingRobot
{
    feed_tx:  UnboundedTx<EngineEvent<DataKind>>,
    feed_rx:  UnboundedRx<EngineEvent<DataKind>>,
}

impl TradingRobot {

        // Initialize the TradingRobot with necessary components and configurations
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>>
    {
        // Initialise Tracing
        init_logging();
        // Initialise Channels
        let (feed_tx, mut feed_rx) = mpsc_unbounded();
        // let (audit_tx, audit_rx) = mpsc_unbounded();

        Ok(TradingRobot {
            feed_tx,
            feed_rx,
        })
    }

    // run the robot, running its core engine
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize channels for communication
        // let (feed_tx, mut feed_rx) = mpsc_unbounded();
        let (audit_tx, audit_rx) = mpsc_unbounded();

        // Construct IndexedInstruments
        let instruments = self.indexed_instruments();

        // Initialise MarketData Stream & forward to Engine feed
        let market_stream =
            init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await?;
        tokio::spawn(market_stream.forward_to(self.feed_tx.clone()));

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
        tokio::spawn(account_stream.forward_to(self.feed_tx.clone()));

        // Initialize Engine
        let mut engine = Engine::new(
            LiveClock,
            state,
            execution_txs,
            TrendStrategy::default(),
            DefaultRiskManager::default(),
        );
        let shutdown_audit = run(&mut self.feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx));
        let engine_task = (engine, shutdown_audit);
        // Start Engine and handle events
        // let feed_rx = Box::new(Arc::new(Mutex::new(&mut self.feed_rx)));  // 将 feed_rx 包装在 Arc 和 Mutex 中

        // let engine_task = tokio::task::spawn_blocking(move || {
        //     let shutdown_audit = run(&mut self.feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx));
        //     (engine, shutdown_audit)
        // });

        // let engine_task = tokio::task::spawn_blocking({
        //     let feed_rx = Arc::clone(&feed_rx);  // 克隆 Arc，以便传入闭包
        //     move || {
        //         let mut feed_rx = feed_rx.lock().unwrap();  // 锁定 feed_rx 进行修改
        //         let shutdown_audit = run(&mut feed_rx, &mut engine, &mut ChannelTxDroppable::new(&audit_tx));
        //         (engine, shutdown_audit)
        //     }
        // });

         // let engine_task = tokio::task::spawn_blocking({
         //     let feed_rx = feed_rx.clone(); // 克隆 Box
         //     move || {
         //         let mut feed_rx = feed_rx.lock().unwrap();
         //         let shutdown_audit = run(&mut *feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx)); (engine, shutdown_audit)
         //     }
         // });
        // let engine_task = tokio::task::spawn_blocking({
        //     let feed_rx = Arc::clone(&feed_rx);  // 克隆 Arc，以便传入闭包
        //     let audit_tx = audit_tx.clone();  // 如果 audit_tx 是可以克隆的，确保传递所有权
        //     move || {
        //         let mut feed_rx = feed_rx.lock().unwrap();  // 锁定 feed_rx
        //         // 解锁 feed_rx 并传递实际的类型（feed_rx 本身是 MutexGuard）
        //         let shutdown_audit = run(&mut *feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx)); // 这里使用解引用（`*feed_rx`）来获取实际的值
        //         (engine, shutdown_audit)
        //     }
        // });


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
        // feed_tx.send(Command::CancelOrders(InstrumentFilter::None))?;
        // feed_tx.send(Command::ClosePositions(InstrumentFilter::None))?;
        // feed_tx.send(EngineEvent::Shutdown)?;

        // Wait for the tasks to finish gracefully
        let (engine, _shutdown_audit) = engine_task; //engine_task.await?;
        let _audit_stream = audit_task.await?;

        Ok(())
    }


    // Get indexed instruments
    fn indexed_instruments(&mut self) -> IndexedInstruments {
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

