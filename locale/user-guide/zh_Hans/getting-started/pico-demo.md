<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/pico-demo.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/pico-demo.md) · 简体中文 · [日本語](../../ja/getting-started/pico-demo.md)

# 你的第一块板子：Raspberry Pi Pico 上的按钮 → 灯

**目标。** 让一个按钮控制一盏灯——并在真实的板子上运行。你将打开一个小产品，告诉 Studio 按钮和灯在哪些引脚上，构建固件，把它放到板子上，然后按下按钮。

**时间。** 十五分钟，外加第一次构建下载板子的库所需的时间。

**需要。** 一块 Raspberry Pi Pico（RP2040 板子）、一根能传数据的 USB 线、一个按钮、两根跳线。不需要 LED：演示驱动 Pico 自带的板载 LED。除了 Studio 和安装页要求的 Rust 工具链（[安装与启动](install-and-launch.md)），无需安装其他东西。

## 1. 接好按钮

按钮接在 **GP2** 和 **GND** 之间：一脚接 GP2，另一脚接任意 GND 引脚。不需要别的。Pico 自己把这条线拉高，所以按住按钮时线读为低电平——查看设备时演示也会这么说。

| Pico 引脚 | 接线 |
| -------- | -------------------------------- |
| GP2 | 按钮的一脚 |
| GND | 另一脚 |
| GP25 | 不接——它就是板载 LED |

## 2. 打开演示

在欢迎页的**演示**下选择 **Button → Lamp, wired**。Studio 会问项目放在哪里（它是一个普通项目：一个含有 `src/main.bdl` 的文件夹），然后在设计页打开它。

![The canvas with the concept rows Pressed and Lit at the top, the Source node pressed with its entry arrow and the word Source, the relationship node lit below it with the formula pressed, and the lamp sink at the right, joined by links; each carries the domain name main.](../../../../docs/user-guide/assets/getting-started/pico-design.png)

_演示的设计：来源 pressed、跟随它的值 lit，以及它驱动的 lamp。_

三个对象。**pressed** 是一个 _来源_：环境提供的值——设计不说明如何提供，画布上也没有引脚。**lit** 是一个关系，公式就是 `pressed`。**lamp** 是物理输出，由 `lit` 驱动。这里没有任何东西与板子有关；那是下一页的事。

另一个演示 **Button → Lamp** 是同一个设计但不带部署：想自己做设备选择时用它（第 3 步会解释）；wired 版本已经做好了。

## 3. 部署

点击**部署**（⌘3），在**目标板**中选择 **Raspberry Pi Pico (RP2040)**。

![The Deploy page: the Target pop-up showing Raspberry Pi Pico (RP2040); the green verdict Feasible on Raspberry Pi Pico (RP2040); two device cards — button, a Digital input for pressed — Source with the provider GPIO input, active low and pin GP2, and led, a Digital output for lamp with the realization GPIO, on/off and pin GP25 — each with its judgments checked; the placement table with button and led on GP2 and GP25; and at the bottom the Firmware section with the steps Deployment done, Build current, Flash and Observe to come, a green dot with Ready to build for Raspberry Pi Pico (RP2040), and one primary button, Build for Raspberry Pi Pico (RP2040).](../../../../docs/user-guide/assets/getting-started/pico-firmware-ready.png)

_部署页上的 wired 演示：选择了 Pico，两个设备都可接受并已放置，只剩一件事要做——构建。_

从上到下读这一页：

- **可部署到 Raspberry Pi Pico (RP2040)。**——设备能放入板子。
- **设备。** _设备_ 是代表设计中某一件事物的硬件。`button` 是 `pressed — 来源` 的**数字输入**，**提供方式**为 _GPIO 输入，低电平有效_（低为真：按钮接地），在引脚 **GP2** 上。`led` 是 `lamp` 的**数字输出**，**实现方式**为 _GPIO，开/关_，在引脚 **GP25** 上。每个下面的勾——来源的 _转换器 · 匹配 · 已放置 · 可读取_，输出的 _编码器 · 匹配 · 已放置_——是编译器的判断；每一项都成立。
- **放置。** 哪个引脚承载什么。
- **固件。** 你在通往运行中的板子的路上走到了哪一步——_✓ 部署 · **构建** · 烧录 · 观察_——以及下一步要做的那一件事。

