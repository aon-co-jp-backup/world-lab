//! World Laboratoryコーディネータへ、実際にワーカー側の暗号化フレームを
//! 送信するデモ/検証プログラム(2026-08-06)。
//!
//! `cargo run --example submit_worker_result --release -- <coordinator_url> <work_unit_id>`

use dream_os_wire::{WorkResultEnvelope, WorkerChannel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let base_url = args.next().unwrap_or_else(|| "http://127.0.0.1:8300".to_string());
    let work_unit_id = args.next().unwrap_or_else(|| "wu-000001".to_string());

    // コーディネータと同じ既定の開発用共有鍵(実運用ではKeyGuardian等の
    // 鍵配布基盤経由でワーカーごとに異なる鍵を配る設計が必要、
    // main.rsのモジュールdoc参照)。
    let shared_key = derive_shared_key("dev-only-shared-key");
    let mut worker = WorkerChannel::new(&shared_key);

    let envelope = WorkResultEnvelope {
        work_unit_id: work_unit_id.clone(),
        worker_id: "demo-worker-1".to_string(),
        result_json: serde_json::json!({ "hashes": 64, "digest_sample": "abcd1234" }),
    };

    let frame = worker.submit(&envelope)?;

    if let Some(dump_path) = std::env::var_os("DUMP_FRAME_TO") {
        std::fs::write(&dump_path, &frame)?;
        println!("wrote {} bytes to {:?}", frame.len(), dump_path);
        return Ok(());
    }

    let url = format!("{}/work-units/{}/submit?worker_id={}", base_url.trim_end_matches('/'), work_unit_id, envelope.worker_id);
    let client = reqwest::Client::new();
    let resp = client.post(&url).body(frame).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    println!("HTTP {status}: {body}");

    Ok(())
}

fn derive_shared_key(seed: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let bytes = seed.as_bytes();
    for (i, b) in key.iter_mut().enumerate() {
        *b = bytes[i % bytes.len().max(1)];
    }
    key
}
