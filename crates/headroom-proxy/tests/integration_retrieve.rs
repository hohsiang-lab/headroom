//! Integration coverage for local Rust CCR retrieve endpoints.

mod common;

use common::start_proxy_with_state;
use headroom_core::ccr::backends::CcrBackendConfig;
use headroom_core::ccr::compute_key;
use reqwest::StatusCode;
use serde_json::{json, Value};

const DEAD_UPSTREAM: &str = "http://127.0.0.1:9";

fn in_memory_ccr(config: &mut headroom_proxy::Config) {
    config.ccr_backend = CcrBackendConfig::InMemory {
        capacity: 16,
        ttl_seconds: 300,
    };
}

#[tokio::test]
async fn retrieve_success_returns_seeded_payload_without_upstream() {
    let payload = r#"{"rows":[{"id":1},{"id":2}]}"#.to_string();
    let hash = compute_key(payload.as_bytes());
    let seed_hash = hash.clone();
    let seed_payload = payload.clone();

    let proxy = start_proxy_with_state(DEAD_UPSTREAM, in_memory_ccr, move |state| {
        state.ccr_store.put(&seed_hash, &seed_payload);
        state
    })
    .await;

    let resp = reqwest::Client::new()
        .post(format!("{}/v1/retrieve", proxy.url()))
        .json(&json!({ "hash": hash }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["hash"], json!(hash));
    assert_eq!(body["original_content"], json!(payload));
    assert!(body["retrieval_count"].is_null());

    let stats: Value = reqwest::get(format!("{}/v1/retrieve/stats", proxy.url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["store"]["entry_count"], json!(1));
    assert_eq!(stats["store"]["backend"], json!("in_memory"));
    assert_eq!(stats["store"]["ttl_seconds"], json!(300));
    assert_eq!(stats["retrievals"]["success_count"], json!(1));
    assert_eq!(stats["retrievals"]["miss_count"], json!(0));
    assert_eq!(stats["retrievals"]["invalid_request_count"], json!(0));
    assert_eq!(stats["retrievals"]["stats_read_count"], json!(1));

    proxy.shutdown().await;
}

#[tokio::test]
async fn retrieve_missing_hash_returns_404_without_upstream() {
    let proxy = start_proxy_with_state(DEAD_UPSTREAM, in_memory_ccr, |state| state).await;

    let resp = reqwest::Client::new()
        .post(format!("{}/v1/retrieve", proxy.url()))
        .json(&json!({ "hash": "0123456789abcdefabcdef01" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"], json!("hash not found"));

    let stats: Value = reqwest::get(format!("{}/v1/retrieve/stats", proxy.url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["retrievals"]["miss_count"], json!(1));
    assert_eq!(stats["retrievals"]["stats_read_count"], json!(1));

    proxy.shutdown().await;
}

#[tokio::test]
async fn retrieve_invalid_requests_return_400_without_upstream() {
    let proxy = start_proxy_with_state(DEAD_UPSTREAM, in_memory_ccr, |state| state).await;
    let client = reqwest::Client::new();
    let url = format!("{}/v1/retrieve", proxy.url());

    let missing = client.post(&url).json(&json!({})).send().await.unwrap();
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        missing.json::<Value>().await.unwrap()["error"],
        json!("hash is required")
    );

    let invalid = client
        .post(&url)
        .json(&json!({ "hash": "0123456789ABCDEFabcdef01" }))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid.json::<Value>().await.unwrap()["error"],
        json!("hash must be 24 lowercase hex characters")
    );

    let malformed = client
        .post(&url)
        .header("content-type", "application/json")
        .body("{\"hash\":")
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        malformed.json::<Value>().await.unwrap()["error"],
        json!("invalid JSON request body")
    );

    let stats: Value = reqwest::get(format!("{}/v1/retrieve/stats", proxy.url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["retrievals"]["invalid_request_count"], json!(3));
    assert_eq!(stats["retrievals"]["stats_read_count"], json!(1));

    proxy.shutdown().await;
}

#[tokio::test]
async fn retrieve_stats_empty_store_and_metrics_are_bounded() {
    let proxy = start_proxy_with_state(DEAD_UPSTREAM, in_memory_ccr, |state| state).await;

    let stats: Value = reqwest::get(format!("{}/v1/retrieve/stats", proxy.url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["store"]["entry_count"], json!(0));
    assert_eq!(stats["retrievals"]["success_count"], json!(0));
    assert_eq!(stats["retrievals"]["miss_count"], json!(0));
    assert_eq!(stats["retrievals"]["invalid_request_count"], json!(0));
    assert_eq!(stats["retrievals"]["stats_read_count"], json!(1));

    let metrics = reqwest::get(format!("{}/metrics", proxy.url()))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(metrics.contains("proxy_ccr_retrieve_requests_total"));
    assert!(metrics.contains("outcome=\"success\""));
    assert!(metrics.contains("outcome=\"miss\""));
    assert!(metrics.contains("outcome=\"invalid_request\""));
    assert!(metrics.contains("outcome=\"stats_read\""));
    assert!(!metrics.contains("0123456789abcdefabcdef01"));

    proxy.shutdown().await;
}
