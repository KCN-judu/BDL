<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/what-is-bdl.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/what-is-bdl.md) · [简体中文](../../zh_Hans/getting-started/what-is-bdl.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# BDL とは？

BDL——Behavior Design Language——は、製品が何をするかを**値の意味**と**値どうしの関係**によって記述する方法です。これにより、設計はプログラミングの前に検査し、シミュレートし、ハードウェアに配置できます。Behavior Designer は BDL の設計を描き、入力し、検査し、実行するためのデスクトップツールです。

このページでは 5 つの考え方でメンタルモデルを示します。それぞれ [コンセプト](../../../../docs/user-guide/concepts/concepts.md) に専用ページがあります。ここでは [最初のチュートリアル](first-behavior.md) の前に必要な分だけ説明します。

## 1. A concept is a kind of value; a block is one such value

A lamp has a _Tilt_, a _Brightness_, maybe an _Ambient light_. Each is a **concept**: a named _kind_ of value the product senses, decides or shows — a type, and a template. A concept has a _value form_ — a quantity with a unit (an angle, a length, a temperature, or a plain number), an on/off state, or a count.

The things that actually hold values are **blocks** (_Sem blocks_ in the [terminology](../../../../docs/user-guide/reference/terminology.md)): a block is one instance of a concept in the design — `tilt`, a Tilt; `brightness`, a Brightness — and it has one value at each tick. A product may have as many blocks of one concept as it has such values: two temperature sensors are two blocks of _Temperature_. Concept : block = type : instance.

Two concepts with the same value form are still different concepts. _Brightness_ and _Opacity_ may both be numbers between 0 and 1; BDL will not let you use a block of one where the other is expected, because they mean different things. This is deliberate, and it is the first thing that makes a BDL design more than a spreadsheet.

## 2. A rule says how one value follows from others; a block's formula applies it

`dimByTilt` reads a _Tilt_ and produces a _Brightness_. That is a **rule**: a named template from some concepts to a concept, with a formula — `Tilt / 90 deg` — that is checked for units and dimensions: dividing an angle by an angle gives a plain number, which is what a brightness is. Adding an angle to a time would be refused, with the reason.

A rule computes nothing by itself. A block gets its value from its own formula — `brightness = dimByTilt(tilt)` — which applies the rule to the blocks it reads; that formula is drawn on the canvas as the block's **mapping block**, joined to the blocks it reads and to the block it defines. Rule : mapping block = template : application. A block with no formula is a **Source**: its value arrives from outside (a sensor reading, a switch). In Studio a rule and a block are both created as a _relationship_ (labelled _Mapping_): one that reads something is a rule; one that reads nothing is a block.

## 3. 設計は意図的に未完成でいられる

You can create `dimByTilt` before you know its formula. It is then _declared_: it exists, it has a signature, and everything that depends on it can be designed around it. The tool marks it as open work, not as an error — and a block that has no formula yet is a Source, provided from outside until you say otherwise. The same goes for a concept whose value form you have not chosen yet. [Incomplete designs](../../../../docs/user-guide/concepts/incomplete-designs.md) explains the states a design can be in and why none of them stops you working.

## 4. タイミングは明示的

製品内の値は異なるリズムで更新されます。タッチセンサーはあるレートで、温度センサーは別のレートで。BDL では、独自のリズムで更新されるすべての値が名前付きの**タイミングドメイン**に属します。別のドメインの値を読む関係はそう明記し、最初の読み値が届く前に使う値を与えなければなりません。記憶——前のティックの値を使うこと——は数式に書かれ、隠されません。[タイミング](../../../../docs/user-guide/concepts/timing.md) は設計者の疑問から出発して規則へ進みます。

## 5. 出力は振る舞いが設計を離れる場所

A **physical output** — the light, the motor, the display — accepts values of one concept and is driven by exactly one block. A second block trying to drive the same output is a conflict the tool names, not a race it resolves. Which board the design will run on is a separate question: **deployment** checks whether the outputs' devices can be placed on a chosen board's pins, and never changes what the design means.

## ランプからシステムへ

設計にまとまりのある関係がいくつかできたら、それらを _振る舞い_ として**グループ化**できます。これは設計を整理し眺める方法であり、設計が何をするかは何も変わりません。振る舞いは、明示的な境界（必要なもの、提供するもの）を持つ再利用可能な _コンポーネント_ として**パッケージ化**できます。コンポーネントは _インスタンス_ として配置され、配線されて _振る舞いシステム_ になります。[コンセプトページ](../../../../docs/user-guide/concepts/behavior-groups.md) はこれらを 1 つずつ扱い、[ワークフロー](../../../../docs/user-guide/workflows/grouping-behavior.md) はチュートリアルで作る同じランプで順に進めます。

## 次へ

[インストールと起動](install-and-launch.md)、続いて [最初の振る舞い](first-behavior.md)。
