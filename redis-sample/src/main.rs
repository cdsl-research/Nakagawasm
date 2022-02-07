use std::collections::HashMap;

use chrono::Local;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = redis::Client::open("redis://0.0.0.0:6379")?;
    let mut conn = client.get_async_connection().await?;

    let _ = conn.set("my_key", "my_value").await?;
    let result: String = conn.get("my_key").await?;
    let _ = conn.del("my_key").await?;

    println!("{result:?}");

    let _ = conn
        .hset_multiple(
            "hkey",
            &[
                ("module_id", "hogehogehoge"),
                ("module_name", "hoge_module.mod"),
                ("created_at", &Local::now().to_rfc3339()),
            ],
        )
        .await?;
    let result: HashMap<String, String> = conn.hgetall("hkey").await?;

    println!("{result:?}");
    Ok(())
}
