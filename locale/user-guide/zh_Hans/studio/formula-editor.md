<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/formula-editor.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/formula-editor.md) · 简体中文 · [日本語](../../ja/studio/formula-editor.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 公式编辑器

关系检查器的**关系**分区是编写公式的地方。它不是普通的文本框：编译器在你输入时就检查你输入的内容，而且在你确认之前，什么都不会进入设计。

编辑器对同一个公式有两种视图，用顶部的**公式 | 文本**开关选择。**公式**把表达式显示为你拼装的部件——引用、带单位的数、运算符、函数——并告诉你每个空槽位期望什么。**文本**是写出来的公式。切换对公式没有影响：两者编辑的是同一份草稿，在一个视图里搭建的就是在另一个视图里读到的。

## 拼装公式

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula as components — a Tilt chip, a division sign and a dashed empty slot with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, and a folded Equations row.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_选中分母槽位的公式视图：编译器说明该槽位期望角度及原因，并提供带角度单位的数字、合适的引用，以及结果类型合适的方程。_

空公式是一个**槽位**——一个写着 `?` 的虚线框，即尚待写入值的位置。点击槽位，编译器会说明它在那里期望什么，并提供合适的选项：

- **一个数。** 输入它，从弹出菜单选择单位，按回车或**插入**。弹出菜单只列出槽位期望的那种值的单位——角度是 _rad_、_deg_、_turn_；绝不会是长度或时间。期望纯数的槽位没有单位可选。
- **一个引用。** 这个关系读取的概念，以及设计中值的种类合适的关系——各自附有它生成的内容。引用按原样插入：它的种类来自其声明，没有单位弹出菜单。值——像 `tilt` 这样的来源，或计算值——只用名字书写（`tilt`，绝不是 `tilt()`）；规则带着参数应用（`dimByTilt(?)`）。
- **一个方程。** 折叠在 _方程_ 之下：库中结果可以是此处期望值的方程（角度可用 `min`、`max`、`clamp`、`sum` …；不包括生成真或假的 `any`）。选择一个会插入它，每个参数一个槽位。
- **A truth value.** Where the slot expects true or false — the side of an `and`, a condition, a concept with the On / off value form — there is no number to type: two buttons, **true** and **false**, stand in its place, and _References_ lists the values that are true or false, the concepts this relationship reads first.
- **A form.** **Choose** opens a choice in the slot, `if ? then ? else ?`; on a slot that expects true or false, **not** opens a negation, `not ?`.

Click a part that is already there and the compiler says what it is. Above it, the actions on that part: **+ − × ÷** put that operator after it with a new slot for the other side, **and** / **or** likewise for a part that is (or may be) true or false, **not** negates that part in place (no new slot), **Compare** puts `<`, `==` and the rest after it, **Function** wraps it in an equation (`clamp(…, ?, ?)`), **Each element** reads a collection element by element (`all reading in readings: ?` — the editor picks a readable name for the element, `reading` for `readings`, `item` otherwise), **Range** asks whether the value lies between two ends (`… in ? .. ?`), **Choose** makes the part one outcome of a choice (`if ? then … else ?`, the condition selected next), and **Remove** turns it back into a slot — removing the slot next to an operator removes the operator with it, and an empty `not ?` goes with its slot. Parentheses are added where the operators need them: a sum divided by something becomes `(a + b) / ?`, an `or` under an `and` becomes `(a or b) and ?`, and a choice under anything is `(if … then … else …)`.

字段下方那行——_期望：角度，因为角度 ÷ 角度 = 无量纲量。_——是编译器用平实语言给出的推理。它根据槽位周围的内容推算槽位必须是什么：关系必须生成的结果，以及运算符的另一侧。速度的 `? / 1 s` 期望长度；力矩的 `Force * ?` 期望长度。当槽位周围什么都还不知道时——两个槽位的乘积——它会这样说明并且不提供单位；先填另一侧。**解释**用编译器的记号显示同样的内容。

**带单位的数**是两个字段：数和它的单位。编辑数是同一单位下的一个新量。从弹出菜单选择另一个单位会保持量不变并重写数：`180 deg` 变成 `3.141592653589793 rad`。这两者是不同的事，弹出菜单永远不做前者。

A choice is drawn the way it reads: `if` and its condition on one line, `then` and `else` with their outcomes indented under it. Each part is an ordinary component: the condition expects true or false, and both outcomes expect what the choice must give — the relationship's result at the top, or, inside a larger formula, whatever the other outcome already is. The logical operators are shown as the words **and**, **or** and **not** (`&&`, `||` and `!` in the text), in the weight of the language's own words.

作用于集合的公式按其读法绘制：`all reading in readings:` 一行，条件缩进在它下面。元素名在出现的所有地方都是斜体——它属于这个公式，而不是设计，所以重命名概念永远不会影响它——选中它会说明一个元素是什么。范围是 `..` 两侧的两端；每一端期望的种类与 `in` 之前的值相同，因此其单位弹出菜单列出该种类的单位。

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads exactly so in the **Text** view, and a formula typed as text appears in the **Formula** view — with `?` wherever text left a slot; `all reading in readings: reading < limit` and `if RoomTemp > 299.15 K && ButtonHeld then true else false` typed as text come back as the same words, every part selectable. Some forms — `match`, a block with `let`, a rule `x => …`, a collection or grouped literal, `delay` / `sync` — are shown as text in the Formula view and edited in the Text view. Text that cannot be read as a formula keeps exactly what you typed; the Formula view shows no parts for it, says _The text cannot be read as a formula._ and offers **Edit as text**. After any change the Formula view waits for the compiler's reading of the new text — _Waiting for the compiler to read the formula…_, the parts dimmed — before it offers the next action, so nothing you click ever acts on text that has already changed.

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
| --------------------------- | ----------------------------------------------------------------------------------------------------- |
| **⌘↩** | 添加 / 保存定义 |
| **Esc** | 还原草稿（补全打开时，第一次 Esc 先关闭它） |
| **Return** | 在文本视图中换行——公式可以跨行；在数字输入中，插入该数 |
| **⌃Space** | 打开补全（文本视图） |
| **↑ / ↓**、**Return / Tab** | 在补全列表中移动并接受 |
| **Tab** | 在公式视图中，移到下一个部件 |
| **+ − \* /**, **< >** | on a selected part in the Formula view: put that operator after it, with a slot for the other side |
| **=**, **&**, **\|** | likewise `==`, `&&` (and), `\|\|` (or); `<=`, `>=` and `!=` are in the **Compare** pop-up |
| **!** | negate the selected part in place (`not …`) |
| **⌫** | remove the selected part (an empty slot takes its operator with it) |
| **⌘S** | _保存项目_——从不保存草稿；未添加的公式和未保存的项目是两种不同的状态 |

## 补全与悬停

在文本视图中，**⌃Space** 在光标处打开一个列表：作用域内的概念、设计中的关系（规则带有 `(`）、数字后的单位、关键字，以及在允许的位置的 `delay(…)` / `sync(…)`。列表由编译器提供，并由它过滤和排序；每一行显示种类和结果类型。打开期间每次按键都会重新询问。

把指针在名字上停留片刻会显示一张**卡片**：这个名字是什么、它的值形式、状态、描述。从不显示形式化术语——那些在 _解释_ 中。

## 检查什么

量纲（单位必须算得通）、含义（结果必须是签名承诺的概念）、形状（期望值的地方出现规则、把值当规则应用、参数个数错误的调用）、名字（未知、有歧义，或者提到了这个关系不读取的概念），以及 `delay` / `sync` 的位置。跨域时序和瞬时环路在公式保存后检查，因为它们依赖设计的其余部分；时序发现项出现在**时序**之下，连接发现项在**驱动**之下，其余的（比如瞬时环路）在编辑器下方的**关系**分区——都在它们涉及的关系上。

## 相关

[关系](../../../../docs/user-guide/concepts/relationships.md) · [公式语言](../../../../docs/user-guide/reference/formula-language.md) · [类型、单位与概念](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)
