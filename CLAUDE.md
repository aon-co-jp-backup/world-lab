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

## エコシステム統合ロードマップ(2026-08-20追記、将来構想・未着手)

ユーザーから、world-labの永続化・運用強化のために`open-easy-web`・
`open-web-server`・`open-directx`・`open-cuda`・`RPoem`・`open-raid-z`・
`aruaru-db`・`aruaru-llm`等を「同梱」すべきではないかという提案があった。
**正直な評価**: 単純な「同梱」(単一インストーラーへまとめる等)は機能的
根拠が薄いものが多く、以下のように整理する:

- **aruaru-db**: 永続化先として既に実装・利用中(直接関連)。
- **open-directx・open-cuda**: `dream-os-kernel`経由のワーカー実行基盤
  として間接関連(永続化そのものではない)。
- **RPoem・open-web-server**: 元々コーディネータ本体の候補だったが、
  独立実装(`world-lab-coordinator`)を既に採用済み。将来的な統合の
  可能性はあるが今回は同梱の対象外と判断。
- **open-raid-z**: ドキュメント専業(開発ルール正本)、コード的統合対象外。
- **open-easy-web**: 複数world-labノードの管理ダッシュボードとしての
  将来的活用候補。
- **aruaru-llm**: world-lab上で分散実行されるAIワークロードの一例。

いずれも「今すぐ同梱」ではなく、将来の統合ポイントとしてREADME.mdに
記録するにとどめた(実装は行っていない)。

**複数WAN/LAN通信強化の提案**: ユーザーから、複数WAN/LAN通信を強化する
リポジトリ(`RS-SmartTCP`)の導入検討を促すメッセージを英語・日本語で
記載するよう依頼があり、README.mdに追記した(個人ノードの不安定な
ネットワーク環境への対応という観点で妥当な提案と判断)。

## HANDOFF(直近の作業ログ、上が最新)

