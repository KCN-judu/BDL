<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/code-view.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/code-view.md) · 简体中文 · [日本語](../../ja/studio/code-view.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 设计、代码与分栏

设计页以三种方式显示同一个项目。画布上方栏右端的控件在它们之间切换：

| 视图 | 你看到什么 | 你做什么 |
| ---------- | ----------------------------------------------- | -------------------------------------------------------------- |
| **设计** | 画布（[画布](canvas.md)） | 绘制设计 |
| **代码** | 编辑器中项目的 `.bdl` 文件 | 输入设计（[语法基础](../../../../docs/user-guide/textual/syntax-basics.md)） |
| **分栏** | 左侧画布，右侧编辑器 | 两者兼有，作用于同一批对象 |

切换不会改变项目中的任何东西：图和文本是同一设计的两幅图像，由 Studio 的编译器服务保持同步。没有 _转换_、_导入_ 或 _导出_。

## 编辑器显示什么

一次显示一个文件的文本。当项目有多个 `.bdl` 文件时，编辑器顶部的弹出菜单列出它们；尚不能构建的文件带有 _— 未构建_ 标记。

The text is the project's files with everything you did on the canvas written in: rename a block on the canvas, or a concept in the sidebar, and the text shows the new name at every place it is used, with your comments and blank lines untouched. Add a block on the canvas and it appears at the end of `src/main.bdl` (or of the first file, when there is no `main.bdl`), or at the end of the component body it belongs to.

## 颜色的含义

![The editor filling the Design page, showing src/main.bdl of the component system. Keywords such as concept, component, mapping and bind are in a quiet grey; concept names like Tilt and Brightness in a blue-grey ink; relationship names in blue; the Source raw in green; the output light in a warm brown; the instances adaptiveLamp and second in teal; comments in a light grey; the number 90 plain with its unit deg in grey. Declared names are in a heavier weight than their uses.](../../../../docs/user-guide/assets/studio/code-view.png)

_组件系统的代码视图：文件按项目保存的样子显示，按每个词是什么着色。_

文本按每个词对项目而言 _是什么_ 着色——而不是按拼写。颜色与画布一致：**概念**名是蓝灰色，**关系**是块的蓝色，**来源**是来源块的绿色，**输出**或**设备**是输出节点的暖色，**实例**是实例节点的青色。关键字、运算符和单位是灰色；注释是更浅的灰色。名字在其 _声明_ 处比在使用处更粗；只存在于公式内部的名字——规则的参数、绑定器的变量——是斜体；公式中留下的 `?` 是橙色，与别处的 _尚待决定_ 颜色相同。

因为颜色来自项目，它们能告诉你拼写无法告诉你的事：`90` 后面的 `deg` 是单位，作为规则参数的 `deg` 不是；`all x in xs: …` 开头的 `all` 是关键字，名为 `all` 的值是值；`clamp` 是库的；一个关系在获得定义的那一刻从来源的绿色变为关系的蓝色。尚不能构建的文件仍保留关键字、数字、注释和单位的颜色，以及最后一个能构建的版本仍认识的名字。颜色跟随外观（浅色或深色）；没有设置项。

## 输入

像在任何编辑器里一样输入。你停下片刻之后，文件会被读作设计：

- **能构建。** 画布随之变化——新关系作为节点出现，替你放在它读取的东西旁边；重命名的概念保留它的节点、颜色、位置和连接，因为条目的身份在于它的种类和名字，而不是它在文本中的拼写（[身份](../../../../docs/user-guide/textual/overview.md#identity)）。
- **尚不能构建。** 编辑器上方的横幅显示 _此文件尚不能构建：设计显示的是最后一个能构建的版本。_ 画布继续显示文件最后一个能构建的版本，并带有自己的横幅：_正在显示最后一个能构建的版本；文本中有尚不能构建的更改。_ 你的文本保持原样。原因列在编辑器下方，每行一条，附有行号；点击一条把光标移到那里。**×** 表示设计无法表达的东西；**空心环**表示设计尚无含义、但不阻止文件构建的东西。修好文本，两条横幅都会消失。

你输入的不会丢失，你画的也不会丢失：不能构建的文件永远不会抹掉图。

**撤销**（⌘Z）撤销对设计的更改——在文本中添加的关系像在画布上添加的一样被撤销，文本随之变化。只涉及注释或空白的更改不是该历史中的步骤。

## 分栏：同一个选择

在画布上选中节点，编辑器滚动到它的声明。点击某个条目的文本内部，它的节点在画布上被选中，检查器随之显示。两个视图中的选择是同一个对象。

## 保存

**⌘S** 按编辑器显示的原样写入文件——包括尚不能构建的文件——然后是布局和辅助文件（[项目文件](../reference/project-files.md)）。重新打开项目显示同样的文本、同样的横幅，设计则是最后一个能构建的版本。

## 编辑器知道什么

编辑器向 Studio 的编译器服务提出的问题，与带 [语言服务器](../../../../docs/user-guide/textual/editor-and-lsp.md) 的代码编辑器提出的相同，针对的是你输入的原样文本。

**补全。** 按 **⌃Space**，光标处打开一个列表，列出这里可以放什么，最合适的在前：`:` 之后是概念，`@` 之后是时序域，行首是允许出现的条目，公式内部是输入、其他关系——规则以调用形式提供，来源或值以名字提供——公式自身绑定的名字、库的方程，数字之后是单位。↑ 和 ↓ 移动，**Return** 或 **Tab** 接受，**Esc** 关闭；列表随输入收窄。插入的是服务给出的文本，从不猜测。

![A pop-up under the caret after brightness() = dimByTilt( in the component body, listing candidates one per row: tiltValue and gain as the body's own values, dimByTilt(Tilt) as a call, then the units and the equations of the library, each with its kind word and the kind of value it gives.](../../../../docs/user-guide/assets/studio/code-completion.png)

_组件主体内、`dimByTilt(` 之后的补全弹出列表：这里可以放什么，来自编译器服务，最合适的在前。_

**悬停。** 把指针停在名字上，一张卡片会说明它是什么：它的声明、它生成什么、它的状态、它的角色（_来源_、_规则_ 或 _值_）、你写的描述。在库的方程上——`clamp`、`min`、`any`——卡片给出它的形状和作用。在关键字、数字或单位上没有卡片。输入或移开就会隐藏它。

![A card beside the word dimByTilt in the component body showing the name in bold, the signature mapping dimByTilt : Tilt -> Brightness in monospace, the words type-valid, and a row role: Rule.](../../../../docs/user-guide/assets/studio/code-hover.png)

_在组件主体应用 dimByTilt 之处悬停显示的卡片：它的声明、角色、状态。_

**跳到定义。** **⌘-点击**一个名字，或把光标放在它上面按 **F12**，编辑器会选中它声明的位置——在本文件或另一个文件中，后者会自动打开。在组件的来源中，端口名指向端口所在的行，绝不会指向某个实例的副本。

**引用。** 在名字上按 **⇧F12**，编辑器下方会列出所有文件中提到它的每一处，附有文件和行号；点击一行就跳过去。Esc 或 × 关闭列表。值形式相同的两个概念绝不会共用一个列表：搜索按身份进行，而不是按拼写。

**格式化。** **⌥⇧F**，或文件栏右侧的 _格式化_，把文件排成规范布局——空格、缩进、条目之间一个空行——并作为一次编辑应用，光标保持在原来的行。尚不能解析的文件保持原样；先修好它。

这里的一切都作用于文本的当前状态，无论它能否构建：最后一个能构建的版本仍认识的东西会得到回答，而任何东西都无法解析的名字没有卡片也没有目的地，绝不会按拼写猜测。

## 尚未实现

在文本中用下划线标出发现项（目前由列表和光标跳转代替）；从编辑器中重命名（在画布上重命名，文本随之变化）；从 Studio 创建第二个源文件（在代码编辑器中创建；重新加载后出现）。

## 相关

[画布](canvas.md) · [文本形式的 BDL](../../../../docs/user-guide/textual/overview.md) · [以文本编写项目](../../../../docs/user-guide/workflows/authoring-as-text.md) · [源文件](../../../../docs/user-guide/troubleshooting/text-project-errors.md)
