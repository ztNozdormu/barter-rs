use std::borrow::Cow;
use super::{futures::BinanceFuturesUsd, Binance};
use crate::subscription::kline::KLines;
use crate::{
    subscription::{
        book::{OrderBooksL1, OrderBooksL2}
        ,
        liquidation::Liquidations,
        ticker::Tickers,
        trade::PublicTrades,
        Subscription,
    },
    Identifier,
};
use serde::Serialize;

/// Type that defines how to translate a Barter [`Subscription`] into a [`Binance`]
/// channel to be subscribed to.
///
/// See docs: <https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams>
/// See docs: <https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams>
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
pub struct BinanceChannel(pub &'static str,pub &'static str);

impl BinanceChannel {

    /// [`Binance`] real-time trades channel name.
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/spot/en/#trade-streams>
    ///
    /// Note:
    /// For [`BinanceFuturesUsd`] this real-time
    /// stream is undocumented.
    ///
    /// See discord: <https://discord.com/channels/910237311332151317/923160222711812126/975712874582388757>
    pub const TRADES: Self = Self("@trade","");

    /// [`Binance`] real-time OrderBook Level1 (top of books) channel name.
    ///
    /// See docs:<https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-book-ticker-streams>
    /// See docs:<https://binance-docs.github.io/apidocs/futures/en/#individual-symbol-book-ticker-streams>
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker","");

    /// [`Binance`] OrderBook Level2 channel name (100ms delta updates).
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/spot/en/#diff-depth-stream>
    /// See docs: <https://binance-docs.github.io/apidocs/futures/en/#diff-book-depth-streams>
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms","");

    /// [`BinanceFuturesUsd`] liquidation orders channel name.
    ///
    /// See docs: <https://binance-docs.github.io/apidocs/futures/en/#liquidation-order-streams>
    pub const LIQUIDATIONS: Self = Self("@forceOrder","");

    /// [`BinanceFuturesUsd`] Ticker channel name.
    ///
    /// See docs: <https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Individual-Symbol-Ticker-Streams>
    pub const TICKERS: Self = Self("@ticker","");

    /// [`BinanceFuturesUsd`] kline channel name. <symbol>@kline_<interval>
    ///
    /// See docs: <https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Kline-Candlestick-Streams>
    /// [`BinanceFuturesUsd`] kline channel name. <symbol>@kline_<interval>@+08:00
    //
    //  See docs: https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#klinecandlestick-streams-with-timezone-offset>
    pub const KLINES: Self = Self("@kline_1m","");

}

impl<Server, Instrument> Identifier<BinanceChannel>
    for Subscription<Binance<Server>, Instrument, PublicTrades>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::TRADES
    }
}

impl<Server, Instrument> Identifier<BinanceChannel>
    for Subscription<Binance<Server>, Instrument, OrderBooksL1>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L1
    }
}

impl<Server, Instrument> Identifier<BinanceChannel>
    for Subscription<Binance<Server>, Instrument, OrderBooksL2>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L2
    }
}

impl<Instrument> Identifier<BinanceChannel>
    for Subscription<BinanceFuturesUsd, Instrument, Liquidations>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::LIQUIDATIONS
    }
}

impl<Instrument> Identifier<BinanceChannel>
    for Subscription<BinanceFuturesUsd, Instrument, Tickers>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::TICKERS
    }
}

impl<Instrument> Identifier<BinanceChannel>
for Subscription<BinanceFuturesUsd, Instrument, KLines>
{
    fn id(&self) -> BinanceChannel {
        BinanceChannel::KLINES
    }
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
        // // let concatenated: Cow<str> = if self.1.is_empty() {
        //     if self.1.is_empty() {
        //     Cow::Owned(format!("{}", self.0)).as_ref()
        //
        // } else {
        //     Cow::Owned(format!("{}_{}", self.0, self.1)).as_ref()
        // }
        // // concatenated.as_ref()
    }
}
