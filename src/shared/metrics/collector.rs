//shared/metrics/collector.rs
// メトリクス収集器
// 2025/7/8

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Metrics collector for tracking application metrics
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, u64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn increment(&self, key: &str) {
        let mut metrics = self.metrics.write();
        *metrics.entry(key.to_string()).or_insert(0) += 1;
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        let metrics = self.metrics.read();
        metrics.get(key).copied()
    }

    pub fn record_query_execution(&self, operation_name: &str, _duration: std::time::Duration) {
        self.increment(&format!("query_{}", operation_name));
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
