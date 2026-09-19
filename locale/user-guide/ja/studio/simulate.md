<!-- scripts/docs_l10n.py が docs/user-guide/studio/simulate.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/studio/simulate.md) · [简体中文](../../zh_Hans/studio/simulate.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# シミュレート

シミュレートページ（⌘2）は _設計は時間とともに何をするか_ に、1 ティックずつ答えます。すべての値はコンパイラサービスの参照評価器——BDL の設計の意味の実行可能な定義——から来ます。Studio は入力した値、選んだ周期、返ってきたサンプルを保持するだけで、自分では何も計算しません。

![The Simulate page: on the left, under Sources, a control for tilt showing 0.785398 rad and the interaction domain's period of every 1 ticks; in the middle the Step, Step ×10 and Reset buttons, tick 3, and a trace with three rows whose tilt, brightness and light columns read Tilt(0.785398 [rad]), Brightness(0.5) and Brightness(0.5); on the right the probe for brightness with Brightness(0.5) now and at each tick over the run.](../assets/studio/simulate-page.png)

_傾き 45° で 3 ステップ後のシミュレートページ：左に入力元、中央にトレース、右にプローブ。_

## 入力元（左）

One control for every [Source](canvas.md) — every relationship that reads nothing and has no formula: the values the environment provides, which the simulation asks you for. The control follows the concept's value form, never its name: a number with the unit beside it for a quantity (in the base unit: radians, metres, seconds, kelvin…), an **off | on** control for on / off, a whole number for a count. The row carries the concept's glyph and colour, and clicking it selects the object — on this page and on Design.

A Source you have not given a value yet says so: a number field shows the hint _no value yet_, and the on / off control is drawn empty with a dashed outline and the words _no value yet_ beside it — the same dashes that mark a declared relationship, for the same reason: nothing has been decided. It is not _off_. One click on **off** or **on** gives exactly that value; the simulator never fills one in for you, because a missing Source is an error at run time, not a default.

入力元の下に**タイミングドメイン**。ドメインごとの周期——_N ティックごと_——評価器が起動するスケジュールです。周期であって、レートではありません。周期を変えると実行は最初からやり直されます。

インスタンスの**オープンな要求ポート**も入力元であり、インスタンスの値は `lampA.brightness` のように列挙されます。

## 準備状態（トレースの上）

何かが動く前に、ページはステップを止めるものを、名前付きオブジェクトについての文として、それぞれを選択する _表示_ リンク付きで列挙します：

- _tilt にはシミュレーションをステップする前に値が必要です。_——値のない入力元
- _Tilt には、tilt に値を与える前に値の形式（量、オン/オフ、個数）が必要です。_——まだ _後で決める_ のままのコンセプト
- _dimByTilt には定義がありません。_ / _level には有効な定義がありません。_
- _これらの関係は同じ瞬間に互いに依存しています：a、b。_

いずれかが列挙されている間、**ステップ**は無効で何もしません。現在の設計の解析が届いていない間、リストは _設計を検査中…_ と述べ、何も問題はありません。ページを開いても実行は始まりません。

Below the blockers, with a hollow dot instead of a filled one, the page lists what does **not** stop a step but explains what the trace will not show: a **rule nothing applies**. A rule — a relationship with inputs — is a function, so it has no value per tick and no column; only a value that calls it does. The note names the rule and the value that would put it to work:

- _AirConditionerCtrl is a rule nothing applies yet._ — _A rule has no value of its own; a value that applies it — `AirConditionerCtrl(TempSensor, ButtonInput)` — is what the simulator and an output can read._

Under it, the fix **Add a value that applies AirConditionerCtrl** and a _Show_ link. The fix creates `airConditionerCtrl : () -> SwitchState = AirConditionerCtrl(TempSensor, ButtonInput)` in one click when each concept the rule reads has exactly one value producing it; when one has several, the button becomes a pop-up of the calls to choose from; when one has none, the button says why (_Not possible yet: no value produces `RoomTemp` yet; add a Source or a computed value that produces it first_). The tool never guesses. A rule that has no definition yet is a blocker first (_has no definition_) and is not repeated here.

![Above the trace, an orange-dotted line saying tilt needs a value before simulation can step, with a Show link under it; the Step, Step ×10 and Reset buttons above it are disabled and the counter reads tick 0.](../../../../docs/user-guide/assets/studio/simulate-readiness.png)

_表示リンク付きの妨げ：入力元 tilt にまだ値がないので、ステップは無効。_

## ステップ、ステップ ×10、リセット

**ステップ**は画面上の入力元で次のティックを評価し、**ステップ ×10** は 10 ティック評価します。すべてのステップは**再生**です。評価器はこれまでの各ティックの入力元とスケジュールとともにティック 0 から再開し、新しいティックまで実行します——だから同じ設計、入力元、周期は常に同じトレースを与えます。**リセット**はティック 0 に戻り、入力元と周期は残ります。

**失敗**したティック——ゼロ除算、数でない値——はそこで止まり、失敗はコントロール行にそのオブジェクトについて書かれます（_bad でゼロ除算が発生しました。_）。バナーにはなりません。

## トレース（中央）

Rows are ticks. Columns are the design's **values** — relationships without inputs — and its **driven outputs**; a rule (a relationship with inputs) has no column, because it is a function, not a value — `dimByTilt` never appears, the value `brightness = dimByTilt(tilt)` does. If a rule seems ignored by the simulation, the readiness area says so and offers the value that applies it. The _active_ column names the domains that ticked. A cell is the evaluator's own rendering, always with the concept and in the value form's words: `Brightness(0.5)`, `Tilt(0.785398 [rad])`, `Held(on)` — a truth value reads _on_ / _off_, a number shows six significant digits. Studio never renders a value itself.

空のセルは、その値のドメインがそのティックで起動しなかったことを意味します。入力元のセルは与えた値で、そのドメインが起動したティックに評価器がエコーしたものです。列の順序は同一性によるもので、時間によるものではありません。列見出しをクリックするとその関係が選択されます。

記憶と転送は値として現れます。`acc = delay(0, acc + x)` はティック 0 で `0`、その後は前のティックの和を読みます。遅いドメインの `y = sync(fast, -1, x)` は、自身の起動より厳密に前の入力元の最後の起動を読むので、同じ瞬間に生成された入力元の値はまだ見えません。

## プローブ（右）

The selected object's value **now** and **over the run**, written as in the trace, with its glyph; for an output, its driver. A concept is shown as what **carries** it — _Carried by_, then each value and Source that produces it with its latest sample; when none does, _Nothing carries Brightness yet: no value or Source produces it._ and, for each rule that produces it, _dimByTilt is a rule; a value whose formula applies it would carry Brightness._ A rule has no value to show: _A rule: it has no value of its own. A value whose formula applies it is what the simulator samples._, then, under _Applied in_, links to the values whose formula applies it — or _No value applies it yet._ with the same fix the readiness area offers. **Explain** under it holds the identity number, the run's revision and, for a failure, the code and technical text.

## 新しいリビジョンが行うこと

設計を編集するとサンプルと処理中の応答は捨てられ、まだ存在する入力に与えた値は保持され、準備状態が再検査されます。トレースは常に設計の 1 つのリビジョンに属します。

## このページにないもの

キャンバス上の値（トレースとプローブが唯一のビューです）、プロット（トレースは表です）、実機からのテレメトリ（それはモニターで、まだ存在しません）。

## 関連

[Your first simulation](../getting-started/first-simulation.md) · [Relationships](../../../../docs/user-guide/concepts/relationships.md) · [Timing](../../../../docs/user-guide/concepts/timing.md) · [Troubleshooting: incomplete design](../../../../docs/user-guide/troubleshooting/incomplete-design.md)
