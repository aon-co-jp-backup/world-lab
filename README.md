# World Laboratory (world-lab)

**構想段階のプロジェクトです。** 世界中の個人が持つPC・タブレット・
スマホの余剰計算資源(GPU/NPU/CPU)を、AI(LLM推論・学習・データ処理等)の
ワークロードへボランティア提供できる分散基盤——BOINC/Folding@home型の
ボランティアコンピューティングを、個人では持てない研究所/大学/メーカー
規模のAI開発環境として実現することを目指します。

## 分離の経緯

元々[`dream-os`](https://github.com/aon-co-jp/dream-os)(Linux×TRON
ハイブリッドカーネル構想)のサブ構想として設計・実装が始まりましたが、
「実行基盤(GPU/NPU上でのカーネルディスパッチ)」と「分散コーディネー
ション(世界中のノードを束ねる仕組み)」は性質の異なる別レイヤーである
ため、独立リポジトリとして分離しました(2026-08-06)。

## エコシステム内での位置づけ

- [`dream-os`](https://github.com/aon-co-jp/dream-os) — 実行基盤層
  (Windows/Android共通Vulkan実行基盤、マイニングカーネル、東芝SBM風
  最適化、open-directx連携)+通信・永続化層(AEAD暗号化+リプレイ対策の
  `SecureChannel`、aruaru-db永続化)を提供する下位レイヤー。
- [`open-web-server`](https://github.com/aon-co-jp/open-web-server)/
  [`RPoem`](https://github.com/aon-co-jp/RPoem) — コーディネータ本体の
  実装候補。
- [`aruaru-db`](https://github.com/aon-co-jp/aruaru-db) — 結果の永続化先
  (ACID互換+Git-on-SQLバージョン管理)。
- [`open-cuda`](https://github.com/aon-co-jp/open-cuda) — GPU実行基盤。
- [`open-directx`](https://github.com/aon-co-jp/open-directx) — Windows
  DirectXシェーダー互換層。**現在このエコシステム内で最優先で完成度を
  高めている最中**のプロジェクト。

## 設計文書

詳細なアーキテクチャ設計は
[`docs/world-laboratory-design.md`](docs/world-laboratory-design.md)を
参照してください(4層構成、BOINC等の先行事例調査、ワークユニット設計、
N-of-M多数決検証、Sybil耐性の課題、段階的ロードマップ)。

## 現状

構想・設計文書のみの段階です。コーディネータ本体・ワークユニット配布
ロジックの実装はまだありません(詳細は[`CLAUDE.md`](CLAUDE.md)の
HANDOFF参照)。

## 関連プロジェクト

- [aon-co-jp/dream-os](https://github.com/aon-co-jp/dream-os)
- [aon-co-jp/open-directx](https://github.com/aon-co-jp/open-directx)
- [aon-co-jp/open-cuda](https://github.com/aon-co-jp/open-cuda)
- [aon-co-jp/aruaru-db](https://github.com/aon-co-jp/aruaru-db)
- [aon-co-jp/open-web-server](https://github.com/aon-co-jp/open-web-server)
- [aon-co-jp/RPoem](https://github.com/aon-co-jp/RPoem)
- [aon-co-jp/RUNO](https://github.com/aon-co-jp/RUNO) —
  エコシステム全体のメタ索引

## License

未定(構想段階のため)。
