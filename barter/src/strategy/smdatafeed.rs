/*
策略市场数据业务 TODO
*/

use barter_data::{exchange::binance::model::KlineSummary, subscription::tiker::Tiker};
use std::sync::{Arc, Mutex};
use tokio::{
    task,
    time::{sleep, Duration},
};

// KlineSummaryManager 负责管理不同时间周期的蜡烛
#[derive(Clone, Debug)]
pub struct KlineSummaryManager {
    // TODO 需要改造适应多周期多币种集合
    KlineSummarys: Arc<Mutex<Vec<KlineSummary>>>,
    current_KlineSummary: Arc<Mutex<Option<KlineSummary>>>,
}

impl KlineSummaryManager {
    pub fn init() -> Self {
        // 从市场实时获取 TODO
        //  let market_KlineSummarys = self.KlineSummarys.lock().unwrap();
        Self {
            KlineSummarys: Arc::new(Mutex::new(vec![])), // 获取市场实时数据 一般大多数据策略获取实时接口即可 如果要更多历史数据需要扩展该方法
            current_KlineSummary: Arc::new(Mutex::new(None)),
        }
    }

    async fn update_KlineSummary(&self, tiker: Tiker) {
        let mut current = self.current_KlineSummary.lock().unwrap();
        if let Some(KlineSummary) = current.as_mut() {
            KlineSummary.update(tiker);
        } else {
            *current = Some(KlineSummary::new(tiker));
        }
    }

    async fn finalize_KlineSummary(&self) {
        let mut KlineSummarys = self.KlineSummarys.lock().unwrap();
        let mut current = self.current_KlineSummary.lock().unwrap();

        if let Some(KlineSummary) = current.take() {
            KlineSummarys.push(KlineSummary);
        }
    }

    fn print_KlineSummarys(&self) {
        let KlineSummarys = self.KlineSummarys.lock().unwrap();
        for KlineSummary in KlineSummarys.iter() {
            println!(
                "Timestamp: {}, Open: {:.2}, Close: {:.2}, High: {:.2}, Low: {:.2}, Volume: {:.2}",
                KlineSummary.close_time,
                KlineSummary.open,
                KlineSummary.close,
                KlineSummary.high,
                KlineSummary.low,
                KlineSummary.volume
            );
        }
    }
}

// ticker_event 用于模拟随机价格和交易量的生成
async fn ticker_event(KlineSummary_manager: Arc<KlineSummaryManager>, interval: u64) {
    // let mut rng = rand::thread_rng();
    loop {
        // let price = Tiker{...};
        // let volume = rng.gen_range(1.0..10.0);
        // KlineSummary_manager.update_KlineSummary(price, volume).await;
        sleep(Duration::from_secs(interval)).await;
    }
}

// finalize_KlineSummarys 按指定周期完成当前蜡烛
async fn finalize_KlineSummarys(KlineSummary_manager: Arc<KlineSummaryManager>, duration: u64) {
    loop {
        sleep(Duration::from_secs(duration)).await;
        KlineSummary_manager.finalize_KlineSummary().await;
    }
}

#[tokio::main]
async fn main() {
    let KlineSummary_manager = Arc::new(KlineSummaryManager::init());

    // 启动 ticker 事件线程
    let cm_clone = Arc::clone(&KlineSummary_manager);
    task::spawn(async move {
        ticker_event(cm_clone, 5).await; // 每 5 秒生成一次价格
    });

    // 启动 finalize_KlineSummarys 线程
    let cm_clone = Arc::clone(&KlineSummary_manager);
    task::spawn(async move {
        finalize_KlineSummarys(cm_clone, 60).await; // 每 60 秒完成一次蜡烛
    });

    // 主线程定期打印蜡烛数据
    loop {
        sleep(Duration::from_secs(60)).await;
        KlineSummary_manager.print_KlineSummarys();
    }
}
