<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/first-deployment.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/first-deployment.md) · [简体中文](../../zh_Hans/getting-started/first-deployment.md) · 日本語

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

判定は **Arduino Nano に配置可能です。** に変わり、_arduino_nano 上の配置_ 表にランプが必要とする 1 行が示されます。`pwmLight`、その _PWM_ 要件、そして割り当てられたピン（`D3` など）です。

**作ったもの。** 1 つのボードに対するデプロイ構成。設計とともにプロジェクトに保存されますが、別の層です。デバイスを追加しても設計ページの判定は変わらず、別のボードを選んでも変わりません。

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the green verdict Feasible on Arduino Nano; the Devices section with a card named pwmLight of kind PWM channel realising light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_デプロイページ：Arduino Nano を選択、light に PWM デバイス 1 つ、判定と配置。_

## 4. わざと失敗させる

各デバイス行には要件ごとにピンフィールドがあります。`pwmLight` のピンフィールドに `D4` と入力します——Nano では PWM ができないピンです。

> Arduino Nano には配置できません。
> arduino_nano 上で D4 は pwmLight の PWM を担えません。手動で選んだピン D4 は、ここでは pwmLight の PWM を担えません。

フィールドをクリアすると、再び配置可能になります。次に**ターゲット**を **Big board (mock)** に切り替えます。そこでも別のピンで配置可能です。元に戻します。

**何が起きたか。** 配置可能性は _このボード上のこの設計_ についての事実です。ボードの選択はセッションの設定で——プロジェクトとともには保存されません——配置は選ばれたボードに対して再計算されます。

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry pwmLight PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../../../../docs/user-guide/assets/getting-started/deploy-dead-end.png)

_配置不可：手動で選んだピンは PWM を担えない。_

## 「配置可能」が意味すること、しないこと

_配置可能_ とは、すべてのデバイスの要件を、適切な能力を持つ互いに異なるピンに配置できることです。回路が電気的に動くことは意味しません。電流、電圧、タイミング分解能はモデル化されていません。複数の配置が可能なときツールは 1 つを示し、どれも不可能なときは最初に突き当たった行き止まりを示します。それは _1 つの_ 衝突であり、唯一のものとは限りません。

## うまくいかないとき

- **ステータス行にまだ _瞬時サイクルあり_ か _出力が未完了_ とある。** デプロイの判定はボードについてだけです。設計自体が準備できていません。まず設計ページで直してください。[デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md) を参照。
- **出力に接続されていません：…**——出力が選ばれていないデバイス。
- **Arduino Nano 上に … を担えるものがありません**——ボードにその能力を持つ空きピンがありません。別の種類のデバイス、別のボード、またはより少ないデバイスを試してください。

## 次へ

振る舞いを作り、シミュレートし、配置しました。使った考え方については [コンセプト](../../../../docs/user-guide/concepts/concepts.md) のページを読むか、直接 [センサーから出力へ](../../../../docs/user-guide/workflows/sensor-to-output.md) に進んで、ランプに環境光センサーを追加してください。
