<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/first-behavior.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/first-behavior.md) · 简体中文 · [日本語](../../ja/getting-started/first-behavior.md)

# 你的第一个行为

**目标。** 一盏台灯，亮度随灯头倾斜的角度变化：竖直时熄灭，放平时全亮。完成时你会得到一个包含两个概念、三个关系、一个时序域和一个物理输出的设计——已检查、已保存，可以 [仿真](first-simulation.md)。

**用时。** 大约二十分钟。

每个想法都会在设计需要它时出现。之后 [概念](../../../../docs/user-guide/concepts/concepts.md) 页面会正式解释每一个。

## 1. 创建项目

1. 启动 Studio（[安装与启动](install-and-launch.md)）。
2. 点击**新建项目…**，选择位置，把文件夹命名为 `lamp`。

工作区在**设计**页打开：左侧是侧栏，中间是空画布，右侧是检查器，底部是状态行和页面栏。画布告诉你先做什么：从侧栏添加一个概念。

## 2. 添加两个概念

**概念**是有含义的值。这盏灯有两个：倾斜了多少，以及有多亮。

1. 在侧栏的**项目**标签页中，点击 _概念_ 旁的 **+**。
2. 在 _新建概念_ 面板中：名称填 `Tilt`。在 _值_ 下选择**量**，单位选择**角度**（符号 `rad` 出现在它自己的一列）。点击**创建**。
3. 再次点击 _概念_ 旁的 **+**：名称 `Brightness`，**量**，单位**无单位**。点击**创建**。

每个概念在画布上是一行，两端各有一个圆形插口。圆形表示 _量_；颜色是该概念专属的，凡是用到 _这个_ 概念的地方都会出现。Brightness 是无单位的量：0 到 1 之间的一个水平。

> 面板会随你输入预览这一行。如果是空心环而不是实心插口，表示值形式尚未选择——这是允许的，指南稍后会回到这一点。

**你做了什么。** 两个有名字的含义。还没涉及数字。侧栏的**库**标签页提供现成的概念（_Tilt_ 和 _Brightness_ 都在其中）；把一个拖到画布上等同于填写面板。

## 3. 添加关系

**关系**说明一个概念如何由其他概念得出。

1. 点击侧栏中 _映射_ 旁的 **+**（面板标题是 _新建映射_；指南中说 _关系_——它们是同一回事）。
2. 名称 `dimByTilt`。在 _读取_ 下打开 **Tilt**。在 _生成_ 下选择 **Brightness**。点击**创建**。

出现一个节点，有一个输入插口（左侧的 Tilt）和一个输出插口（右侧的 Brightness）。它以**虚线**绘制，标题里写着 _已声明_：关系存在并有签名，但还没有公式。这不是错误。你可以就此停下、保存，明天再回来。

