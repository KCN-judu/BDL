<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/pico-demo.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/pico-demo.md) · [简体中文](../../zh_Hans/getting-started/pico-demo.md) · 日本語

# はじめてのボード：Raspberry Pi Pico でボタン → ランプ

**目標。** 押しボタンでランプを制御し、本物のボードで動かします。小さな製品を開き、ボタンとランプがどのピンにあるかを Studio に伝え、ファームウェアをビルドし、ボードに載せ、ボタンを押します。

**時間。** 15 分。加えて、最初のビルドでボードのライブラリをダウンロードする時間。

**必要なもの。** Raspberry Pi Pico（RP2040 ボード）、データ通信できる USB ケーブル、押しボタン 1 個、ジャンパー線 2 本。LED は不要です。デモは Pico の基板上 LED を駆動します。Studio と、インストールページが求める Rust ツールチェーン（[インストールと起動](install-and-launch.md)）以外にインストールするものはありません。

## 1. ボタンを配線する

ボタンは **GP2** と **GND** の間に付けます。片方の足を GP2 に、もう片方を任意の GND ピンに。それだけです。Pico が自分で線をプルアップするので、ボタンを押している間は線が low として読まれます——デバイスを見ればデモもそう述べています。

| Pico のピン | 配線 |
| -------- | -------------------------------- |
| GP2 | ボタンの片方の足 |
| GND | もう片方の足 |
| GP25 | 何も付けない — 基板上の LED です |

## 2. デモを開く

ようこそページの**デモ**で **Button → Lamp, wired** を選びます。Studio がプロジェクトの置き場所を尋ね（普通のプロジェクトです。`src/main.bdl` を含むフォルダです）、設計ページで開きます。

![The canvas arranged left to right: the Source node pressed with its entry arrow and the word Source at the left, the concept row Pressed and the relationship node lit with the formula pressed in the middle, the concept row Lit and the lamp sink at the right, joined by links; each carries the domain name main.](../../../../docs/user-guide/assets/getting-started/pico-design.png)

_デモの設計：入力元 pressed、それに追従する値 lit、そしてそれが駆動する lamp。_

3 つのオブジェクトです。**pressed** は _入力元_：環境が与える値で、設計はその方法を述べず、キャンバスにピンはありません。**lit** は数式が単に `pressed` である関係です。**lamp** は物理出力で、`lit` に駆動されます。ここにはボードに関するものは何もありません。それは次のページです。

もう一つのデモ **Button → Lamp** はデプロイのない同じ設計です。デバイスの選択を自分で行いたい時に選んでください（手順 3 で説明します）。wired の方は選択済みです。

## 3. デプロイ

**デプロイ**（⌘3）をクリックし、**ターゲット**で **Raspberry Pi Pico (RP2040)** を選びます。

![The Deploy page: the Target pop-up showing Raspberry Pi Pico (RP2040); the green verdict Feasible on Raspberry Pi Pico (RP2040); two device cards — button, a Digital input for pressed — Source with the provider GPIO input, active low and pin GP2, and led, a Digital output for lamp with the realization GPIO, on/off and pin GP25 — each with its judgments checked; the placement table with button and led on GP2 and GP25; and at the bottom the Firmware section with the steps Deployment done, Build current, Flash and Observe to come, a green dot with Ready to build for Raspberry Pi Pico (RP2040), and one primary button, Build for Raspberry Pi Pico (RP2040).](../../../../docs/user-guide/assets/getting-started/pico-firmware-ready.png)

_デプロイページの wired デモ：Pico を選択、両デバイスとも受け入れ可能で配置済み、残る一つのこと — ビルド。_

ページを上から下へ読みます。

- **Raspberry Pi Pico (RP2040) に配置可能です。** — デバイスがボードに収まります。
- **デバイス。** _デバイス_ は設計の一つのものを表すハードウェアです。`button` は `pressed — 入力元` の**デジタル入力**で、**提供方式**は _GPIO 入力、アクティブ Low_（low が真：グランドへのボタン）、ピンは **GP2**。`led` は `lamp` の**デジタル出力**で、**実現方式**は _GPIO、オン/オフ_、ピンは **GP25**。それぞれの下のチェック——入力元では _変換器 · 適合 · 配置済み · 読み取り可_、出力では _エンコーダ · 適合 · 配置済み_——はコンパイラの判断で、すべて成り立っています。
- **配置。** どのピンが何を担うか。
- **ファームウェア。** 動くボードへの道のりで今どこにいるか——_✓ デプロイ · **ビルド** · 書き込み · 観察_——と、次にすべき一つのこと。

