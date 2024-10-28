/*
策略市场数据业务 TODO
*/
use chrono::{DateTime, Local};
use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tokio::task;

// 定义 Candle 结构体，类似于 Python 的 Candle 类
#[derive(Debug, Clone)]
struct Candle {
    timestamp: DateTime<Local>,
    open_price: f64,
    close_price: f64,
    high_price: f64,
    low_price: f64,
    volume: f64,
}

impl Candle {
    fn new(open_price: f64) -> Self {
        Self {
            timestamp: Local::now(),
            open_price,
            close_price: open_price,
            high_price: open_price,
            low_price: open_price,
            volume: 0.0,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.close_price = price;
        self.high_price = self.high_price.max(price);
        self.low_price = self.low_price.min(price);
        self.volume += volume;
    }
}

// CandleManager 负责管理不同时间周期的蜡烛
struct CandleManager {
    candles: Arc<Mutex<Vec<Candle>>>,
    current_candle: Arc<Mutex<Option<Candle>>>,
}

impl CandleManager {
    fn new() -> Self {
        Self {
            candles: Arc::new(Mutex::new(vec![])),
            current_candle: Arc::new(Mutex::new(None)),
        }
    }

    async fn update_candle(&self, price: f64, volume: f64) {
        let mut current = self.current_candle.lock().unwrap();
        if let Some(candle) = current.as_mut() {
            candle.update(price, volume);
        } else {
            *current = Some(Candle::new(price));
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
                candle.timestamp, candle.open_price, candle.close_price,
                candle.high_price, candle.low_price, candle.volume
            );
        }
    }
}

// ticker_event 用于模拟随机价格和交易量的生成
async fn ticker_event(candle_manager: Arc<CandleManager>, interval: u64) {
    let mut rng = rand::thread_rng();
    loop {
        let price = rng.gen_range(100.0..200.0);
        let volume = rng.gen_range(1.0..10.0);
        candle_manager.update_candle(price, volume).await;
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
    let candle_manager = Arc::new(CandleManager::new());

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