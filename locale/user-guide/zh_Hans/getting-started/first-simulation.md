<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/first-simulation.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/first-simulation.md) · 简体中文 · [日本語](../../ja/getting-started/first-simulation.md)

# 你的第一次仿真

**目标。** 倾斜 [第一个教程](first-behavior.md) 中的灯，一拍一拍地观察它的亮度。

**用时。** 十分钟。

## 1. 打开仿真页

点击底部页面栏中的**仿真**（或按 **⌘2**）。

这个页面有三个部分：

- 左侧的**输入**——每个来自外部的值一个控件。这里是 `tilt`，一个数字栏，旁边是单位 `rad`。输入下方，**时序域**列出了 `interaction` 及其周期：_每 1_ 拍。
- 中间的**轨迹**——随你步进而填充的表格：每拍一行，每个计算值和每个被驱动的输出一列。上方是**步进**、**步进 ×10** 和**重置**，以及拍计数器。
- 右侧的**探针**——当前选中对象的值，包括现在的值和整个运行中的值。

在你步进之前什么都不会运行。如果设计中有任何东西会阻止步进，它会在轨迹上方以一句关于该对象的话列出——_tilt 需要一个值仿真才能步进。_——并附有一个 _显示_ 链接来选中它。只要有这样一行，步进就被禁用。

## 2. 给倾角一个值

在 `tilt` 栏输入 `0.7854`（45°；该栏使用旁边显示的基本单位——角度用弧度）。

## 3. 步进

点击**步进**。

轨迹得到第一行：第 0 拍，活动域 _interaction_，以及设计计算出的值——`brightness` 为 `Brightness(0.5)`（或者相差几位小数，取决于你输入的值），_light_ 列显示同样的值，因为 `brightness` 驱动它。值总是和它所属的概念一起书写。再点两次**步进**：三行，同样的值，因为输入没变。

把栏改为 `1.5708`（90°）并**步进**：约为 `Brightness(1)`。设为 `0` 再步进：`Brightness(0)`。竖直时熄灭，放平时全亮。

**发生了什么。** 每一拍，_interaction_ 域中的每个值都根据你提供的输入重新计算。产生这些数字的求值器就是定义 BDL 设计含义的那个；为设备生成的代码必须与之一致。

![The Simulate page: on the left an input control for tilt showing 0.785398 rad and the interaction domain's period of every 1 ticks; in the middle the Step, Step ×10 and Reset buttons, tick 3, and a trace with three rows whose tilt, brightness and light columns read 0.785398, Brightness(0.5) and Brightness(0.5); on the right the probe for brightness with 0.5 now and at each tick over the run.](../../../../docs/user-guide/assets/studio/simulate-page.png)

_倾角为 45° 时步进三次后的仿真页：左侧是输入，中间是轨迹，右侧是探针。_

## 4. 查看一个值

点击 `brightness` 列标题，或在设计页上选中 `brightness` 再回来。**探针**显示它现在的值和整个运行中的值，带有概念的图形符号。展开其下的**解释**可查看形式化细节——标识号、运行的修订号、原始渲染值。

## 5. 重置并试试节奏

**重置**回到第 0 拍并保留你的输入。

在 _时序域_ 下，把 _interaction_ 设为 _每 2_。**步进 ×10**：_活动_ 列现在只在每隔一拍时写着 _interaction_，`tilt` 的单元格也只在那些拍显示——其余拍该域没有激活，所以没有读取任何值。周期是仿真的选择；设计本身只说明值属于 _哪个_ 域。

## 一次步进实际做了什么

每次步进都是重放：仿真器带着你迄今输入的所有值从第 0 拍重新开始，运行到新的一拍。同样的设计、同样的输入和同样的周期每次都给出同样的轨迹。在设计页上修改设计会丢弃轨迹；你的输入值保留。

## 如果有什么不对

- **步进被禁用，并有一句话指出某个关系。** 读一读：没有值的输入、没有值形式的概念、没有有效定义的关系，或者在同一瞬间相互依赖的关系。每一条都有 _显示_ 链接。见 [未完成的设计](../../../../docs/user-guide/troubleshooting/incomplete-design.md)。
- **某一拍停止，控件行上出现消息**（例如除以零）。消息会指出关系。修改公式或输入，再次步进。
- **编辑设计后轨迹为空。** 这是预期的——运行属于设计的某一个版本。再次步进。

## 下一步

[你的第一次部署检查](first-deployment.md)——这盏灯能放进 Arduino Nano 吗？
