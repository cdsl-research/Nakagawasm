use std::{
    process::{Output, Stdio},
};

use tokio::{process::Command, signal::ctrl_c};

mod domain;
mod repository;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();

    let handle = tokio::spawn(async move {
        let child = Command::new("wasmedge")
            .args(&[
                "--dir",
                ".:../server-contents-setup/static",
                "--enable-all",
                "../wasmedge-app/target/wasm32-wasi/release/wasmedge-app.wasm",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let Output {
            status,
            stderr,
            stdout,
        } = child.wait_with_output().await.unwrap();

        let stdout = String::from_utf8(stdout).unwrap();
        let stderr = String::from_utf8(stderr).unwrap();

        println!("stdout: {stdout}");
        println!("stderr: {stderr}");
        println!("status: {status:?}");
    });

    ctrl_c().await.ok();
    handle.abort();
    handle.await.unwrap_err().is_cancelled();
}
