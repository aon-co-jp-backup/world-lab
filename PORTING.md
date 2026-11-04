# PORTING.md — world-lab お引越しファイル / Porting notes

> 対象バージョン: 構想段階(2026-08-06、`dream-os`から分離直後、
> コーディネータ本体(`world-lab-coordinator`)を実装・実機検証済み)。
> / Target version: concept stage (2026-08-06, just split off from
> `dream-os`; the coordinator itself (`world-lab-coordinator`) has been
> implemented and verified on real hardware).

## 0. このリポジトリのスコープ / Scope of this repository

日本語: `world-lab`はBOINC/Folding@home型のボランティア分散
コンピューティング構想のリポジトリです。実行基盤(`dream-os-kernel`)・
通信/永続化層(`dream-os-wire`)は`dream-os`側に実装済みで、
`world-lab`はそれらを利用するコーディネータ層を担います。コーディネータ
本体(`crates/world-lab-coordinator`、Rust+tokio/hyper直接実装)は
実装済みで、実プロセス・実HTTP・実AEAD暗号処理による実機検証(正常な
結果送信・リプレイ攻撃の拒否)も完了しています。N-of-M多数決検証・
ワークユニットの永続化・ワーカーごとの鍵配布は未実装です(詳細は
`CLAUDE.md`のHANDOFF参照)。

English: `world-lab` is a concept-stage repository for a BOINC/
Folding@home-style volunteer distributed computing effort. The
execution backend (`dream-os-kernel`) and communication/persistence
layer (`dream-os-wire`) already live in `dream-os`; `world-lab` hosts
the coordinator layer that uses them. The coordinator itself
(`crates/world-lab-coordinator`, implemented directly with Rust +
tokio/hyper) has been implemented and verified end-to-end with real
processes, real HTTP, and real AEAD cryptography (normal result
submission and replay-attack rejection). N-of-M majority-vote
verification, work-unit persistence, and per-worker key distribution
are not yet implemented (see the HANDOFF section of `CLAUDE.md` for
details).

## 1. 持っていくもの(ファイル一覧) / Files to bring along

```
world-lab/
├── README.md          # 日本語(主言語) / Japanese (primary language)
├── README-English.md  # 英語 / English
├── CLAUDE.md           # 開発方針・エコシステム内の位置づけ・HANDOFF
├── PORTING.md          # 本ファイル / this file
├── Cargo.toml           # ワークスペース定義(2026-08-06新設)
├── docs/
│   └── world-laboratory-design.md  # アーキテクチャ設計(2026-08-06に
│                                     # dream-osから移設)
└── crates/
    └── world-lab-coordinator/  # コーディネータ本体(Rust+tokio/hyper
        │                        # 直接実装、Poem非依存)
        ├── Cargo.toml           # ../../../dream-os/crates/dream-os-wire
        │                        # へのpath依存
        ├── src/main.rs
        └── examples/submit_worker_result.rs
```

## 2. VPS/ローカルとの対応 / Local & VPS locations

- ローカル / Local: `F:\runo\world-lab`
- VPS(conoha): `/root/world-lab`(2026-08-06新設、プレースホルダの
  `README.md`のみ、git管理下ではない — created 2026-08-06, placeholder
  `README.md` only, not yet under git)

## 3. `dream-os`との依存関係 / Dependency on `dream-os`

`crates/world-lab-coordinator`は`dream-os-wire`
(`../../../dream-os/crates/dream-os-wire`)へのpath依存を持つ
(`WorkerChannel`/`CoordinatorChannel`/`AruaruDbStore`を再利用)。
さらに`dream-os-wire`自体が`open-web-server-wire`
(`../../../open-web-server/crates/open-web-server-wire`)・
`aruaru-db`への実行時ネットワーク接続(`WORLD_LAB_ARUARU_PG_CONN`環境
変数、pgwire)を持つ。移設する場合は`dream-os`・`open-web-server`・
`RS-SmartTCP`を同じ相対位置関係でcloneしておくこと。

`crates/world-lab-coordinator`has a path dependency on `dream-os-wire`
(`../../../dream-os/crates/dream-os-wire`, reusing `WorkerChannel`/
`CoordinatorChannel`/`AruaruDbStore`). `dream-os-wire` itself has a path
dependency on `open-web-server-wire`
(`../../../open-web-server/crates/open-web-server-wire`) and a runtime
network dependency on `aruaru-db` (via the `WORLD_LAB_ARUARU_PG_CONN`
env var, pgwire). When relocating, keep `dream-os`, `open-web-server`,
and `RS-SmartTCP` cloned at the same relative positions.

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

### Deutsch (KI-Übersetzung, noch nicht von Muttersprachlern geprüft)
- **REST wird abgeschafft. Neue APIs werden nicht als REST gebaut.** APIs, die aus dem Browser oder mit `curl` aufgerufen werden, nutzen **Connect-Unary-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body); zwischen Diensten gilt **gRPC / gRPC-web** (wie bei der Admin-API von RPoem, `open_runo.admin.v1.Admin`). Streaming erst, wenn es wirklich nötig ist.
- **Bestehende REST-Endpunkte werden einzeln migriert, sobald man sie anfasst.** Nach der Migration wird die alte REST-Route entfernt. Bis dahin bleiben sie bestehen, es werden aber **keine neuen hinzugefügt**.
- **JSON wird auf RS-JSON vereinheitlicht**: Lesen mit `from_str_secure` / `parse_secure` (Grenzen für Größe, Tiefe und doppelte Schlüssel), Schreiben mit `rust_json::fast`. `prost` kommt nicht in Browser-WASM (die Datei würde größer).
- **Ausnahmen (Formate, die ein fremder Standard vorgibt, nicht unsere eigene API)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Dritten (z. B. GitHub), Health-Checks, statische Dateien, OTLP usw. Diese nehmen schlichtes HTTP entgegen, wie es der Standard verlangt.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines Repositorys diese Richtlinie schon befolgt, hängt vom jeweiligen Repository ab. Dieser Abschnitt hält die Richtlinie fest; er bedeutet nicht, dass der Code bereits geändert wurde.

### Русский (машинный перевод, носителями языка ещё не проверен)
- **REST полностью отменяется. Новые API не создаются как REST.** API, вызываемые из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело — JSON); между сервисами — **gRPC / gRPC-web** (так же, как административный API RPoem, `open_runo.admin.v1.Admin`). Потоковая передача — только когда она действительно нужна.
- **Существующие REST-эндпоинты переносятся по одному, когда до них доходят руки.** После переноса старый REST-маршрут удаляется. До завершения переноса они остаются, но **новые не добавляются**.
- **JSON унифицируется на RS-JSON**: чтение через `from_str_secure` / `parse_secure` (ограничения на размер, глубину и повторяющиеся ключи), запись через `rust_json::fast`. `prost` не добавляется в браузерный WASM (размер вырос бы).
- **Исключения (форматы, заданные чужим стандартом, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и т. п. Они принимают обычный HTTP, как требует стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Этот раздел фиксирует политику и не означает, что код уже изменён.
