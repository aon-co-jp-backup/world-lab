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
