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

文本就是项目的文件，你在画布上做的一切都写在里面：在画布上重命名概念，文本会在它被使用的每个地方显示新名字，你的注释和空行保持不变。在画布上添加关系，它会出现在 `src/main.bdl` 的末尾（没有 `main.bdl` 时是第一个文件的末尾），或者它所属组件主体的末尾。

## What the colours mean

![The editor filling the Design page, showing src/main.bdl of the component system. Keywords such as concept, component, mapping and bind are in a quiet grey; concept names like Tilt and Brightness in a blue-grey ink; relationship names in blue; the Source raw in green; the output light in a warm brown; the instances adaptiveLamp and second in teal; comments in a light grey; the number 90 plain with its unit deg in grey. Declared names are in a heavier weight than their uses.](../../../../docs/user-guide/assets/studio/code-view.png)

_The Code view of the component system: the file as the project holds it, coloured by what each word is._

The text is coloured by what each word _is_ to the project — not by how it is spelled. The colours are the canvas's: a **concept** name has the concept nodes' blue-grey, a **relationship** the relationship nodes' blue, a **Source** the green of a Source node, an **output** or a **device** the warm tone of an output node, an **instance** the teal of an instance node. Keywords, operators and units are grey; comments lighter grey. A name where it is _declared_ is heavier than where it is used; a name that exists only inside a formula — a rule's parameter, a binder's variable — is italic; a `?` left in a formula is orange, the same _still to decide_ colour as elsewhere.

Because the colours come from the project, they tell you things spelling cannot: `deg` after `90` is a unit, `deg` as a rule's parameter is not; `all` at the head of `all x in xs: …` is a keyword, a value named `all` is a value; `clamp` is the library's; a relationship turns from Source green to relationship blue the moment it is given a definition. A file that does not build yet keeps its keywords, numbers, comments and units coloured, and the names the last version that built still knows. The colours follow the appearance (light or dark); there is no setting.

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

## What the editor knows

The editor asks Studio's compiler service the same questions a code editor with the [language server](../../../../docs/user-guide/textual/editor-and-lsp.md) asks, about the text exactly as you have typed it.

**Completion.** Press **⌃Space** and a list opens at the cursor with what can go here, best first: after a `:` the concepts, after an `@` the timing domains, at the start of a line the items allowed there, and inside a formula the inputs, the other relationships — a rule offered as a call, a Source or a value as its name — the names bound in the formula itself, the equations of the library, and after a number the units. ↑ and ↓ move, **Return** or **Tab** accept, **Esc** closes; the list narrows as you type. What is inserted is the service's text, never a guess.

![A pop-up under the caret after brightness() = dimByTilt( in the component body, listing candidates one per row: tiltValue and gain as the body's own values, dimByTilt(Tilt) as a call, then the units and the equations of the library, each with its kind word and the kind of value it gives.](../../../../docs/user-guide/assets/studio/code-completion.png)

_The completion pop-up inside the component's body, after dimByTilt(: what can go here, from the compiler service, best first._

**Hover.** Rest the pointer on a name and a card says what it is: its declaration, what it produces, its state, its role (_Source_, _Rule_ or _Value_), the description you wrote. On an equation of the library — `clamp`, `min`, `any` — the card gives its shape and what it does. Over a keyword, a number or a unit there is no card. Typing or moving away hides it.

![A card beside the word dimByTilt in the component body showing the name in bold, the signature mapping dimByTilt : Tilt -> Brightness in monospace, the words type-valid, and a row role: Rule.](../../../../docs/user-guide/assets/studio/code-hover.png)

_The hover card over dimByTilt where the component's body applies it: its declaration, its role, its state._

**Go to definition.** **⌘-click** a name, or put the cursor on it and press **F12**, and the editor selects where it is declared — in this file or in another, which opens. Inside a component's source a port's name leads to the port's line, never to an instance's copy.

**References.** **⇧F12** on a name lists, under the editor, every place that names it, across all files, with the file and line; a row takes you there. Esc or the × closes the list. Two concepts with the same value form never share a list: the search is by identity, not by spelling.

**Format.** **⌥⇧F**, or _Format_ at the right of the file bar, lays the file out the canonical way — spacing, indentation, one blank line between items — and applies it as one edit, with the cursor kept on its line. A file that does not parse yet is left exactly as it is; fix it first.

Everything here works on the text as it stands, whether or not it builds: what the last version that built still knows is answered, and what nothing resolves gets no card and no destination, never a guess by spelling.

## 尚未实现

Findings underlined in the text (the list and the cursor jump stand in); rename from the editor (rename on the canvas, and the text follows); creating a second source file from Studio (make it in a code editor; it appears on reload).

## 相关

[画布](canvas.md) · [文本形式的 BDL](../../../../docs/user-guide/textual/overview.md) · [以文本编写项目](../../../../docs/user-guide/workflows/authoring-as-text.md) · [源文件](../../../../docs/user-guide/troubleshooting/text-project-errors.md)