![Two concept rows, Tilt and Brightness, and between them the relationship node dimByTilt drawn with a dashed outline and the word declared in its header; a link runs from Tilt into the node's input socket and from its hollow output socket to Brightness.](../../../../docs/user-guide/assets/getting-started/declared-relationship.png)

_一个已声明的关系：虚线轮廓，标题中写着“已声明”。它的输出插口是空心的：在有值应用这条规则之前，规则不产生任何值。_

## 4. 写公式

1. 点击 `dimByTilt` 节点。检查器会显示它。
2. 在**关系**区，编辑器以**公式**视图打开：一个空槽位 `?`，以及文字 _生成 Brightness_。点击槽位。它下方的 _引用_ 列出了 **Tilt**——点击它。槽位变为 `Tilt`。
3. 点击 `Tilt` 并按 **÷**。公式显示为 `Tilt ÷ ?`，新槽位被选中：_期望：角度，因为角度 ÷ 角度 = 无量纲量。_
4. 在数字输入框中输入 `90`，从单位弹出菜单中选择 **deg**（只提供角度单位），按回车。公式显示为 `Tilt ÷ 90 deg`，字段下方一行显示 _有效的定义_。

如果你更愿意打字，切换到**文本**并以文本形式写：

   ```text
   Tilt / 90 deg
   ```

两个视图编辑的是同一个公式。按 **⌘↩** 或点击**添加定义**。

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula as components — a Tilt chip, a division sign and a dashed empty slot with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_选中分母槽位的公式视图：编译器说明该槽位期望角度及原因，并提供带角度单位的数字、合适的引用，以及结果类型合适的方程。_

节点现在在主体中显示公式，并以实线绘制。

**你做了什么。** 一条规则：亮度等于倾角除以九十度。在**文本**视图里试试：把 `90 deg` 改成 `90 s`，看字段下方那行变红：它说明 Brightness 是什么，以及公式实际生成的是什么——角度除以时间不是纯数。把 `deg` 改回来。每个公式都会这样检查单位和含义，无论你是打字还是拼装——而且在按下 _添加定义_ 之前，什么都不会保存到设计中。

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_公式字段中有一个未通过检查的草稿：红色结论行说明 Brightness 是什么，以及公式实际生成的是什么。_

> 在文本视图中，**⌃Space** 打开补全：此处可用的名字（_Tilt_）、数字后的单位、关键字。把鼠标在名字上停留片刻会显示它是什么。公式视图什么都不需要你记：每个槽位都列出适合的内容。

## 5. 把倾角从环境引进来

`dimByTilt` 是规则，不是值：它需要一个倾角来处理。倾角从哪里来？来自环境——设计之外的一个传感器。在 BDL 中，由环境提供的值是一个**来源**：一个**不读取任何东西**、生成该概念、并且**没有公式**的关系。

1. 点击 _映射_ 旁的 **+**：名称 `tilt`，不读取任何东西，生成 **Tilt**。创建。（右键点击画布 → **添加来源 ▸** → _新建来源…_ 会在你在那里选择的概念——Tilt——上做出同样的东西，也可以顺带新建一个概念。）

节点的标题写着 _来源_，左边缘有一条竖条，没有输入插口。它不是虚线的：这里没有任何缺失。在仿真器中你将输入它的值；在设备上由传感器提供。

## 6. 计算灯的亮度

灯实际显示的值是把 `dimByTilt` 应用于 `tilt`。这又是一个不读取任何东西、生成 Brightness 的关系——这次带有公式。

1. 点击 _映射_ 旁的 **+**：名称 `brightness`，不读取任何东西，生成 **Brightness**。创建。
2. 选中它并输入公式 `dimByTilt(tilt)`。_添加定义。_

补全会提供 `dimByTilt(`，因为它是带输入的关系；也会提供 `tilt`，因为它是值。公式字段是唯一组合关系的地方；画布上的连线显示关系读取 _哪些概念_，而不是算术。

**你做了什么。** 三个关系：一个来源（`tilt`）、一条规则（`dimByTilt`）和一个计算值（`brightness`）。底部的状态行会计数。

## 7. 给值一个节奏

来自外部的值有节奏：传感器每隔一段时间报告一次。在 BDL 中，这个节奏是一个有名字的**时序域**，每个自行更新的值都属于某一个。

1. 点击 _时序域_ 旁的 **+**：命名为 `interaction`。创建。
2. 选中 `tilt`。在检查器的**时序**区，把**更新于**设为 _interaction_。
3. 选中 `brightness`，做同样的事。

`dimByTilt` 保持在 _任意时序域_：它是纯规则，跟随应用它的那一方的节奏。域的名字安静地出现在两个节点的右边缘。

**你做了什么。** 一个把“这个什么时候更新”作为显式决定的设计。域是名字，不是速率——_interaction_ 多久走一拍是在仿真或部署时选择的，不在这里。

## 8. 添加灯

Brightness 是设计内部的值。灯本身是一个**物理输出**：值离开设计、进入世界的地方。

1. 点击 _输出_ 旁的 **+**。在 _新建输出_ 面板中：名称 `light`，**接受** Brightness，**更新于** interaction，打开**必需**。创建。

画布右边缘出现一个汇节点，以虚线绘制：它有域，但还没有东西驱动它。状态行显示 _输出未完成_。

1. 从 `brightness` 的输出插口拖到汇节点的插口上。（或者选中输出，在其检查器的**连接**弹出菜单中选择 `brightness`；弹出菜单会标记 _有输入_ 的关系，它们不能驱动输出。）

汇节点变为实线。只有 _不读取任何东西_、并在同一域中生成所接受概念的关系才能驱动输出。`brightness` 符合条件；如果改连 `dimByTilt`，编辑会被接受，然后在输出下方被报告为不合适的连接。

**你做了什么。** 一个完整的设计。状态行不再显示 _输出未完成_，也没有任何 _尚未定义_ 的项：来源 `tilt` 本来就应该没有公式，状态行把它计为 _1 个来源_。按 **⌘S** 保存。

## 你现在拥有的

![The canvas with the concept rows Tilt and Brightness at the top, the Source node tilt (a bar at its left edge, an entry arrow and the word Source in its header, no input socket), the relationship nodes dimByTilt and brightness below them, and the light sink at the right, joined by links; tilt, brightness and the output carry the domain name interaction at their right edge.](../../../../docs/user-guide/assets/getting-started/complete-lamp.png)

_完成的灯：来源 tilt、规则 dimByTilt、值 brightness，以及被驱动的 light。_

| 对象 | 种类 | 公式 | 更新于 |
| -------------- | -------------------------------------- | ----------------- | ------------- |
| **Tilt** | 概念，角度 |  |  |
| **Brightness** | 概念，纯数 |  |  |
| **tilt** | 不读取任何东西的关系：来源 | _无_ | _interaction_ |
| **dimByTilt** | 读取 Tilt 的关系：规则 | `Tilt / 90 deg` | 任意 |
| **brightness** | 不读取任何东西的关系：值 | `dimByTilt(tilt)` | _interaction_ |
| **light** | 由 brightness 驱动的物理输出 |  | _interaction_ |

这个设计是 _可执行的_：输出所依赖的每个关系要么已定义要么是来源，通过检查，有节奏，并且输出恰好有一个驱动方。

## 如果有什么不对

- **公式行是红色的。** 读一读：它用你的概念来说明问题（“这把角度除以了时间”）。见 [类型、单位与概念](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md)。
- **`brightness` 连不上输出。** 检查它的 _更新于_ 是否与输出一致，以及它是否不读取任何东西。见 [连接](../../../../docs/user-guide/troubleshooting/connection-errors.md)。
- **状态行显示 _N 个定义未添加_。** 公式已输入但未添加；选中节点，按 _添加定义_ 或 _还原_。
- **什么都不检查，底部一行显示 _编译器未连接_。** 见 [安装与启动](install-and-launch.md)。

## 下一步

[你的第一次仿真](first-simulation.md)——倾斜灯，观察亮度。
