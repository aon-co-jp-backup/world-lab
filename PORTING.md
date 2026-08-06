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
└── docs/
    └── world-laboratory-design.md  # アーキテクチャ設計(2026-08-06に
                                      # dream-osから移設)
```

## 2. VPS/ローカルとの対応 / Local & VPS locations

- ローカル / Local: `F:\runo\world-lab`
- VPS(conoha): `/root/world-lab`(2026-08-06新設、プレースホルダの
  `README.md`のみ、git管理下ではない — created 2026-08-06, placeholder
  `README.md` only, not yet under git)

## 3. `dream-os`との依存関係 / Dependency on `dream-os`

コーディネータ実装が始まった際、`dream-os-kernel`/`dream-os-wire`への
依存方法(path依存 vs. ネットワーク越しの利用)を確定させる必要が
あります。移設する場合は`dream-os`(`../dream-os`、同じ`F:\runo`直下に
cloneされている前提)も同じ相対位置関係でcloneしておくことを推奨します
(実際の依存方式が確定次第、このセクションを更新すること)。

When coordinator work begins, decide how it depends on `dream-os-kernel`/
`dream-os-wire` (path dependency vs. network client). Until then, keep
`dream-os` (`../dream-os`) cloned at the same relative position when
relocating this repo (update this section once the actual dependency
mechanism is settled).
