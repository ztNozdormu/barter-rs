// use crate::engine::state::{EngineState,trading::TradingState};
// use crate::strategy::trend_strategy::{TrendStrategy, TrendStrategyState};
// use crate::risk::{DefaultRiskManager, DefaultRiskManagerState, RiskManager};
// use crate::execution::builder::ExecutionBuilder;
// use fnv::FnvHashMap;
// use rust_decimal::Decimal;
// use rust_decimal_macros::dec;
// use futures::StreamExt;
// use tracing::{info};
// use barter_data::streams::builder::dynamic::indexed::init_indexed_multi_exchange_market_stream;
// use barter_data::streams::reconnect::stream::ReconnectingStream;
// use barter_data::subscription::SubKind;
// use barter_execution::balance::Balance;
// use barter_execution::client::mock::MockExecutionConfig;
// use barter_instrument::asset::Asset;
// use barter_instrument::exchange::{ExchangeId, ExchangeIndex};
// use barter_instrument::index::IndexedInstruments;
// use barter_instrument::instrument::{Instrument, InstrumentIndex};
// use barter_instrument::instrument::kind::InstrumentKind;
// use barter_instrument::instrument::spec::{InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity, OrderQuantityUnits};
// use barter_instrument::Underlying;
// use barter_integration::channel::{mpsc_unbounded, ChannelTxDroppable, Tx};
// use crate::engine::clock::{EngineClock, LiveClock};
// use crate::engine::command::Command;
// use crate::engine::{run, Engine};
// use crate::engine::audit::EngineAudit;
// use crate::engine::execution_tx::ExecutionTxMap;
// use crate::engine::state::instrument::filter::InstrumentFilter;
// use crate::engine::state::instrument::market_data::FeedMarketData;
// use crate::EngineEvent;
//
// const EXCHANGE: ExchangeId = ExchangeId::BinanceFuturesUsd;
// const STARTING_BALANCE_BTC: Balance = Balance {
//     total: dec!(0.1),
//     free: dec!(0.1),
// };
//
// const MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS: u64 = 100;
//
// const MOCK_EXCHANGE_FEES_PERCENT: Decimal = dec!(0.05);
//
// /// Generic TradingRobot struct
// pub struct TradingRobot
// where
//     // ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,  // Constraint: ExecutionTxMap for executing orders
//     // Strategy: TradingStrategy,                                    // Constraint: Strategy for trading decisions
//     // Risk: RiskManager,                                             // Constraint: RiskManager for managing trade risk
// {
//     // execution_tx_map: ExecutionTxs,  // The map of execution transmitters
//     // strategy: Strategy,              // The trading strategy
//     // risk: Risk,                      // The risk manager
// }
//
// impl TradingRobot
// where
//     // ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,
//     // Strategy: TradingStrategy,
//     // Risk: RiskManager,
// {
//     pub fn new() -> Self {
//         TradingRobot {
//             // execution_tx_map,
//             // strategy,
//             // risk,
//         }
//     }
//
//     pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         // Initialize channels for communication
//         let (feed_tx, mut feed_rx) = mpsc_unbounded();
//         let (audit_tx, audit_rx) = mpsc_unbounded();
//
//         // Construct IndexedInstruments
//         let instruments = self.indexed_instruments();
//
//         // Initialise MarketData Stream & forward to Engine feed
//         let market_stream =
//             init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await?;
//         tokio::spawn(market_stream.forward_to(feed_tx.clone()));
//
//         // Construct Engine clock
//         let clock = LiveClock;
//
//         // Construct EngineState from IndexedInstruments and hard-coded exchange asset Balances
//         let state = EngineState::<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>::builder(&instruments)
//             .time_engine_start(clock.time())
//             .trading_state(TradingState::Enabled)
//             .balances([
//                 (EXCHANGE, "btc", STARTING_BALANCE_BTC),
//                 // (EXCHANGE, "eth", STARTING_BALANCE_ETH),
//                 // (EXCHANGE, "sol", STARTING_BALANCE_SOL),
//                 // You can add more balances if needed.
//             ])
//             .build();
//
//         // Generate initial AccountSnapshot from EngineState for BinanceSpot MockExchange
//         // Note: for live-trading this would be automatically fetched via the AccountStream init
//         let mut initial_account = FnvHashMap::from(&state);
//         assert_eq!(initial_account.len(), 1);
//
//         // Initialise ExecutionManager & forward Account Streams to Engine feed
//         let (execution_txs, account_stream) = ExecutionBuilder::new(&instruments)
//             .add_mock(MockExecutionConfig::new(
//                 EXCHANGE,
//                 initial_account.remove(&EXCHANGE).unwrap(),
//                 MOCK_EXCHANGE_ROUND_TRIP_LATENCY_MS,
//                 MOCK_EXCHANGE_FEES_PERCENT,
//             ))?
//             .init()
//             .await?;
//         tokio::spawn(account_stream.forward_to(feed_tx.clone()));
//
//         // Initialize Engine
//         let mut engine = Engine::new(
//             LiveClock,
//             state,
//             execution_txs,
//             TrendStrategy::default(),
//             DefaultRiskManager::default(),
//         );
//
//         // Start Engine and handle events
//         let engine_task = tokio::task::spawn_blocking(move || {
//             let shutdown_audit = run(&mut feed_rx, &mut engine, &mut ChannelTxDroppable::new(audit_tx));
//             (engine, shutdown_audit)
//         });
//
//         // Run dummy asynchronous AuditStream consumer
//         let audit_task = tokio::spawn(async move {
//             let mut audit_stream = audit_rx.into_stream();
//             while let Some(audit) = audit_stream.next().await {
//                 info!(?audit, "AuditStream consumed AuditTick");
//                 if let EngineAudit::Shutdown(_) = audit.event {
//                     break;
//                 }
//             }
//         });
//
//         // Wait for the engine to perform tasks (e.g., sleep for a while)
//         tokio::time::sleep(std::time::Duration::from_secs(4)).await;
//         feed_tx.send(Command::CancelOrders(InstrumentFilter::None))?;
//         feed_tx.send(Command::ClosePositions(InstrumentFilter::None))?;
//         feed_tx.send(EngineEvent::Shutdown)?;
//
//         // Wait for the tasks to finish gracefully
//         let (engine, _shutdown_audit) = engine_task.await?;
//         let _audit_stream = audit_task.await?;
//
//         Ok(())
//     }
//
//     fn indexed_instruments(&mut self) -> IndexedInstruments {
//         IndexedInstruments::builder()
//             .add_instrument(Instrument::new(
//                 EXCHANGE,
//                 "binance_perpetual_btc_usdt",
//                 "BTCUSDT",
//                 Underlying::new("btc", "usdt"),
//                 InstrumentKind::Perpetual {
//                     settlement_asset: Asset::from("btc"),
//                 },
//                 Some(InstrumentSpec::new(
//                     InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
//                     InstrumentSpecQuantity::new(
//                         OrderQuantityUnits::Quote,
//                         dec!(0.00001),
//                         dec!(0.00001),
//                     ),
//                     InstrumentSpecNotional::new(dec!(5.0)),
//                 )),
//             ))
//             // .add_instrument(Instrument::new(
//             //     EXCHANGE,
//             //     "binance_spot_eth_usdt",
//             //     "ETHUSDT",
//             //     Underlying::new("eth", "usdt"),
//             //     InstrumentKind::Spot,
//             //     Some(InstrumentSpec::new(
//             //         InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
//             //         InstrumentSpecQuantity::new(OrderQuantityUnits::Quote, dec!(0.0001), dec!(0.0001)),
//             //         InstrumentSpecNotional::new(dec!(5.0)),
//             //     )),
//             // ))
//             // .add_instrument(Instrument::new(
//             //     EXCHANGE,
//             //     "binance_spot_sol_usdt",
//             //     "SOLUSDT",
//             //     Underlying::new("sol", "usdt"),
//             //     InstrumentKind::Spot,
//             //     Some(InstrumentSpec::new(
//             //         InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
//             //         InstrumentSpecQuantity::new(OrderQuantityUnits::Quote, dec!(0.001), dec!(0.001)),
//             //         InstrumentSpecNotional::new(dec!(5.0)),
//             //     )),
//             // ))
//             .build()
//     }
// }
//
