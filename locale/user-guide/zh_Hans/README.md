<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/README.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../docs/user-guide/README.md) · 简体中文 · [日本語](../ja/README.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# Behavior Designer — 用户指南

Behavior Designer 是一款桌面工具，用于设计**产品如何行为**：它感知什么、决定什么、显示或驱动什么——在写任何固件之前。你把行为描述为一组有名字的值以及它们之间的关系；工具在你构建的同时检查设计，让你逐拍仿真，并告诉你它能否放进某块特定的板子。

工具背后的语言叫做 **BDL**。画布、检查器和公式栏是书写它的一种方式；文本形式是另一种，适合更喜欢编辑器的人——见 [文本形式的 BDL](../../../docs/user-guide/textual/overview.md)。无论哪种方式，你都是在用一门语言描述行为；工具替你挡在外面的是机器：用哪个引脚、哪种协议、哪个 HAL 调用，都留到部署时再决定。

## 它解决什么问题

产品的行为通常在草图和表格里决定，然后在代码中被重新发明，而每个数字的含义就此丢失。在 Behavior Designer 中，值有**含义**（_亮度_ 不是 _不透明度_，即使两者都是 0 到 1 之间的数），值之间的关系会做单位和量纲检查，时序是显式的，一个物理输出只能由一个关系驱动。未完成的设计是常态：你可以在知道公式之前先给关系起名，工具会告诉你还有什么没定，而不是拒绝工作。

## 你可以在这里做什么

|  |  |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **设计** | 为产品的概念命名，用公式把它们关联起来，决定它们的时序，并把结果接到物理输出 |
| **仿真** | 喂入值，逐拍推进设计，观察每一个值——用的是定义这门语言的同一个求值器 |
| **部署** | 选择一块板子，看设计的输出能否放到它的引脚上；放不下时看原因 |
| **组织与复用** | 把相关的关系归为一个行为，把行为打包成可复用的组件，再把组件实例组合成更大的系统 |

工具尚不支持：烧录板子、来自运行中设备的实时值、从其他项目导入组件。指南会在相关处说明。

## 从哪里开始

| 如果你是… | 从这里开始 |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| 想描述一个行为并试一试的产品或工业设计师 | [什么是 BDL？](getting-started/what-is-bdl.md) → [你的第一个行为](getting-started/first-behavior.md) |
| 熟悉传感器和执行器的嵌入式原型开发者 | [你的第一个行为](getting-started/first-behavior.md)，然后是 [从传感器到输出](../../../docs/user-guide/workflows/sensor-to-output.md) 和 [部署](studio/deploy.md) |
| 更愿意直接敲 BDL 的代码型用户 | [文本形式的 BDL — 概览](../../../docs/user-guide/textual/overview.md)，它准确说明了今天哪些可用 |
| 想了解语义的研究者或语言爱好者 | 每个概念页末尾的 _深入_ 一节，以及 [术语](../../../docs/user-guide/reference/terminology.md) → `docs/` 中的技术文档 |

## 目录

**Getting started** [What is BDL?](getting-started/what-is-bdl.md) · [Install and launch](getting-started/install-and-launch.md) · [Your first behavior](getting-started/first-behavior.md) · [Your first simulation](getting-started/first-simulation.md) · [Your first deployment check](getting-started/first-deployment.md) · [Your first board](getting-started/pico-demo.md)

**概念** — 每个想法一页 [概念](../../../docs/user-guide/concepts/concepts.md) · [关系](../../../docs/user-guide/concepts/relationships.md) · [未完成的设计](../../../docs/user-guide/concepts/incomplete-designs.md) · [时序](../../../docs/user-guide/concepts/timing.md) · [物理输出](../../../docs/user-guide/concepts/physical-outputs.md) · [行为组](../../../docs/user-guide/concepts/behavior-groups.md) · [组件](../../../docs/user-guide/concepts/components.md) · [行为系统](../../../docs/user-guide/concepts/behavior-systems.md)

**Studio** — 界面，每个面板或页面一页 [工作区](studio/workspace.md) · [画布](studio/canvas.md) · [库](../../../docs/user-guide/studio/library.md) · [检查器](studio/inspector.md) · [公式编辑器](studio/formula-editor.md) · [仿真](studio/simulate.md) · [部署](studio/deploy.md) · [系统项目](../../../docs/user-guide/studio/system-projects.md) · [设计、代码与分栏](studio/code-view.md)

**工作流** — 每个目标一页，都围绕同一盏灯 [从传感器到输出](../../../docs/user-guide/workflows/sensor-to-output.md) · [第二个输出](../../../docs/user-guide/workflows/multi-output-behavior.md) · [给行为分组](../../../docs/user-guide/workflows/grouping-behavior.md) · [把行为打包成组件](../../../docs/user-guide/workflows/package-as-component.md) · [组合组件](../../../docs/user-guide/workflows/composing-components.md) · [跨时序域传递值](../../../docs/user-guide/workflows/cross-domain-transport.md) · [组件版本管理](../../../docs/user-guide/workflows/component-versioning.md) · [以文本编写项目](../../../docs/user-guide/workflows/authoring-as-text.md)

**文本形式的 BDL** [概览与当前状态](../../../docs/user-guide/textual/overview.md) · [语法基础](../../../docs/user-guide/textual/syntax-basics.md) · [编辑器与语言服务器](../../../docs/user-guide/textual/editor-and-lsp.md) · [以文本编写项目](../../../docs/user-guide/workflows/authoring-as-text.md)

**故障排除** [从这里开始](../../../docs/user-guide/troubleshooting/README.md) · [未完成的设计](../../../docs/user-guide/troubleshooting/incomplete-design.md) · [类型、单位与概念](../../../docs/user-guide/troubleshooting/type-and-concept-errors.md) · [时序](../../../docs/user-guide/troubleshooting/timing-errors.md) · [连接](../../../docs/user-guide/troubleshooting/connection-errors.md) · [部署](../../../docs/user-guide/troubleshooting/deployment-errors.md) · [源文件](../../../docs/user-guide/troubleshooting/text-project-errors.md)

**参考** [术语](../../../docs/user-guide/reference/terminology.md) · [键盘与鼠标](reference/keyboard-and-mouse.md) · [状态含义](../../../docs/user-guide/reference/status-meanings.md) · [公式语言](../../../docs/user-guide/reference/formula-language.md) · [项目文件](reference/project-files.md) · [命令行](../../../docs/user-guide/reference/cli.md)

**关于本指南** [风格指南](../../../docs/user-guide/STYLE_GUIDE.md) · [验证矩阵](../../../docs/user-guide/VERIFICATION.md) · [截图计划](../../../docs/user-guide/SCREENSHOT_PLAN.md) · [文档调研](../../../docs/user-guide/DOCUMENTATION_RESEARCH.md)

面向工具开发者的技术文档在上一级的 `docs/` 中（架构、编译流水线、协议、形式化说明）。本指南在 _深入_ 小节中链接到它们，不再重复。
