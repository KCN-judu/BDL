<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/first-deployment.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/first-deployment.md) · [简体中文](../../zh_Hans/getting-started/first-deployment.md) · 日本語

# 最初のデプロイ確認

**目標。** [最初のチュートリアル](first-behavior.md) のランプが Arduino Nano に収まるかを確かめ、「収まらない」がどう見えるかを見ます。

**所要時間。** 10 分。

このページは _配置チェック_ です。各物理出力のデバイスがボードのどのピンを使うか、あるいはなぜどのピンも使えないかを調べます。ファームウェアをビルドしてボードに載せるのは次のページ——[はじめてのボード](pico-demo.md)——で、このビルドがファームウェアを持つボードが必要です。ここでの Nano は配置の練習です。

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

`light` についての行が消え、_arduino_nano 上の配置_ の表にランプが必要とする唯一の行が現れます。`pwmLight`、その _PWM_ 要件、そして割り当てられたピン（`D3` など）です。判定は**今のところ Arduino Nano に収まります — バインディングは未完了です。**のままで、その下に一行残ります。_tilt には Arduino Nano 上のデバイスがありません。_ ランプの出力は配置されましたが、その入力元はまだです。

**作ったもの。** 1 つのボードに対するデプロイ構成。設計とともにプロジェクトに保存されますが、別の層です。デバイスを追加しても設計ページの判定は変わらず、別のボードを選んでも変わりません。

**`tilt` について。** [入力元](../../../../docs/user-guide/concepts/relationships.md)は環境が与える値で、ボード上ではデバイスがそれを提供しなければなりません——出力と同じ種類のバインディングを、デバイスのポップアップ（_tilt — 入力元_）で選び、実現方式の代わりに**提供方式**を選びます。このビルドの提供方式はデジタル線をオン/オフとして読むものだけです。`tilt` は角度なので、まだ適合するものがなく、デプロイは正直に _未完了_ のままです。それは状態であってエラーではありません。設計はシミュレートでき、`light` の配置は本物です。

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_デプロイページ：Arduino Nano を選択、light に 1 つの PWM デバイス、配置、そしてまだ提供されていない入力元。_

## 4. わざと失敗させる

各デバイス行には要件ごとにピンフィールドがあります。`pwmLight` のピンフィールドに `D4` と入力します——Nano では PWM ができないピンです。

> Arduino Nano には配置できません。
> arduino_nano 上で D4 は pwmLight の PWM を担えません。手動で選んだピン D4 は、ここでは pwmLight の PWM を担えません。

フィールドを空にすると、また収まります。**ターゲット**を **Big board (mock)** に切り替えてみてください。そこでも別のピンで収まります。元に戻します。

**何が起きたか。** 配置可能性は _このボード上のこの設計_ についての事実です。ボードの選択はセッションの設定で——プロジェクトとともには保存されません——配置は選ばれたボードに対して再計算されます。

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry pwmLight PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../../../../docs/user-guide/assets/getting-started/deploy-dead-end.png)

_配置不可：手動で選んだピンは PWM を担えない。_

## 「配置可能」が意味すること、しないこと

_配置可能_ とは、すべてのデバイスの要件を、適切な能力を持つ互いに異なるピンに配置できることです。回路が電気的に動くことは意味しません。電流、電圧、タイミング分解能はモデル化されていません。複数の配置が可能なときツールは 1 つを示し、どれも不可能なときは最初に突き当たった行き止まりを示します。それは _1 つの_ 衝突であり、唯一のものとは限りません。

## うまくいかないとき

- **ステータス行にまだ _瞬時サイクルあり_ か _出力が未完了_ とある。** デプロイの判定はボードについてだけです。設計自体が準備できていません。まず設計ページで直してください。[デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md) を参照。
- **出力または入力元に未接続：…** — ポップアップで何も選ばれていないデバイスです。
- **… には Arduino Nano 上のデバイスがありません。** — どのデバイスも提供していない入力元です。提供されるまでデプロイは _未完了_ のままです。
- **Arduino Nano 上に … を担えるものがありません**——ボードにその能力を持つ空きピンがありません。別の種類のデバイス、別のボード、またはより少ないデバイスを試してください。

## 次へ

振る舞いを作り、シミュレートし、配置しました。手元に Raspberry Pi Pico があれば、[はじめてのボード](pico-demo.md)で設計をビルドして動かせます。あるいは使った考え方を[コンセプト](../../../../docs/user-guide/concepts/concepts.md)の各ページで読むか、[センサーから出力へ](../../../../docs/user-guide/workflows/sensor-to-output.md)に進んでランプに環境光センサーを足してください。
