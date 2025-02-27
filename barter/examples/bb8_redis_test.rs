use bb8_redis::bb8;
use bb8_redis::redis::cmd;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = bb8_redis::RedisConnectionManager::new("192.168.1.248:6379");
    let pool = bb8::Pool::builder()
        .max_size(15)
        .build(manager)
        .await
        .unwrap();

    for _ in 0..20 {
        let pool = pool.clone();
        tokio::spawn(async move {
            {
                let mut conn = pool.get().await.unwrap();
                // set value
                cmd("SET")
                    .arg(&["bb8/test_key", "42"])
                    .query_async::<()>(&mut conn)
                    .await.unwrap();
            }
            {
                let mut conn = pool.get().await.unwrap();
                // Get the value from Redis
                let value: String = cmd("GET")
                    .arg(&["bb8/test_key"])
                    .query_async(&mut conn)
                    .await.unwrap();

                info!("bb8/test_key key_{}: {}", value);
            }


        });
    }

    Ok(())
}