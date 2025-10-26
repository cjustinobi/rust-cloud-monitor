use crate::config::Target;
use prometheus::{IntCounterVec, HistogramVec, Encoder, TextEncoder, register_int_counter_vec, register_histogram_vec};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use std::sync::Arc;
use std::fs;
use serde_json::to_string_pretty;
use anyhow::{Result, Context};
use std::path::PathBuf;

pub struct Monitor {
    client: Client,
    request_count: IntCounterVec,
    latency_histogram: HistogramVec,
    pub targets: Arc<RwLock<Vec<Target>>>,
    pub file_path: PathBuf,
}

impl Monitor {
    pub fn new(targets: Vec<Target>, file_path: impl Into<PathBuf>) -> Self {
        Self {
            client: Client::new(),
            request_count: register_int_counter_vec!(
                "monitor_requests_total",
                "Total number of requests made",
                &["target"]
            ).unwrap(),
            latency_histogram: register_histogram_vec!(
                "monitor_latency_seconds",
                "Request latency in seconds",
                &["target"]
            ).unwrap(),
            targets: Arc::new(RwLock::new(targets)),
            file_path: file_path.into(),
        }
    }

    pub async fn get_targets(&self) -> Vec<Target> {
        self.targets.read().await.clone()
    }

    pub async fn add_target(&self, target: Target) -> Result<()> {
        let mut targets = self.targets.write().await;

        if targets.iter().any(|t| t.name == target.name) {
            anyhow::bail!("Target '{}' already exists", target.name);
        }

        targets.push(target);

        // Persist targets to JSON file
        let json = to_string_pretty(&*targets)
            .context("Failed to serialize targets to JSON")?;

        let file_path: PathBuf = self.file_path.clone();
        fs::write(&file_path, json)
            .context(format!("Failed to write targets to file {:?}", file_path))?;

        Ok(())
    }

    pub async fn remove_target(&self, name: &str) -> Result<()> {
        let mut targets = self.targets.write().await;

        if let Some(pos) = targets.iter().position(|t| t.name == name) {
            targets.remove(pos);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Target '{}' not found", name))
        }
    }

    pub async fn run(&self) {
        let targets = self.targets.read().await.clone();
        for target in targets {
            let client = self.client.clone();
            let counter = self.request_count.clone();
            let hist = self.latency_histogram.clone();
            let target = target.clone();

            tokio::spawn(async move {
                loop {
                    let start = std::time::Instant::now();
                    let res = client.get(&target.url).send().await;

                    counter.with_label_values(&[&target.name]).inc();

                    match res {
                        Ok(_resp) => {
                            let elapsed = start.elapsed().as_secs_f64();
                            hist.with_label_values(&[&target.name]).observe(elapsed);

                            info!(
                                "[{}] ✅ {} responded in {:.3}s",
                                target.name,
                                target.url,
                                elapsed
                            );

                            if elapsed * 1000.0 > target.alert_threshold_ms as f64 {
                                warn!(
                                    "[{}] ⚠️ High latency {:.0}ms > {}ms",
                                    target.name,
                                    elapsed * 1000.0,
                                    target.alert_threshold_ms
                                );
                            }
                        }
                        Err(e) => {
                            error!("[{}] ❌ Request failed: {}", target.name, e);
                        }
                    }

                    sleep(Duration::from_secs(target.interval)).await;
                }
            });
        }
    }

    pub fn gather_metrics(&self) -> String {
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}