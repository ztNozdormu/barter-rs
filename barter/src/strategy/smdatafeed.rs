/*
策略市场数据业务 TODO
*/

use barter_data::{exchange::binance::model::KlineSummary, subscription::{candle::Candle, tiker::Tiker}};
use std::sync::{Arc, Mutex};
use tokio::{
    task,
    time::{sleep, Duration},
};

// CandleManager 负责管理不同时间周期的蜡烛
#[derive(Clone, Debug)]
pub struct CandleManager {
    // TODO 需要改造适应多周期多币种集合
    candles: Arc<Mutex<Vec<KlineSummary>>>,
    current_candle: Arc<Mutex<Option<KlineSummary>>>,
}

impl CandleManager {
    pub fn init() -> Self {
        // 从市场实时获取 TODO
        //  let market_candles = self.candles.lock().unwrap();
        Self {
            candles: Arc::new(Mutex::new(vec![])), // 获取市场实时数据 一般大多数据策略获取实时接口即可 如果要更多历史数据需要扩展该方法
            current_candle: Arc::new(Mutex::new(None)),
        }
    }

    async fn update_candle(&self, tiker: Tiker) {
        let mut current = self.current_candle.lock().unwrap();
        if let Some(candle) = current.as_mut() {
            candle.update(tiker);
        } else {
            *current = Some(KlineSummary::new(tiker));
        }
    }

    async fn finalize_candle(&self) {
        let mut candles = self.candles.lock().unwrap();
        let mut current = self.current_candle.lock().unwrap();

        if let Some(candle) = current.take() {
            candles.push(candle);
        }
    }

    fn print_candles(&self) {
        let candles = self.candles.lock().unwrap();
        for candle in candles.iter() {
            println!(
                "Timestamp: {}, Open: {:.2}, Close: {:.2}, High: {:.2}, Low: {:.2}, Volume: {:.2}",
                candle.close_time,
                candle.open,
                candle.close,
                candle.high,
                candle.low,
                candle.volume
            );
        }
    }
}

// ticker_event 用于模拟随机价格和交易量的生成
async fn ticker_event(candle_manager: Arc<CandleManager>, interval: u64) {
    // let mut rng = rand::thread_rng();
    loop {
        // let price = Tiker{...};
        // let volume = rng.gen_range(1.0..10.0);
        // candle_manager.update_candle(price, volume).await;
        sleep(Duration::from_secs(interval)).await;
    }
}

// finalize_candles 按指定周期完成当前蜡烛
async fn finalize_candles(candle_manager: Arc<CandleManager>, duration: u64) {
    loop {
        sleep(Duration::from_secs(duration)).await;
        candle_manager.finalize_candle().await;
    }
}

#[tokio::main]
async fn main() {
    let candle_manager = Arc::new(CandleManager::init());

    // 启动 ticker 事件线程
    let cm_clone = Arc::clone(&candle_manager);
    task::spawn(async move {
        ticker_event(cm_clone, 5).await; // 每 5 秒生成一次价格
    });

    // 启动 finalize_candles 线程
    let cm_clone = Arc::clone(&candle_manager);
    task::spawn(async move {
        finalize_candles(cm_clone, 60).await; // 每 60 秒完成一次蜡烛
    });

    // 主线程定期打印蜡烛数据
    loop {
        sleep(Duration::from_secs(60)).await;
        candle_manager.print_candles();
    }
}
