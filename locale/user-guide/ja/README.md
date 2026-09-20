<!-- scripts/docs_l10n.py が docs/user-guide/README.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../docs/user-guide/README.md) · [简体中文](../zh_Hans/README.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# Behavior Designer — ユーザーガイド

Behavior Designer は、**製品がどう振る舞うか**——何を感知し、何を判断し、何を表示・駆動するか——をファームウェアを書く前に設計するためのデスクトップツールです。振る舞いを、名前の付いた値とそれらの間の関係として記述します。ツールは構築中の設計を検査し、ティックごとにシミュレートし、特定のボードに収まるかどうかを教えてくれます。

ツールの背後にある言語は **BDL** と呼ばれます。キャンバス、インスペクター、数式フィールドはそれを書く一つの方法で、テキスト形式はエディタを好む人のためのもう一つの方法です——[テキスト形式の BDL](../../../docs/user-guide/textual/overview.md) を参照してください。どちらでも、あなたは言語で振る舞いを記述しています。ツールが遠ざけてくれるのは機械の側です。どのピン、どのプロトコル、どの HAL 呼び出しかは、後でデプロイ時に決めます。

## 解決する問題

製品の振る舞いはたいていスケッチや表計算で決められ、その後コードの中で再発見されます。そこでは各数値の意味が失われています。Behavior Designer では値に**意味**があり（_明るさ_ は _不透明度_ ではありません。どちらも 0 から 1 の数であっても）、値の間の関係は単位と次元が検査され、タイミングは明示的で、物理出力は 1 つの関係からしか駆動できません。未完成の設計は普通のことです。数式が分かる前に関係に名前を付けられ、ツールは動作を拒むのではなく、何がまだ決まっていないかを教えてくれます。

## ここでできること

|  |  |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **設計** | 製品のコンセプトに名前を付け、数式で関係づけ、タイミングを決め、結果を物理出力につなぐ |
| **シミュレート** | 値を与え、設計をティックごとに進め、すべての値を観察する——言語を定義するのと同じ評価器で |
| **デプロイ** | ボードを選び、設計の出力をそのピンに配置できるか、できないならなぜかを確認する |
| **整理と再利用** | 関連する関係を振る舞いにまとめ、振る舞いを再利用可能なコンポーネントとしてパッケージ化し、コンポーネントのインスタンスをより大きなシステムに組み合わせる |

まだツールにないもの：ボードへの書き込み、動作中のデバイスからのライブ値、他のプロジェクトからのコンポーネントのインポート。ガイドは必要な箇所でそう明記しています。

## どこから始めるか

| あなたが… | ここから |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| 振る舞いを記述して試したいプロダクトデザイナーや工業デザイナーなら | [BDL とは？](getting-started/what-is-bdl.md) → [最初の振る舞い](getting-started/first-behavior.md) |
| センサーとアクチュエータに詳しい組み込みプロトタイパーなら | [最初の振る舞い](getting-started/first-behavior.md)、続いて [センサーから出力へ](../../../docs/user-guide/workflows/sensor-to-output.md) と [デプロイ](studio/deploy.md) |
| BDL を直接書きたいコード志向のユーザーなら | [テキスト形式の BDL — 概要](../../../docs/user-guide/textual/overview.md)。今日何が動くかを正確に述べています |
| 意味論を知りたい研究者や言語の人なら | 各コンセプトページ末尾の _さらに深く_ の節、および [用語集](../../../docs/user-guide/reference/terminology.md) → `docs/` の技術文書 |

## 目次

**Getting started** [What is BDL?](getting-started/what-is-bdl.md) · [Install and launch](getting-started/install-and-launch.md) · [Your first behavior](getting-started/first-behavior.md) · [Your first simulation](getting-started/first-simulation.md) · [Your first deployment check](getting-started/first-deployment.md) · [Your first board](getting-started/pico-demo.md)

**コンセプト** — 考え方を 1 ページずつ [コンセプト](../../../docs/user-guide/concepts/concepts.md) · [関係](../../../docs/user-guide/concepts/relationships.md) · [未完成の設計](../../../docs/user-guide/concepts/incomplete-designs.md) · [タイミング](../../../docs/user-guide/concepts/timing.md) · [物理出力](../../../docs/user-guide/concepts/physical-outputs.md) · [振る舞いグループ](../../../docs/user-guide/concepts/behavior-groups.md) · [コンポーネント](../../../docs/user-guide/concepts/components.md) · [振る舞いシステム](../../../docs/user-guide/concepts/behavior-systems.md)

**Studio** — インターフェースをパネルやページごとに [ワークスペース](studio/workspace.md) · [キャンバス](studio/canvas.md) · [ライブラリ](../../../docs/user-guide/studio/library.md) · [インスペクター](studio/inspector.md) · [数式エディタ](studio/formula-editor.md) · [シミュレート](studio/simulate.md) · [デプロイ](studio/deploy.md) · [システムプロジェクト](../../../docs/user-guide/studio/system-projects.md) · [設計・コード・分割](studio/code-view.md)

**ワークフロー** — 同じランプで目標を 1 つずつ [センサーから出力へ](../../../docs/user-guide/workflows/sensor-to-output.md) · [2 つ目の出力](../../../docs/user-guide/workflows/multi-output-behavior.md) · [振る舞いをグループ化する](../../../docs/user-guide/workflows/grouping-behavior.md) · [振る舞いをコンポーネントとしてパッケージ化する](../../../docs/user-guide/workflows/package-as-component.md) · [コンポーネントを組み合わせる](../../../docs/user-guide/workflows/composing-components.md) · [タイミングドメインをまたいで値を運ぶ](../../../docs/user-guide/workflows/cross-domain-transport.md) · [コンポーネントのバージョン管理](../../../docs/user-guide/workflows/component-versioning.md) · [プロジェクトをテキストで書く](../../../docs/user-guide/workflows/authoring-as-text.md)

**テキスト形式の BDL** [概要と現状](../../../docs/user-guide/textual/overview.md) · [構文の基本](../../../docs/user-guide/textual/syntax-basics.md) · [エディタと言語サーバー](../../../docs/user-guide/textual/editor-and-lsp.md) · [プロジェクトをテキストで書く](../../../docs/user-guide/workflows/authoring-as-text.md)

**トラブルシューティング** [ここから](../../../docs/user-guide/troubleshooting/README.md) · [未完成の設計](../../../docs/user-guide/troubleshooting/incomplete-design.md) · [型・単位・コンセプト](../../../docs/user-guide/troubleshooting/type-and-concept-errors.md) · [タイミング](../../../docs/user-guide/troubleshooting/timing-errors.md) · [接続](../../../docs/user-guide/troubleshooting/connection-errors.md) · [デプロイ](../../../docs/user-guide/troubleshooting/deployment-errors.md) · [ソースファイル](../../../docs/user-guide/troubleshooting/text-project-errors.md)

**リファレンス** [用語集](../../../docs/user-guide/reference/terminology.md) · [キーボードとマウス](reference/keyboard-and-mouse.md) · [状態の意味](../../../docs/user-guide/reference/status-meanings.md) · [数式言語](../../../docs/user-guide/reference/formula-language.md) · [プロジェクトファイル](reference/project-files.md) · [コマンドライン](../../../docs/user-guide/reference/cli.md)

**このガイドについて** [スタイルガイド](../../../docs/user-guide/STYLE_GUIDE.md) · [検証マトリクス](../../../docs/user-guide/VERIFICATION.md) · [スクリーンショット計画](../../../docs/user-guide/SCREENSHOT_PLAN.md) · [ドキュメント調査](../../../docs/user-guide/DOCUMENTATION_RESEARCH.md)

ツール自体を作る人のための技術文書は、1 つ上の `docs/`（アーキテクチャ、コンパイラパイプライン、プロトコル、形式的な注記）にあります。このガイドは _さらに深く_ の節からそこへリンクし、内容を繰り返しません。
