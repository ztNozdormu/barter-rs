
use barter::{
    EngineEvent,
    engine::{
        Engine, Processor,
        audit::EngineAudit,
        clock::LiveClock,
        state::{
            EngineState,
            global::DefaultGlobalData,
            instrument::{
                data::{DefaultInstrumentMarketData, InstrumentDataState},
                filter::InstrumentFilter,
            },
            order::in_flight_recorder::InFlightRequestRecorder,
            position::PositionManager,
            trading::TradingState,
        },
    },
    logging::init_logging,
    risk::DefaultRiskManager,
    statistic::{summary::instrument::TearSheetGenerator, time::Daily},
    strategy::{
        DefaultStrategy,
        algo::AlgoStrategy,
        close_positions::{ClosePositionsStrategy, build_ioc_market_order_to_close_position},
        on_disconnect::OnDisconnectStrategy,
        on_trading_disabled::OnTradingDisabled,
    },
    system::{
        builder::{AuditMode, EngineFeedMode, SystemArgs, SystemBuilder},
        config::SystemConfig,
    },
};
use barter_data::{
    event::{DataKind, MarketEvent},
    streams::builder::dynamic::indexed::init_indexed_multi_exchange_market_stream,
    subscription::SubKind,
};
use barter_execution::{
    AccountEvent, AccountEventKind,
    order::{
        id::{ClientOrderId, StrategyId},
        request::{OrderRequestCancel, OrderRequestOpen},
    },
};
use barter_instrument::{asset::AssetIndex, exchange::{ExchangeId, ExchangeIndex}, index::IndexedInstruments, instrument::InstrumentIndex, Underlying};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use smol_str::SmolStr;
use std::{fs::File, io::BufReader, time::Duration};
use tracing::{debug, info};
use barter_instrument::asset::{Asset, QuoteAsset};
use barter_instrument::instrument::Instrument;
use barter_instrument::instrument::kind::InstrumentKind;
use barter_instrument::instrument::kind::perpetual::PerpetualContract;
use barter_instrument::instrument::quote::InstrumentQuoteAsset;
use barter_instrument::instrument::spec::{InstrumentSpec, InstrumentSpecNotional, InstrumentSpecPrice, InstrumentSpecQuantity, OrderQuantityUnits};
use crate::bot::data::trend_data::TrendStrategyInstrumentData;
use crate::bot::strategy::trendst::TrendStrategy;

const FILE_PATH_SYSTEM_CONFIG: &str = "barter-engine/config/system_config.json";
const RISK_FREE_RETURN: Decimal = dec!(0.05);

#[tokio::main]
pub(crate) async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise Tracing
    init_logging();


    // Load SystemConfig
    let SystemConfig {
        instruments,
        executions,
    } = load_config()?;

    // Construct IndexedInstruments
    let instruments = IndexedInstruments::new(instruments);


    // Initialise MarketData Stream
    let market_stream = init_indexed_multi_exchange_market_stream(
        &instruments,
        &[SubKind::KLines(1)],
    )
        .await?;

    // Construct System Args
    let args = SystemArgs::new(
        &instruments,
        executions,
        LiveClock,
        TrendStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
        DefaultGlobalData::default(),
        || TrendStrategyInstrumentData::init(Utc::now()),
    );

    // Build & run System:
    // See SystemBuilder for all configuration options
    let mut system = SystemBuilder::new(args)
        // Engine feed in Sync mode (Iterator input)
        .engine_feed_mode(EngineFeedMode::Iterator)
        // Audit feed is enabled (Engine sends audits)
        .audit_mode(AuditMode::Enabled)
        // Engine starts with TradingState::Disabled
        .trading_state(TradingState::Disabled)
        // Build System, but don't start spawning tasks yet
        .build::<EngineEvent, _>()?
        // Init System, spawning component tasks on the current runtime
        .init_with_runtime(tokio::runtime::Handle::current())
        .await?;

    // Take ownership of Engine audit receiver
    let audit_rx = system.audit_rx.take().unwrap();

    // Run dummy asynchronous AuditStream consumer
    // Note: you probably want to use this Stream to replicate EngineState, or persist events, etc.
    //  --> eg/ see examples/engine_sync_with_audit_replica_engine_state
    let audit_task = tokio::spawn(async move {
        let mut audit_stream = audit_rx.into_stream();
        while let Some(audit) = audit_stream.next().await {
            debug!(?audit, "AuditStream consumed AuditTick");
            if let EngineAudit::Shutdown(_) = audit.event {
                break;
            }
        }
        audit_stream
    });

    // Enable trading
    system.trading_state(TradingState::Enabled);

    // Let the example run for 5 seconds...
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Before shutting down, CancelOrders and then ClosePositions
    system.cancel_orders(InstrumentFilter::None);
    system.close_positions(InstrumentFilter::None);

    // // Shutdown
    // let (engine, _shutdown_audit) = system.shutdown().await?;
    // let _audit_stream = audit_task.await?;
    //
    // // Generate TradingSummary<Daily>
    // let trading_summary = engine
    //     .trading_summary_generator(RISK_FREE_RETURN)
    //     .generate(Daily);
    //
    // // Print TradingSummary<Daily> to terminal (could save in a file, send somewhere, etc.)
    // trading_summary.print_summary();

    Ok(())
}

fn load_config() -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let file = File::open(FILE_PATH_SYSTEM_CONFIG)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn gen_config_json() -> Result<String, Box<dyn std::error::Error>> {
    let instrumentss = indexed_instruments();
    info!("IndexedInstruments: {}", serde_json::to_string_pretty(&instrumentss).unwrap_or_else(|e| format!("Error serializing: {}", e)));
}
    // Get indexed instruments
    fn indexed_instruments() -> IndexedInstruments {
        // instruments
        IndexedInstruments::builder()
            .add_instrument(Instrument::new(
                ExchangeId::BinanceFuturesUsd,
                "binance_perpetual_btc_usdt",
                "BTCUSDT",
                Underlying::new("btc", "usdt"),
                InstrumentQuoteAsset::UnderlyingQuote,
                InstrumentKind::Perpetual (PerpetualContract { contract_size: Default::default(), settlement_asset: Asset::from("btc") }),
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
            .add_instrument(Instrument::new(
                ExchangeId::BinanceFuturesUsd,
                "binance_perpetual_eth_usdt",
                "ETHUSDT",
                Underlying::new("eth", "usdt"),
                InstrumentQuoteAsset::UnderlyingQuote,
                InstrumentKind::Perpetual (PerpetualContract { contract_size: Default::default(), settlement_asset: Asset::from("eth") }),
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
            .add_instrument(Instrument::new(
                ExchangeId::BinanceFuturesUsd,
                "binance_perpetual_sol_usdt",
                "SOLUSDT",
                Underlying::new("sol", "usdt"),
                InstrumentQuoteAsset::UnderlyingQuote,
                InstrumentKind::Perpetual (PerpetualContract { contract_size: Default::default(), settlement_asset: Asset::from("sol") }),
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