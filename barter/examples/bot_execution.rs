use barter::bot::TradingRobot;
use barter_integration::channel::Tx;
use std::sync::{Arc, OnceLock};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, OnceCell};
use tracing::info;

// pub static GLOBAL_ROBOT: Lazy<OnceCell<Arc<Mutex<TradingRobot>>>> = Lazy::new(|| OnceCell::new());

// pub static GLOBAL_ROBOT: OnceLock<Arc<Mutex<TradingRobot>>> = OnceLock::new();

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Use tokio to spawn the asynchronous initialization
    // let robot = TradingRobot::launch().await.expect("Cannot launch TradingRobot");

    // Wrap the robot in Arc and Mutex
    // let robot_arc = Arc::new(Mutex::new(robot));

    // Set the global variable once
    // GLOBAL_ROBOT.set(robot_arc).expect("Failed to set GLOBAL_ROBOT");

    // println!("robot global initialized successfully");

    // let robot = GLOBAL_ROBOT.get().expect("GLOBAL ROBOT not initialized").lock().await;


    // // 创建一个线程安全的 time 变量
    // let time_arc = Arc::new(Mutex::new(5));
    //
    // loop {
    //     let mut time = time_arc.lock().await;
    //     *time += 1;
    //     // info!(?time,"time 计数");
    //     // Simulated send forbid generate trade order command
    //     if *time == 100 {
    //         // robot.disable().await?;
    //         info!(?time,"time 计数;Trade order command is forbidden after time == 100");
    //     }
    //
    //     // Simulate the cancel all/specify order command
    //     if *time == 160 {
    //         robot.cancel_orders().await?;
    //         info!(?time,"time 计数;Orders have been canceled after time == 160");
    //     }
    //
    //     // Simulate sending the liquidation command
    //     if *time == 220  {
    //         robot.close_position().await?;
    //         info!(?time,"time 计数;Liquidation command sent after time == 220");
    //     }
    //
    //     //  Simulated sending robot stop command
    //     if *time == 1500 {
    //         robot.stop().await?;
    //         info!(?time,"time 计数;Robot stopped after time == 1500");
    //         break;
    //     }
    // }

    // println!("Server stopped");
    //
    // Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Get Global Robot
    let robot = robot().await;
    // let robot2 = init_robot().await;
    // 创建一个线程安全的 time 变量
    let time_arc = Arc::new(Mutex::new(5));

    loop {
        let mut time = time_arc.lock().await;
        *time += 1;
        // info!(?time,"time 计数");
    //     // Simulated send forbid generate trade order command
    //     if *time == 100 {
    //         // robot.disable().await?;
    //         info!(?time,"time 计数;Trade order command is forbidden after time == 100");
    //     }
    //
    //     // Simulate the cancel all/specify order command
    //     if *time == 160 {
    //         robot.cancel_orders().await?;
    //         info!(?time,"time 计数;Orders have been canceled after time == 160");
    //     }
    //
    //     // Simulate sending the liquidation command
    //     if *time == 220  {
    //         robot.close_position().await?;
    //         info!(?time,"time 计数;Liquidation command sent after time == 220");
    //     }
    //
        //  Simulated sending robot stop command
        if *time == 3500 {
            robot.stop().await?;
            info!(?time,"time 计数;Robot stopped after time == 3500");
            break;
        }
    }
    println!("Server stopped");

    Ok(())
}


async fn robot() -> &'static TradingRobot {
    static GLOBAL_ROBOT: Lazy<OnceCell<TradingRobot>> = Lazy::new(|| OnceCell::new());
    GLOBAL_ROBOT.get_or_init(|| async {
        let robot = TradingRobot::launch().await.expect("Cannot launch TradingRobot");
        info!("robot global initialized successfully");
        robot
    }).await
}