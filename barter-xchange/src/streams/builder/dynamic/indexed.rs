

/// Initialise an indexed [`DynamicStreams`] using batches of indexed [`Subscription`] batches.
///
/// This function:
/// 1. Generates indexed market data Subscriptions from all Instrument-SubKind combinations found
///    in the provided `IndexedInstruments` and `SubKind` slice.
/// 2. Initialise an indexed [`DynamicStreams`] .
/// 3. Combines all market hstreams into a single `Stream` via
///    [`select_all`](futures_util::stream::select_all::select_all)
/// 4. Handles recoverable errors by logging them at `warn` level.
///
/// See [`generate_indexed_market_data_subscription_batches`] for how indexed `Subscriptions` can
/// be conveniently generated from an [`IndexedInstruments`] collection.
///
/// See [`index_market_data_subscription_batches`] for how unindexed `Subscriptions` can be
/// indexed using an [`IndexedInstruments`] collection.
pub async fn init_indexed_multi_exchange_market_hwstream(
    instruments: &IndexedInstruments,
    sub_kinds: &[SubKind],
) -> Result<impl Stream<Item = MarketStreamEvent<InstrumentIndex, DataKind>>, DataError> {
    // Generate indexed market data Subscriptions
    let subscriptions = generate_indexed_market_data_subscription_batches(instruments, sub_kinds);
    
    // Initialise an indexed MarketStream via DynamicStreams
    let stream = DynamicStreams::init(subscriptions)
        .await?
        .select_all::<MarketStreamResult<InstrumentIndex, DataKind>>()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    Ok(stream)
}