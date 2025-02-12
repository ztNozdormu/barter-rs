use barter::{
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
use barter_integration::channel::{mpsc_unbounded, ChannelTxDroppable, Tx};
use fnv::FnvHashMap;
use futures::StreamExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise Tracing
    init_logging();

    // Initialise Channels
    let (feed_tx, mut feed_rx) = mpsc_unbounded();
    let (audit_tx, audit_rx) = mpsc_unbounded();

    // Construct IndexedInstruments
    let instruments = indexed_instruments();

    // Initialise MarketData Stream & forward to Engine feed
    let market_stream =
        init_indexed_multi_exchange_market_stream(&instruments, &[SubKind::KLines(1)]).await?;
    tokio::spawn(market_stream.forward_to(feed_tx.clone()));

    // Construct Engine clock
    let clock = LiveClock;

    // Construct EngineState from IndexedInstruments and hard-coded exchange asset Balances
    let state =
        EngineState::<FeedMarketData, TrendStrategyState, DefaultRiskManagerState>::builder(
            &instruments,
        )
        .time_engine_start(clock.time())
        // Note: you may want to start to engine with TradingState::Disabled and turn on later
        .trading_state(TradingState::Enabled)
        .balances([
            (EXCHANGE, "btc", STARTING_BALANCE_BTC),
            // (EXCHANGE, "eth", STARTING_BALANCE_ETH),
            // (EXCHANGE, "sol", STARTING_BALANCE_SOL),
        ])
        // Note: can add other initial data via this builder (eg/ exchange asset balances)
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
    let mut engine = Engine::new(
        clock,
        state,
        execution_txs,
        TrendStrategy::default(),
        DefaultRiskManager::default(),
    );

    // Run synchronous Engine on blocking task
    let engine_task = tokio::task::spawn_blocking(move || {
        let shutdown_audit = run(
            &mut feed_rx,
            &mut engine,
            &mut ChannelTxDroppable::new(audit_tx),
        );
        (engine, shutdown_audit)
    });

    // Run dummy asynchronous AuditStream consumer
    // Note: you probably want to use this Stream to replicate EngineState, or persist events, etc.
    //  --> eg/ see examples/engine_with_replica_engine_state.rs
    /**
       * event examples---
       * 2025-02-12T07:42:52.828873Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick audit=AuditTick { event: Process(Process(Account(Item(AccountEvent { exchange: ExchangeIndex(0), kind: Sn
       * apshot(AccountSnapshot { exchange: ExchangeIndex(0), balances: [AssetBalance { asset: AssetIndex(0), balance: Balance { total: 0.1, free: 0.1 }, time_exchange: 2025-02-12T07:41:50.645351800Z }, AssetBalance { asset: AssetIn
       * dex(1), balance: Balance { total: 0, free: 0 }, time_exchange: 2025-02-12T07:41:50.645351800Z }], instruments: [] }) })))), context: EngineContext { sequence: Sequence(1), time: 2025-02-12T07:42:52.827023300Z } }
       * 2025-02-12T07:42:52.834185Z  INFO barter::engine: Engine actioning user Command::CancelOrders filter=None
       * 2025-02-12T07:43:28.138053Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick audit=AuditTick { event: Process(ProcessWithOutput(TradingStateUpdate(Disabled), One(OnTradingDisabled(()
       * )))), context: EngineContext { sequence: Sequence(2), time: 2025-02-12T07:42:52.834079600Z } }
       * 2025-02-12T07:43:28.138729Z  INFO barter::engine: Engine actioning user Command::ClosePositions filter=None
       * 2025-02-12T07:43:32.249377Z  INFO barter::engine: Engine shutting down shutdown_audit=Commanded(Shutdown)
       * 2025-02-12T07:43:32.249015Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick audit=AuditTick { event: Process(ProcessWithOutput(Command(CancelOrders(None)), One(Commanded(CancelOrder
       *  s(SendRequestsOutput { sent: None, errors: None }))))), context: EngineContext { sequence: Sequence(3), time: 2025-02-12T07:43:28.138703600Z } }
       * 2025-02-12T07:43:33.171938Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick audit=AuditTick { event: Process(ProcessWithOutput(Command(ClosePositions(None)), One(Commanded(ClosePosi
       * tions(SendCancelsAndOpensOutput { cancels: SendRequestsOutput { sent: None, errors: None }, opens: SendRequestsOutput { sent: None, errors: None } }))))), context: EngineContext { sequence: Sequence(4), time: 2025-02-12T07:
       * 43:32.249344700Z } }
       * 2025-02-12T07:43:42.256996Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick audit=AuditTick { event: Shutdown(Commanded(Shutdown)), context: EngineContext { sequence: Sequence(5), t
       * ime: 2025-02-12T07:43:32.249353800Z } }
       * 2025-02-12T07:43:42.261016Z  INFO engine_with_live_market_data_and_strategy_execution: AuditStream consumed AuditTick shutdown audit=AuditTick { event: Shutdown(Commanded(Shutdown)), context: EngineContext { sequence: Seque
       * nce(5), time: 2025-02-12T07:43:32.249353800Z } }
       *
       *
       */

    let audit_task = tokio::spawn(async move {
        let mut audit_stream = audit_rx.into_stream();
        while let Some(audit) = audit_stream.next().await {
            info!(?audit, "AuditStream consumed AuditTick");
            if let EngineAudit::Shutdown(_) = audit.event {
                info!(?audit, "AuditStream consumed AuditTick shutdown");
                break;
            }
        }
        audit_stream
    });

    // Let the example run for 4 seconds..., then:
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    // 1. Disable Strategy order generation (still continues to update EngineState)
    feed_tx.send(TradingState::Disabled)?;
    // // 2. Cancel all open orders
    feed_tx.send(Command::CancelOrders(InstrumentFilter::None))?;
    // // 3. Send orders to close current positions
    feed_tx.send(Command::ClosePositions(InstrumentFilter::None))?;
    // 4. Stop Engine run loop
    feed_tx.send(EngineEvent::Shutdown)?;
    // feed_tx.send(EngineEvent::TradingStateUpdate(TradingState::Enabled))?;
    // Await Engine & AuditStream task graceful shutdown
    // Note: Engine & AuditStream returned, ready for further use
    let (engine, _shutdown_audit) = engine_task.await?;
    let _audit_stream = audit_task.await?;

    // Generate TradingSummary<Daily>
    // let trading_summary = engine
    //     .trading_summary_generator(RISK_FREE_RETURN)
    //     .generate(Daily);
    //
    // // Print TradingSummary<Daily> to terminal (could save in a file, send somewhere, etc.)
    // trading_summary.print_summary();

    Ok(())
}