ガイド付きデモではページは代わりに**まだビルドできません**と述べ、最初に欠けているものを名指しします——_pressed には Raspberry Pi Pico 上のデバイスがありません。_——そして 2 つのデバイスをあなたが作ります。**デバイスを追加**、名前を付け、種類を選び、3 つ目のポップアップで何のためかを選び（`pressed — 入力元` または `lamp`）、提供方式または実現方式を選び、ピンを入力します。進めるにつれて文が変わり、_ビルドできます_ になります。

## 4. ビルド

**Raspberry Pi Pico (RP2040) 向けにビルド**をクリックします。カードに進行中のことが表示され——_デプロイを確認中_、_クレートを生成中_、_ツールチェーンを準備中_、_コンパイル中_（コンパイル済みの数付き）、_イメージを書き出し中_——最後に **_時刻_ にファームウェアをビルドしました**とイメージのサイズが出ます。最初のビルドはボードのライブラリを取得するので数分かかります。以後は数秒です。**停止**でビルドを途中で終えられます。

**ビルドは完了しませんでした**で終わったら、カードがどの段階でなぜかを述べます——例えば _Raspberry Pi Pico 向けの Rust ターゲットがインストールされていません。_ と実行すべき一つのコマンド——そして**詳細**にコンパイラ自身の言葉があります。[トラブルシューティング：デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md)を参照してください。

## 5. 書き込み

ビルドされたファームウェアの下で、カードは**接続できるボードがありません。**と述べ、接続できるようにする方法を示します。**Pico の BOOTSEL ボタンを押したまま** USB で差し込みます。Pico はコンピューターに `RPI-RP2` という小さなドライブとして現れます。**再検出**をクリックすると、カードがそれを名指しし——_Raspberry Pi Pico in BOOTSEL mode (RPI-RP2)_——**書き込む**ボタンが現れます。

![The Firmware card after a build: a green dot and Firmware built at a time with the image's size in bytes, a Build again link; below, an orange dot with No board is reachable and the line Hold BOOTSEL while plugging the board in over USB; it appears as a drive named RPI-RP2, and a Look again link.](../../../../docs/user-guide/assets/getting-started/pico-firmware-built.png)

_ビルド済み：イメージは最新、ボードはまだ差し込まれておらず、ページが何をすべきかを述べています。_

**書き込む**をクリックします。イメージがドライブにコピーされ、Pico はそれで再起動し、ドライブが消えます。カードは **_時刻_ に Raspberry Pi Pico in BOOTSEL mode (RPI-RP2) へ書き込みました**と述べ、続けて：

> 試してみましょう。pressed を操作すると、lamp が設計どおりに追従するはずです。

Pico が 2 枚差し込まれていれば、カードはどちらかを尋ねます。勝手に選ぶことはありません。

## 6. 試す

ボタンを押します。押している間は基板上の LED が点き、離すと消えます。`lit` は `pressed` で、`lamp` は `lit` です。

Studio が伝えられるのは書き込みが成功したことまでです。ボードが正しく振る舞うかは自分の目で確かめます。LED がボタンに追従しなければ、まず配線（GP2 と GND、実際に閉じるボタン）を確認し、次に[トラブルシューティング：デプロイ](../../../../docs/user-guide/troubleshooting/deployment-errors.md)を見てください。

## 7. 変えて、もう一度

設計を変えます——例えば `lit` を `!pressed` にして、押すまでランプが点いているようにする——あるいはデプロイを変えます——ボタンを GP3 に。ファームウェアのカードはすぐに述べます。**このファームウェアは以前の設計またはデプロイのものです。**そして書き込み済みなら _ボードは以前の設計で動いています。更新するには再ビルドして書き込んでください。_ 提供されるのは**再ビルド**だけで、イメージが最新になれば書き込みが戻ります。Studio は古いイメージを画面上の設計として通すことは決してありません。

## 設計の言葉で言うと、何をしたのか

設計は GP2 も GP25 も知らないままです。**入力元**は環境が与えるもの、**提供方式**——デプロイページで選ぶカタログのプロファイル——はデバイスがボードのためにそれを読む方法、**実現方式**はデバイスが出力の値を外へ運ぶ方法です。ボードを変えても設計はそのまま、提供方式を変えても設計はそのままです。その分離こそが 2 つのページの要点です（[デプロイ](../studio/deploy.md)）。

## Studio を使わずに

スクリプト用に、またアプリなしでビルドを確認するために、ターミナルから同じ道をたどれます（[コマンドライン](../../../../docs/user-guide/reference/cli.md)）。

```bash
bdld init my-lamp --template button-lamp-configured
bdld build my-lamp --target rp2040_pico
bdld flash my-lamp --target rp2040_pico
```

## 次へ

[センサーから出力へ](../../../../docs/user-guide/workflows/sensor-to-output.md)は設計に 2 つ目の入力元を足します。[デプロイ](../studio/deploy.md)にはファームウェアカードのあらゆる状態があります。
