// /*
// 策略市场数据业务 TODO
// */
// use barter_data::{exchange::binance::{api::Binance, futures::market::FuturesMarket, model::{KlineSummaries, KlineSummary}}, subscription::Ticker::Ticker};
// use chrono::{DateTime, NaiveDateTime, Utc};
// use rand::Rng;
// use ta::Volume;

// use std::sync::{Arc, Mutex};
// use tokio::{
//     task,
//     time::{sleep, Duration},
// };

// // KlineSummaryManager 负责管理不同时间周期的蜡烛
// #[derive(Clone, Debug)]
// pub struct CandleManager {
//     // TODO 需要改造适应多周期多币种集合
//     candles: Arc<Mutex<Vec<KlineSummary>>>,
//     current_candle: Arc<Mutex<Option<KlineSummary>>>,
// }

// #[derive(Clone, Debug)]
// pub struct QueryParam {
//    pub symbol: String,
//    pub interval: String,
//    pub limit: Option<u16>,
//    pub start_time: Option<u64>,
//    pub end_time: Option<u64>,
// }

// impl CandleManager {
//      pub async fn init(param: QueryParam) -> Self {
//         let market: FuturesMarket = Binance::new(None, None);

//         match market.get_klines(param.symbol, param.interval, param.limit , param.start_time, param.end_time).await {
//             Ok(KlineSummaries::AllKlineSummaries(answer)) => Self {
//                 candles: Arc::new(Mutex::new(answer)), // 获取市场实时数据 一般大多数据策略获取实时接口即可 如果要更多历史数据需要扩展该方法
//                 current_candle: Arc::new(Mutex::new(None)),
//             },
//             Err(e) => Self {
//                 candles: Arc::new(Mutex::new(vec![])), // 获取市场实时数据 一般大多数据策略获取实时接口即可 如果要更多历史数据需要扩展该方法
//                 current_candle: Arc::new(Mutex::new(None)),
//             },
//         }

//     }

//     async fn update_candle(&self, Ticker: Ticker) {
//         let mut current = self.current_candle.lock().unwrap();
//         if let Some(KlineSummary) = current.as_mut() {
//             KlineSummary.update(Ticker);
//         } else {
//             *current = Some(KlineSummary::new(Ticker));
//         }
//     }

//     async fn finalize_candle(&self) {
//         let mut candles = self.candles.lock().unwrap();
//         let mut current = self.current_candle.lock().unwrap();

//         if let Some(candle) = current.take() {
//             candles.push(candle);
//         }
//     }

//     fn print_candle(&self) {
//         let candles = self.candles.lock().unwrap();
//         for candle in candles.iter() {
//             println!(
//                 "Timestamp: {}, Open: {:.2}, Close: {:.2}, High: {:.2}, Low: {:.2}, Volume: {:.2}",
//                 candle.close_time,
//                 candle.open,
//                 candle.close,
//                 candle.high,
//                 candle.low,
//                 candle.volume
//             );
//         }
//     }
// }

// // ticker_event 用于模拟随机价格和交易量的生成
// async fn ticker_event(candle_manager: Arc<CandleManager>, interval: u64) {
//     let mut rng = rand::thread_rng();
//     loop {
//         let open = rng.gen_range(70000.0..81000.0);
//         let high =  rng.gen_range(70000.0..81000.0);
//         let low =  rng.gen_range(70000.0..81000.0);
//         let close =  rng.gen_range(70000.0..81000.0);
//         let volume = rng.gen_range(100.0..500.0);
//         let Ticker = Ticker{ price_change: 0f64, price_change_percent: 0f64, weighted_avg_price: 0f64, last_qty: 0f64, open: open, high: high, low: low, last_price: close, volume: volume, quote_volume: volume, open_time: Utc::now(), close_time: Utc::now(), first_id: 0u64, last_id: 0u64, count: 0u64};
//         candle_manager.update_candle(Ticker).await;
//         sleep(Duration::from_secs(interval)).await;
//     }
// }

// // finalize_candles 按指定周期完成当前蜡烛
// async fn finalize_candle_manager(candle_manager: Arc<CandleManager>, duration: u64) {
//     loop {
//         sleep(Duration::from_secs(duration)).await;
//         candle_manager.finalize_candle().await;
//     }
// }

// #[tokio::main]
// async fn main() {
//     let param: QueryParam = QueryParam{ symbol: "btcusdt".to_string(), interval: "5m".to_string(), limit: None, start_time: None, end_time: None };
//     let candle_manager = Arc::new(CandleManager::init(param).await);

//     // 启动 ticker 事件线程
//     let cm_clone = Arc::clone(&candle_manager);
//     // task::spawn(async move {
//         ticker_event(cm_clone, 5).await; // 每 5 秒生成一次价格
//     // });

//     // 启动 finalize_candles 线程
//     let cm_clone = Arc::clone(&candle_manager);
//     task::spawn(async move {
//         finalize_candle_manager(cm_clone, 60).await; // 每 60 秒完成一次蜡烛
//     });

//     // 主线程定期打印蜡烛数据
//     loop {
//         sleep(Duration::from_secs(60)).await;
//         candle_manager.print_candle();
//     }
// }
