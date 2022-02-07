use chrono::Local;
use redis::{AsyncCommands, Client, Commands};
use std::collections::HashMap;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = redis::Client::open("redis://0.0.0.0:6379")?;
    let mut conn = client.get_async_connection().await?;

    let _ = conn.set("my_key", "my_value").await?;
    let result: String = conn.get("my_key").await?;
    let _ = conn.del("my_key").await?;

    println!("{result:?}");

    async_without_connection_pool_example().await?;

    sync_with_connection_pool_example().await?;

    Ok(())
}

async fn async_without_connection_pool_example() -> anyhow::Result<()> {
    let client = redis::Client::open("redis://0.0.0.0:6379")?;
    let mut conn = client.get_async_connection().await?;
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
    let _ = conn.del("hkey").await?;

    println!("{result:?}");
    Ok(())
}

async fn sync_with_connection_pool_example() -> anyhow::Result<()> {
    let client = Client::open("redis://0.0.0.0:6379")?;
    let pool = r2d2::Pool::builder().build(client)?;

    let pool = pool.clone();
    let handler: JoinHandle<anyhow::Result<()>> = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        let _ = conn.hset_multiple(
            "sync",
            &[
                ("module_id", "hogehogehoge"),
                ("module_name", "hoge_module.mod"),
                ("created_at", &Local::now().to_rfc3339()),
            ],
        )?;
        let result: HashMap<String, String> = conn.hgetall("sync")?;
        let _ = conn.del("sync")?;
        println!("{result:?}");
        Ok(())
    });
    handler.await??;
    Ok(())
}
