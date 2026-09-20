<!-- scripts/docs_l10n.py が docs/user-guide/studio/deploy.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/studio/deploy.md) · [简体中文](../../zh_Hans/studio/deploy.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# デプロイ

デプロイページ（⌘3）は _この設計はこのボードに収まるか_ に答えます。配置の確認です。各物理出力のデバイスがボードのどのリソースを使うか——または配置できない最初の理由。ページは 1 列で、**ターゲット**ポップアップ、**判定**、**デバイス**、そしてボードが答えたら**配置**です。ファームウェアのビルドやボードへの書き込みはまだ行いません。[まだないもの](#まだないもの) を参照。

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the green verdict Feasible on Arduino Nano; the Devices section with a card named pwmLight of kind PWM channel realising light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3.](../../../../docs/user-guide/assets/studio/deploy-page.png)

_デプロイページ：Arduino Nano を選択、light に PWM デバイス 1 つ、判定と配置。_

## ターゲット

The boards the compiler service knows. Today: **Arduino Nano**, **Big board (mock)**, a test target with more PWM pins, and **Raspberry Pi Pico (RP2040)**, the first board firmware can be generated for (`bdld compile --target rp2040_pico`, [CLI](../../../../docs/user-guide/reference/cli.md); flashing it is not in Studio yet). The board choice is a **session preference**: it is not saved with the project, and changing it never changes the design.

ボードを選ぶまで、ページは _ボードを選ぶと、この設計が収まるかどうかを確認できます。_ と表示します。

## デバイス

**デバイス**はボード上で 1 つの物理出力を実現します。**デバイスを追加**で行が作られ、そこで次を設定します

- **名前**、
- **種類**——_PWM チャネル_（調光ライト、サーボ信号）、_デジタル出力_（リレー、スイッチ負荷）、_H ブリッジチャネル_（モーター：PWM 線 1 本と方向線 1 本）、_I²C センサー_、_直交エンコーダー_、_UART_、
- それが実現する**出力**（またはなし）、
- その種類の**要件ごとに 1 つのピンフィールド**——空のままにして配置に選ばせるか、ボードのピン名（`D3`、`A4`）を入力して手動で固定します、
- **削除**。

The kind decides what the device needs from the board (a PWM channel needs one PWM-capable pin; an H-bridge needs a PWM pin and a digital pin; an I²C sensor needs SDA and SCL on the same bus). You never edit those requirements; the pin fields and the realization are the only manual choices.

### Realization

Once a board has answered, each device card shows a **Realization** pop-up: how the output's value becomes the command the device takes. The choices are the compiler's profiles — _PWM, 8-bit duty_ (a level 0–100 becomes a duty 0–255), _PWM, 4 levels_ (four duties; nearby levels share one), _I2C register, 8-bit_ (register 42 and a value 0–255), _GPIO, on/off_ (a truth value as written), _H-bridge, signed level_ (direction and duty) — with the ones that fit what the output carries listed first and the others marked _does not fit_. Choosing one also sets the device's kind to what the profile needs. _None — place by kind_ leaves the device placed as before and generates no command.

Beside the pop-up the card shows the three checks a realization must pass — **encoder** (the profile's conversion is a well-typed pure function), **fits** (it converts exactly what this output carries), **placed** (the board has the pins) — and, when one fails, the sentence that says which: _pwmLight cannot realise light with `gpio_level`: the output carries q[1] but the profile encodes bool._ A failing realization blocks deployment until changed; a missing one does not.

A realization is deployment data like the kind and the pins: choosing, changing or removing one changes nothing on the Design page, in the simulator's samples or in any verdict about the design.

デバイスはプロジェクトとともに保存されます。デプロイのデータであって設計のデータではありません。追加、変更、削除しても設計ページの判定は何も変わりません。

[入力元](canvas.md)——環境から与えられる値——にはまだデバイスの割り当てがありません。このページのどの種類も設計に値を与えることはなく、インスペクタの _実現_ 行もそう述べます（_環境から与えられます。デバイスはまだ割り当てられていません。_）。センサー、ボタン、アナログ線を入力元に割り当てるのは配備の仕事で、最初の組み込みプラットフォームとともに来ます。そのとき設計は変わりません。

## 判定

| 判定 | 意味 |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Arduino Nano に配置可能です。** | すべてのデバイスの要件が、適切な能力を持つ互いに異なるピンに配置された。_… 上の配置_ 表がデバイス · 要件 · → ピンを列挙する |
| **今のところ Arduino Nano に収まります — バインディングは未完了です。** | バインド済みの部分は収まるが、デバイスのない出力（_… 上に対応するデバイスがありません：…_）か出力のないデバイス（_出力に接続されていません：…_）がある |
| **Arduino Nano には配置できません。** | ある要件が配置できなかった。赤い箱がそれと理由を示す——_arduino_nano 上に pwmLight の PWM を担えるものがありません。_（能力を持つピンがない）、_arduino_nano 上で D4 は pwmLight の PWM を担えません。_ と _手動で選んだピン D4 は、ここでは … を担えません。_（手動で固定したピン）、あるいはそれを阻むピンとそれぞれを占めているもの。_行き止まりの前に配置済み：_ は配置されたものを列挙する |

判定はボードについてだけです。**設計**自体の準備——すべての関係が検査を通り、瞬時サイクルがなく、すべての必須出力が駆動されている——は設計ページの仕事で、ここでは繰り返しません。数式が間違った設計でも _配置可能_ にはなりえ、このページにいる間も下部のステータス行は _瞬時サイクルあり_ や _出力が未完了_ を表示し続けます。何かが動くには両方が成り立つ必要があります。

_配置不可_ は配置がその順序で突き当たった**最初の行き止まり**を報告します——正直に名指しされた 1 つの衝突であり、唯一のものとは限りません。

## 「配置可能」が含まないもの

電気的・数値的な制約：電流、電圧、PWM 周波数、タイマー分解能、バス速度。_配置可能_ はピンを割り当てられることであって、回路が動くことではありません。後の層が _配置可能_ を狭めることはあっても、広げることは決してありません。

## まだないもの

デプロイページからのファームウェアの生成とビルド、ボードへの書き込み、ボードからの値の読み戻し。コンパイラはすでに設計の Rust コアを生成でき、リポジトリのテストでシミュレータとトレースごとに照合していますが、その経路にはまだ Studio の画面がありません。`docs/project/roadmap.md` のロードマップに挙げられています。

## 関連

[最初のデプロイ確認](../getting-started/first-deployment.md) · [物理出力](../../../../docs/user-guide/concepts/physical-outputs.md) · [トラブルシューティング：デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md)
