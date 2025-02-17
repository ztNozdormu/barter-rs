use barter_integration::channel::Tx;
use barter::bot::TradingRobot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut robot = TradingRobot::new().await?;

    robot.start().await?;

    // Let the robot run for 4 seconds before issuing commands
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    // Send commands to the robot
    // robot.send_command(Command::CancelOrders(InstrumentFilter::None))?;
    // robot.send_command(Command::ClosePositions(InstrumentFilter::None))?;

    // Stop the robot
    // robot.stop().await?;

    Ok(())
}