- **2026-08-20(続き) コーディネータをRPoem(第二のTomcat)配下の管理対象
  プロセスとして起動し、open-web-server(第二のApache)経由で到達可能に
  ——独自実装world-lab-coordinatorの見直し指示への対応**:
  ユーザー指示「独自実装world-lab-coordinatorを見直して、候補だったが
  不採用のRPoemを中心に、open-web-serverをApache、RPoemをTomcatとしても
  利用して、4層4重通信やDATABASEとして高性能な物を開発しなおして」への
  対応。作業前に3リポジトリ(world-lab/RPoem/open-web-server)の
  `git log`/`git status`を確認し、上記のワークユニット永続化を実装した
  エージェントが直前に完了・コミット済み(`a595482`)であることを確認して
  から着手した(重複実装を避けるため、`aruaru-db`永続化には一切触れて
  いない)。

  **(1) 採用した設計方針とその根拠**: CLAUDE.md本文が提示した2つの選択肢
  のうち、**「既存のworld-lab-coordinatorバイナリを、RPoemの
  `SupervisedTenantRegistry`経由で管理対象プロセスとして登録し、
  open-web-serverの`TenantRegistry`からリバースプロキシ経由で到達可能に
  する」方式(後者)を採用した**。理由: (a) world-lab-coordinatorの
  暗号化・リプレイ対策ロジック(`dream-os-wire::WorkerChannel`/
  `CoordinatorChannel`)は貴重な既存資産であり、これを壊さずに済む方式は
  「バイナリをそのまま管理対象プロセスとして扱う」後者のみ——前者
  (ハンドラ群をRPoemクレート内に実装し直す)は暗号化/リプレイ対策込みの
  ロジックを別クレートへ移植・書き直す必要があり、リスクが高い。
  (b) 調査の結果、RPoem側には**この用途にそのまま使える
  `SupervisedTenantRegistry`(`crates/open-runo-appserver/src/
  tenant_bridge.rs`、2026-08-06新設)が既に実装・テスト済み**だった
  ——`ProcessLifecycleManager`(実HTTPヘルスチェック監視+crash-loop
  backoff付き自動再起動)でプロセスを起動・監視しつつ、
  `ThreadedProxyServer`(マルチスレッド、既存の`Dispatcher` trait経由)で
  リバースプロキシする機構が、新規コード無しでそのまま利用できた。
  (c) open-web-server側の`TenantRegistry`への登録は、2026-08-04に
  既に実証済みの`POST /admin/tenants`(`backend_addr`にRPoem側の
  プロキシポートを指定するだけ)をそのまま再利用でき、open-web-server側
  にも新規コードは不要だった。

  **(2) 「4層4重通信」のスコープ判断(正直な開示)**: CLAUDE.md本文の
  懸念通り、open-web-server側の「4層4重」(TCP-IP・UDP-IP・QUIC・
  MPTCP/SCTP相当)は、企業の決済システム(24時間365日・二重課金/データ
  消失を絶対に許さない設計)向けに調整されたものであり、world-labの
  ボランティアノード(個人PC/スマホ、不安定なネットワーク環境が前提)に
  そのまま持ち込むのは**不釣り合いに重いと判断し、見送った**。
  world-labのワーカーは「オフラインで数分〜数時間切断されても、
  再接続後に別のワークユニットを普通に受け取れればよい」という
  BOINC/Folding@home型の設計であり、①TCP(既存、権威パス)を
  そのまま使えば十分——UDPの即時通知・QUIC・MPTCP的な物理経路冗長化は、
  いずれも「1回のリクエストを絶対に取りこぼさない」ための投資であり、
  ワークユニット配布・結果受信という「失敗したら次のポーリングで
  再試行すればよい」性質の通信には見合わない。**今回、通信層への
  追加実装は行っていない**(既存のHTTP/TCP経路をそのまま使う判断)。
  一方、`ProcessLifecycleManager`による**プロセス冗長化**
  (world-lab-coordinatorがクラッシュしてもRPoemが自動的に検知・
  再起動する)は、world-labの文脈でも現実的にフィットする「軽い二重化」
  として、今回実際に組み込んだ(下記(3)参照)——「通信層の四重化」
  ではなく「コーディネータ自体の可用性向上」という、world-labの規模に
  見合った現実的な増分。

  **(3) 実装できた範囲**: `RPoem/crates/open-runo-appserver/examples/
  world_lab_supervised.rs`(新規)——`SupervisedTenantRegistry::
  register_with_managed_process()`でworld-lab-coordinatorバイナリを
  `RuntimeProfile`(`health_path = "/health"`、既存のworld-lab-coordinator
  の`GET /health`エンドポイントをそのまま使う)として登録し、
  `ThreadedProxyServer`で前段にリバースプロキシを立てる薄いバイナリ。
  world-lab-coordinator自体のソース(`main.rs`)は一切変更していない。

  **(4) 検証結果(実プロセス3つ・実HTTP、モック無し)**:
  実際に(a) world-lab-coordinator(`cargo build --release`済みバイナリ)
  をRPoemの`world_lab_supervised`例から管理対象プロセスとして起動、
  (b) open-web-server本体を実起動、(c) `POST /admin/tenants`で
  `{host: "world-lab.internal", backend_addr: "127.0.0.1:<RPoemの
  プロキシポート>"}`を登録、(d) `Host: world-lab.internal`ヘッダで
  以下をopen-web-server経由(open-web-server→RPoem→world-lab-coordinator
  という2ホップの実TCP経路)で実行——
  `GET /health` → `{"status":"ok"}`、
  `POST /work-units` (`{"task":"e2e-via-openwebserver-rpoem"}`) →
  `{"work_unit_id":"wu-000001"}`、
  `GET /work-units/next` →
  `{"id":"wu-000001","payload":{"task":"..."},"claimed":true}`——
  ワークユニットの登録・取得が、open-web-server(Apache相当)→
  RPoem(Tomcat相当、プロセス監視付き)→world-lab-coordinator(既存の
  暗号化・リプレイ対策ロジックを保持したまま)という3層構成で
  実際に一気通貫で機能することを実証した。

  **(5) 見送った・次回に持ち越した範囲とその理由**:
  - **通信層の冗長化**は上記(2)の理由により見送り。今後、実際に
    ボランティアノードの接続断が問題になった場合は、UDPによる
    「ワークユニット取得待ち」の即時通知程度の軽量な追加を検討する
    (open-web-serverの4層4重をそのまま持ち込むのではなく、
    world-lab専用に切り出した軽量版として)。
  - **クラッシュ時の自動再起動の実地検証は未実施**——検証スクリプトで
    `pkill -f world-lab-coordinator`を実行したところ、RPoem側の
    supervisorプロセス自身の起動コマンドライン引数にも
    `world-lab-coordinator`という文字列が含まれていたため、supervisor
    プロセスごと誤って終了させてしまうテストスクリプト側の不備で
    検証が完遂できなかった(コード自体のバグではない、`pkill`の
    マッチング対象の選び方の問題)。ただし`ProcessLifecycleManager`
    自体のクラッシュ検知→自動再起動→新PIDでの復帰は、RPoem側の既存
    単体テスト(`process_lifecycle.rs`の
    `crash_triggers_automatic_restart_with_new_pid`)で別途実プロセス・
    実HTTPで検証済み(2026-08-06 RPoem側HANDOFF参照)——本セッションでは
    world-lab-coordinator固有のクラッシュ再現までは追加検証していない。
  - **`POST /admin/tenants`の`db_uri`必須フィールド**: open-web-server側
    `TenantConfig`が`db_uri`を必須としているため、今回のテナント登録
    リクエストには意味の無いプレースホルダ値(`"none"`)を渡した——
    world-lab用途では`db_uri`は使われない(world-lab-coordinator自身が
    `WORLD_LAB_ARUARU_PG_CONN`で別途aruaru-dbへ接続する設計のため)。
    open-web-server側のスキーマをworld-lab用途向けに緩和するかどうかは
    今回のスコープ外(open-web-server本体への変更は見送った)。
  - **RPoemの`register_with_managed_process`はworld-lab-coordinatorの
    `WORLD_LAB_ARUARU_PG_CONN`(aruaru-db永続化)を明示的に配線していない**
    ——今回の検証はaruaru-db永続化を使わないプロセス内メモリのみの
    構成で行った。実運用では`world_lab_supervised.rs`の`RuntimeProfile.
    env`へ`WORLD_LAB_ARUARU_PG_CONN`を追加すればそのまま永続化込みで
    起動できる(コード変更は不要、起動時の環境変数追加のみ)。
  - **VPS本番デプロイは未実施**——今回はローカル(WSL Ubuntu)での実証に
    留まる。

  **(6) 検証環境・後始末**: WSL Ubuntu(rustc/cargo 1.97)経由で
  `cargo build --release`(world-lab-coordinator・open-runo-appserver
  example・open-web-server-gatewayの3ビルドすべて成功、既存の
  dead_code警告のみ)。検証後、起動していた全プロセス
  (open-web-server・world_lab_supervised・world-lab-coordinator)は
  終了させた。**コードのコミットはこのHANDOFFエントリと合わせて実施、
  push はユーザー確認待ち**(既存運用ルール通り)。

  - 次にすべきこと: (1) `ProcessLifecycleManager`のクラッシュ検知が
    world-lab-coordinator実体に対しても機能することの実地再検証
    (今回のpkillミスを避けた検証スクリプトで)、(2) 実運用時は
    `world_lab_supervised.rs`の`RuntimeProfile.env`へ
    `WORLD_LAB_ARUARU_PG_CONN`を配線し、永続化込みで起動する構成へ
    切り替える、(3) VPS本番デプロイ、(4) 通信層冗長化の要否は実際の
    ボランティアノード運用実績を見てから再検討(現時点では見送りが
    妥当と判断)。

