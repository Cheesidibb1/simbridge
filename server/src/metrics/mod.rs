// Performance metrics collection

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, MetricData>>>,
}

#[derive(Debug, Clone)]
struct MetricData {
    name: String,
    value: f64,
    timestamp: DateTime<Utc>,
    metric_type: MetricType,
}

#[derive(Debug, Clone, Copy)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(name.to_string())
            .or_insert_with(|| MetricData {
                name: name.to_string(),
                value: 0.0,
                timestamp: Utc::now(),
                metric_type: MetricType::Counter,
            });
        entry.value += value;
        entry.timestamp = Utc::now();
    }

    /// Set a gauge metric
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(
            name.to_string(),
            MetricData {
                name: name.to_string(),
                value,
                timestamp: Utc::now(),
                metric_type: MetricType::Gauge,
            },
        );
    }

    /// Record a histogram value
    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics
            .entry(name.to_string())
            .or_insert_with(|| MetricData {
                name: name.to_string(),
                value: 0.0,
                timestamp: Utc::now(),
                metric_type: MetricType::Histogram,
            });
        entry.value += value;
        entry.timestamp = Utc::now();
    }

    /// Get a metric
    pub async fn get_metric(&self, name: &str) -> Option<MetricData> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Vec<MetricData> {
        let metrics = self.metrics.read().await;
        metrics.values().cloned().collect()
    }

    /// Clear all metrics
    pub async fn clear(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
