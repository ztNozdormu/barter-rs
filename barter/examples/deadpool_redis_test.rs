use std::env;
use std::fmt::Debug;
use deadpool_redis::{redis::Commands, Config, ConnectionAddr, ConnectionInfo, ProtocolVersion, RedisConnectionInfo, Runtime};
use deadpool_redis::redis::AsyncCommands;
use tracing::info;

#[tokio::main]
async fn main() {

    let rci = RedisConnectionInfo{
        db: 0,
        username: None,
        password: Some("knd@123456".to_string()),
        protocol: ProtocolVersion::RESP2,
    };

    let conn_info = ConnectionInfo {
        addr: ConnectionAddr::Tcp("192.168.1.248".to_string(), 6379),
        redis: rci,
    };

    let mut cfg = Config::from_connection_info(conn_info);

    // $ENV:ROCKET_REDIS = 'redis://:[password]@[host]:[port]/';
    // redis://:[password]@[host]:[port]/
    // let mut cfg = Config::from_url(env::var("redis://:knd@123456@192.168.1.248:6379/").unwrap()); 应该是密码有@关键字需要转义?
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        let _: () = conn.set("deadpool/test_key", 88).await.unwrap();

    }
    {
        let mut conn = pool.get().await.unwrap();
        let res: String = conn.get("deadpool/test_key").await.unwrap();

        // let value: String = cmd("GET")
        //     .arg(&["deadpool/test_key"])
        //     .query_async(&mut conn)
        //     .await.unwrap();
        assert_eq!(res, "88".to_string());
    }
}