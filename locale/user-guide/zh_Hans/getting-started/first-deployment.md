<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/first-deployment.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/first-deployment.md) · 简体中文 · [日本語](../../ja/getting-started/first-deployment.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 你的第一次部署检查

**目标。** 弄清 [第一个教程](first-behavior.md) 中的灯能否放进 Arduino Nano——并看看“放不下”是什么样子。

**用时。** 十分钟。

今天 Behavior Designer 中的部署指的是 _检查布置_：每个物理输出的设备会用板子的哪个引脚，或者为什么没有引脚可用。它还不会构建固件或烧录板子。

## 1. 打开部署页

点击页面栏中的**部署**（或 **⌘3**）。左侧是**目标板**弹出菜单和**设备**列表；中间是所选板子的结论。

在你选择板子之前，页面显示 _选择一块板子，看这个设计能否放进去。_

## 2. 选择板子

在**目标板**中选择 **Arduino Nano**。结论显示：

> 目前能放进 Arduino Nano——绑定尚未完成。

其下：_arduino_nano 上没有对应的设备：light。_（这一行用板子的短 id 称呼它）。设计有一个输出，但没有说明哪件硬件实现它。

## 3. 添加设备

**设备**是在板子上实现一个输出的硬件：可调光灯用 PWM 通道，继电器用数字输出，电机用 H 桥。

1. 点击**添加设备**。_设备_ 中出现一行。
2. 命名为 `pwmLight`。在它的种类弹出菜单中选择 **PWM 通道**。在它的输出弹出菜单中选择 **light**。

The line about `light` disappears and a _Placement on arduino_nano_ table shows the one line the lamp needs: `pwmLight`, its _PWM_ requirement, and the pin it was given, such as `D3`. The verdict stays **Fits Arduino Nano so far — the binding is not finished.**, and one line remains under it: _tilt has no device on Arduino Nano._ The lamp's output is placed; its Source is not.

**你做了什么。** 一块板子的部署配置。它和设计一起保存在项目中，但是独立的一层：添加设备时设计页的结论没有变，换另一块板子它们也不会变。

**About `tilt`.** A [Source](../../../../docs/user-guide/concepts/relationships.md) is a value the environment provides, and on a board a device has to provide it — the same kind of binding as for an output, chosen in the device's pop-up (_tilt — Source_) with a **Provider** instead of a Realization. The providers this build has read a digital line as on/off; `tilt` is an angle, so nothing fits it yet, and the deployment honestly stays _not finished_. That is a state, not an error: the design simulates, and the placement of `light` is real.

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the placement, and the Source still to provide._

## 4. 故意让它失败

每个设备行的每个需求都有一个引脚栏。在 `pwmLight` 的引脚栏输入 `D4`——Nano 上一个不能做 PWM 的引脚。

> 无法部署到 Arduino Nano。
> 在 arduino_nano 上 D4 无法承载 pwmLight 的 PWM。手动选择的引脚 D4 在这里无法承载 pwmLight 的 PWM。

Clear the field: it fits again. Now switch **Target** to **Big board (mock)**: it fits there too, on a different pin. Switch back.

**发生了什么。** 可部署性是关于 _这个设计在这块板子上_ 的事实。板子的选择是会话偏好——不随项目保存——布置会针对所选的任何板子重新计算。

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry pwmLight PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../../../../docs/user-guide/assets/getting-started/deploy-dead-end.png)

_无法部署：手动选择的引脚无法承载 PWM。_

## “可部署”意味着什么，不意味着什么

_可部署_ 意味着每个设备的需求都能放到具备相应能力的、互不相同的引脚上。它不意味着电路在电气上能工作：电流、电压和时序分辨率没有建模。当有多种布置可能时，工具显示其中一种；当一种都没有时，它显示遇到的第一个死路，那是 _一个_ 冲突，不一定是唯一的。

## 如果有什么不对

- **状态行仍然显示 _存在瞬时环路_ 或 _输出未完成_。** 部署结论只关乎板子；设计本身还没准备好。先在设计页修好它。见 [部署](../../../../docs/user-guide/troubleshooting/deployment-errors.md)。
- **Not connected to an output or Source: …** — a device with nothing chosen in its pop-up.
- **… has no device on Arduino Nano.** — a Source no device provides; the deployment stays _not finished_ until one does.
- **Arduino Nano 上没有东西能承载 …**——板子上没有具备该能力的空闲引脚；试试另一种设备、另一块板子，或更少的设备。

## 下一步

你已经搭建、仿真并布置了一个行为。阅读 [概念](../../../../docs/user-guide/concepts/concepts.md) 页面了解你用到的想法，或者直接去 [从传感器到输出](../../../../docs/user-guide/workflows/sensor-to-output.md)，给灯加上环境光传感器。
