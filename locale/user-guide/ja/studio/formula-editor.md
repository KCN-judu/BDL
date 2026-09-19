<!-- scripts/docs_l10n.py が docs/user-guide/studio/formula-editor.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/studio/formula-editor.md) · [简体中文](../../zh_Hans/studio/formula-editor.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# 数式エディタ

関係のインスペクターの**関係**セクションが数式を書く場所です。ただのテキストボックスではありません。コンパイラは入力中の内容をその場で検査し、あなたが確定するまで何も設計には届きません。

エディタには同じ数式の 2 つのビューがあり、上部の**数式 | テキスト**スイッチで選びます。**数式**は式を組み立てる部品——参照、単位付きの数、演算子、関数——として示し、各空スロットが何を期待するかを教えます。**テキスト**は書かれたままの数式です。切り替えは数式に何もしません。両方が 1 つの下書きを編集し、一方で組み立てたものが他方で読めます。

## 数式を組み立てる

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula as components — a Tilt chip, a division sign and a dashed empty slot with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, and a folded Equations row.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_分母のスロットを選択した数式ビュー：コンパイラはスロットが角度を期待することとその理由を述べ、角度の単位付きの数値、適合する参照、結果が適合する方程式を提示する。_

空の数式は 1 つの**スロット**——`?` と書かれた破線の箱、値がまだ書かれていない場所——です。スロットをクリックすると、コンパイラはそこに何を期待するかを述べ、適合するものを提示します：

- **数。** 入力し、ポップアップから単位を選び、Return か**挿入**を押します。ポップアップにはスロットが期待する種類の値の単位だけが並びます——角度なら _rad_、_deg_、_turn_。長さや時間は決して出ません。無次元の数を期待するスロットには選ぶ単位がありません。
- **参照。** この関係が読み取るコンセプトと、値が適切な種類である設計の関係——それぞれが生成するものとともに。参照はそのまま挿入されます。種類は宣言から来て、単位のポップアップは付きません。値——`tilt` のような入力元や計算された値——は名前だけで書きます（`tilt` であって、決して `tilt()` ではありません）。規則は引数を付けて適用します（`dimByTilt(?)`）。
- **方程式。** _方程式_ の下に折りたたまれています。結果がここで期待される値になりうるライブラリの方程式です（角度なら `min`、`max`、`clamp`、`sum` …。真偽を生成する `any` は除く）。1 つ選ぶと、引数ごとにスロット付きで挿入されます。
- **A truth value.** Where the slot expects true or false — the side of an `and`, a condition, a concept with the On / off value form — there is no number to type: two buttons, **true** and **false**, stand in its place, and _References_ lists the values that are true or false, the concepts this relationship reads first.
- **A form.** **Choose** opens a choice in the slot, `if ? then ? else ?`; on a slot that expects true or false, **not** opens a negation, `not ?`.

Click a part that is already there and the compiler says what it is. Above it, the actions on that part: **+ − × ÷** put that operator after it with a new slot for the other side, **and** / **or** likewise for a part that is (or may be) true or false, **not** negates that part in place (no new slot), **Compare** puts `<`, `==` and the rest after it, **Function** wraps it in an equation (`clamp(…, ?, ?)`), **Each element** reads a collection element by element (`all reading in readings: ?` — the editor picks a readable name for the element, `reading` for `readings`, `item` otherwise), **Range** asks whether the value lies between two ends (`… in ? .. ?`), **Choose** makes the part one outcome of a choice (`if ? then … else ?`, the condition selected next), and **Remove** turns it back into a slot — removing the slot next to an operator removes the operator with it, and an empty `not ?` goes with its slot. Parentheses are added where the operators need them: a sum divided by something becomes `(a + b) / ?`, an `or` under an `and` becomes `(a or b) and ?`, and a choice under anything is `(if … then … else …)`.

フィールドの下の行——_期待：角度。角度 ÷ 角度 = 無次元量だから。_——はコンパイラの推論を平易な言葉で述べたものです。スロットが何であるべきかを周囲から導きます。関係が生成しなければならない結果と、演算子の反対側です。速度の `? / 1 s` は長さを期待し、トルクの `Force * ?` は長さを期待します。スロットの周囲がまだ何も分からないとき——2 つのスロットの積——はそう述べ、単位を提示しません。先に反対側を埋めてください。**説明**は同じことをコンパイラの記法で示します。

**単位付きの数**は 2 つのフィールド、数とその単位です。数を編集すると同じ単位の新しい量になります。ポップアップから別の単位を選ぶと量は保たれ、数が書き換えられます。`180 deg` は `3.141592653589793 rad` になります。この 2 つは別のことであり、ポップアップは決して前者を行いません。

A choice is drawn the way it reads: `if` and its condition on one line, `then` and `else` with their outcomes indented under it. Each part is an ordinary component: the condition expects true or false, and both outcomes expect what the choice must give — the relationship's result at the top, or, inside a larger formula, whatever the other outcome already is. The logical operators are shown as the words **and**, **or** and **not** (`&&`, `||` and `!` in the text), in the weight of the language's own words.

コレクションに対する数式は読むとおりに描かれます。`all reading in readings:` が 1 行、その下に条件がインデントされます。要素の名前は現れる場所すべてでイタリックです——それはこの数式に属し設計には属さないので、コンセプトの名前変更は決してそれに触れません——そして選択すると 1 つの要素が何かを述べます。範囲は `..` を挟む両端です。各端は `in` の前の値と同じ種類を期待するので、その単位ポップアップにはその種類の単位が並びます。

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads exactly so in the **Text** view, and a formula typed as text appears in the **Formula** view — with `?` wherever text left a slot; `all reading in readings: reading < limit` and `if RoomTemp > 299.15 K && ButtonHeld then true else false` typed as text come back as the same words, every part selectable. Some forms — `match`, a block with `let`, a rule `x => …`, a collection or grouped literal, `delay` / `sync` — are shown as text in the Formula view and edited in the Text view. Text that cannot be read as a formula keeps exactly what you typed; the Formula view shows no parts for it, says _The text cannot be read as a formula._ and offers **Edit as text**. After any change the Formula view waits for the compiler's reading of the new text — _Waiting for the compiler to read the formula…_, the parts dimmed — before it offers the next action, so nothing you click ever acts on text that has already changed.

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
| --------------------------- | ----------------------------------------------------------------------------------------------------- |
| **⌘↩** | 定義を追加 / 保存 |
| **Esc** | 下書きを元に戻す（補完が開いているときは最初の Esc がそれを閉じる） |
| **Return** | テキストビューでは改行——数式は複数行にまたがれる。数値入力では数を挿入 |
| **⌃Space** | 補完を開く（テキストビュー） |
| **↑ / ↓**、**Return / Tab** | 補完リスト内を移動し、確定する |
| **Tab** | 数式ビューで次の部品へ移動 |
| **+ − \* /**, **< >** | on a selected part in the Formula view: put that operator after it, with a slot for the other side |
| **=**, **&**, **\|** | likewise `==`, `&&` (and), `\|\|` (or); `<=`, `>=` and `!=` are in the **Compare** pop-up |
| **!** | negate the selected part in place (`not …`) |
| **⌫** | remove the selected part (an empty slot takes its operator with it) |
| **⌘S** | _プロジェクトを保存_——下書きは決して保存しない。未追加の数式と未保存のプロジェクトは別の状態 |

## 補完とホバー

テキストビューでは **⌃Space** がキャレット位置にリストを開きます。スコープ内のコンセプト、設計の関係（規則には `(` が付く）、数の後の単位、キーワード、そして許される場所での `delay(…)` / `sync(…)` です。リストはコンパイラのもので、コンパイラが絞り込み順序付けします。各行は種類と結果の型を示します。開いている間、キー入力のたびに再び問い合わせます。

名前の上にポインタをしばらく置くと**カード**が表示されます。その名前が何か、値の形式、状態、説明です。形式的な用語は決して出ません——それらは _説明_ にあります。

## 検査されるもの

次元（単位が合わなければならない）、意味（結果はシグネチャが約束するコンセプトでなければならない）、形（値が期待される場所の規則、規則のように適用された値、引数の数が違う呼び出し）、名前（未知、曖昧、あるいはこの関係が読み取らないコンセプトへの言及）、そして `delay` / `sync` の配置。ドメインをまたぐタイミングと瞬時サイクルは、設計の残りに依存するため、数式が保存されてから検査されます。タイミングの検出項目は**タイミング**の下、接続の検出項目は**駆動先**の下、残り（例えば瞬時サイクル）はエディタの下の**関係**セクションに——すべて関係する関係の上に現れます。

## 関連

[関係](../../../../docs/user-guide/concepts/relationships.md) · [数式言語](../../../../docs/user-guide/reference/formula-language.md) · [型・単位・コンセプト](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)
