//! World Laboratory コーディネータ本体(2026-08-06新設)。
//!
//! ユーザー指示「open-directx優先で同時進行で、world-lab側のコーディネータ
//! 本体(RPoem/open-web-server上へのHTTP API実装)を進めて」への対応。
//!
//! **設計判断**: RPoem/open-web-serverのクレートへ直接依存するのではなく、
//! このエコシステムの既存方針(Poem非依存、Rust標準+tokio/hyper直接実装、
//! `open-web-server-gateway`の`accept_loop`+`service_fn`+`http1::Builder`
//! パターン)をそのまま踏襲した**新規の薄いHTTPサーバー**として実装した。
//! `open-web-server-gateway`のコードそのものを再利用(path依存)しなかった
//! 理由: そちらは課金/決済ドメイン専用の重量級ゲートウェイであり、
//! World Laboratory固有のワークユニット配布ロジックとは関心事が異なる
//! ため、同じ**設計パターン**を踏襲しつつ独立実装とする方が疎結合になる
//! と判断した(RPoem/open-web-server自体は、このエコシステムの標準的な
//! Rust+tokio/hyperサーバーの書き方の「参考実装」として機能している)。
//!
//! ## 実装したAPI(フェーズ1、`docs/world-laboratory-design.md`参照)
//!
//! - `GET /health` — 死活確認。
//! - `POST /work-units` — ワークユニットを登録する(テスト/管理用途、
//!   本来はより大きなジョブから自動分割する上位ロジックが必要——今回は
//!   その前段の「配布・結果受信」経路の実証に絞る)。
//! - `GET /work-units/next` — 未着手のワークユニットを1件返す(単純な
//!   FIFO、複数ワーカーへの同時配布制御・タイムアウト再配布は未実装)。
//! - `POST /work-units/:id/submit?worker_id=X` — ワーカーが
//!   `dream_os_wire::WorkerChannel::submit()`で暗号化した結果フレームを
//!   送信する。コーディネータは`CoordinatorChannel::receive()`で復号・
//!   改ざん検知・リプレイ拒否を行った上で、`aruaru-db`(設定されていれば)
//!   へ永続化する。
//!
//! ## 正直な開示・スコープの限界
//!
//! - **鍵共有はMVP簡略化**: 全ワーカーが同一の共有鍵(起動時の
//!   `WORLD_LAB_SHARED_KEY`環境変数、無指定時は開発用固定値)を使う設計。
//!   `docs/world-laboratory-design.md`のフェーズ3(外部ボランティア参加・
//!   認証・Sybil耐性)で、ワーカーごとに異なる鍵を発行する設計へ拡張が
//!   必要——現時点は「単一の信頼できるノード群」(フェーズ1)を対象とした
//!   簡略化であることを明記する。
//! - **N-of-M多数決検証は未実装**: 現状は1件の結果をそのまま受理する
//!   (§3.3のN-of-M複製配布検証はフェーズ2)。
//! - **ワークユニットの永続化(2026-08-20実装)**: `WORLD_LAB_ARUARU_PG_CONN`
//!   環境変数が設定されている場合、`dream_os_wire::AruaruDbStore`
//!   (既存の`aruaru_persistence`モジュール、`world_lab_results`テーブルと
//!   同じ接続を再利用)経由で`world_lab_work_units`テーブルへ
//!   ワークユニットの登録内容・claimedフラグを永続化する。起動時に
//!   このテーブルから全件読み出して`AppState`(`work_units`/`queue`/
//!   `next_id`)を復元するため、コーディネータを再起動してもワーク
//!   ユニットの状態(未着手/進行中)が失われない。未設定時は従来通り
//!   プロセス内メモリのみで動作する(後方互換、開発・テスト用途)。
//!   **正直な開示**: (1) `claimed`は「未着手→取得済み」の1方向の
//!   フラグのみで、`submit`後の「完了」状態は`work_units`テーブルには
//!   反映されない(結果自体は既存の`world_lab_results`テーブルへ
//!   別途永続化される、`handle_submit_result`参照)——ワークユニットの
//!   3値ステータス(未着手/進行中/完了)のうち「進行中」と「完了」は
//!   現状どちらも`claimed=true`のまま区別されない、これは今回の
//!   スコープでは対応していない今後の拡張余地。(2) `aruaru-query`の
//!   SQLパーサーが`UPDATE ... SET col = 'v' WHERE col2 = 'v2'`という
//!   単一の等価条件しかサポートしないため、`claimed`はbool型ではなく
//!   `'true'`/`'false'`の文字列として保存している(`aruaru_persistence`
//!   モジュール側の制約、詳細はそちらのdoc参照)。(3) 起動時の復元は
//!   `AruaruDbStore::load_all_work_units`で全件を1回のSELECTで読み出す
//!   単純な実装であり、ワークユニット数が非常に多い場合のページング等は
//!   実装していない。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use dream_os_wire::{AruaruDbStore, CoordinatorChannel, WorkResultEnvelope};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkUnit {
    id: String,
    payload: serde_json::Value,
    claimed: bool,
}

