use std::env;
use deadpool_redis::{redis::{cmd, FromRedisValue}, Config, ConnectionAddr, ConnectionInfo, ProtocolVersion, RedisConnectionInfo, Runtime};

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

    // let mut cfg = Config::from_url(env::var("192.168.1.248:6379").unwrap());
    let mut cfg = Config::from_connection_info(conn_info);

    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<()>(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}