mod config;
mod monitor;
mod routes;

use routes::targets::{get_targets, add_target, remove_target};

use crate::config::{Config, Target};
use crate::monitor::Monitor;
use tokio::net::TcpListener;
use axum::{routing::{get, delete}, Router};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_path = PathBuf::from("targets.json");
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    let targets: Vec<Target> = if file_path.exists() {
        let data = fs::read_to_string(&file_path).await?;
        serde_json::from_str(&data).unwrap_or_else(|e| {
            error!("Failed to parse JSON: {}", e);
            Vec::new()
        })
    } else {
        info!("No existing targets.json found. Creating new one...");
        fs::write(&file_path, "[]").await?;
        Vec::new()
    };

    let monitor = Arc::new(Monitor::new(targets, file_path));

    let m = monitor.clone();
    tokio::spawn(async move {
        m.run().await;
    });

    let app = Router::new()
        .route("/metrics", get({
            let monitor = monitor.clone();
            move || async move { monitor.gather_metrics() }
        }))
        .route("/targets", get(get_targets).post(add_target))
        .route("/targets/{name}", delete(remove_target))
        .with_state(monitor.clone());

    let listener = TcpListener::bind(&config.addr).await?;
    println!("🚀 Metrics server running on http://{}", config.addr);

    axum::serve(listener, app).await?;
    Ok(())
}