<!-- scripts/docs_l10n.py が docs/user-guide/studio/code-view.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/studio/code-view.md) · [简体中文](../../zh_Hans/studio/code-view.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# 設計・コード・分割

設計ページは 1 つのプロジェクトを 3 通りに表示します。キャンバス上部のバーの右端にあるコントロールで切り替えます：

| ビュー | 見えるもの | すること |
| ---------- | ----------------------------------------------- | -------------------------------------------------------------- |
| **設計** | キャンバス（[キャンバス](canvas.md)） | 設計を描く |
| **コード** | エディタ内のプロジェクトの `.bdl` ファイル | 設計を入力する（[構文の基本](../../../../docs/user-guide/textual/syntax-basics.md)） |
| **分割** | 左にキャンバス、右にエディタ | 両方を、同じオブジェクトに対して |

切り替えはプロジェクトの何も変えません。グラフとテキストは同じ設計の 2 つの絵で、Studio のコンパイラサービスが同期を保ちます。_変換_、_インポート_、_エクスポート_ はありません。

## エディタが示すもの

一度に 1 ファイルのテキスト。プロジェクトに複数の `.bdl` ファイルがあるとき、エディタ上部のポップアップがそれらを列挙します。まだビルドできないファイルは _— 未ビルド_ 付きで示されます。

テキストはプロジェクトのファイルそのもので、キャンバスで行ったことがすべて書き込まれています。キャンバスでコンセプトの名前を変えると、テキストは使われているすべての場所で新しい名前を示し、コメントと空行はそのままです。キャンバスで関係を追加すると、`src/main.bdl` の末尾（`main.bdl` がないときは最初のファイルの末尾）、または属するコンポーネント本体の末尾に現れます。

## What the colours mean

![The editor filling the Design page, showing src/main.bdl of the component system. Keywords such as concept, component, mapping and bind are in a quiet grey; concept names like Tilt and Brightness in a blue-grey ink; relationship names in blue; the Source raw in green; the output light in a warm brown; the instances adaptiveLamp and second in teal; comments in a light grey; the number 90 plain with its unit deg in grey. Declared names are in a heavier weight than their uses.](../../../../docs/user-guide/assets/studio/code-view.png)

_The Code view of the component system: the file as the project holds it, coloured by what each word is._

The text is coloured by what each word _is_ to the project — not by how it is spelled. The colours are the canvas's: a **concept** name has the concept nodes' blue-grey, a **relationship** the relationship nodes' blue, a **Source** the green of a Source node, an **output** or a **device** the warm tone of an output node, an **instance** the teal of an instance node. Keywords, operators and units are grey; comments lighter grey. A name where it is _declared_ is heavier than where it is used; a name that exists only inside a formula — a rule's parameter, a binder's variable — is italic; a `?` left in a formula is orange, the same _still to decide_ colour as elsewhere.

Because the colours come from the project, they tell you things spelling cannot: `deg` after `90` is a unit, `deg` as a rule's parameter is not; `all` at the head of `all x in xs: …` is a keyword, a value named `all` is a value; `clamp` is the library's; a relationship turns from Source green to relationship blue the moment it is given a definition. A file that does not build yet keeps its keywords, numbers, comments and units coloured, and the names the last version that built still knows. The colours follow the appearance (light or dark); there is no setting.

## 入力

どのエディタでも同じように入力します。手を止めて少し経つと、ファイルは設計として読まれます：

- **ビルドできる。** キャンバスが合わせて変わります——新しい関係はノードとして現れ、読み取るものの横に自動で配置されます。名前を変えたコンセプトはノード、色、場所、接続を保ちます。項目の同一性はその種類と名前にあり、テキスト中の綴りにはないからです（[同一性](../../../../docs/user-guide/textual/overview.md#identity)）。
- **まだビルドできない。** エディタ上部のバナーが _このファイルはまだビルドできません。設計には最後にビルドできたバージョンが表示されています。_ と述べます。キャンバスはファイルの最後にビルドできたバージョンを、独自のバナー _最後にビルドできたバージョンを表示しています。テキストにはまだビルドできない変更があります。_ とともに表示し続けます。テキストは入力したままです。理由はエディタの下に 1 行ずつ行番号付きで列挙され、クリックするとカーソルがそこへ移ります。**×** は設計が意味しえないもの、**中空の輪**は設計にまだ意味がなく、ファイルのビルドを止めないものです。テキストを直せば両方のバナーが消えます。

入力したものも描いたものも失われません。ビルドできないファイルがグラフを消すことは決してありません。

**取り消し**（⌘Z）は設計への変更を取り消します——テキストで追加した関係はキャンバスで追加したものと同じように取り消され、テキストが追従します。コメントや空白だけに触れる変更はその履歴のステップではありません。

## 分割：1 つの選択

キャンバスでノードを選択するとエディタはその宣言までスクロールします。項目のテキスト内をクリックするとキャンバスでそのノードが選択され、インスペクターに表示されます。選択は両方のビューで同じオブジェクトです。

## 保存

**⌘S** はファイルをエディタに表示されているとおりに書き込み——まだビルドできないファイルも含めて——次にレイアウトとサイドカーを書き込みます（[プロジェクトファイル](../reference/project-files.md)）。プロジェクトを再び開くと、同じテキストが同じバナーとともに、設計は最後にビルドできたバージョンとして表示されます。

## What the editor knows

The editor asks Studio's compiler service the same questions a code editor with the [language server](../../../../docs/user-guide/textual/editor-and-lsp.md) asks, about the text exactly as you have typed it.

**Completion.** Press **⌃Space** and a list opens at the cursor with what can go here, best first: after a `:` the concepts, after an `@` the timing domains, at the start of a line the items allowed there, and inside a formula the inputs, the other relationships — a rule offered as a call, a Source or a value as its name — the names bound in the formula itself, the equations of the library, and after a number the units. ↑ and ↓ move, **Return** or **Tab** accept, **Esc** closes; the list narrows as you type. What is inserted is the service's text, never a guess.

![A pop-up under the caret after brightness() = dimByTilt( in the component body, listing candidates one per row: tiltValue and gain as the body's own values, dimByTilt(Tilt) as a call, then the units and the equations of the library, each with its kind word and the kind of value it gives.](../../../../docs/user-guide/assets/studio/code-completion.png)

_The completion pop-up inside the component's body, after `dimByTilt(`: what can go here, from the compiler service, best first._

**Hover.** Rest the pointer on a name and a card says what it is: its declaration, what it produces, its state, its role (_Source_, _Rule_ or _Value_), the description you wrote. On an equation of the library — `clamp`, `min`, `any` — the card gives its shape and what it does. Over a keyword, a number or a unit there is no card. Typing or moving away hides it.

![A card beside the word dimByTilt in the component body showing the name in bold, the signature mapping dimByTilt : Tilt -> Brightness in monospace, the words type-valid, and a row role: Rule.](../../../../docs/user-guide/assets/studio/code-hover.png)

_The hover card over `dimByTilt` where the component's body applies it: its declaration, its role, its state._

**Go to definition.** **⌘-click** a name, or put the cursor on it and press **F12**, and the editor selects where it is declared — in this file or in another, which opens. Inside a component's source a port's name leads to the port's line, never to an instance's copy.

**References.** **⇧F12** on a name lists, under the editor, every place that names it, across all files, with the file and line; a row takes you there. Esc or the × closes the list. Two concepts with the same value form never share a list: the search is by identity, not by spelling.

**Format.** **⌥⇧F**, or _Format_ at the right of the file bar, lays the file out the canonical way — spacing, indentation, one blank line between items — and applies it as one edit, with the cursor kept on its line. A file that does not parse yet is left exactly as it is; fix it first.

Everything here works on the text as it stands, whether or not it builds: what the last version that built still knows is answered, and what nothing resolves gets no card and no destination, never a guess by spelling.

## 未実装

Findings underlined in the text (the list and the cursor jump stand in); rename from the editor (rename on the canvas, and the text follows); creating a second source file from Studio (make it in a code editor; it appears on reload).

## 関連

[キャンバス](canvas.md) · [テキスト形式の BDL](../../../../docs/user-guide/textual/overview.md) · [プロジェクトをテキストで書く](../../../../docs/user-guide/workflows/authoring-as-text.md) · [ソースファイル](../../../../docs/user-guide/troubleshooting/text-project-errors.md)