换成引导式演示时，页面会写**尚不能构建**并点名第一件缺失的事——_pressed 在 Raspberry Pi Pico 上没有设备。_——由你来创建两个设备：**添加设备**，命名，选择种类，在第三个弹出菜单中选择它是为了什么（`pressed — 来源` 或 `lamp`），选择提供方式或实现方式，输入引脚。这句话会随着你的操作变化，直到写着 _已可构建_。

## 4. 构建

点击**为 Raspberry Pi Pico (RP2040) 构建**。卡片显示正在发生的事——_检查部署_、_生成代码_、_准备工具链_、_编译_（附已编译的件数）、_写入镜像_——最后是**固件已于 _时间_ 构建**和镜像大小。第一次构建要获取板子的库，需要几分钟；之后只要几秒。**停止**可提前结束构建。

如果以**构建未完成**结束，卡片会说明是哪一步、为什么——例如 _尚未安装 Raspberry Pi Pico 的 Rust 目标。_ 附带要运行的那一条命令——**详细信息**里有编译器自己的话。见[故障排除：部署](../../../../docs/user-guide/troubleshooting/deployment-errors.md)。

## 5. 烧录

在已构建的固件下方，卡片写着**未找到可连接的开发板。**以及如何让板子可连接：**按住 Pico 上的 BOOTSEL 键，同时通过 USB 插入**。Pico 会以名为 `RPI-RP2` 的小磁盘出现在电脑上；点击**重新查找**，卡片会点名它——_Raspberry Pi Pico in BOOTSEL mode (RPI-RP2)_——并带有**烧录**按钮。

![The Firmware card after a build: a green dot and Firmware built at a time with the image's size in bytes, a Build again link; below, an orange dot with No board is reachable and the line Hold BOOTSEL while plugging the board in over USB; it appears as a drive named RPI-RP2, and a Look again link.](../../../../docs/user-guide/assets/getting-started/pico-firmware-built.png)

_已构建：镜像是最新的，还没有插入板子，页面说明该做什么。_

点击**烧录**。镜像被复制到磁盘上；Pico 重启进入新固件，磁盘消失。卡片写着**已于 _时间_ 烧录到 Raspberry Pi Pico in BOOTSEL mode (RPI-RP2)**，然后是：

> 试一试：操作 pressed；lamp 应按设计响应。

如果插了两块 Pico，卡片会问是哪一块；它从不替你选。

## 6. 试一试

按下按钮。按住时板载 LED 亮起，松开时熄灭：`lit` 就是 `pressed`，`lamp` 就是 `lit`。

Studio 只能告诉你烧录成功了；板子是否按预期工作要你自己看。如果 LED 不跟随按钮，先检查接线（GP2 与 GND，按钮确实能闭合），再看[故障排除：部署](../../../../docs/user-guide/troubleshooting/deployment-errors.md)。

## 7. 改一改，再来一次

改动设计——比如把 `lit` 改为 `!pressed`，让灯在按下之前一直亮着——或者改动部署——把按钮放到 GP3。固件卡片立刻写着：**固件来自更早的设计或部署。**如果你已经烧录过，还有 _开发板运行的是更早的设计——重新构建并烧录以更新。_ 只提供**重新构建**；镜像变为最新后烧录才会回来。Studio 从不让旧镜像冒充屏幕上的设计。

## 用设计的语言说，你做了什么

设计从未知道 GP2 或 GP25。**来源**是环境提供的东西；**提供方式**——在部署页选择的目录配置——是设备为板子读取它的方式，**实现方式**是设备把输出的值送出去的方式。换板子，设计不变；换提供方式，设计不变。这种分离正是两个页面的意义所在（[部署](../studio/deploy.md)）。

## 不用 Studio

从终端走同一条路，用于脚本或不开应用检查构建（[命令行](../../../../docs/user-guide/reference/cli.md)）：

```bash
bdld init my-lamp --template button-lamp-configured
bdld build my-lamp --target rp2040_pico
bdld flash my-lamp --target rp2040_pico
```

## 下一步

[从传感器到输出](../../../../docs/user-guide/workflows/sensor-to-output.md)给设计加第二个来源；[部署](../studio/deploy.md)列出了固件卡片的每一种状态。