fn indexed_instruments() -> IndexedInstruments {
    IndexedInstruments::builder()
        .add_instrument(Instrument::new(
            EXCHANGE,
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
        // .add_instrument(Instrument::new(
        //     EXCHANGE,
        //     "binance_spot_eth_usdt",
        //     "ETHUSDT",
        //     Underlying::new("eth", "usdt"),
        //     InstrumentKind::Spot,
        //     Some(InstrumentSpec::new(
        //         InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
        //         InstrumentSpecQuantity::new(OrderQuantityUnits::Quote, dec!(0.0001), dec!(0.0001)),
        //         InstrumentSpecNotional::new(dec!(5.0)),
        //     )),
        // ))
        // .add_instrument(Instrument::new(
        //     EXCHANGE,
        //     "binance_spot_sol_usdt",
        //     "SOLUSDT",
        //     Underlying::new("sol", "usdt"),
        //     InstrumentKind::Spot,
        //     Some(InstrumentSpec::new(
        //         InstrumentSpecPrice::new(dec!(0.01), dec!(0.01)),
        //         InstrumentSpecQuantity::new(OrderQuantityUnits::Quote, dec!(0.001), dec!(0.001)),
        //         InstrumentSpecNotional::new(dec!(5.0)),
        //     )),
        // ))
        .build()
}
