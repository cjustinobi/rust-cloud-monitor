use serde::Deserialize;
use std::env;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Target {
    pub name: String,
    pub url: String,
    pub interval: u64,
    pub alert_threshold_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub targets: Vec<Target>,
    pub addr: String,
}

impl Config {
    /// Create a new Config by reading from environment variables.
    /// Defaults to 127.0.0.1:8080 if not set.
    pub fn from_env() -> Result<Self> {
        // Try to get the ADDR environment variable (e.g., "0.0.0.0:8080")
        let addr = env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        
        // Load targets from config file
        let targets = Self::load_targets_from_file("config.yaml")?;
        
        Ok(Self { addr, targets })
    }

    /// Load config from a YAML file
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Helper to load just targets from file
    fn load_targets_from_file(path: &str) -> Result<Vec<Target>> {
        #[derive(Deserialize)]
        struct TargetsWrapper {
            targets: Vec<Target>,
        }
        
        let contents = std::fs::read_to_string(path)?;
        let wrapper: TargetsWrapper = serde_yaml::from_str(&contents)?;
        Ok(wrapper.targets)
    }
}