struct AppState {
    work_units: RwLock<HashMap<String, WorkUnit>>,
    /// 挿入順を保持するFIFOキュー(未着手ワークユニットのID列)。
    queue: Mutex<Vec<String>>,
    next_id: Mutex<u64>,
    /// ワーカー鍵は現状全ワーカー共通(フェーズ1の簡略化、モジュールdoc参照)。
    shared_key: [u8; 32],
    /// ワーカーIDごとの`CoordinatorChannel`(重要: `SecureChannel`の
    /// リプレイガードは呼び出し元インスタンスの内部状態
    /// 〈既知シーケンス番号集合〉に依存するため、リクエストのたびに
    /// 新規`CoordinatorChannel::new()`を作ると`seen`集合が毎回空になり
    /// リプレイ検知が機能しない実バグになる——ワーカーごとに1つの
    /// `CoordinatorChannel`を保持し続けることで、複数HTTPリクエストに
    /// またがってリプレイガードの状態を正しく蓄積する)。
    channels: Mutex<HashMap<String, CoordinatorChannel>>,
    aruaru: Option<AruaruDbStore>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let bind_addr: SocketAddr = std::env::var("WORLD_LAB_BIND").unwrap_or_else(|_| "127.0.0.1:8300".to_string()).parse()?;

    let shared_key = derive_shared_key(&std::env::var("WORLD_LAB_SHARED_KEY").unwrap_or_else(|_| "dev-only-shared-key".to_string()));

    let aruaru = match std::env::var("WORLD_LAB_ARUARU_PG_CONN") {
        Ok(conn_str) => match AruaruDbStore::connect(&conn_str).await {
            Ok(store) => {
                store.ensure_schema().await?;
                store.ensure_work_units_schema().await?;
                tracing::info!("connected to aruaru-db for result and work-unit persistence");
                Some(store)
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to connect to aruaru-db; work units and results will only be kept in memory");
                None
            }
        },
        Err(_) => {
            tracing::warn!("WORLD_LAB_ARUARU_PG_CONN not set; work units and results will only be kept in memory");
            None
        }
    };

    // 永続化先が設定されていれば、起動時に既存のワークユニット状態を
    // 読み込んで`AppState`を復元する(未着手のワークユニットのみFIFO
    // キューへ積み戻す、`claimed=true`のものはキューへは戻さない
    // ——モジュールdocの通り「進行中」と「完了」を区別できない現状の
    // 制約から、再起動を跨いだ再配布はしない安全側の判断)。
    let mut restored_work_units: HashMap<String, WorkUnit> = HashMap::new();
    let mut restored_queue: Vec<String> = Vec::new();
    let mut restored_next_id: u64 = 0;
    if let Some(store) = &aruaru {
        match store.load_all_work_units().await {
            Ok(mut rows) => {
                rows.sort_by(|a, b| a.0.cmp(&b.0));
                for (id, payload, claimed) in rows {
                    if let Some(n_str) = id.strip_prefix("wu-") {
                        if let Ok(n) = n_str.parse::<u64>() {
                            restored_next_id = restored_next_id.max(n);
                        }
                    }
                    if !claimed {
                        restored_queue.push(id.clone());
                    }
                    restored_work_units.insert(id.clone(), WorkUnit { id, payload, claimed });
                }
                tracing::info!(
                    restored_count = restored_work_units.len(),
                    queued_count = restored_queue.len(),
                    "restored work units from aruaru-db on startup"
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to restore work units from aruaru-db; starting with an empty state");
            }
        }
    }

    let state = Arc::new(AppState {
        work_units: RwLock::new(restored_work_units),
        queue: Mutex::new(restored_queue),
        next_id: Mutex::new(restored_next_id),
        shared_key,
        channels: Mutex::new(HashMap::new()),
        aruaru,
    });

    let listener = TcpListener::bind(bind_addr).await?;
    tracing::info!(addr = %bind_addr, "world-lab-coordinator listening");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req| route(state.clone(), req));
            if let Err(err) = AutoBuilder::new(hyper_util::rt::TokioExecutor::new()).serve_connection(io, service).await {
                tracing::debug!(error = %err, %peer_addr, "connection error");
            }
        });
    }
}

fn derive_shared_key(seed: &str) -> [u8; 32] {
    // 開発用の決定的な鍵導出(単純な繰り返し充填、本番運用では鍵管理基盤
    // 〈KeyGuardian等〉経由のランダム鍵配布に置き換えるべき——フェーズ1の
    // 簡略化として明記)。
    let mut key = [0u8; 32];
    let bytes = seed.as_bytes();
    for (i, b) in key.iter_mut().enumerate() {
        *b = bytes[i % bytes.len().max(1)];
    }
    key
}

