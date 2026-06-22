//! Health endpoints: own /healthz always 200; /healthz/upstream reflects upstream.

mod common;

use common::start_proxy;
use headroom_core::ccr::backends::CcrBackendConfig;
use headroom_proxy::{AppState, Config, ProxyError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn healthz_ok_when_upstream_down() {
    let proxy = start_proxy("http://127.0.0.1:1").await; // unroutable port
    let resp = reqwest::get(format!("{}/healthz", proxy.url()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    proxy.shutdown().await;
}

#[tokio::test]
async fn healthz_upstream_503_when_upstream_down() {
    let proxy = start_proxy("http://127.0.0.1:1").await;
    let resp = reqwest::get(format!("{}/healthz/upstream", proxy.url()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    proxy.shutdown().await;
}

#[tokio::test]
async fn healthz_upstream_200_when_upstream_healthy() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&upstream)
        .await;
    let proxy = start_proxy(&upstream.uri()).await;
    let resp = reqwest::get(format!("{}/healthz/upstream", proxy.url()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    proxy.shutdown().await;
}

#[test]
fn app_state_fails_loudly_when_ccr_sqlite_init_fails() {
    let mut config = Config::for_test("http://127.0.0.1:1".parse().unwrap());
    config.ccr_backend = CcrBackendConfig::Sqlite {
        path: std::path::PathBuf::from("/definitely/missing/headroom-ccr.sqlite"),
        ttl_seconds: headroom_core::ccr::DEFAULT_TTL.as_secs(),
    };

    let err = match AppState::new(config) {
        Ok(_) => panic!("invalid CCR backend must fail startup"),
        Err(err) => err,
    };
    assert!(
        matches!(err, ProxyError::CcrStartup(_)),
        "expected CCR startup error, got {err:?}"
    );
}
