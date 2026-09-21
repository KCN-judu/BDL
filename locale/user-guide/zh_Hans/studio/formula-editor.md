<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/formula-editor.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/formula-editor.md) · 简体中文 · [日本語](../../ja/studio/formula-editor.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 公式编辑器

关系检查器的**关系**分区是编写公式的地方。它不是普通的文本框：编译器在你输入时就检查你输入的内容，而且在你确认之前，什么都不会进入设计。

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

## 拼装公式

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_选中分母槽位的公式视图：商画成分数；编译器说明该槽位需要一个角度以及原因，并提供带角度单位的数字、匹配的引用以及结果匹配的方程。_

空公式是一个**槽位**——一个写着 `?` 的虚线框，即尚待写入值的位置。点击槽位，编译器会说明它在那里期望什么，并提供合适的选项：

- **一个数。** 输入它，从弹出菜单选择单位，按回车或**插入**。弹出菜单只列出槽位期望的那种值的单位——角度是 _rad_、_deg_、_turn_；绝不会是长度或时间。期望纯数的槽位没有单位可选。
- **一个引用。** 这个关系读取的概念，以及设计中值的种类合适的关系——各自附有它生成的内容。引用按原样插入：它的种类来自其声明，没有单位弹出菜单。值——像 `tilt` 这样的来源，或计算值——只用名字书写（`tilt`，绝不是 `tilt()`）；规则带着参数应用（`dimByTilt(?)`）。
- **一个方程。** 折叠在 _方程_ 之下：库中结果可以是此处期望值的方程（角度可用 `min`、`max`、`clamp`、`sum` …；不包括生成真或假的 `any`）。选择一个会插入它，每个参数一个槽位。
- **一个真值。** 当槽位期望真或假时——`and` 的一侧、一个条件、值形式为开/关的概念——没有数字可输入：两个按钮 **true** 和 **false** 取而代之，_引用_ 列出为真或为假的值，这个关系读取的概念排在最前。
- **一种形式。** **选择**在槽位中打开一个选择 `if ? then ? else ?`；在期望真或假的槽位上，**not** 打开一个否定 `not ?`。

点击已有的部件，编译器会说明它是什么。它上方是针对该部件的操作：**+ − × ÷** 把该运算符放在它后面，并为另一侧添加新槽位；对于是（或可能是）真或假的部件，**and** / **or** 同理；**not** 原地否定该部件（不添加槽位）；**比较**把 `<`、`==` 等放在它后面；**函数**把它包进一个方程（`clamp(…, ?, ?)`）；**每个元素**逐个元素读取集合（`all reading in readings: ?`——编辑器为元素选一个可读的名字，`readings` 对应 `reading`，否则用 `item`）；**范围**询问值是否在两端之间（`… in ? .. ?`）；**选择**把该部件变成一个选择的一个结果（`if ? then … else ?`，接着选中条件）；**移除**把它变回槽位——移除运算符旁的槽位会连同运算符一起移除，空的 `not ?` 也随其槽位一起移除。括号会在运算符需要的地方自动添加：和除以某物变成 `(a + b) / ?`，`and` 下的 `or` 变成 `(a or b) and ?`，任何东西之下的选择都是 `(if … then … else …)`。

字段下方那行——_期望：角度，因为角度 ÷ 角度 = 无量纲量。_——是编译器用平实语言给出的推理。它根据槽位周围的内容推算槽位必须是什么：关系必须生成的结果，以及运算符的另一侧。速度的 `? / 1 s` 期望长度；力矩的 `Force * ?` 期望长度。当槽位周围什么都还不知道时——两个槽位的乘积——它会这样说明并且不提供单位；先填另一侧。**解释**用编译器的记号显示同样的内容。

A **number with a unit** is its number and its unit; a unit made of several — `m per s^2` in the text — is drawn the way it is read, `m/s²`. Selecting the number shows the unit pop-up. Editing the number is a new quantity in the same unit. Choosing another unit from the pop-up keeps the quantity and rewrites the number: `180 deg` becomes `3.141592653589793 rad`. The two are different things, and the pop-up never does the first.

A **division** is drawn as a fraction, the numerator over the denominator; the line between them is the division itself — click it to select the whole quotient. A **choice** is drawn as a branch: `if` and its condition on the spine, `then` and `else` with their outcomes on the rows under it. Each part is an ordinary component: the condition expects true or false, and both outcomes expect what the choice must give — the relationship's result at the top, or, inside a larger formula, whatever the other outcome already is. The logical operators are shown as the words **and**, **or** and **not** (`&&`, `||` and `!` in the text), in the weight of the language's own words. A **`match`** is its subject on the spine and one row per case — the pattern, `⇒`, the outcome; a block with **`let`** is one row per local binding over a line over the result; a rule `x => …` is its parameter, `⇒`, the body; a collection `[…]` or a group `(…)` its items; **`delay`** and **`sync`** a shaded region with a bar on its left, the word and its arguments. Every part reads aloud to a screen reader as what it is — _Tilt over 90 deg_, _a choice: if Held, then 1, else 0_.

作用于集合的公式按其读法绘制：`all reading in readings:` 一行，条件缩进在它下面。元素名在出现的所有地方都是斜体——它属于这个公式，而不是设计，所以重命名概念永远不会影响它——选中它会说明一个元素是什么。范围是 `..` 两侧的两端；每一端期望的种类与 `in` 之前的值相同，因此其单位弹出菜单列出该种类的单位。

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads exactly so in the **Text** view, and a formula typed as text appears in the **Formula** view — with `?` wherever text left a slot; `all reading in readings: reading < limit` and `if RoomTemp > 299.15 K && ButtonHeld then true else false` typed as text come back as the same words, every part selectable. Only an empty group `()` is shown as text. Text that cannot be read as a formula keeps exactly what you typed; the Formula view shows no parts for it, says _The text cannot be read as a formula._ and offers **Edit as text**. After any change the Formula view waits for the compiler's reading of the new text — _Waiting for the compiler to read the formula…_, the parts dimmed — before it offers the next action, so nothing you click ever acts on text that has already changed.

## 文本字段及其结论

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_公式字段中有一个未通过检查的草稿：红色结论行说明 Brightness 是什么，以及公式实际生成的是什么。_

- 空字段中显示的**提示**指出你可以基于什么来写：_基于 Tilt、Held 的表达式_——这个关系读取的概念——或者对于值，_没有输入的表达式_。
- **结论行**：编译器检查时显示 _检查中…_，然后是 _有效的定义_，或者一行橙色表示还有待决定的事（_Tilt 还没有表示形式。_），或者一行红色带有第一个发现项的消息。橙色是 _还没做_；红色是 _现在就错了_。
- **下划线**在字段内标出每个发现项涉及的范围，每个发现项在下方重复出现，附有摘录、解释和可用的修复。上图中整个公式就是那个范围：`Tilt / 90 s` 是角度除以时间，即角速率，而 Brightness 必须是纯数。
- **按钮**：关系还没有公式时是**添加定义**，已有公式时是**保存定义**；文本与已保存内容不同时显示**还原**；**分离定义**移除公式并把关系恢复为 _已声明_。两种视图中它们相同：仍含槽位的公式可以保存，在槽位填满之前它是 _无效的_。

## 草稿

输入会创建一份**草稿**。草稿是项目的一部分：切换选择和切换页面后仍然保留，随项目一起保存（无论是否通过检查，甚至字段为空时也保存），重新打开时回到编辑器中。状态行计数 _N 个定义未添加_；添加它是一次设计改动。画布不会变化——草稿不能让未解决的关系看起来像已定义。

当草稿之下的设计发生变化——你更改了某个概念的单位，或撤销替换了已保存的公式——草稿会针对新设计重新检查；如果你输入期间已保存的公式本身变了，字段会显示一条提示，带有**重新加载**（采用已保存的）或**保留我的**（继续在它之上输入）。什么都不会被悄悄覆盖。

允许保存**无效的**公式：关系变为 _无效_，带有你输入时看到的同一个发现项。设计可以在公式正确之前记录它，正如概念可以在选择值形式之前存在。

## 按键

| 键 | 作用 |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **⌘↩** | 添加 / 保存定义 |
| **Esc** | 还原草稿（补全打开时，第一次 Esc 先关闭它） |
| **Return** | 在文本视图中换行——公式可以跨行；在数字输入中，插入该数 |
| **⌃Space** | open completion (both views; the Formula view also opens it as you type a name) |
| **↑ / ↓**、**Return / Tab** | 在补全列表中移动并接受 |
| **← →** | Formula view: the previous / next place — out of a denominator, past a parenthesis, into the next part |
| **↑ ↓** | Formula view: the row above / below (a numerator from its denominator, a branch from the next) |
| **Home / End** | Formula view: the ends of the enclosing part; again, the ends of the formula |
| **Tab / ⇧Tab** | Formula view: the next / previous empty slot |
| **) ,** | Formula view: leave the parentheses / move to the next argument |
| **letters, digits, space** | Formula view: type into the slot or the name or number at the caret; a space after a number starts its unit |
| **+ − \* /**、**< >** | Formula view: put that operator after the part at the caret (or the selected part), with a slot for the other side |
| **=**、**&**、**\|** | 同理 `==`、`&&`（and）、`\|\|`（or）；`<=`、`>=` 和 `!=` 在**比较**弹出菜单中 |
| **!** | negate the part in place (`not …`) |
| **(** | Formula view: apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot |
| **⌫ / ⌦** | Formula view: a character of a name or number, or the whole part beside the caret (an empty slot takes its operator with it) |
| **⌘S** | _保存项目_——从不保存草稿；未添加的公式和未保存的项目是两种不同的状态 |

## 补全与悬停

**⌃Space** opens a list at the caret — in the Formula view it also opens as you type a name: the concepts in scope, the design's relationships (rules come with a `(`), units after a number, keywords, and `delay(…)` / `sync(…)` where they are allowed. The list is the compiler's, filtered and ordered by it — in a formula, by what the place you are typing in expects: in `? / CycleTime` the lengths come before a speed — and each row shows the kind and the resulting type. It re-asks on every keystroke while open.

把指针在名字上停留片刻会显示一张**卡片**：这个名字是什么、它的值形式、状态、描述。从不显示形式化术语——那些在 _解释_ 中。

## 检查什么

量纲（单位必须算得通）、含义（结果必须是签名承诺的概念）、形状（期望值的地方出现规则、把值当规则应用、参数个数错误的调用）、名字（未知、有歧义，或者提到了这个关系不读取的概念），以及 `delay` / `sync` 的位置。跨域时序和瞬时环路在公式保存后检查，因为它们依赖设计的其余部分；时序发现项出现在**时序**之下，连接发现项在**驱动**之下，其余的（比如瞬时环路）在编辑器下方的**关系**分区——都在它们涉及的关系上。

## 相关

[关系](../../../../docs/user-guide/concepts/relationships.md) · [公式语言](../../../../docs/user-guide/reference/formula-language.md) · [类型、单位与概念](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)
