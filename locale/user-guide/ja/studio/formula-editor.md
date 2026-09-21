<!-- scripts/docs_l10n.py が docs/user-guide/studio/formula-editor.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/studio/formula-editor.md) · [简体中文](../../zh_Hans/studio/formula-editor.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# 数式エディタ

関係のインスペクターの**関係**セクションが数式を書く場所です。ただのテキストボックスではありません。コンパイラは入力中の内容をその場で検査し、あなたが確定するまで何も設計には届きません。

The editor has two views of the same formula, chosen with the **Formula | Text** switch at its top. **Formula** shows the expression as the mathematics it is — references, numbers with their units, a fraction for a division, a function and its arguments, a choice as a branch — with a caret you type at and a selected part the buttons beneath act on, and tells you what each empty slot expects. **Text** is the formula as written. Switching does nothing to the formula: both edit one draft, and what you build in one is what you read in the other.

## Typing a formula

The Formula view is written at a **caret**, like a text field, but the caret moves through the formula's parts rather than its characters: before and after each part, inside an empty slot, inside a name or a number, just inside a pair of parentheses. Click where you want to write, or start typing in an empty formula (_Type to write, or choose a part_).

- **Letters and digits** type into the slot or the name or number the caret touches. Typing `Til` opens the completion list at the caret — the concepts this relationship reads, the design's relationships, the equations, ranked by what this place expects — and **Return** or **Tab** takes the highlighted row. A **space after a number** starts its unit: `90` `⎵` `deg`.
- **`+ − * /`, `< >`, `=`, `&`, `|`** put that operator after the part the caret touches, with a slot for the other side; `/` draws a fraction and puts the caret in the denominator. **`!`** negates the part.
- **`(`** after an equation's name applies it: `clamp` becomes `clamp(?, ?, ?)` with the first slot ready to type into. `(` in an empty slot opens a group.
- **← →** move to the previous / next place — out of a denominator, past a parenthesis, into the next part. **↑ ↓** move between the rows of a fraction or a choice. **Home / End** go to the ends of the enclosing part, and again to the ends of the formula. **Tab / ⇧Tab** jump to the next / previous empty slot. **`)`** leaves the parentheses you are in; **`,`** moves to the next argument.
- **⌫ / ⌦** delete a character of a name or a number, or a whole part when the caret is beside one — a slot's operator goes with it.

So `clamp(Tilt / 90 deg, 0, 1)` is typed as `clamp` `(` `Tilt` `/` `90` `⎵deg` `,` `0` `,` `1` — 22 keys, and the completion list would have taken `clamp` after `cl` and `Tilt` after `Ti`. The part you are typing into is shown as text until the compiler has read it — a moment — and the rest of the formula keeps its shape. A key that cannot act where the caret is says why beneath the field (_Type an operator before adding a value here._) and changes nothing.

The pointer works alongside: clicking a part places the caret there _and_ selects the part, so the buttons beneath the field (below) act on it; the caret follows every action to the part that comes next. There is no mode to switch.

## 数式を組み立てる

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_分母のスロットを選択した式ビュー：商は分数として描かれ、コンパイラはそのスロットが角度を期待する理由を示し、角度の単位付きの数値、適合する参照、結果が適合する方程式を提示します。_

空の数式は 1 つの**スロット**——`?` と書かれた破線の箱、値がまだ書かれていない場所——です。スロットをクリックすると、コンパイラはそこに何を期待するかを述べ、適合するものを提示します：

- **数。** 入力し、ポップアップから単位を選び、Return か**挿入**を押します。ポップアップにはスロットが期待する種類の値の単位だけが並びます——角度なら _rad_、_deg_、_turn_。長さや時間は決して出ません。無次元の数を期待するスロットには選ぶ単位がありません。
- **参照。** この関係が読み取るコンセプトと、値が適切な種類である設計の関係——それぞれが生成するものとともに。参照はそのまま挿入されます。種類は宣言から来て、単位のポップアップは付きません。値——`tilt` のような入力元や計算された値——は名前だけで書きます（`tilt` であって、決して `tilt()` ではありません）。規則は引数を付けて適用します（`dimByTilt(?)`）。
- **方程式。** _方程式_ の下に折りたたまれています。結果がここで期待される値になりうるライブラリの方程式です（角度なら `min`、`max`、`clamp`、`sum` …。真偽を生成する `any` は除く）。1 つ選ぶと、引数ごとにスロット付きで挿入されます。
- **真偽値。** スロットが真か偽を期待する場所——`and` の片側、条件、オン/オフの値の形式を持つコンセプト——では入力する数はありません。代わりに **true** と **false** の 2 つのボタンがあり、_参照_ には真か偽である値が、この関係が読み取るコンセプトを先頭に並びます。
- **形。** **選択**はスロットに選択 `if ? then ? else ?` を開きます。真か偽を期待するスロットでは、**not** が否定 `not ?` を開きます。

すでにある部品をクリックすると、コンパイラはそれが何かを述べます。その上に、その部品への操作があります。**+ − × ÷** はその演算子を後ろに置き、反対側に新しいスロットを作ります。真か偽である（かもしれない）部品では **and** / **or** も同様です。**not** はその部品をその場で否定します（新しいスロットなし）。**比較**は `<`、`==` などを後ろに置きます。**関数**は方程式で包みます（`clamp(…, ?, ?)`）。**各要素**はコレクションを要素ごとに読みます（`all reading in readings: ?`——エディタは要素に読みやすい名前を選びます。`readings` なら `reading`、それ以外は `item`）。**範囲**は値が両端の間にあるかを問います（`… in ? .. ?`）。**選択**はその部品を選択の一方の結果にします（`if ? then … else ?`、次に条件が選択されます）。**削除**はスロットに戻します——演算子の隣のスロットを削除すると演算子も一緒に消え、空の `not ?` はそのスロットとともに消えます。括弧は演算子が必要とする場所に追加されます。和を何かで割ると `(a + b) / ?`、`and` の下の `or` は `(a or b) and ?`、何かの下の選択は `(if … then … else …)` になります。

フィールドの下の行——_期待：角度。角度 ÷ 角度 = 無次元量だから。_——はコンパイラの推論を平易な言葉で述べたものです。スロットが何であるべきかを周囲から導きます。関係が生成しなければならない結果と、演算子の反対側です。速度の `? / 1 s` は長さを期待し、トルクの `Force * ?` は長さを期待します。スロットの周囲がまだ何も分からないとき——2 つのスロットの積——はそう述べ、単位を提示しません。先に反対側を埋めてください。**説明**は同じことをコンパイラの記法で示します。

A **number with a unit** is its number and its unit; a unit made of several — `m per s^2` in the text — is drawn the way it is read, `m/s²`. Selecting the number shows the unit pop-up. Editing the number is a new quantity in the same unit. Choosing another unit from the pop-up keeps the quantity and rewrites the number: `180 deg` becomes `3.141592653589793 rad`. The two are different things, and the pop-up never does the first.

A **division** is drawn as a fraction, the numerator over the denominator; the line between them is the division itself — click it to select the whole quotient. A **choice** is drawn as a branch: `if` and its condition on the spine, `then` and `else` with their outcomes on the rows under it. Each part is an ordinary component: the condition expects true or false, and both outcomes expect what the choice must give — the relationship's result at the top, or, inside a larger formula, whatever the other outcome already is. The logical operators are shown as the words **and**, **or** and **not** (`&&`, `||` and `!` in the text), in the weight of the language's own words. A **`match`** is its subject on the spine and one row per case — the pattern, `⇒`, the outcome; a block with **`let`** is one row per local binding over a line over the result; a rule `x => …` is its parameter, `⇒`, the body; a collection `[…]` or a group `(…)` its items; **`delay`** and **`sync`** a shaded region with a bar on its left, the word and its arguments. Every part reads aloud to a screen reader as what it is — _Tilt over 90 deg_, _a choice: if Held, then 1, else 0_.

コレクションに対する数式は読むとおりに描かれます。`all reading in readings:` が 1 行、その下に条件がインデントされます。要素の名前は現れる場所すべてでイタリックです——それはこの数式に属し設計には属さないので、コンセプトの名前変更は決してそれに触れません——そして選択すると 1 つの要素が何かを述べます。範囲は `..` を挟む両端です。各端は `in` の前の値と同じ種類を期待するので、その単位ポップアップにはその種類の単位が並びます。

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads exactly so in the **Text** view, and a formula typed as text appears in the **Formula** view — with `?` wherever text left a slot; `all reading in readings: reading < limit` and `if RoomTemp > 299.15 K && ButtonHeld then true else false` typed as text come back as the same words, every part selectable. Only an empty group `()` is shown as text. Text that cannot be read as a formula keeps exactly what you typed; the Formula view shows no parts for it, says _The text cannot be read as a formula._ and offers **Edit as text**. After any change the Formula view waits for the compiler's reading of the new text — _Waiting for the compiler to read the formula…_, the parts dimmed — before it offers the next action, so nothing you click ever acts on text that has already changed.

## テキストフィールドとその判定

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_検査を通らない下書きのある数式フィールド：赤い判定行は Brightness が何であり、数式が代わりに何を生成するかを述べる。_

- 空のフィールドに表示される**ヒント**は、何をもとに書けるかを示します。_Tilt、Held を使う式_——この関係が読み取るコンセプト——または値なら _入力のない式_ です。
- **判定行**：コンパイラが調べている間は _検査中…_、次に _有効な定義_、あるいはまだ決めるべきことがあればオレンジの行（_Tilt にはまだ表現形式がありません。_）、あるいは最初の検出項目のメッセージを持つ赤い行。オレンジは _まだ_、赤は _今、間違っている_ です。
- **下線**はフィールド内で各検出項目が対象とする範囲を示し、各検出項目は下に抜粋、説明、修正とともに繰り返されます。上の例では数式全体が範囲です。`Tilt / 90 s` は角度を時間で割った角速度であり、Brightness は無次元の数でなければなりません。
- **ボタン**：関係にまだ数式がなければ**定義を追加**、あれば**定義を保存**。テキストが保存済みのものと異なる間は**元に戻す**。**定義を外す**は数式を取り除き、関係を _宣言済み_ に戻します。両方のビューで同じです。スロットが残った数式も保存でき、スロットが埋まるまで _無効_ です。

## 下書き

入力すると**下書き**ができます。下書きはプロジェクトの一部です。選択やページを切り替えても残り、プロジェクトとともに保存され（検査を通るかどうかにかかわらず、フィールドが空でも）、再び開くとエディタに戻ってきます。ステータス行は _未追加の定義 N 件_ を数えます。追加は設計の変更です。キャンバスは変わりません——下書きが未解決の関係を定義済みに見せてはならないからです。

下書きの下で設計が変わったとき——コンセプトの単位を変えた、取り消しが保存済みの数式を置き換えた——下書きは新しい設計に対して再検査され、入力中に保存済みの数式自体が変わった場合、フィールドは**再読み込み**（保存済みのものを取る）か**自分の内容を保持**（その上に入力を続ける）の通知を表示します。何も黙って上書きされません。

**無効な**数式の保存は許されます。関係は入力中に見たのと同じ検出項目を持って _無効_ になります。コンセプトが値の形式を選ぶ前に存在できるのと同じように、設計は数式が正しくなる前にそれを記録できます。

## キー

| キー | 動作 |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **⌘↩** | 定義を追加 / 保存 |
| **Esc** | 下書きを元に戻す（補完が開いているときは最初の Esc がそれを閉じる） |
| **Return** | テキストビューでは改行——数式は複数行にまたがれる。数値入力では数を挿入 |
| **⌃Space** | open completion (both views; the Formula view also opens it as you type a name) |
| **↑ / ↓**、**Return / Tab** | 補完リスト内を移動し、確定する |
| **← →** | Formula view: the previous / next place — out of a denominator, past a parenthesis, into the next part |
| **↑ ↓** | Formula view: the row above / below (a numerator from its denominator, a branch from the next) |
| **Home / End** | Formula view: the ends of the enclosing part; again, the ends of the formula |
| **Tab / ⇧Tab** | Formula view: the next / previous empty slot |
| **) ,** | Formula view: leave the parentheses / move to the next argument |
| **letters, digits, space** | Formula view: type into the slot or the name or number at the caret; a space after a number starts its unit |
| **+ − \* /**、**< >** | Formula view: put that operator after the part at the caret (or the selected part), with a slot for the other side |
| **=**、**&**、**\|** | 同様に `==`、`&&`（and）、`\|\|`（or）。`<=`、`>=`、`!=` は**比較**ポップアップにある |
| **!** | negate the part in place (`not …`) |
| **(** | Formula view: apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot |
| **⌫ / ⌦** | Formula view: a character of a name or number, or the whole part beside the caret (an empty slot takes its operator with it) |
| **⌘S** | _プロジェクトを保存_——下書きは決して保存しない。未追加の数式と未保存のプロジェクトは別の状態 |

## 補完とホバー

**⌃Space** opens a list at the caret — in the Formula view it also opens as you type a name: the concepts in scope, the design's relationships (rules come with a `(`), units after a number, keywords, and `delay(…)` / `sync(…)` where they are allowed. The list is the compiler's, filtered and ordered by it — in a formula, by what the place you are typing in expects: in `? / CycleTime` the lengths come before a speed — and each row shows the kind and the resulting type. It re-asks on every keystroke while open.

名前の上にポインタをしばらく置くと**カード**が表示されます。その名前が何か、値の形式、状態、説明です。形式的な用語は決して出ません——それらは _説明_ にあります。

## 検査されるもの

次元（単位が合わなければならない）、意味（結果はシグネチャが約束するコンセプトでなければならない）、形（値が期待される場所の規則、規則のように適用された値、引数の数が違う呼び出し）、名前（未知、曖昧、あるいはこの関係が読み取らないコンセプトへの言及）、そして `delay` / `sync` の配置。ドメインをまたぐタイミングと瞬時サイクルは、設計の残りに依存するため、数式が保存されてから検査されます。タイミングの検出項目は**タイミング**の下、接続の検出項目は**駆動先**の下、残り（例えば瞬時サイクル）はエディタの下の**関係**セクションに——すべて関係する関係の上に現れます。

## 関連

[関係](../../../../docs/user-guide/concepts/relationships.md) · [数式言語](../../../../docs/user-guide/reference/formula-language.md) · [型・単位・コンセプト](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)
