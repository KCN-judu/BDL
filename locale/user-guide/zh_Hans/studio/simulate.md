<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/simulate.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/simulate.md) · 简体中文 · [日本語](../../ja/studio/simulate.md)

# 仿真

仿真页（⌘2）回答 _设计随时间做什么_，一拍一拍地回答。每个值都来自编译器服务的参考求值器——BDL 设计含义的可执行定义。Studio 保存你输入的值、你选择的周期和返回的采样；它自己不做任何计算。

![The Simulate page: on the left an input control for tilt showing 0.785398 rad and the interaction domain's period of every 1 ticks; in the middle the Step, Step ×10 and Reset buttons, tick 3, and a trace with three rows whose tilt, brightness and light columns read 0.785398, Brightness(0.5) and Brightness(0.5); on the right the probe for brightness with 0.5 now and at each tick over the run.](../../../../docs/user-guide/assets/studio/simulate-page.png)

_倾角为 45° 时步进三次后的仿真页：左侧是输入，中间是轨迹，右侧是探针。_

## 输入（左侧）

每个来自外部的值一个控件——每个不读取任何东西且没有公式的关系。控件跟随概念的值形式，而不是它的名字：量是旁边带单位的数（用基本单位：弧度、米、秒、开尔文…），开/关是开关，计数是整数。这一行带有概念的图形符号和颜色，点击它会选中该对象——在本页和设计页上都是。

输入下方是**时序域**：每个域一个周期——_每 N_ 拍——求值器按此调度激活。是周期，从不是速率。更改周期会重新开始运行。

实例的**开放的必需端口**也是输入，实例的值以 `lampA.brightness` 的形式列出。

## 就绪状态（轨迹上方）

在任何东西运行之前，页面用关于具名对象的句子列出会阻止步进的事项，每条附有选中它的 _显示_ 链接：

- _tilt 需要一个值仿真才能步进。_——没有值的输入
- _Tilt 需要一个值形式（量、开/关或计数），tilt 才能被赋值。_——仍处于 _稍后决定_ 的概念
- _dimByTilt 没有定义。_ / _level 没有有效的定义。_
- _这些关系在同一瞬间相互依赖：a、b。_

只要列出了任何一条，**步进**就被禁用且不做任何事。当前设计的分析尚未到达时，列表显示 _正在检查设计…_，这并不表示有错。打开页面永远不会开始运行。

![Above the trace, an orange-dotted line saying tilt needs a value before simulation can step, with a Show link under it; the Step, Step ×10 and Reset buttons above it are disabled and the counter reads tick 0.](../../../../docs/user-guide/assets/studio/simulate-readiness.png)

_一个阻碍项及其显示链接：输入 tilt 还没有值，所以步进被禁用。_

## 步进、步进 ×10、重置

**步进**用屏幕上的输入求值下一拍；**步进 ×10** 求值十拍。每次步进都是**重放**：求值器带着迄今每一拍的输入和调度从第 0 拍重新开始，然后运行到新的一拍——因此同样的设计、输入和周期总是给出同样的轨迹。**重置**回到第 0 拍；输入和周期保留。

**失败**的一拍——除以零、不是数的值——就停在那里，失败以关于该对象的句子写在控件行上（_bad 除以了零。_），从不作为横幅。

## 轨迹（中间）

行是拍。列是设计的**值**——没有输入的关系——和它的**被驱动的输出**；规则（有输入的关系）没有列，因为它是函数而不是值。_活动_ 列指出走了一拍的域。单元格是求值器自己的渲染，总是带着概念：`Brightness(0.5)`、`Held(true)`。

空单元格表示该值的域在那一拍没有激活。输入的单元格显示你喂入的值，只在其域激活的拍显示。列的顺序按身份，不按时间；点击列标题选中该关系。

记忆和传输以值的形式出现：`acc = delay(0, acc + x)` 在第 0 拍读到 `0`，之后读到上一拍的和；较慢域中的 `y = sync(fast, -1, x)` 读取来源在严格早于自身激活之前的最后一次激活，所以同一瞬间产生的来源值还看不到。

## 探针（右侧）

选中对象**现在**和**整个运行中**的值，带有它的图形符号；对于输出，还有它的驱动方。其下的**解释**保存标识号、运行的修订号、渲染值，以及失败时的代码和技术文本。

## 新修订会做什么

编辑设计会丢弃采样和任何仍在途中的回答，保留仍然存在的输入的喂入值，并重新检查就绪状态。轨迹总是属于设计的某一个修订。

## 本页没有的

画布上的值（轨迹和探针是仅有的视图）、图表（轨迹是表格），以及来自真实设备的遥测（那是监视，尚不存在）。

## 相关

[你的第一次仿真](../getting-started/first-simulation.md) · [时序](../../../../docs/user-guide/concepts/timing.md) · [故障排除：未完成的设计](../../../../docs/user-guide/troubleshooting/incomplete-design.md)
