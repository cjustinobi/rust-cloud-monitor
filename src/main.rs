mod config;
mod monitor;

use crate::config::Config;
use crate::monitor::Monitor;
use tokio::net::TcpListener;
use axum::{routing::get, Router};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Create monitor with targets from config
    let monitor = Arc::new(Monitor::new(config.targets.clone()));

    let m = monitor.clone();
    tokio::spawn(async move {
        m.run().await;
    });

    // Create API server to expose /metrics
    let app = Router::new().route("/metrics", get({
        let monitor = monitor.clone();
        move || async move {
            monitor.gather_metrics()
        }
    }));

    let listener = TcpListener::bind(&config.addr).await?;
    println!("🚀 Metrics server running on http://{}", config.addr);

    axum::serve(listener, app).await?;
    Ok(())
}