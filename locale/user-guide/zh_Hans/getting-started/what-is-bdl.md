<!-- 由 scripts/docs_l10n.py 从 docs/user-guide/getting-started/what-is-bdl.md 生成；请编辑 locale/user-guide/zh_Hans/user-guide.po，不要编辑本文件。 -->

> 语言: [English](../../../../docs/user-guide/getting-started/what-is-bdl.md) · 简体中文 · [日本語](../../ja/getting-started/what-is-bdl.md)
>
> 本页尚未完全翻译；未翻译的段落以英文显示。

# 什么是 BDL？

BDL——行为设计语言（Behavior Design Language）——是一种描述产品做什么的方式，它从**值的含义**和**值之间如何关联**出发，使设计能在编程之前被检查、仿真并放到硬件上。Behavior Designer 是用来绘制、输入、检查和运行 BDL 设计的桌面工具。

本页用五个想法给出心智模型。每一个在 [概念](../../../../docs/user-guide/concepts/concepts.md) 中都有自己的页面；这里只讲到 [第一个教程](first-behavior.md) 之前你需要的程度。

## 1. A concept is a kind of value; a block is one such value

A lamp has a _Tilt_, a _Brightness_, maybe an _Ambient light_. Each is a **concept**: a named _kind_ of value the product senses, decides or shows — a type, and a template. A concept has a _value form_ — a quantity with a unit (an angle, a length, a temperature, or a plain number), an on/off state, or a count.

The things that actually hold values are **blocks** (_Sem blocks_ in the [terminology](../../../../docs/user-guide/reference/terminology.md)): a block is one instance of a concept in the design — `tilt`, a Tilt; `brightness`, a Brightness — and it has one value at each tick. A product may have as many blocks of one concept as it has such values: two temperature sensors are two blocks of _Temperature_. Concept : block = type : instance.

Two concepts with the same value form are still different concepts. _Brightness_ and _Opacity_ may both be numbers between 0 and 1; BDL will not let you use a block of one where the other is expected, because they mean different things. This is deliberate, and it is the first thing that makes a BDL design more than a spreadsheet.

## 2. A rule says how one value follows from others; a block's formula applies it

`dimByTilt` reads a _Tilt_ and produces a _Brightness_. That is a **rule**: a named template from some concepts to a concept, with a formula — `Tilt / 90 deg` — that is checked for units and dimensions: dividing an angle by an angle gives a plain number, which is what a brightness is. Adding an angle to a time would be refused, with the reason.

A rule computes nothing by itself. A block gets its value from its own formula — `brightness = dimByTilt(tilt)` — which applies the rule to the blocks it reads; that formula is drawn on the canvas as the block's **mapping block**, joined to the blocks it reads and to the block it defines. Rule : mapping block = template : application. A block with no formula is a **Source**: its value arrives from outside (a sensor reading, a switch). In Studio a rule and a block are both created as a _relationship_ (labelled _Mapping_): one that reads something is a rule; one that reads nothing is a block.

## 3. 设计可以有意地保持未完成

You can create `dimByTilt` before you know its formula. It is then _declared_: it exists, it has a signature, and everything that depends on it can be designed around it. The tool marks it as open work, not as an error — and a block that has no formula yet is a Source, provided from outside until you say otherwise. The same goes for a concept whose value form you have not chosen yet. [Incomplete designs](../../../../docs/user-guide/concepts/incomplete-designs.md) explains the states a design can be in and why none of them stops you working.

## 4. 时序是显式的

产品中的值以不同节奏更新：触摸传感器一种速率，温度传感器另一种。在 BDL 中，每个按自己节奏更新的值都属于一个有名字的**时序域**；读取另一个域的值的关系必须明确说明，并给出第一个读数到来之前要使用的值。记忆——使用上一拍的值——写在公式里，而不是藏起来。[时序](../../../../docs/user-guide/concepts/timing.md) 从设计师会问的问题出发，逐步引出规则。

## 5. 输出是行为离开设计的地方

A **physical output** — the light, the motor, the display — accepts values of one concept and is driven by exactly one block. A second block trying to drive the same output is a conflict the tool names, not a race it resolves. Which board the design will run on is a separate question: **deployment** checks whether the outputs' devices can be placed on a chosen board's pins, and never changes what the design means.

## 从一盏灯长成一个系统

当设计中有几个关系应当放在一起时，你可以把它们**分组**为一个 _行为_——这是组织和查看设计的一种方式，不改变设计做什么。行为可以被**打包**为可复用的 _组件_，带有明确的边界（它需要什么，提供什么）。组件随后作为 _实例_ 被放置，并连接成一个 _行为系统_。[概念页面](../../../../docs/user-guide/concepts/behavior-groups.md) 逐个讲解这些内容；[工作流](../../../../docs/user-guide/workflows/grouping-behavior.md) 在你于教程中搭建的同一盏灯上演练它们。

## 下一步

[安装与启动](install-and-launch.md)，然后是 [你的第一个行为](first-behavior.md)。
