use headroom_core::ccr::backends::InMemoryCcrStore;
use headroom_core::ccr::CcrStore;
use headroom_core::transforms::{
    compress_openai_chat_live_zone_with_ccr, compress_openai_responses_live_zone_with_ccr,
    AuthMode, LiveZoneOutcome,
};
use serde_json::{json, Value};

fn compressible_rows_payload() -> String {
    let rows: Vec<Value> = (0..1500)
        .map(|i| {
            json!({
                "id": i,
                "kind": "row",
                "value": format!("repeat-{}", i % 5),
                "status": "ok",
            })
        })
        .collect();
    serde_json::to_string(&rows).unwrap()
}

fn compressible_shell_log() -> String {
    let mut log = String::new();
    for i in 0..240 {
        log.push_str(&format!(
            "[2024-01-01 00:00:00] INFO build.rs:42 compiled module foo_{i}\n"
        ));
    }
    log
}

fn extract_ccr_hash(body: &str) -> Option<String> {
    let start = body.find("<<ccr:")? + "<<ccr:".len();
    let rest = &body[start..];
    let end = rest.find(">>")?;
    Some(rest[..end].to_string())
}

#[test]
fn openai_chat_with_ccr_writes_marker_backing_store() {
    let store = InMemoryCcrStore::new();
    let tool_payload = compressible_rows_payload();
    let body = serde_json::to_vec(&json!({
        "model": "gpt-4o",
        "messages": [
            {"role": "user", "content": "summarize rows"},
            {"role": "assistant", "content": "fetching"},
            {"role": "tool", "tool_call_id": "t1", "content": tool_payload},
        ]
    }))
    .unwrap();

    let outcome =
        compress_openai_chat_live_zone_with_ccr(&body, AuthMode::Payg, "gpt-4o", Some(&store))
            .expect("dispatcher");
    let LiveZoneOutcome::Modified { new_body, .. } = outcome else {
        panic!("expected modified chat body");
    };
    let hash = extract_ccr_hash(new_body.get()).expect("CCR marker");
    assert_eq!(store.get(&hash).as_deref(), Some(tool_payload.as_str()));
}

#[test]
fn openai_responses_with_ccr_writes_marker_backing_store() {
    let store = InMemoryCcrStore::new();
    let output = compressible_shell_log();
    let body = serde_json::to_vec(&json!({
        "model": "gpt-4o",
        "input": [{
            "type": "local_shell_call_output",
            "id": "lso_1",
            "call_id": "call_1",
            "output": output
        }]
    }))
    .unwrap();

    let outcome =
        compress_openai_responses_live_zone_with_ccr(&body, AuthMode::Payg, "gpt-4o", Some(&store))
            .expect("dispatcher");
    let LiveZoneOutcome::Modified { new_body, .. } = outcome else {
        panic!("expected modified responses body");
    };
    let hash = extract_ccr_hash(new_body.get()).expect("CCR marker");
    assert_eq!(store.get(&hash).as_deref(), Some(output.as_str()));
}
