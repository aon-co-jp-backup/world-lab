# PORTING.md — world-lab お引越しファイル / Porting notes

> 対象バージョン: 構想段階(2026-08-06、`dream-os`から分離直後、
> コーディネータ実装は未着手)。 / Target version: concept stage
> (2026-08-06, just split off from `dream-os`, coordinator not yet
> implemented).

## 0. このリポジトリのスコープ / Scope of this repository

日本語: `world-lab`はBOINC/Folding@home型のボランティア分散
コンピューティング構想のリポジトリです。実行基盤(`dream-os-kernel`)・
通信/永続化層(`dream-os-wire`)は`dream-os`側に実装済みで、
`world-lab`はそれらを利用するコーディネータ層(未実装)を担う想定です。

English: `world-lab` is a concept-stage repository for a BOINC/
Folding@home-style volunteer distributed computing effort. The
execution backend (`dream-os-kernel`) and communication/persistence
layer (`dream-os-wire`) already live in `dream-os`; `world-lab` is
meant to hold the (not yet implemented) coordinator layer that uses
them.

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
