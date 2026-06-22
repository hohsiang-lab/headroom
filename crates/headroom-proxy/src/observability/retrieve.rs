//! Bounded Prometheus counters for local CCR retrieve endpoints.

use std::sync::OnceLock;

use prometheus::{IntCounterVec, Opts, Registry};

const METRIC_CCR_RETRIEVE_REQUESTS_TOTAL: &str = "proxy_ccr_retrieve_requests_total";
const LABEL_OUTCOME: &str = "outcome";

pub const OUTCOME_SUCCESS: &str = "success";
pub const OUTCOME_MISS: &str = "miss";
pub const OUTCOME_INVALID_REQUEST: &str = "invalid_request";
pub const OUTCOME_STATS_READ: &str = "stats_read";

pub fn retrieve_requests_counter(registry: &Registry) -> &'static IntCounterVec {
    static COUNTER: OnceLock<IntCounterVec> = OnceLock::new();
    COUNTER.get_or_init(|| {
        let opts = Opts::new(
            METRIC_CCR_RETRIEVE_REQUESTS_TOTAL,
            "Total Rust proxy CCR retrieve endpoint requests by bounded outcome.",
        );
        let counter = IntCounterVec::new(opts, &[LABEL_OUTCOME])
            .expect("proxy_ccr_retrieve_requests_total descriptor well-formed");
        registry
            .register(Box::new(counter.clone()))
            .expect("proxy_ccr_retrieve_requests_total registers exactly once");
        counter
    })
}

pub fn record_ccr_retrieve_request(outcome: &'static str) {
    retrieve_requests_counter(super::prometheus::registry())
        .with_label_values(&[outcome])
        .inc();
    tracing::debug!(
        event = "metric_recorded",
        metric = METRIC_CCR_RETRIEVE_REQUESTS_TOTAL,
        outcome = outcome,
        "incremented proxy_ccr_retrieve_requests_total"
    );
}
