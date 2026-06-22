//! Local CCR retrieval endpoints.
//!
//! These routes must never forward upstream: they resolve CCR hashes from the
//! Rust proxy's configured store and expose bounded counters for operator
//! readback.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::proxy::AppState;

const HASH_LEN: usize = 24;

#[derive(Debug, Default)]
pub struct RetrieveStats {
    success_count: AtomicU64,
    miss_count: AtomicU64,
    invalid_request_count: AtomicU64,
    stats_read_count: AtomicU64,
}

impl RetrieveStats {
    pub fn new() -> Self {
        Self::default()
    }

    fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        crate::observability::record_ccr_retrieve_request("success");
    }

    fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
        crate::observability::record_ccr_retrieve_request("miss");
    }

    fn record_invalid_request(&self) {
        self.invalid_request_count.fetch_add(1, Ordering::Relaxed);
        crate::observability::record_ccr_retrieve_request("invalid_request");
    }

    fn record_stats_read(&self) {
        self.stats_read_count.fetch_add(1, Ordering::Relaxed);
        crate::observability::record_ccr_retrieve_request("stats_read");
    }

    fn snapshot(&self) -> RetrievalCounters {
        RetrievalCounters {
            success_count: self.success_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            invalid_request_count: self.invalid_request_count.load(Ordering::Relaxed),
            stats_read_count: self.stats_read_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrieveRequest {
    #[serde(default)]
    hash: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

#[derive(Debug, Serialize)]
struct RetrieveResponse {
    hash: String,
    original_content: String,
    retrieval_count: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug, Serialize)]
struct RetrieveStatsResponse {
    store: StoreStats,
    retrievals: RetrievalCounters,
}

#[derive(Debug, Serialize)]
struct StoreStats {
    entry_count: usize,
    backend: Option<&'static str>,
    ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
struct RetrievalCounters {
    success_count: u64,
    miss_count: u64,
    invalid_request_count: u64,
    stats_read_count: u64,
}

pub async fn handle_retrieve(
    State(state): State<AppState>,
    body: Result<Json<RetrieveRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(body) => body,
        Err(error) => {
            state.retrieve_stats.record_invalid_request();
            tracing::warn!(
                event = "ccr_retrieve_invalid_request",
                reason = "invalid_json",
                error = %error,
                "invalid CCR retrieve request"
            );
            return error_response(StatusCode::BAD_REQUEST, "invalid JSON request body");
        }
    };

    let Some(hash) = req.hash.as_deref() else {
        state.retrieve_stats.record_invalid_request();
        tracing::warn!(
            event = "ccr_retrieve_invalid_request",
            reason = "missing_hash",
            query_present = req.query.is_some(),
            "invalid CCR retrieve request"
        );
        return error_response(StatusCode::BAD_REQUEST, "hash is required");
    };

    if !is_canonical_ccr_hash(hash) {
        state.retrieve_stats.record_invalid_request();
        tracing::warn!(
            event = "ccr_retrieve_invalid_request",
            reason = "invalid_hash",
            hash_len = hash.len(),
            query_present = req.query.is_some(),
            "invalid CCR retrieve hash"
        );
        return error_response(
            StatusCode::BAD_REQUEST,
            "hash must be 24 lowercase hex characters",
        );
    }

    match state.ccr_store.get(hash) {
        Some(original_content) => {
            state.retrieve_stats.record_success();
            tracing::info!(
                event = "ccr_retrieve_success",
                query_present = req.query.is_some(),
                "served CCR payload from local store"
            );
            Json(RetrieveResponse {
                hash: hash.to_string(),
                original_content,
                retrieval_count: None,
            })
            .into_response()
        }
        None => {
            state.retrieve_stats.record_miss();
            tracing::info!(
                event = "ccr_retrieve_miss",
                query_present = req.query.is_some(),
                "CCR payload missing or expired"
            );
            error_response(StatusCode::NOT_FOUND, "hash not found")
        }
    }
}

pub async fn handle_retrieve_stats(State(state): State<AppState>) -> Response {
    state.retrieve_stats.record_stats_read();
    tracing::info!(
        event = "ccr_retrieve_stats",
        entry_count = state.ccr_store.len(),
        "served CCR retrieve stats"
    );

    Json(RetrieveStatsResponse {
        store: store_stats(&state),
        retrievals: state.retrieve_stats.snapshot(),
    })
    .into_response()
}

fn error_response(status: StatusCode, error: &'static str) -> Response {
    (status, Json(ErrorResponse { error })).into_response()
}

fn is_canonical_ccr_hash(hash: &str) -> bool {
    hash.len() == HASH_LEN
        && hash
            .as_bytes()
            .iter()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn store_stats(state: &AppState) -> StoreStats {
    match &state.config.ccr_backend {
        headroom_core::ccr::backends::CcrBackendConfig::InMemory { ttl_seconds, .. } => {
            StoreStats {
                entry_count: state.ccr_store.len(),
                backend: Some("in_memory"),
                ttl_seconds: Some(*ttl_seconds),
            }
        }
        headroom_core::ccr::backends::CcrBackendConfig::Sqlite { ttl_seconds, .. } => StoreStats {
            entry_count: state.ccr_store.len(),
            backend: Some("sqlite"),
            ttl_seconds: Some(*ttl_seconds),
        },
        headroom_core::ccr::backends::CcrBackendConfig::Redis { ttl_seconds, .. } => StoreStats {
            entry_count: state.ccr_store.len(),
            backend: Some("redis"),
            ttl_seconds: Some(*ttl_seconds),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::is_canonical_ccr_hash;

    #[test]
    fn canonical_hash_validation_accepts_24_lower_hex_only() {
        assert!(is_canonical_ccr_hash("0123456789abcdefabcdef01"));
        assert!(!is_canonical_ccr_hash(""));
        assert!(!is_canonical_ccr_hash("0123456789abcdefabcdef0"));
        assert!(!is_canonical_ccr_hash("0123456789abcdefabcdef012"));
        assert!(!is_canonical_ccr_hash("0123456789ABCDEFabcdef01"));
        assert!(!is_canonical_ccr_hash("0123456789abcdefabcdeg01"));
    }
}
