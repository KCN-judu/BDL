<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/deploy.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/deploy.md) · 简体中文 · [日本語](../../ja/studio/deploy.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 部署

部署页（⌘3）回答 _这个设计能放进这块板子吗_。这是一次布置检查：对每个物理输出的设备，它会使用板子的哪些资源——或者无法布置的第一个原因。页面是一列：**目标板**弹出菜单、**结论**、**设备**，以及板子回答之后的**布置**。它还不会构建固件或烧录板子；见 [尚未实现的](#尚未实现的)。

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the green verdict Feasible on Arduino Nano; the Devices section with a card named pwmLight of kind PWM channel realising light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_部署页：选中 Arduino Nano，light 上有一个 PWM 设备，以及结论和布置。_

## 目标板

The boards the compiler service knows. Today: **Arduino Nano**, **Big board (mock)**, a test target with more PWM pins, and **Raspberry Pi Pico (RP2040)**, the first board firmware can be generated for (`bdld compile --target rp2040_pico`, [CLI](../../../../docs/user-guide/reference/cli.md); flashing it is not in Studio yet). The board choice is a **session preference**: it is not saved with the project, and changing it never changes the design.

在选择板子之前，页面显示 _选择一块板子，看这个设计能否放进去。_

## 设备

**设备**在板子上实现一个物理输出。**添加设备**创建一行；在其中你设置

- 一个**名称**；
- 一个**种类**——_PWM 通道_（可调光灯、舵机信号）、_数字输出_（继电器、开关负载）、_H 桥通道_（电机：一条 PWM 线加一条方向线）、_I²C 传感器_、_正交编码器_、_UART_；
- 它实现的**输出**（或者无）；
- 该种类的每个需求一个**引脚栏**——留空让布置自行选择，或输入板子引脚名（`D3`、`A4`）手动固定；
- **移除**。

The kind decides what the device needs from the board (a PWM channel needs one PWM-capable pin; an H-bridge needs a PWM pin and a digital pin; an I²C sensor needs SDA and SCL on the same bus). You never edit those requirements; the pin fields and the realization are the only manual choices.

### Realization

Once a board has answered, each device card shows a **Realization** pop-up: how the output's value becomes the command the device takes. The choices are the compiler's profiles — _PWM, 8-bit duty_ (a level 0–100 becomes a duty 0–255), _PWM, 4 levels_ (four duties; nearby levels share one), _I2C register, 8-bit_ (register 42 and a value 0–255), _GPIO, on/off_ (a truth value as written), _H-bridge, signed level_ (direction and duty) — with the ones that fit what the output carries listed first and the others marked _does not fit_. Choosing one also sets the device's kind to what the profile needs. _None — place by kind_ leaves the device placed as before and generates no command.

Beside the pop-up the card shows the three checks a realization must pass — **encoder** (the profile's conversion is a well-typed pure function), **fits** (it converts exactly what this output carries), **placed** (the board has the pins) — and, when one fails, the sentence that says which: _pwmLight cannot realise light with `gpio_level`: the output carries q[1] but the profile encodes bool._ A failing realization blocks deployment until changed; a missing one does not.

A realization is deployment data like the kind and the pins: choosing, changing or removing one changes nothing on the Design page, in the simulator's samples or in any verdict about the design.

设备随项目保存。它们是部署数据，不是设计数据：添加、更改或移除设备不会改变设计页的任何结论。

[来源](canvas.md)——由环境提供的值——还没有设备绑定：本页上没有任何种类会向设计供值，检查器的 _实现_ 一行也这样说明（_由环境提供；尚未绑定设备。_）。把传感器、按钮或模拟线路绑定到来源是部署工作，会随第一个嵌入式平台到来；到那时设计也不会改变。

## 结论

| 结论 | 含义 |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **可部署到 Arduino Nano。** | 每个设备的需求都被放到了具备相应能力的、互不相同的引脚上；_… 上的布置_ 表格列出设备 · 需求 · → 引脚 |
| **目前能放进 Arduino Nano——绑定尚未完成。** | 已绑定的部分能放下，但某个输出没有设备（_… 上没有对应的设备：…_）或某个设备没有输出（_未连接到输出：…_） |
| **无法部署到 Arduino Nano。** | 某个需求无法布置；红框指出它和原因——_arduino_nano 上没有东西能承载 pwmLight 的 PWM。_（没有引脚具备该能力）、_在 arduino_nano 上 D4 无法承载 pwmLight 的 PWM。_ 加上 _手动选择的引脚 D4 在这里无法承载 …_（手动固定的引脚），或者阻塞它的引脚及各自被占用的原因；_死路之前已布置：_ 列出已布置的内容 |

结论只关乎板子。**设计**本身是否就绪——每个关系通过检查、没有瞬时环路、每个必需输出都被驱动——是设计页的事，这里不再重述：公式错误的设计仍然可以 _可部署_，而你在本页时底部状态行仍会显示 _存在瞬时环路_ 或 _输出未完成_。两者都成立，才谈得上运行。

_无法部署_ 报告的是布置按自身顺序遇到的**第一个死路**——一个如实指出的冲突，不一定是唯一的。

## “可部署”没有涵盖的

电气和数值约束：电流、电压、PWM 频率、定时器分辨率、总线速度。_可部署_ 意味着引脚可以分配，而不是电路能工作。以后的层可能收窄 _可部署_，但永远不会放宽它。

## 尚未实现的

从部署页生成并构建固件、烧录板子、从板子读回值。编译器已经能为设计生成 Rust 核心，并在仓库测试中逐条轨迹地与仿真器对照检查，但这条路径在 Studio 中还没有界面；`docs/project/roadmap.md` 中的路线图列出了它。

## 相关

[你的第一次部署检查](../getting-started/first-deployment.md) · [物理输出](../../../../docs/user-guide/concepts/physical-outputs.md) · [故障排除：部署](../../../../docs/user-guide/troubleshooting/deployment-errors.md)
