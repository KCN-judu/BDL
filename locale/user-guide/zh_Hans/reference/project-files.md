<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/reference/project-files.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/reference/project-files.md) · 简体中文 · [日本語](../../ja/reference/project-files.md)

# 项目文件

项目是一个由普通文件组成的文件夹。Studio 读写它们；你可以把文件夹纳入版本控制、复制或做差异比较。面向工具开发者的权威描述在 `docs/spec/project-format.md`。

```text
lamp/
├── bdl.toml                 the manifest: name, schema version
├── src/
│   ├── concepts.bdl         the design — any number of .bdl files, any names, read in path order
│   └── main.bdl
├── .bdl/
│   ├── identities.json      the stable identity of every item — owned by the tools
│   └── authoring.json       behavior groups — owned by the tools
└── ui/
    └── layout.json          where things are on the canvas — not part of the design
```

项目只有一种。它的设计就是 `.bdl` 文件（[语法基础](../../../../docs/user-guide/textual/syntax-basics.md)）；Studio 把它们显示为图、文本或两者（[设计、代码与分栏](../studio/code-view.md)）。

## 设计中有什么

一切有含义的东西：概念（名称、描述、值形式）、关系（名称、描述、读取、生成、公式、时序域、它驱动的输出）、时序域、物理输出（名称、接受什么、域、是否必需）、设备（名称、种类、输出、固定引脚）、组件（各自带有自己的设计和端口）、实例、绑定和导出。

每个对象都有一个稳定的数字身份，永不改变、永不复用。**名称是文件对它的拼写**：重命名一个条目只会改变它在所有书写处的名字，别的什么都不变，因为身份保存在 `.bdl/identities.json` 中，按种类和名称与条目匹配。两个项目中同名的两个对象彼此毫无关系。

名称必须是文件能拼写的名称：字母、数字和下划线，不以数字开头，不是语言的关键字（`light`、`pwmLight`、`dimByTilt`）。Studio 拒绝其他任何拼写，并显示可行的拼写（[源文件](../../../../docs/user-guide/troubleshooting/text-project-errors.md)）。

## 辅助文件中有什么

`.bdl/identities.json` 按种类和名称记录每个条目的身份，以及分配新身份的计数器。当源文件需要该文件中没有的身份时，它在打开时写入；每次保存之后写入；在编辑器中保存后由语言服务器写入。请把它和源文件一起纳入版本控制；不要编辑它。删除它会给每个条目一个全新的身份——语义上不会丢失任何东西，但以身份为键的画布位置和组成员关系会丢失。

`.bdl/authoring.json` 保存行为组，文本不表达它们。成员从文本中消失的组会失去这些成员。

## 布局文件中有什么

概念、关系、输出、实例和行为框的位置；折叠状态；每个画布的平移和缩放。没有这个文件项目也能打开；删除它只会丢失摆放位置。没有位置的条目在项目打开时和每次创建条目时都会被分配一个位置——在它所属种类的列中，紧挨着它读取或生成的东西——已有位置的东西不会被移动（[画布](../studio/canvas.md)）。

## 不保存的东西

- 部署页上选择的**板子**——会话偏好；
- **仿真**——输入、周期和轨迹；
- **分析**——每个结论在打开时重新计算；
- **修订历史**——撤销以会话为单位；
- 你所在的位置——视图、页面、打开的文件、来源被打开的组件——Studio 按用户自行记录，下次打开时把你带回那里。

你能看到和编辑的一切都在项目中。你输入但未添加的公式——即使尚未通过检查，即使字段为空——都原样保存并回到编辑器中。代码视图中尚不能构建的文本按输入原样保存：磁盘上的文件就是你输入的内容，设计继续显示它最后一个能构建的版本，直到它再次能构建（[设计、代码与分栏](../studio/code-view.md)）。

## 保存

**⌘S** 或**保存**写入完整的当前状态：改动过的源文件以条目级编辑写入——保存只改变你改过的条目，文本的其余部分、注释、空行和你的排列顺序保持原样；尚不能构建的文件按输入原样写入——然后是辅助文件（身份、组、你未完成的公式，以及不能构建的文件最后一份正确的文本）、布局和清单，每个都通过临时文件和原子重命名写入，因此被中断的保存永远不会留下写了一半的项目。保存从不要求公式通过检查或文件能构建。状态行中的 _已保存_ 表示上述每个文件都已写入；在此之前项目是 _已编辑_。在画布上新建的条目放到 `src/main.bdl` 的末尾（没有 `main.bdl` 时是按路径顺序的第一个文件的末尾），或它们所属组件主体的末尾。更新版本的文件会被拒绝而不是误读。

保存会检查自读取以来没有源文件在磁盘上被修改；如果有，Studio 会拒绝并提供重新加载或覆盖的选项（[以文本编写项目](../../../../docs/user-guide/workflows/authoring-as-text.md)）。

## 旧项目

早期版本的 Studio 保存的项目把设计放在 JSON 文件中（`design/project.bdl.json` 或 `design/system.bdl.json`），并在 `bdl.toml` 中如此声明。打开这样的项目——在 Studio 中、从命令行或通过语言服务器——会原地转换它，只转换一次：

- 设计写入 `src/main.bdl`；
- 每个条目保留其身份，因此位置、颜色和组成员关系不变；
- 文件无法拼写的名称会被重新拼写（`Light Output` 变为 `Light_Output`，语言关键字会加上尾随的 `_`）；
- JSON 文件被重命名为 `….migrated`，不再被读取；确认无误后可删除，或作为记录保留；
- `bdl.toml` 被重写。

同时含有 JSON 设计文件和 `src/` 下 `.bdl` 文件的文件夹会被拒绝而不是猜测：只保留其中一种。没有回到 JSON 形式的途径。

## 相关

[设计、代码与分栏](../studio/code-view.md) · [以文本编写项目](../../../../docs/user-guide/workflows/authoring-as-text.md) · [文本形式的 BDL](../../../../docs/user-guide/textual/overview.md)
