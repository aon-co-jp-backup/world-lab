# 設計思想＆開発方針＆開発環境ルール(World Laboratory / world-lab)

作業ドライブは`F:\runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の
`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。
GitHubリポジトリ: [aon-co-jp/world-lab](https://github.com/aon-co-jp/world-lab)。

**構想開始日: 2026-08-06**(このリポジトリへの構想文書・コード移設着手日。
[`dream-os`](https://github.com/aon-co-jp/dream-os)のサブ構想として
2026-08-06に発案され、同日中に独立リポジトリとして分離された)。

## このプロジェクトの役割

**World Laboratory**(リポジトリ名・VPS/ローカルフォルダ名は`world-lab`)は、
世界中の個人が持つPC/タブレット/スマホの余剰計算資源(GPU/NPU/CPU)を、
AI(LLM推論・学習・データ処理等)のワークロードへボランティア提供できる
分散基盤——BOINC/Folding@home型のボランティアコンピューティングを、
個人では持てない研究所/大学/メーカー規模のAI開発環境として実現する構想。

**分離の経緯**: 元々`dream-os`(Linux×TRONハイブリッドカーネル構想、
Windows/Android共通実行基盤PoC)のサブ構想として設計・実装が始まったが、
「実行基盤(GPU/NPU上でのカーネルディスパッチ)」と「分散コーディネー
ション(世界中のノードを束ねる仕組み)」は性質の異なる別レイヤーであり、
スコープを明確化するため独立リポジトリとして分離した(ユーザー指示、
2026-08-06)。

## エコシステム内での位置づけ

- **[`dream-os`](https://github.com/aon-co-jp/dream-os)**: 実行基盤層
  (`dream-os-kernel`、Windows/Android共通Vulkan実行基盤+マイニング
  カーネル+東芝SBM風最適化+open-directx連携)+通信・永続化層
  (`dream-os-wire`、`SecureChannel`によるAEAD暗号化+リプレイ対策+
  aruaru-db永続化)を提供する**下位レイヤー**。World Laboratoryは
  これらをpath依存で再利用する想定(具体的な依存方法は次回設計)。
- **[`open-web-server`](https://github.com/aon-co-jp/open-web-server)/
  [`RPoem`](https://github.com/aon-co-jp/RPoem)**: コーディネータ本体
  (ワークユニット配布・結果集約のHTTP API)の実装候補。
- **[`aruaru-db`](https://github.com/aon-co-jp/aruaru-db)**: 結果の
  永続化先(ACID互換+Git-on-SQLバージョン管理)、`dream-os-wire`の
  `aruaru_persistence`モジュール経由で既に実機接続実績あり。
- **[`open-cuda`](https://github.com/aon-co-jp/open-cuda)**: GPU実行
  基盤(`dream-os-kernel`経由の間接依存)。
- **[`open-directx`](https://github.com/aon-co-jp/open-directx)**:
  Windows DirectXシェーダー互換層(`dream-os-kernel`経由の間接依存)。
  **現在このエコシステム内で最優先で完成度を高めている最中**の
  プロジェクト。

## 設計文書

詳細なアーキテクチャ設計(4層構成: コーディネータ/通信・永続化層/
ワーカー/実行基盤、BOINC等の先行事例調査、ワークユニット設計、N-of-M
多数決検証、Sybil耐性の課題、フェーズ0〜4ロードマップ)は
[`docs/world-laboratory-design.md`](docs/world-laboratory-design.md)
(`dream-os`から2026-08-06に移設)を参照。

## 現状(2026-08-06、コーディネータ本体を実装・実機検証済み)

**`crates/world-lab-coordinator`(新規)**: Rust+tokio/hyper直接実装
(Poem非依存、`open-web-server-gateway`と同じ`accept_loop`+
`service_fn`+`http1::Builder`パターンを踏襲、ただしRPoem/open-web-server
自体のクレートには依存せず独立実装)のHTTPサーバー。

**実装したAPI(フェーズ1)**:
- `GET /health` — 死活確認。
- `POST /work-units` — ワークユニット登録(テスト/管理用途)。
- `GET /work-units/next` — 未着手ワークユニットを1件取得(FIFO)。
- `POST /work-units/:id/submit?worker_id=X` — ワーカーが`dream-os-wire::
  WorkerChannel`で暗号化した結果フレームを送信、コーディネータが
  `CoordinatorChannel`で復号・改ざん検知・リプレイ拒否した上で
  `aruaru-db`(設定時)へ永続化。

**実機検証(実プロセス3つ・実HTTP・実暗号処理)**: コーディネータを実際に
起動し、`examples/submit_worker_result.rs`(実ワーカー役、
`dream_os_wire::WorkerChannel`で実際にAEAD暗号化)から実HTTP POSTで結果を
送信。以下を実証:
1. 正常な結果送信が受理されること(`{"accepted":true,...}`)。
2. **同一暗号化フレームを2回目に送信するとリプレイとして拒否される**
   こと(`{"error":"rejected: failed to decrypt/verify work result
   frame"}`)——ワーカーごとの`CoordinatorChannel`が複数HTTPリクエストに
   またがって正しく状態(既知シーケンス番号集合)を保持している直接証拠。

**実装過程で発見・修正した実際の設計上の罠**: 当初`CoordinatorChannel::
new()`をリクエストのたびに新規生成する実装にしていたが、
`SecureChannel`のリプレイガードは呼び出し元インスタンスの内部状態
(既知シーケンス番号集合)に依存するため、これでは`seen`集合が毎回
空になりリプレイ検知が機能しない実バグになることに気付いた——
ワーカーIDごとに1つの`CoordinatorChannel`を`AppState`内で保持し続ける
設計へ修正した(詳細は`main.rs`のコメント参照)。

**正直な開示・スコープの限界**: (1) 鍵共有は全ワーカー共通の開発用
固定鍵(フェーズ1の簡略化、フェーズ3で認証・鍵配布基盤が必要)、
(2) N-of-M多数決検証は未実装(フェーズ2)、(3) ワークユニットの永続化は
現状プロセス内メモリのみ(再起動で失われる)、(4) `aruaru-db`永続化は
実接続確認済み(`WORLD_LAB_ARUARU_PG_CONN`設定時)だが、今回の実機検証
では未設定のまま実施(`aruaru_commit_id: null`)——別途`dream-os`側で
`aruaru-db`永続化自体は実機検証済み。

## HANDOFF(直近の作業ログ、上が最新)

- **2026-08-06(続き) コーディネータ本体(`world-lab-coordinator`)を実装・
  実機検証**: ユーザー指示「open-directx優先で同時進行で、world-lab側の
  コーディネータ本体(RPoem/open-web-server上へのHTTP API実装)を進めて」
  への対応。詳細は上記「現状」節参照。`dream-os-wire`(`WorkerChannel`/
  `CoordinatorChannel`/`AruaruDbStore`)へのpath依存で実装、実プロセス・
  実HTTP・実AEAD暗号処理で正常送信・リプレイ拒否を実証。open-directx側は
  並行してバックグラウンドエージェントが境界チェック付きチェーンを
  3項→4項へ拡張・実機検証済み(`cf95ba2`)。
  - 次にすべきこと: (1) `POST /work-units`の自動分割ロジック(現状は
    手動登録のみ)、(2) N-of-M多数決検証の実装(フェーズ2)、(3) ワーク
    ユニットの永続化(現状プロセス内メモリのみ)、(4) ワーカーごとの
    鍵配布(フェーズ3、KeyGuardian等の活用検討)。

- **2026-08-06 新規作成・リポジトリ分離**: ユーザー指示「ワールド
  ラボラトリー構想のリポジトリ名はworld-labとして作成して、VPSと
  ローカルドライブにも」を受けて着手。GitHub上に`aon-co-jp/world-lab`
  を新規作成(空リポジトリでは無くこのタイミングで新規作成、
  `dream-os`/`open-directx`のような「既存の空リポジトリを再利用」
  パターンとは異なる)。ローカル(`F:\runo\world-lab`)へclone、
  VPS(conoha、`/root/world-lab`)にもプレースホルダフォルダを新設。
  `dream-os`の`docs/world-laboratory-design.md`をこのリポジトリへ
  移設(`dream-os`側は移設先へのポインタのみ残す)。
  - 次にすべきこと: (1) コーディネータ本体(RPoem/open-web-server上
    への実HTTP API)の設計・実装、(2) `dream-os-kernel`/`dream-os-wire`
    への依存方法の確定(path依存か、ネットワーク越しの利用か)、
    (3) open-directxの完成度向上が優先方針のため、引き続き小さく
    育てる。
