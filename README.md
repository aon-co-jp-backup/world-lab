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

## 現状(2026-08-06、コーディネータ本体を実機検証済み)

`crates/world-lab-coordinator`(Rust+tokio/hyper直接実装、Poem非依存)が
ワークユニット配布・結果受信のHTTP APIを提供します。`dream-os-wire`の
`WorkerChannel`/`CoordinatorChannel`(AEAD暗号化+リプレイ対策)を実際に
使い、正常な結果送信・**リプレイ攻撃の拒否**を実プロセス・実HTTP・実暗号
処理で検証済みです。N-of-M多数決検証・ワークユニットの永続化・
ワーカーごとの鍵配布は未実装(詳細は[`CLAUDE.md`](CLAUDE.md)の
HANDOFF参照)。

## エコシステム統合ロードマップ(将来構想、未着手)

ユーザーから、world-labの将来的な発展として、エコシステム内の以下の
リポジトリとの統合を検討すべきというアイデアの共有があった。**現時点
ではいずれも「同梱」(単一インストーラーへの統合等)ではなく、機能的な
関連性に応じた個別の統合ポイントとして整理するに留める**(実装ではなく
構想の記録)。

- **open-easy-web** — 複数のworld-labノード(コーディネータ・ワーカー)
  を横断的に管理するダッシュボードとしての活用が考えられる。
- **RPoem・open-web-server** — 元々コーディネータ本体の実装候補だった
  (現在は独立実装`world-lab-coordinator`を採用済み)。将来、これらの
  Federation Gateway機能(RPoem)やTenantRegistry(open-web-server)を、
  複数コーディネータの束ね役として再利用する可能性がある。
- **open-directx・open-cuda** — ワーカー側の実行基盤(GPU/NPU上での
  計算実行)。`dream-os-kernel`経由の間接依存という位置づけは変わらず。
- **open-raid-z** — 開発ルールの正本(ドキュメント専業)。コード的な
  統合対象ではない。
- **aruaru-db** — 結果・ワークユニットの永続化先(既に実装・利用中)。
- **aruaru-llm** — world-lab上で分散実行されるAIワークロードの一例
  として位置づけられる可能性がある。

**複数WAN/LAN通信の強化についての検討提案 / Suggestion: consider
multi-WAN/LAN communication reinforcement**:

世界中の個人ノードを束ねるworld-labの性質上、単一の通信経路に依存しない
複数WAN/LAN対応の通信基盤([`RS-SmartTCP`](https://github.com/aon-co-jp/RS-SmartTCP)
等)の導入を検討する価値があります。個人宅のネットワーク環境は不安定・
非対称(上りが遅い等)なことが多く、複数経路の集約・冗長化がボランティア
ノードの安定した参加率向上に寄与する可能性があります。

Given world-lab's nature of coordinating individual nodes worldwide, it
may be worth considering a communication foundation that doesn't rely on
a single network path — for example, multi-WAN/LAN support via
[`RS-SmartTCP`](https://github.com/aon-co-jp/RS-SmartTCP). Individual
home network environments are often unstable or asymmetric (e.g., slow
upload speeds), and aggregating/redundifying multiple paths could help
improve the stability and participation rate of volunteer nodes.

## 関連プロジェクト

- [aon-co-jp/dream-os](https://github.com/aon-co-jp/dream-os)
- [aon-co-jp/open-directx](https://github.com/aon-co-jp/open-directx)
- [aon-co-jp/open-cuda](https://github.com/aon-co-jp/open-cuda)
- [aon-co-jp/aruaru-db](https://github.com/aon-co-jp/aruaru-db)
- [aon-co-jp/aruaru-llm](https://github.com/aon-co-jp/aruaru-llm)
- [aon-co-jp/open-web-server](https://github.com/aon-co-jp/open-web-server)
- [aon-co-jp/RPoem](https://github.com/aon-co-jp/RPoem)
- [aon-co-jp/open-easy-web](https://github.com/aon-co-jp/open-easy-web)
- [aon-co-jp/open-raid-z](https://github.com/aon-co-jp/open-raid-z)
- [aon-co-jp/RS-SmartTCP](https://github.com/aon-co-jp/RS-SmartTCP)
- [aon-co-jp/RUNO](https://github.com/aon-co-jp/RUNO) —
  エコシステム全体のメタ索引

## License

未定(構想段階のため)。
