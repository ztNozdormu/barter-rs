
/// Defines the [`MultiStreamBuilder`](multi::MultiStreamBuilder) API for ergonomically
/// initialising a common [`Streams<Output>`](Streams) from multiple
/// [`StreamBuilder<SubscriptionKind>`](StreamBuilder)s.
pub mod multi;

/// Defines the [`DynamicStreams`](dynamic::DynamicStreams) API for initialising an arbitrary number
/// of `MarketStream`s from the [`ExchangeId`] and [`SubKind`](crate::subscription::SubKind) enums, rather than concrete
/// types.
pub mod dynamic;