async fn route(state: Arc<AppState>, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    let response = match (method.clone(), path.as_str()) {
        (Method::GET, "/health") => json_response(StatusCode::OK, &serde_json::json!({"status": "ok"})),
        (Method::POST, "/work-units") => handle_create_work_unit(&state, req).await,
        (Method::GET, "/work-units/next") => handle_next_work_unit(&state).await,
        (Method::POST, p) if p.starts_with("/work-units/") && p.ends_with("/submit") => {
            let id = p.trim_start_matches("/work-units/").trim_end_matches("/submit").to_string();
            handle_submit_result(&state, req, &id, &query).await
        }
        _ => json_response(StatusCode::NOT_FOUND, &serde_json::json!({"error": "not found"})),
    };

    Ok(response)
}

async fn handle_create_work_unit(state: &Arc<AppState>, req: Request<Incoming>) -> Response<Full<Bytes>> {
    let body = match req.into_body().collect().await {
        Ok(b) => b.to_bytes(),
        Err(e) => return json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": format!("failed to read body: {e}")})),
    };
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": format!("invalid JSON: {e}")})),
    };

    let id = {
        let mut n = state.next_id.lock();
        *n += 1;
        format!("wu-{n:06}")
    };
    let unit = WorkUnit { id: id.clone(), payload: payload.clone(), claimed: false };
    state.work_units.write().insert(id.clone(), unit);
    state.queue.lock().push(id.clone());

    if let Some(store) = &state.aruaru {
        if let Err(e) = store.insert_work_unit(&id, &payload, false).await {
            tracing::error!(work_unit_id = %id, error = %e, "failed to persist new work unit into aruaru-db (kept in memory only)");
        }
    }

    json_response(StatusCode::CREATED, &serde_json::json!({"work_unit_id": id}))
}

async fn handle_next_work_unit(state: &Arc<AppState>) -> Response<Full<Bytes>> {
    let claimed_id = {
        let mut queue = state.queue.lock();
        let mut result = None;
        while let Some(id) = queue.first().cloned() {
            let mut units = state.work_units.write();
            if let Some(unit) = units.get_mut(&id) {
                if !unit.claimed {
                    unit.claimed = true;
                    let unit = unit.clone();
                    drop(units);
                    queue.remove(0);
                    result = Some(unit);
                    break;
                }
            }
            queue.remove(0);
        }
        result
    };

    match claimed_id {
        Some(unit) => {
            if let Some(store) = &state.aruaru {
                if let Err(e) = store.update_work_unit_claimed(&unit.id, true).await {
                    tracing::error!(work_unit_id = %unit.id, error = %e, "failed to persist claimed flag into aruaru-db (kept in memory only)");
                }
            }
            json_response(StatusCode::OK, &unit)
        }
        None => json_response(StatusCode::NO_CONTENT, &serde_json::json!({"message": "no work units available"})),
    }
}

async fn handle_submit_result(state: &Arc<AppState>, req: Request<Incoming>, work_unit_id: &str, query: &str) -> Response<Full<Bytes>> {
    let worker_id = match parse_query_param(query, "worker_id") {
        Some(w) => w,
        None => return json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing ?worker_id=... query parameter"})),
    };

    let frame = match req.into_body().collect().await {
        Ok(b) => b.to_bytes(),
        Err(e) => return json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": format!("failed to read body: {e}")})),
    };

    let envelope: WorkResultEnvelope = {
        let mut channels = state.channels.lock();
        let channel = channels.entry(worker_id.clone()).or_insert_with(|| CoordinatorChannel::new(&state.shared_key));
        match channel.receive(&frame) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(work_unit_id, worker_id, error = %e, "rejected work result (tamper/replay/decrypt failure)");
                return json_response(StatusCode::UNAUTHORIZED, &serde_json::json!({"error": format!("rejected: {e}")}));
            }
        }
    };

    if envelope.work_unit_id != work_unit_id {
        return json_response(
            StatusCode::BAD_REQUEST,
            &serde_json::json!({"error": "work_unit_id in URL does not match envelope"}),
        );
    }

    let commit_id = if let Some(store) = &state.aruaru {
        match store.persist_result(&envelope).await {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::error!(work_unit_id, error = %e, "failed to persist result into aruaru-db");
                None
            }
        }
    } else {
        None
    };

    if let Some(unit) = state.work_units.write().get_mut(work_unit_id) {
        unit.claimed = true;
    }

    json_response(
        StatusCode::OK,
        &serde_json::json!({
            "accepted": true,
            "worker_id": envelope.worker_id,
            "aruaru_commit_id": commit_id,
        }),
    )
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k == key {
            Some(v.to_string())
        } else {
            None
        }
    })
}

fn json_response<T: Serialize>(status: StatusCode, body: &T) -> Response<Full<Bytes>> {
    let bytes = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    Response::builder().status(status).header("content-type", "application/json").body(Full::new(Bytes::from(bytes))).unwrap()
}
