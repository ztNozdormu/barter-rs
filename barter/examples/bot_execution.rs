use barter_integration::channel::Tx;
use barter::bot::TradingRobot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let mut robot = TradingRobot::new().await?;
    //
    // robot.start()?;
    //
    // // Let the robot run for 4 seconds before issuing commands
    // tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    //
    // // Send commands to the robot
    // robot.send_command(Command::CancelOrders(InstrumentFilter::None))?;
    // robot.send_command(Command::ClosePositions(InstrumentFilter::None))?;
    //
    // // Stop the robot
    // robot.stop()?;

    // let execution_tx_map = MultiExchangeTxMap::<UnboundedTx<ExecutionRequest>>::new();
    // let strategy = TrendStrategy::default();
    // let risk_manager = DefaultRiskManagerState::default();

    let mut robot = TradingRobot::new().await?;
    robot.run().await?;
    Ok(())
}
