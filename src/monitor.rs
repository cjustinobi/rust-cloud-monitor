use crate::config::Target;
use prometheus::{IntCounterVec, HistogramVec, Encoder, TextEncoder, register_int_counter_vec, register_histogram_vec};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

pub struct Monitor {
    client: Client,
    request_count: IntCounterVec,
    latency_histogram: HistogramVec,
    targets: Vec<Target>,
}

impl Monitor {
    pub fn new(targets: Vec<Target>) -> Self {
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
            targets,
        }
    }

    pub async fn run(&self) {
        for target in &self.targets {
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