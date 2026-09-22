<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/first-behavior.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/first-behavior.md) · 简体中文 · [日本語](../../ja/getting-started/first-behavior.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 你的第一个行为

**目标。** 一盏台灯，亮度随灯头倾斜的角度变化：竖直时熄灭，放平时最亮。完成后你会得到一个设计，包含两个概念、两个由它们生成的块、一条规则、一个时序域和一个物理输出——已检查、已保存，可以[仿真](first-simulation.md)。

**用时。** 大约二十分钟。

每个想法都会在设计需要它时出现。之后 [概念](../../../../docs/user-guide/concepts/concepts.md) 页面会正式解释每一个。

## 1. 创建项目

1. 启动 Studio（[安装与启动](install-and-launch.md)）。
2. 点击**新建项目…**，选择位置，把文件夹命名为 `lamp`。

工作区在**设计**页打开：左侧是侧栏，中间是空白画布，右侧是检查器，底部是状态行和页面栏。

## 2. 添加两个概念

A **concept** is a _kind_ of value with a meaning — a type, and the template the blocks on the canvas are made from; a **block** of it is one such value in your design, one per tick. The lamp has two kinds of value: how far it is tilted, and how bright it is.

1. 在侧栏的**项目**标签页中，点击 _概念_ 旁的 **+**。
2. In the _New concept_ sheet: under _Value_ choose **Angle** — the _Measured in_ row lists the units an angle is written in (`rad`, `deg`, `turn`); that is a fact, not a choice. Name it `Tilt`. Click **Create Concept**.
3. Again **+** next to _Concepts_: Value **Level** (a plain number with no unit), Name `Brightness`. **Create Concept**.

两个概念出现在侧栏里，而不是画布上：概念是模板，它的**块**才是持有值的东西。表单预览中的圆形表示 _物理量_；颜色是这个概念专属的，会标记每一个承载 _该_ 概念的插口。Brightness 是没有单位的物理量：0 到 1 之间的程度。

> 如果插口不是实心而是空心圆环，表示值的形式尚未选择——这是允许的，指南后面会再谈到。

**你做了什么。** 两个有名字的含义。还没有任何数字。侧栏的**库**标签页提供现成的概念（_Tilt_ 和 _Brightness_ 都在其中）。

## 3. 在画布上各放一个块

**块**是某个概念的一个值，每个 tick 更新一次。这盏灯需要一个倾斜和一个亮度。

1. 右键点击空白画布 → **添加块 ▸** → **Tilt 的块**。名为 `tilt` 的块落在你点击的位置。
2. 再次右键 → **添加块 ▸** → **Brightness 的块**：得到块 `brightness`。

每个块右侧有一个输出插口，颜色是其概念的颜色；标题带有 _来源_ 一词，左边缘有一条竖线：没有公式的块由**外部提供**——环境、传感器——直到你说明它如何计算为止。这不是错误。你可以在这里停下，保存，明天再来。

![Two blocks one above the other, each with a bar at its left edge, an entry arrow and the word Source in its header and one output socket on the right: tilt with a socket labelled Tilt, and brightness with a socket labelled Brightness; no link joins them yet.](../../../../docs/user-guide/assets/getting-started/declared-relationship.png)

_公式写出之前的两个块：tilt，以及仍由外部提供的 brightness。_

（右键点击画布并选择**添加块 ▸ 新概念 ▸**，会在你点击的位置一次创建一个概念 _以及_ 它的一个块；把库中的一行拖到画布上效果相同。）

## 4. 写出规则

亮度如何由倾斜得出？这是一条**规则**：读取一个概念并生成另一个概念的关系。规则也是模板——它在块的公式里被应用，本身并不是画布上的块。

1. 点击侧栏中 _映射_ 旁的 **+**（面板标题是 _新建映射_；指南中说 _关系_——它们是同一回事）。
2. 名称 `dimByTilt`。在 _读取_ 下打开 **Tilt**。在 _生成_ 下选择 **Brightness**。点击**创建**。

`dimByTilt` 出现在侧栏的 _映射_ 列表中并被选中，检查器显示它：_规则_，读取 Tilt，生成 Brightness，尚无公式。

1. 在**关系**区，编辑器以**公式**视图打开：一个空槽位 `?`，以及文字 _生成 Brightness_。点击槽位。它下方的 _引用_ 列出了 **Tilt**——点击它。槽位变为 `Tilt`。
2. 点击 `Tilt` 并按 **÷**。公式显示为 `Tilt ÷ ?`，新槽位被选中：_期望：角度，因为角度 ÷ 角度 = 无量纲量。_
3. 在数字输入框中输入 `90`，从单位弹出菜单中选择 **deg**（只提供角度单位），按回车。公式显示为 `Tilt ÷ 90 deg`，字段下方一行显示 _有效的定义_。

如果你更愿意打字，切换到**文本**并以文本形式写：

   ```text
   Tilt / 90 deg
   ```

两个视图编辑的是同一个公式。按 **⌘↩** 或点击**添加定义**。

![The Relationship section of the inspector in Formula view: a Formula | Text switch with an Edit… button at its right, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_选中分母槽位的公式视图：商画成分数；编译器说明该槽位需要一个角度以及原因，并提供带角度单位的数字、匹配的引用以及结果匹配的方程。_

**你做了什么。** 一条规则：亮度等于倾斜除以九十度。在**文本**视图里试试：把 `90 deg` 改成 `90 s`，看字段下方那一行变红：它说明 Brightness 是什么、而公式实际生成的又是什么——角度除以时间不是一个普通数字。把 `deg` 改回来。每个公式都会这样检查单位和含义，无论是键入还是拼装——而在你按下 _添加定义_ 之前，什么都不会保存到设计中。

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_公式字段中有一个未通过检查的草稿：红色结论行说明 Brightness 是什么，以及公式实际生成的是什么。_

> 在文本视图中，**⌃Space** 打开补全：此处可用的名字（_Tilt_）、数字后的单位、关键字。把鼠标在名字上停留片刻会显示它是什么。公式视图什么都不需要你记：每个槽位都列出适合的内容。

## 5. 倾斜来自环境

倾斜从哪里来？来自环境——设计之外的传感器。在 BDL 中，环境提供的值是**来源**：**没有公式**的块。`tilt` 已经是一个来源：标题写着 _来源_，左边缘有竖线，没有输入插口。什么都不缺。在仿真器中你将键入它的值；在设备上由传感器提供。

## 6. 计算灯的亮度

灯实际显示的值，是把规则应用到倾斜上得到的。这就是 `brightness` 的公式。

1. 点击 `brightness` 块。检查器显示它。
2. 输入公式 `dimByTilt(tilt)`——在公式视图中点击槽位，在 _引用_ 下选择 **dimByTilt**（它变成 `dimByTilt(?)`），再点击新槽位并选择 **tilt**；或者在文本视图中键入。_添加定义。_

补全会提供 `dimByTilt(`，因为它是规则；也提供 `tilt`，因为它是块。画布上，`brightness` 旁出现一个**映射块**——公式画成的节点：标题写着它应用的规则（`dimByTilt`），左侧有一个标着 `tilt` 的输入插槽——公式读取的每个块各一个——一条连线从 `tilt` 的输出插槽接入；一条短连线把它接到它所定义的块 `brightness`。`brightness` 的标题不再写 _Source_：这个值是算出来的。规则名在公式里，在应用它的地方；规则本身留在侧栏。

> 也可以拖拽连线：把 `tilt` 的输出插槽放到一个还没有公式的块上，或放到某个映射块中一个空缺位置的空心 `?` 插槽上，公式就得到这个名字。画布画的正是公式所说的——公式是把块组合起来的唯一地方。

**你做了什么。** 两个块——一个来源（`tilt`）和一个计算值（`brightness`）——以及该值所应用的一条规则（`dimByTilt`）。底部的状态行计为三个关系和一个来源。

## 7. 给值一个节奏

来自外部的值有节奏：传感器每隔一段时间报告一次。在 BDL 中，这个节奏是一个有名字的**时序域**，每个自行更新的值都属于某一个。

1. 点击 _时序域_ 旁的 **+**：命名为 `interaction`。创建。
2. 选中 `tilt`。在检查器的**时序**区，把**更新于**设为 _interaction_。
3. 选中 `brightness`，做同样的事。

`dimByTilt` 保持在 _任意时序域_：它是纯规则，跟随应用它的那个块的节奏。域名会安静地出现在两个块的右边缘。

**你做了什么。** 一个把“这个什么时候更新”作为显式决定的设计。域是名字，不是速率——_interaction_ 多久走一拍是在仿真或部署时选择的，不在这里。

## 8. 添加灯

Brightness 是设计内部的值。灯本身是一个**物理输出**：值离开设计、进入世界的地方。

1. 点击 _输出_ 旁的 **+**。在 _新建输出_ 面板中：名称 `light`，**接受** Brightness，**更新于** interaction，打开**必需**。创建。

画布右边缘出现一个汇节点，以虚线绘制：它有域，但还没有东西驱动它。状态行显示 _输出未完成_。

1. 从 `brightness` 的输出插口拖到接收端的插口上。（或者选中输出，在其检查器的**连接**弹出菜单中选择 `brightness`；弹出菜单会标出规则——_有输入_ 的关系——它们不能驱动输出。）

接收端变为实线。只有在同一时序域中生成所接受概念的块才能驱动输出：`brightness` 符合，`tilt` 不符合（概念不对，插口会拒绝），而规则没有值可给。

**你做了什么。** 一个完整的设计。状态行不再显示 _输出未完成_，也没有任何 _尚未定义_ 的项：来源 `tilt` 本来就应该没有公式，状态行把它计为 _1 个来源_。按 **⌘S** 保存。

## 你现在拥有的

![The canvas left to right: the Source block tilt (a bar at its left edge, an entry arrow and the word Source in its header, one output socket labelled Tilt), a link from it into the mapping block dimByTilt, whose left socket is labelled tilt and whose formula line reads dimByTilt(tilt), a short link from its output socket into the block brightness, whose socket is labelled Brightness, and a link from brightness to the light sink at the right; tilt, brightness and the output carry the domain name interaction.](../../../../docs/user-guide/assets/getting-started/complete-lamp.png)

_完成的灯：来源 tilt、应用规则 dimByTilt 的映射块、它定义的块 brightness，以及被驱动的 light。_

| 对象 | 种类 | 公式 | 更新于 |
| -------------- | ----------------------------------------------- | ----------------- | ------------- |
| **Tilt** | 概念（模板），角度 |  |  |
| **Brightness** | 概念（模板），普通数字 |  |  |
| **tilt** | Tilt 的块，无公式：来源 | _无_ | _interaction_ |
| **dimByTilt** | 读取 Tilt、生成 Brightness 的规则 | `Tilt / 90 deg` | 任意 |
| **brightness** | Brightness 的块，计算得出：应用该规则 | `dimByTilt(tilt)` | _interaction_ |
| **light** | 由 brightness 驱动的物理输出 |  | _interaction_ |

这个设计是 _可执行的_：输出所依赖的每个块要么是计算得出、要么是来源，都通过检查、都有节奏，而输出恰好有一个驱动者。

## 如果有什么不对

- **公式行是红色的。** 读一读：它用你的概念来说明问题（“这把角度除以了时间”）。见 [类型、单位与概念](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)。
- **`brightness` 连不上输出。** 检查它的 _更新于_ 是否与输出一致，以及它是否不读取任何东西。见 [连接](../../../../docs/user-guide/troubleshooting/connection-errors.md)。
- **状态行显示 _N 个定义未添加_。** 公式已键入但未添加；选中该块并按 _添加定义_ 或 _还原_。
- **什么都不检查，底部一行显示 _编译器未连接_。** 见 [安装与启动](install-and-launch.md)。

## 下一步

[你的第一次仿真](first-simulation.md)——倾斜灯，观察亮度。