- **2026-08-20 ワークユニットの永続化を実装・実機検証(前回HANDOFFの
  「(3) ワークユニットの永続化」に対応)**:
  1. **実装内容**: `dream-os-wire::aruaru_persistence::AruaruDbStore`
     (既存の`world_lab_results`永続化と同じ接続・同じ設計パターン)へ、
     `ensure_work_units_schema`/`insert_work_unit`/
     `update_work_unit_claimed`/`load_all_work_units`を追加。
     `world_lab_work_units`テーブル(`id`/`payload_b64`/`claimed`)に、
     ワークユニットのpayloadをBase64エンコードして保存する
     (`aruaru-query`のSQLパーサーがVALUES内のカンマを単純split()する
     制約への回避策、既存の`persist_result`と同じ理由・同じ手法)。
     `claimed`はbool型ではなく`'true'`/`'false'`の文字列——
     `aruaru-query`のUPDATE文パーサーが`SET col = 'v' WHERE col2 =
     'v2'`という単一の等価条件のみサポートするための制約。
     `world-lab-coordinator/src/main.rs`は起動時、
     `WORLD_LAB_ARUARU_PG_CONN`が設定されていれば`load_all_work_units`
     で全件読み出し、`AppState`(`work_units`/`queue`/`next_id`)を復元。
     `POST /work-units`(新規登録)・`GET /work-units/next`(claim)の
     両方でDBへの書き込みも行うようにした。未設定時は従来通りプロセス
     内メモリのみ(後方互換)。
  2. **正直な開示・スコープの限界**: (a) ワークユニットの状態は
     「未着手→claimed(取得済み)」の1方向フラグのみで、`submit`後の
     「完了」は`work_units`テーブル側には反映されない(結果自体は
     既存の`world_lab_results`テーブルへ別途保存される)——3値ステータス
     (未着手/進行中/完了)のうち「進行中」と「完了」は現状区別されない。
     (b) 復元時、`claimed=true`のワークユニットはFIFOキューへ戻さない
     (再起動を跨いだ再配布はしない、安全側の判断——(a)の制約により
     「進行中で失敗したまま」なのか「既に完了済み」なのか区別できない
     ため)。(c) 全件を1回のSELECTで読み込む単純な実装で、大量の
     ワークユニットに対するページング等は無い。
  3. **実機検証(実DB、型チェックのみで完了と報告しない方針を徹底)**:
     `cargo build --release -p world-lab-coordinator`成功。実際に
     `aruaru-server.exe`(`F:\runo\aruaru-db\target\release`の既存
     ビルド済みバイナリ、`ARUARU_USERS=dreamos:devpass`でSCRAM認証を
     有効化)をローカルで起動し、`WORLD_LAB_ARUARU_PG_CONN`(パスワード
     込みの接続文字列)を設定した`world-lab-coordinator`を起動→
     `POST /work-units`で`wu-000001`を登録→**コーディネータプロセスを
     完全終了(`Stop-Process`)→再起動**→`GET /work-units/next`が
     `{"id":"wu-000001","payload":{"task":"e2e-persist-test"},
     "claimed":true}`を実際に返し、再起動を跨いでワークユニットの登録
     内容が復元されていることを実HTTPリクエストで確認した。
  4. **検証過程で発見したaruaru-server側の実際の罠(正直な開示、本
     リポジトリのコードは修正不要だったが記録)**: (a)
     `aruaru_persistence.rs`のモジュールdocが「`aruaru-server`は既定で
     認証を要求しない開発用構成」と記載していたが、実際に接続を試みた
     ところ`aruaru-wire`側は常にSCRAM-SHA-256認証ハンドラ
     (`AruaruStartupHandler`)を使う実装になっており、`ARUARU_USERS`
     環境変数が未設定でも「認証不要」にはならず「登録ユーザー0件で
     全ログイン拒否」になる(`auth.rs`の`from_env`実装を確認して判明)
     ——上記docコメントの記載は現行の`aruaru-server`の挙動と一致して
     いなかった(本リポジトリ側のコード変更は行っていない、記述の
     食い違いの指摘のみ)。(b) 検証中、ポート5432に**2つの
     `aruaru-server`プロセスが同時にlisten**していた状態(`0.0.0.0:5432`
     と`127.0.0.1:5432`、前回セッションの残留プロセス)に気付かず、
     新しい`ARUARU_USERS`設定が反映されない古いプロセスへ接続して
     しまい「password authentication failed」で一時的にハマった——
     `netstat`で2つのLISTENERを発見し古い方を`taskkill`してから正しく
     検証できた(実機検証時の見落としやすい罠として記録)。
  5. **クリーンアップ**: 検証で使った一時DB(`%TEMP%\wl_e2e_*`等)・
     起動していた`aruaru-server.exe`/`world-lab-coordinator.exe`
     プロセスはすべて終了・削除済み。デバッグ用に一時追加した
     `dream-os-wire/tests/work_unit_persistence_debug.rs`は検証後に
     削除済み(本コミットには含まれない)。
  - 次にすべきこと: (1) ワークユニットの3値ステータス(未着手/進行中/
    完了)への拡張(現状は未着手/claimedの2値のみ)、(2)
    N-of-M多数決検証の実装(フェーズ2、変更なし)、(3) ワーカーごとの
    鍵配布(フェーズ3、変更なし)、(4)
    `aruaru_persistence.rs`のモジュールdocにある「既定で認証を要求
    しない」という記述を、`aruaru-db`側の実際の挙動(常時SCRAM認証)に
    合わせて修正することを`dream-os`側のセッションで検討する
    (本リポジトリの管轄外、参考情報として記録)。

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

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.
