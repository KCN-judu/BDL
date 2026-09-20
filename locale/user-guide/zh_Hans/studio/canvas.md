<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/canvas.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/canvas.md) · 简体中文 · [日本語](../../ja/studio/canvas.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 画布

画布是设计页的中心：设计结构的一幅图。上面的一切都有含义，而每个视觉通道只表示一件事。

## 标记的含义

| 你看到 | 含义 |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **插口颜色** | _哪个概念_——每个概念有自己的色相，在它出现的所有地方都相同 |
| **插口形状** | 概念的 _值形式_：○ 量、◇ 开/关、□ 计数；值形式为 _稍后决定_ 时是**空心环** |
| **概念颜色的连线，从插槽到插槽** | 签名：_这个关系读取那个概念_（概念 → 关系输入）、_这个关系产出那个概念_（关系 → 概念），或 _这个值驱动那个输出_（关系 → 汇） |
| **止于公式行的细灰线** | 引用：_这个公式提到了那个关系_——一个值应用某条规则，或提到某个值或来源。它两端都不能拖拽；公式在检查器中编辑 |
| **虚线轮廓** | _已声明_：读取了某些东西但没有公式的关系；没有驱动方或没有域的输出 |
| **节点左边缘的竖条、一个进入箭头、_来源_ 一词** | 一个**来源**：由环境提供的值——一个不读取任何东西且没有公式的关系。这里没有任何缺失 |
| **公式行上的红色标记** | 公式未通过检查——_现在就错了_，而不是 _还没做_ |
| **_已声明_、_有争议_、_不合式_、_无域_ 这些词** | 节点可以携带的唯一状态词，只在该状态成立时显示（必须被驱动的输出在此期间显示 _必需_；来源显示 _来源_） |
| **词 _规则_** | 读取了某些概念的关系——一个函数；在某个值的公式应用它之前，它本身没有值。仅在没有状态词占用该位置时显示 |
| **节点右边缘的小名字** | 它的时序域 |
| **强调色** | 选中——仅此而已 |

位置、大小和连线方向**没有含义**。数据从左到右绘制只是为了可读。彩色连线表示签名，灰色连线表示什么依赖什么；两者都不表示运行顺序，也没有按运算符划分的节点——算术在公式字段里。

## 节点

![Concept rows Tilt and Brightness with a round socket at each end; relationship nodes with a name header, one input socket per concept read on the left, one output socket on the right and the formula in the body; dimByTilt, with the word rule in its header, outlined in the accent colour because it is selected; the Source tilt with a bar at its left edge, an entry arrow and the word Source in its header and no input socket; brightness with no input socket and two thin grey links arriving at its formula line from the output sockets of tilt and dimByTilt; the light sink at the right with the word required in its header, a single input socket and a bar at its right edge; tilt, brightness and the output carry the domain name interaction at their right edge.](../../../../docs/user-guide/assets/studio/node-anatomy.png)

_倾角灯上的节点构造：概念行、关系节点（选中了 dimByTilt）和 light 汇。_

**概念**是一行：它的名字，左侧一个输入插口（有东西生成这个概念），右侧一个输出插口（关系从这里读取它）。两个插口都带有概念的颜色和形状。

A **relationship** is a box: a header with the name and, while it applies, the one state word (_declared_ on a relationship that reads something and has no formula yet); one input socket per concept it reads, on the left, each labelled with the concept; one output socket on the right, labelled with the concept it produces; and the formula on the line below, with the timing domain's name at the right edge when the relationship has one. A relationship with a formula carries a small chevron at that line: click it (or **Show Formula** in its menu) and the node unfolds to show the formula the way the [Formula editor](formula-editor.md) draws it — a fraction, a branch, the units — with its first finding beneath and **Edit formula**, which opens the inspector. The unfolded formula is for reading: clicking it selects the node and changes nothing. Which nodes are unfolded is not saved with the project.

![The relationship node dimByTilt with its formula line Tilt / 90 deg and a downward chevron, extended below by a region showing Tilt over a rule over 90 deg and the link Edit formula.](../../../../docs/user-guide/assets/studio/formula-unfolded.png)

_dimByTilt unfolded: the chevron at its formula line turned down, and the saved formula Tilt / 90 deg drawn as a fraction inside the node, with Edit formula beneath._

不读公式也能从方框分辨三种形态。读取了某些概念的关系——上图的 `dimByTilt`——是一条**规则**：它有输入插槽，并且在标题词位置没有被其他词占用时显示 _规则_。它是一个函数；在某个值的公式应用它之前，设计不会用它计算任何东西。不读取任何概念且有公式的关系——`brightness`——是一个**值**：没有输入插槽、没有词，它的公式提到它所依赖的值和规则。每一个被提到的关系都通过一条**引用连线**与它相连：从那个关系的输出插槽到 `brightness` 公式行左端的一条细灰线。引用连线是设计的依赖关系，读自编译器对公式的分析；分析到达时它们才出现，且不能拖拽——改变公式才能改变它们。没有任何公式应用的规则，不会有引用连线通向任何公式行。

**来源**是一个不读取任何东西且没有公式的关系——上图中的 `tilt`：它的值由环境提供，每次激活一次。它的标题写着 _来源_，名字前有一个进入箭头，**左**边缘有一条竖条（那是环境的一侧；设计中没有任何东西喂给它），有一个输出插口，没有输入插口。它不是虚线的，也不是 _已声明_：这里没有任何缺失。在真实产品上由什么提供这个值——传感器、按钮、模拟线路——是部署的事，不是画布上的标记；给来源一个公式，它就变成在设计内部计算的普通关系。

**物理输出**是一个汇：着色的标题带有名字，右侧一个词——设计完成前必须驱动的输出显示 _必需_，或者覆盖它的状态（_无域_、_有争议_、_不合式_）；一个输入插口，标有它接受的概念；右边缘是时序域；右侧一条竖条——没有东西从它流出。

选中的节点（上图的 `dimByTilt`）以强调色描边；画布上没有其他东西使用这个颜色。

组合组件的项目还会显示**实例节点**（每个端口一行，主体中是组件名）和**行为区域**或折叠的**行为框**；见 [系统项目](../../../../docs/user-guide/studio/system-projects.md)。

每个节点都有位置。不是你放置的节点——在代码视图、代码编辑器中创建的，或旧项目首次打开时产生的——会替你放置：放在它所属种类的列中（概念在左，关系在中间，输出在右），紧挨着它读取或生成的东西，在已有内容的下方。你放置的东西不会移动；把它拖到你喜欢的地方（[设计、代码与分栏](code-view.md)）。

## 选择

画布的选择方式与桌面 CAD 工具相同。

| 操作 | 结果 |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| 点击节点 | 单独选中它（检查器随之切换）；点击已在选区中的节点会保留选区，并把它设为当前对象 |
| 点击空白画布 | 清空选区 |
| ⌘-点击（Windows 和 Linux 上为 Ctrl-点击） | 把节点加入选区，或把已在其中的节点移出 |
| ⇧-点击 | 选中从当前对象到这个节点的连接链——当这样的链恰有一条时；否则只把这个节点加入选区 |
| 在空白画布上**从左向右**拖动 | **窗口**：完全落在矩形内的每个节点被选中（实线框） |
| 在空白画布上**从右向左**拖动 | **交叉**：矩形内_或被矩形碰到_的每个节点被选中（虚线框）；向上还是向下拖没有区别 |
| ⌘-拖动 / ⇧-拖动出一个矩形 | 把矩形内的节点加入选区 / 从选区移出（指针旁有 `+` 或 `−`）；松开之前，节点就会显示将要发生的结果 |
| ⌘A | 选中视图中的所有节点 |
| Esc | 取消进行中的操作——框选、移动、连线；没有进行中的操作时，清空选区 |

选中的节点以强调色描边；选中多个时，_当前_对象——你最后点击的那个，检查器最先显示的那个——外面多一圈。项目侧栏的选择方式相同：点击选一行，⌘-点击加入或移出，⇧-点击选中当前行到它之间的所有行。

## 移动与查看

| 操作 | 结果 |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| 拖动选中的节点 | 整个选区一起移动，保持间距；松开后位置被保存（不是设计改动） |
| 拖动未选中的节点 | 它成为选区并移动 |
| 拖动行为的标题条 | 它的关系一起移动 |
| ← → ↑ ↓ | 把选区微移一个网格步长；按住 ⇧ 则移动一点 |
| 中键拖动、Space + 拖动、触控板双指 | 平移 |
| 滚轮、捏合、⌘ + 双指 | 以指针为中心缩放 |
| Home 或 ⌘0 | 显示整个设计 |

## 连接

| 操作 | 结果 |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 从输出插口拖到输入插口 | 建立连线；拖动时，每个可接受它的插口显示光晕，不兼容的插口显示禁止光标 |
| 把**概念**的输出插口拖到接受它的**输出**上 | 把提供该概念的关系连接为输出的驱动方（见下） |
| 从已连接的输入插口拖开，在空白画布上松开 | 断开 |
| 把拖动中的连线放在空白画布上 | 无（不会创建节点） |
| ⌫ / Delete | 删除选区——可一次删除多个对象；若其中某个仍被选区之外的东西使用，则什么也不删除，并由横幅说明原因 |
| 双击概念或关系节点 | 原地重命名（输出在检查器中重命名；实例则打开它的来源） |
| 右键点击，或 Control-点击 | 上下文菜单（见下） |
| 把库标签页中的一行拖到画布上 | open the concept sheet for that category; the concept you name lands at the drop point |

**从概念驱动输出。** 输出由关系驱动——一个值或一个来源，恰好生成输出所接受的概念，且位于输出的时序域——而从不由概念本身驱动。你不必先去找那个关系：把概念的输出插口拖到输出上即可。只有一个关系能驱动它时，立刻连接。有多个时，一个小菜单列出它们的名字（_由 brightness 驱动_、_由 dimmer 驱动_），由你选择；工具不会替你选。没有关系能驱动时，菜单会说明——_Servo 接受 ServoPosition，但当前没有任何关系可以驱动它。_——并把输出交给检查器。已被驱动的输出会得到替换选项（_用 rest 替换 lifted_）：旧的驱动方先放开，新的再连上，中间不会出现输出有两个驱动方的时刻。画布上的连线仍然从驱动关系出发——驱动输出的正是它——输出的上下文菜单也会点名（_显示驱动方：brightness_）。

<!-- figure F7: a link in mid-drag with the halo — pending, see SCREENSHOT_PLAN.md -->

## 上下文菜单

菜单只关于你右键点击的那个对象。菜单打开期间，画布等待：菜单后面没有任何东西会移动、滚动或高亮；在外面第一次点击只会关闭菜单，不做别的；在别处右键点击则把菜单移到那里。↑ ↓ ⏎ 和 Esc 与任何菜单中一样。

On **empty canvas**: **Add Concept ▸** — _Recent_, the four kinds of value (_On / off_, _Count_, _Level_, _Decide later_), _Quantities ▸_ (_Angle_, _Length_, …), _More…_ (which opens the Library tab), each opening the [concept sheet](../../../../docs/user-guide/studio/library.md#creating-a-concept) where you name the concept; **Add Source ▸** — _New source…_, opening the [Source sheet](../../../../docs/user-guide/studio/library.md#sources), where you choose the concept the Source provides — an existing one, or a new one made with it; **Add Instance ▸** _component_ and **New Behavior Group**; then **Select All** and **Frame All**.

On a **relationship**: **Edit Definition** (not for a Source — the environment provides its value), **Show Formula** / **Hide Formula** (when it has one), **Rename**, **Reveal in Code** (the Split view opens at its declaration); **Fix ▸** — the fixes the compiler offers for it, as in the inspector's Fixes section: a ready fix runs, one that needs a choice lists the choices, one the language cannot express yet is shown greyed with the reason; then **Group as Behavior**, **Add to Group ▸** or **Remove from …**; and **Delete _name_**.

在**概念**上：**重命名**、**在代码中显示**、**修正 ▸**、**删除 _名字_**。在**输出**上：**显示驱动方：_名字_**、**重命名**、**在代码中显示**、**修正 ▸**（连接一个值、断开驱动方）、**删除 _名字_**。在**连线**上：**显示 _一端_**、**显示 _另一端_**、**断开**（组件之间的绑定还有**显示绑定**）。在**实例**上：**编辑来源**、**重命名**、**在代码中显示**、**删除 _名字_**。在**行为**上：**重命名**、**折叠** / **展开**、**打包为可复用组件…**、**取消分组**。

在**多个选中节点**之一上：菜单关于它们全体——**归为行为（_n_ 个关系）**和**删除 _n_ 个对象**。

## 画布永远不显示的

草稿公式（已输入但未添加的公式不会改变画布上的任何东西——包括它的引用连线；状态行会计数），值（在模拟页上），关系对自身上一个值的记忆（`delay` 不画连线），以及标识符或类型名（在解释中）。

## 相关

[工作区](workspace.md) · [检查器](inspector.md) · [系统项目](../../../../docs/user-guide/studio/system-projects.md) · [键盘与鼠标](../reference/keyboard-and-mouse.md)
