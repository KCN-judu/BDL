<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/studio/simulate.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/studio/simulate.md) · 简体中文 · [日本語](../../ja/studio/simulate.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 仿真

仿真页（⌘2）回答 _设计随时间做什么_，一拍一拍地回答。每个值都来自编译器服务的参考求值器——BDL 设计含义的可执行定义。Studio 保存你输入的值、你选择的周期和返回的采样；它自己不做任何计算。

![The Simulate page: on the left, under Sources, a control for tilt showing 0.785398 rad and the interaction domain's period of every 1 ticks; in the middle the Step, Step ×10 and Reset buttons, tick 3, and a trace with three rows whose tilt, brightness and light columns read Tilt(0.785398 [rad]), Brightness(0.5) and Brightness(0.5); on the right the probe for brightness with Brightness(0.5) now and at each tick over the run.](../assets/studio/simulate-page.png)

_倾角为 45° 时步进三次后的仿真页：左侧是来源，中间是轨迹，右侧是探针。_

## 来源（左侧）

One control for every [Source](canvas.md) — every relationship that reads nothing and has no formula: the values the environment provides, which the simulation asks you for. The control follows the concept's value form, never its name: a number with the unit beside it for a quantity (in the base unit: radians, metres, seconds, kelvin…), an **off | on** control for on / off, a whole number for a count. The row carries the concept's glyph and colour, and clicking it selects the object — on this page and on Design.

A Source you have not given a value yet says so: a number field shows the hint _no value yet_, and the on / off control is drawn empty with a dashed outline and the words _no value yet_ beside it — the same dashes that mark a declared relationship, for the same reason: nothing has been decided. It is not _off_. One click on **off** or **on** gives exactly that value; the simulator never fills one in for you, because a missing Source is an error at run time, not a default.

来源下方是**时序域**：每个域一个周期——_每 N_ 拍——求值器按此调度激活。是周期，从不是速率。更改周期会重新开始运行。

实例的**开放的必需端口**也是来源，实例的值以 `lampA.brightness` 的形式列出。

## 就绪状态（轨迹上方）

在任何东西运行之前，页面用关于具名对象的句子列出会阻止步进的事项，每条附有选中它的 _显示_ 链接：

- _tilt 需要一个值仿真才能步进。_——没有值的来源
- _Tilt 需要一个值形式（量、开/关或计数），tilt 才能被赋值。_——仍处于 _稍后决定_ 的概念
- _dimByTilt 没有定义。_ / _level 没有有效的定义。_
- _这些关系在同一瞬间相互依赖：a、b。_

只要列出了任何一条，**步进**就被禁用且不做任何事。当前设计的分析尚未到达时，列表显示 _正在检查设计…_，这并不表示有错。打开页面永远不会开始运行。

Below the blockers, with a hollow dot instead of a filled one, the page lists what does **not** stop a step but explains what the trace will not show: a **rule nothing applies**. A rule — a relationship with inputs — is a function, so it has no value per tick and no column; only a value that calls it does. The note names the rule and the value that would put it to work:

- _AirConditionerCtrl is a rule nothing applies yet._ — _A rule has no value of its own; a value that applies it — `AirConditionerCtrl(TempSensor, ButtonInput)` — is what the simulator and an output can read._

Under it, the fix **Add a value that applies AirConditionerCtrl** and a _Show_ link. The fix creates `airConditionerCtrl : () -> SwitchState = AirConditionerCtrl(TempSensor, ButtonInput)` in one click when each concept the rule reads has exactly one value producing it; when one has several, the button becomes a pop-up of the calls to choose from; when one has none, the button says why (_Not possible yet: no value produces `RoomTemp` yet; add a Source or a computed value that produces it first_). The tool never guesses. A rule that has no definition yet is a blocker first (_has no definition_) and is not repeated here.

![Above the trace, an orange-dotted line saying tilt needs a value before simulation can step, with a Show link under it; the Step, Step ×10 and Reset buttons above it are disabled and the counter reads tick 0.](../../../../docs/user-guide/assets/studio/simulate-readiness.png)

_一个阻碍项及其显示链接：来源 tilt 还没有值，所以步进被禁用。_

## 步进、步进 ×10、重置

**步进**用屏幕上的来源求值下一拍；**步进 ×10** 求值十拍。每次步进都是**重放**：求值器带着迄今每一拍的来源和调度从第 0 拍重新开始，然后运行到新的一拍——因此同样的设计、来源和周期总是给出同样的轨迹。**重置**回到第 0 拍；来源和周期保留。

**失败**的一拍——除以零、不是数的值——就停在那里，失败以关于该对象的句子写在控件行上（_bad 除以了零。_），从不作为横幅。

## 轨迹（中间）

Rows are ticks. Columns are the design's **values** — relationships without inputs — and its **driven outputs**; a rule (a relationship with inputs) has no column, because it is a function, not a value — `dimByTilt` never appears, the value `brightness = dimByTilt(tilt)` does. If a rule seems ignored by the simulation, the readiness area says so and offers the value that applies it. The _active_ column names the domains that ticked. A cell is the evaluator's own rendering, always with the concept and in the value form's words: `Brightness(0.5)`, `Tilt(0.785398 [rad])`, `Held(on)` — a truth value reads _on_ / _off_, a number shows six significant digits. Studio never renders a value itself.

空单元格表示该值的域在那一拍没有激活。来源的单元格是你喂入的值，由求值器在其域激活的拍回显。列的顺序按身份，不按时间；点击列标题选中该关系。

记忆和传输以值的形式出现：`acc = delay(0, acc + x)` 在第 0 拍读到 `0`，之后读到上一拍的和；较慢域中的 `y = sync(fast, -1, x)` 读取来源在严格早于自身激活之前的最后一次激活，所以同一瞬间产生的来源值还看不到。

## 探针（右侧）

The selected object's value **now** and **over the run**, written as in the trace, with its glyph; for an output, its driver. A concept is shown as what **carries** it — _Carried by_, then each value and Source that produces it with its latest sample; when none does, _Nothing carries Brightness yet: no value or Source produces it._ and, for each rule that produces it, _dimByTilt is a rule; a value whose formula applies it would carry Brightness._ A rule has no value to show: _A rule: it has no value of its own. A value whose formula applies it is what the simulator samples._, then, under _Applied in_, links to the values whose formula applies it — or _No value applies it yet._ with the same fix the readiness area offers. **Explain** under it holds the identity number, the run's revision and, for a failure, the code and technical text.

## 新修订会做什么

编辑设计会丢弃采样和任何仍在途中的回答，保留仍然存在的输入的喂入值，并重新检查就绪状态。轨迹总是属于设计的某一个修订。

## 本页没有的

画布上的值（轨迹和探针是仅有的视图）、图表（轨迹是表格），以及来自真实设备的遥测（那是监视，尚不存在）。

## 相关

[Your first simulation](../getting-started/first-simulation.md) · [Relationships](../../../../docs/user-guide/concepts/relationships.md) · [Timing](../../../../docs/user-guide/concepts/timing.md) · [Troubleshooting: incomplete design](../../../../docs/user-guide/troubleshooting/incomplete-design.md)
