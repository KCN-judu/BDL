<!-- scripts/docs_l10n.py が docs/user-guide/reference/keyboard-and-mouse.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/reference/keyboard-and-mouse.md) · [简体中文](../../zh_Hans/reference/keyboard-and-mouse.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# キーボードとマウス

ショートカットは macOS 向けに列挙しています。Windows ではインターフェースに同じキーが Ctrl で表示されますが、バインディングはまだ有効ではありません——ツールバーとメニューを使ってください。

## アプリケーション

| キー | 動作 |
| --- | --- |
| ⌘S | プロジェクトを保存——未完成の数式やビルドできないテキストも含め、すべてをそのまま |
| ⌘W | プロジェクトを閉じる——編集済みなら _変更を保存しますか？_ と尋ねる |
| ⌘Q | 終了——先に同じ質問 |
| ⌘Z / ⇧⌘Z | 取り消し / やり直し——設計の編集と振る舞いグループの編集、1 つの履歴 |
| ⌘1 · ⌘2 · ⌘3 · ⌘4 | 設計 · シミュレート · デプロイ · モニター |

プロジェクトマネージャーの _新規プロジェクト…_（⌘N）と _プロジェクトを開く…_（⌘O）はショートカットを表示しますが、現在はクリックで操作します。

## キャンバス

| 操作 | 結果 |
| --- | --- |
| 空のキャンバスを左 → 右にドラッグ | 完全に囲まれたノードを選択（ウィンドウ） |
| 空のキャンバスを右 → 左にドラッグ | 囲まれた、または触れたノードを選択（交差） |
| ⌘-ドラッグ / ⇧-ドラッグで矩形 | その中のノードを選択に加える / から外す |
| 中ボタンドラッグ · Space + ドラッグ · トラックパッドの 2 本指 | パン |
| スクロールホイール · ピンチ · ⌘ + 2 本指 | ポインタを中心にズーム |
| Home · ⌘0 | 全体を表示 |
| クリック | 1 つを選択 |
| ⌘-クリック（Windows と Linux では Ctrl） | 選択に加える / から外す |
| ⇧-クリック | アクティブなノードからの接続の連鎖を選択（それが 1 本だけあるとき） |
| ⌘A | 表示中のすべてのノードを選択 |
| Esc | 進行中の操作をキャンセル。その後は選択を解除 |
| ← → ↑ ↓（⇧：1 ポイント） | 選択を少し動かす |
| ノードをドラッグ | 選択を移動（レイアウトのみ） |
| コンセプトの出力ソケットを出力へドラッグ | それを駆動する関係を接続 |
| ソケット → ソケットへドラッグ | リンク |
| 接続済みの入力ソケットを引き離す → 空のキャンバス | 切断 |
| ⌫ / Delete | 選択を削除（または選択中のバインディングリンクを切断） |
| コンセプトまたは関係をダブルクリック | その場で名前を変更 |
| インスタンスをダブルクリック | そのコンポーネントのソースを開く |
| 振る舞いのタイトルをダブルクリック | 名前を変更 |
| 右クリック · Control-クリック | コンテキストメニュー |
| ライブラリの行をキャンバスへドラッグ | コンセプトを挿入 |

## 数式フィールド

| キー | 動作 |
| --- | --- |
| ⌘↩ | 定義を追加 / 保存 |
| Esc | 下書きを元に戻す。補完が開いていれば先にそれを閉じる |
| Return | 改行 |
| ⌃Space | completion (in the Formula view it also opens as you type a name) |
| ↑ / ↓ · Return / Tab | 補完リスト内を移動 · 確定 |

## Formula view

| キー | 動作 |
| --- | --- |
| ← → | the previous / next place — out of a denominator, past a parenthesis, into the next part |
| ↑ ↓ | the row above / below (a numerator from its denominator, a branch from the next) |
| Home / End | the ends of the enclosing part; again, the ends of the formula |
| Tab / ⇧Tab | the next / previous empty slot |
| ) , | leave the parentheses / the next argument |
| letters, digits | type into the slot or the name or number at the caret; a space after a number starts its unit |
| + − \* / < > = & \| | the operator after the part at the caret, with a slot for the other side |
| ! | negate the part |
| ( | apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot |
| ⌫ / ⌦ | a character, or the whole part beside the caret |
| クリック | place the caret and select the part |

## コードビュー

| キー | 動作 |
| --- | --- |
| ⌃Space | キャレット位置で補完 |
| ↑ / ↓ · Return / Tab · Esc | 補完リスト内を移動 · 確定 · 閉じる |
| 名前の上にポインタを置く | そのカード。入力するか離れると隠れる |
| 名前を ⌘-クリック · F12 | 宣言されている場所へ移動（別のファイルが開く） |
| ⇧F12 | それを名指しするすべての場所を列挙。Esc か × でリストを閉じる |
| ⌥⇧F · _フォーマット_ | ファイルを正規のレイアウトに整える、1 つの編集として |

## インライン名前変更（キャンバス、ライブラリの行）

| キー | 動作 |
| --- | --- |
| Return | 確定 |
| Esc | 元の名前を保つ |
| 他の場所をクリック | 入力した内容を確定 |

## シート

Return で送信、Esc でキャンセル。キャンセルは主ボタンの左にあります。
