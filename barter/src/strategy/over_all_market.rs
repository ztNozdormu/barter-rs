use super::{smdatafeed::CandleManager, Decision, Signal, SignalGenerator, SignalStrength};
use crate::data::MarketMeta;
use barter_data::event::{DataKind, MarketEvent};
use barter_instrument::instrument::{market_data::MarketDataInstrument, Instrument};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ta::{indicators::RelativeStrengthIndex, Next};
/**
 * TODO 市场宏观策略 多空选币 市场指数 形态选币
 */
/// Configuration for constructing a [`OverAllStrategy`] via the new() constructor method.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Config {
    pub rsi_period: usize,
}

#[derive(Clone, Debug)]
/// ewo based strategy that implements [`SignalGenerator`].
pub struct OverAllStrategy {
    rsi: RelativeStrengthIndex,
    candle_manager: CandleManager,
}

impl SignalGenerator for OverAllStrategy {
    fn generate_signal(&mut self, market: &MarketEvent<MarketDataInstrument, DataKind>) -> Option<Signal> {
        // Check if it's a MarketEvent with a candle
        let candle_close = match &market.kind {
            DataKind::Candle(candle) => candle.close,
            _ => return None,
        };

        // Calculate the next RSI value using the new MarketEvent Candle data
        let rsi = self.rsi.next(candle_close);

        // Generate advisory signals map
        let signals = OverAllStrategy::generate_signals_map(rsi);

        // If signals map is empty, return no SignalEvent
        if signals.is_empty() {
            return None;
        }

        Some(Signal {
            time: Utc::now(),
            exchange: market.exchange,
            instrument: market.instrument.clone(),
            market_meta: MarketMeta {
                close: candle_close,
                time: market.time_exchange,
            },
            signals,
        })
    }
}

impl OverAllStrategy {
    /// Constructs a new [`OverAllStrategy`] component using the provided configuration struct.
    // pub fn new(config: Config) -> Self {
    //     let rsi_indicator = RelativeStrengthIndex::new(config.rsi_period)
    //         .expect("Failed to construct RSI indicator");
    //     let candle_manager = CandleManager::init().await;
    //     Self {
    //         rsi: rsi_indicator,
    //         candle_manager,
    //     }
    // }

    /// Given the latest RSI value for a symbol, generates a map containing the [`SignalStrength`] for
    /// [`Decision`] under consideration.
    fn generate_signals_map(rsi: f64) -> HashMap<Decision, SignalStrength> {
        let mut signals = HashMap::with_capacity(4);
        if rsi < 40.0 {
            signals.insert(Decision::Long, OverAllStrategy::calculate_signal_strength());
        }
        if rsi > 60.0 {
            signals.insert(
                Decision::CloseLong,
                OverAllStrategy::calculate_signal_strength(),
            );
        }
        if rsi > 60.0 {
            signals.insert(
                Decision::Short,
                OverAllStrategy::calculate_signal_strength(),
            );
        }
        if rsi < 40.0 {
            signals.insert(
                Decision::CloseShort,
                OverAllStrategy::calculate_signal_strength(),
            );
        }
        signals
    }

    /// Calculates the [`SignalStrength`] of a particular [`Decision`].
    fn calculate_signal_strength() -> SignalStrength {
        SignalStrength(1.0)
    }
}
