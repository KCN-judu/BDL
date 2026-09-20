<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/first-deployment.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/first-deployment.md) · [简体中文](../../zh_Hans/getting-started/first-deployment.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# 最初のデプロイ確認

**目標。** [最初のチュートリアル](first-behavior.md) のランプが Arduino Nano に収まるかを確かめ、「収まらない」がどう見えるかを見ます。

**所要時間。** 10 分。

今日の Behavior Designer におけるデプロイとは _配置の確認_ です。各物理出力のデバイスがボードのどのピンを使うか、あるいはなぜどのピンも使えないか。ファームウェアのビルドやボードへの書き込みはまだ行いません。

## 1. デプロイページを開く

ページバーで**デプロイ**をクリックします（または **⌘3**）。左に**ターゲット**ポップアップと**デバイス**リスト、中央に選んだボードに対する判定があります。

ボードを選ぶまで、ページには _ボードを選ぶと、この設計が収まるかどうかを確認できます。_ と表示されます。

## 2. ボードを選ぶ

**ターゲット**で **Arduino Nano** を選びます。判定は次のとおりです：

> 今のところ Arduino Nano に収まります — バインディングは未完了です。

その下に：_arduino_nano 上に対応するデバイスがありません：light。_（この行はボードを短い id で呼びます）。設計には出力がありますが、どのハードウェアがそれを実現するかを何も述べていません。

## 3. デバイスを追加する

**デバイス**はボード上で 1 つの出力を実現するハードウェアです。調光ライトには PWM チャネル、リレーにはデジタル出力、モーターには H ブリッジ。

1. **デバイスを追加**をクリックします。_デバイス_ に行が現れます。
2. `pwmLight` と名付けます。種類のポップアップで **PWM チャネル**を選びます。出力のポップアップで **light** を選びます。

The line about `light` disappears and a _Placement on arduino_nano_ table shows the one line the lamp needs: `pwmLight`, its _PWM_ requirement, and the pin it was given, such as `D3`. The verdict stays **Fits Arduino Nano so far — the binding is not finished.**, and one line remains under it: _tilt has no device on Arduino Nano._ The lamp's output is placed; its Source is not.

**作ったもの。** 1 つのボードに対するデプロイ構成。設計とともにプロジェクトに保存されますが、別の層です。デバイスを追加しても設計ページの判定は変わらず、別のボードを選んでも変わりません。

**About `tilt`.** A [Source](../../../../docs/user-guide/concepts/relationships.md) is a value the environment provides, and on a board a device has to provide it — the same kind of binding as for an output, chosen in the device's pop-up (_tilt — Source_) with a **Provider** instead of a Realization. The providers this build has read a digital line as on/off; `tilt` is an angle, so nothing fits it yet, and the deployment honestly stays _not finished_. That is a state, not an error: the design simulates, and the placement of `light` is real.

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the placement, and the Source still to provide._

## 4. わざと失敗させる

各デバイス行には要件ごとにピンフィールドがあります。`pwmLight` のピンフィールドに `D4` と入力します——Nano では PWM ができないピンです。

> Arduino Nano には配置できません。
> arduino_nano 上で D4 は pwmLight の PWM を担えません。手動で選んだピン D4 は、ここでは pwmLight の PWM を担えません。

Clear the field: it fits again. Now switch **Target** to **Big board (mock)**: it fits there too, on a different pin. Switch back.

**何が起きたか。** 配置可能性は _このボード上のこの設計_ についての事実です。ボードの選択はセッションの設定で——プロジェクトとともには保存されません——配置は選ばれたボードに対して再計算されます。

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry pwmLight PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../../../../docs/user-guide/assets/getting-started/deploy-dead-end.png)

_配置不可：手動で選んだピンは PWM を担えない。_

## 「配置可能」が意味すること、しないこと

_配置可能_ とは、すべてのデバイスの要件を、適切な能力を持つ互いに異なるピンに配置できることです。回路が電気的に動くことは意味しません。電流、電圧、タイミング分解能はモデル化されていません。複数の配置が可能なときツールは 1 つを示し、どれも不可能なときは最初に突き当たった行き止まりを示します。それは _1 つの_ 衝突であり、唯一のものとは限りません。

## うまくいかないとき

- **ステータス行にまだ _瞬時サイクルあり_ か _出力が未完了_ とある。** デプロイの判定はボードについてだけです。設計自体が準備できていません。まず設計ページで直してください。[デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md) を参照。
- **Not connected to an output or Source: …** — a device with nothing chosen in its pop-up.
- **… has no device on Arduino Nano.** — a Source no device provides; the deployment stays _not finished_ until one does.
- **Arduino Nano 上に … を担えるものがありません**——ボードにその能力を持つ空きピンがありません。別の種類のデバイス、別のボード、またはより少ないデバイスを試してください。

## 次へ

振る舞いを作り、シミュレートし、配置しました。使った考え方については [コンセプト](../../../../docs/user-guide/concepts/concepts.md) のページを読むか、直接 [センサーから出力へ](../../../../docs/user-guide/workflows/sensor-to-output.md) に進んで、ランプに環境光センサーを追加してください。
