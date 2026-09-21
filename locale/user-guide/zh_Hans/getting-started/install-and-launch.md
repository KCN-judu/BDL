<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/install-and-launch.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/install-and-launch.md) · 简体中文 · [日本語](../../ja/getting-started/install-and-launch.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 安装与启动

Behavior Designer 从源码构建。目前还没有安装程序或打包下载。构建会产生两样东西：Studio 应用，以及它所对话的编译器服务（`bdld`），后者由 Studio 替你启动。

## 你需要什么

| 工具 | 版本 | 说明 |
| ------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust | 1.89 — 由仓库锁定，通过 `rustup` 安装 | 构建编译器服务，以及板子的固件（板子的目标——Raspberry Pi Pico 为 `thumbv6m-none-eabi`——在第一次构建需要时由 `rustup` 安装） |
| Flutter | 3.47 | 构建 Studio；macOS 上用 `brew install --cask flutter` |
| `just` | 任意 | 仓库使用的任务运行器 |

macOS 和 Windows 是受支持的桌面平台。Linux 能顺带构建，但未经测试。

## 构建并运行

在仓库根目录：

```bash
just studio
```

这会构建 `bdld` 并启动指向它的 Studio。第一次构建需要几分钟；之后就很快。

**请从终端或由 Finder 启动的 shell 运行**，而不是从嵌在其他应用中的终端。macOS 拒绝为从沙盒宿主启动的应用显示文件对话框；Studio 会检测到这一点并改为提供手动输入路径（_按路径打开…_、_在路径新建…_），但原生对话框才是预期的方式。

## 你首先看到的

Studio opens on the **project manager**: the wordmark on the left with _Start_ actions under it; _Recent_ projects on the right, and under them the _Demos_, each with a line saying what it is.

![Studio's start screen with the wordmark, then New Project, Open Project and Preferences on the left; on the right an empty Recent list, then Demos — Button → Lamp and Button → Lamp, wired, each with a sentence saying what it is; the status line at the bottom says no project and names the compiler version.](../../../../docs/user-guide/assets/getting-started/project-manager.png)

_The project manager: Start actions on the left; Recent and the Demos on the right._

| 操作 | 作用 |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **新建项目…** | 询问文件夹名称和位置，在那里创建项目，并打开工作区 |
| **打开项目…** | 打开一个已有的项目文件夹 |
| _演示_ 中的一行 | 一个开始就带有设计的新项目：**Button → Lamp**（只有设计）和 **Button → Lamp, wired**（带有 Raspberry Pi Pico 上的设备和引脚）——[你的第一块板子](pico-demo.md) |
| 一行 _最近_ 项目 | 重新打开该项目；文件夹已不存在的行会变灰并标为 _未找到_ |

**项目**就是一个文件夹。其中的一切都保存为普通文件（[项目文件](../reference/project-files.md)）：设计以 `.bdl` 文本保存，也可以在代码编辑器中编辑，画布布局保存在旁边。项目只有一种；行为组、组件和实例（[系统项目](../../../../docs/user-guide/studio/system-projects.md)）以及**代码**视图（[设计、代码与分栏](../studio/code-view.md)）在每个项目中都可用。

窗口底部一行显示与编译器服务的连接：已连接时显示 _编译器 0.1.0_，启动中显示 _正在连接编译器_，失败时显示 _编译器未连接_。Studio 自身不做任何语义工作——指南中的每一次检查、每一个值和结论都来自该服务——所以如果应用看起来没有反应，先看这一行。

## 试试完整示例

仓库附带一个完整项目 `examples/smart_lamp`。_打开项目…_ → 选择该文件夹。它就是教程中搭建的那盏灯，加上了一个环境光传感器：两个输入、三个关系、一个必需输出、一个 PWM 设备。在搭建自己的项目之前，你可以先浏览它的设计、仿真和部署页面。

## 下一步

[你的第一个行为](first-behavior.md)——或者，手边有 Raspberry Pi Pico 的话，[你的第一块板子](pico-demo.md)。
