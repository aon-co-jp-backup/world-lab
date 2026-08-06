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

## 現状(2026-08-06、リポジトリ分離直後)

構想・設計文書のみ移設した段階。コーディネータ本体・ワークユニット
配布ロジックの実装はまだ無い(`dream-os`側の`dream-os-wire`/
`dream-os-kernel`が提供する部品を、このリポジトリからどう呼び出すか
〈path依存/HTTPクライアント等〉は次回設計)。

## HANDOFF(直近の作業ログ、上が最新)

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
