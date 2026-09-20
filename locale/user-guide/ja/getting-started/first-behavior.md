<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/first-behavior.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/first-behavior.md) · [简体中文](../../zh_Hans/getting-started/first-behavior.md) · 日本語
>
> このページはまだ完全には翻訳されていません。未翻訳の箇所は英語で表示されます。

# 最初の振る舞い

**目標。** ヘッドの傾きに明るさが追従するデスクランプ。直立で消灯、水平で全灯。終わる頃には、コンセプト 2 つ、関係 3 つ、タイミングドメイン 1 つ、物理出力 1 つの設計が、検査され、保存され、[シミュレート](first-simulation.md) できる状態になっています。

**所要時間。** 約 20 分。

各概念は設計がそれを必要とするときに登場します。その後 [コンセプト](../../../../docs/user-guide/concepts/concepts.md) のページがそれぞれをきちんと説明します。

## 1. プロジェクトを作る

1. Studio を起動します（[インストールと起動](install-and-launch.md)）。
2. **新規プロジェクト…** をクリックし、場所を選び、フォルダに `lamp` と名前を付けます。

ワークスペースは**設計**ページで開きます。左にサイドバー、中央に空のキャンバス、右にインスペクター、下部にステータス行とページバーです。キャンバスが最初にすることを教えてくれます。サイドバーからコンセプトを追加します。

## 2. 2 つのコンセプトを追加する

**コンセプト**は意味を持つ値です。ランプには 2 つあります。どれだけ傾いているか、そしてどれだけ明るいかです。

1. サイドバーの**プロジェクト**タブで、_コンセプト_ の横の **+** をクリックします。
2. _新規コンセプト_ シートで：名前は `Tilt`。_値_ の下で**量**を選び、単位に**角度**を選びます（記号 `rad` が独自の列に表示されます）。**作成**をクリックします。
3. もう一度 _コンセプト_ の横の **+**：名前 `Brightness`、**量**、単位は**単位なし**。**作成**をクリックします。

各コンセプトはキャンバス上の 1 行で、両端に丸いソケットがあります。丸い形は _量_ を意味し、色はそのコンセプト固有のもので、_その_ コンセプトが使われる場所すべてに現れます。Brightness は単位のない量で、0 から 1 のレベルです。

> シートは入力に合わせて行をプレビューします。塗りつぶしのソケットではなく中空の輪なら、値の形式がまだ選ばれていないことを意味します。それは許されており、ガイドは後でそこに戻ります。

**作ったもの。** 名前の付いた 2 つの意味。数値についてはまだ何もありません。サイドバーの**ライブラリ**タブには既製のコンセプトがあり（_Tilt_ も _Brightness_ もあります）、キャンバスにドラッグするのはシートを埋めるのと同じです。

## 3. 関係を追加する

**関係**は、あるコンセプトが他のコンセプトからどう導かれるかを述べます。

1. サイドバーの _マッピング_ の横の **+** をクリックします（シートのタイトルは _新規マッピング_ です。ガイドでは _関係_ と言いますが、同じものです）。
2. 名前は `dimByTilt`。_読み取り_ の下で **Tilt** をオンにします。_生成_ の下で **Brightness** を選びます。**作成**をクリックします。

入力ソケット 1 つ（左の Tilt）と出力ソケット 1 つ（右の Brightness）を持つノードが現れます。**破線**で描かれ、ヘッダーに _宣言済み_ とあります。関係は存在しシグネチャを持ちますが、数式はまだありません。これはエラーではありません。ここで止めて保存し、明日戻ってきても構いません。

![Two concept rows, Tilt and Brightness, and between them the relationship node dimByTilt drawn with a dashed outline and the word declared in its header; a link runs from Tilt into the node's input socket and from its hollow output socket to Brightness.](../../../../docs/user-guide/assets/getting-started/declared-relationship.png)

_A declared relationship: dashed outline and the word declared in its header. Its output socket is hollow: no value comes out of a rule until a value applies it._

## 4. 数式を書く

1. `dimByTilt` ノードをクリックします。インスペクターに表示されます。
2. **関係**セクションで、エディタは**数式**ビューで開きます。空のスロット `?` と、_Brightness を生成_ という語です。スロットをクリックします。その下の _参照_ に **Tilt** が並びます——クリックします。スロットが `Tilt` になります。
3. `Tilt` をクリックして **÷** を押します。数式は `Tilt ÷ ?` になり、新しいスロットが選択されます。_期待：角度。角度 ÷ 角度 = 無次元量だから。_
4. 数値入力に `90` と入力し、単位ポップアップから **deg** を選び（角度の単位だけが提示されます）、Return を押します。数式は `Tilt ÷ 90 deg` になり、フィールドの下の行に _有効な定義_ と表示されます。

タイプする方がよければ、**テキスト**に切り替えてテキストで書きます：

   ```text
   Tilt / 90 deg
   ```

どちらのビューも同じ数式を編集します。**⌘↩** を押すか、**定義を追加**をクリックします。

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula as components — a Tilt chip, a division sign and a dashed empty slot with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_分母のスロットを選択した数式ビュー：コンパイラはスロットが角度を期待することとその理由を述べ、角度の単位付きの数値、適合する参照、結果が適合する方程式を提示する。_

ノードは本体に数式を示し、実線で描かれます。

**作ったもの。** 規則です。明るさは傾きを 90 度で割ったもの。**テキスト**ビューで試してみてください。`90 deg` を `90 s` に変えると、フィールドの下の行が赤くなります。Brightness が何であり、数式が代わりに何を生成するかを述べます——角度を時間で割ったものは無次元の数ではありません。`deg` に戻します。すべての数式はこのように、入力中も組み立て中も、単位と意味が検査されます——そして _定義を追加_ を押すまで設計には何も保存されません。

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_検査を通らない下書きのある数式フィールド：赤い判定行は Brightness が何であり、数式が代わりに何を生成するかを述べる。_

> テキストビューでは **⌃Space** で補完が開きます。ここで使える名前（_Tilt_）、数値の後の単位、キーワードです。名前に少しホバーすると、それが何かが表示されます。数式ビューは何も求めません。各スロットが適合するものを列挙します。

## 5. 傾きを環境から取り込む

`dimByTilt` は規則であって値ではありません。処理する傾きが必要です。傾きはどこから来るのでしょう？ 環境から——設計の外にあるセンサーからです。BDL では、環境から与えられる値は**入力元**です。**何も読み取らず**、そのコンセプトを生成し、**数式を持たない**関係です。

1. _マッピング_ の横の **+**：名前 `tilt`、何も読み取らず、**Tilt** を生成。作成。（キャンバスを右クリック → **入力元を追加 ▸** → _新しい入力元…_ でも、そこで選んだコンセプト（Tilt）の上に同じものが作れ、新しいコンセプトを一緒に作ることもできます。）

ノードのヘッダーには _入力元_ とあり、左端に縦のバーがあって、入力ソケットはありません。破線ではありません。欠けているものは何もないのです。シミュレータでは値を入力し、デバイスではセンサーがそれを与えます。

## 6. ランプの明るさを計算する

ランプが実際に示す値は、`tilt` に `dimByTilt` を適用したものです。これもまた、何も読み取らず Brightness を生成する関係です——今回は数式付きで。

1. _マッピング_ の横の **+**：名前 `brightness`、何も読み取らず、**Brightness** を生成。作成。
2. 選択して数式 `dimByTilt(tilt)` を入力します。_定義を追加。_

補完は、入力を持つ関係だから `dimByTilt(` を、値だから `tilt` を提示します。数式フィールドは関係を組み合わせる唯一の場所です。キャンバス上のリンクは関係が _どのコンセプト_ を読むかを示し、算術は示しません。

**作ったもの。** 3 つの関係：入力元（`tilt`）、規則（`dimByTilt`）、計算された値（`brightness`）。下部のステータス行がそれらを数えます。

## 7. 値にリズムを与える

外から来る値にはリズムがあります。センサーは一定間隔で報告します。BDL ではそのリズムは名前付きの**タイミングドメイン**であり、自分で更新されるすべての値はいずれかに属します。

1. _タイミングドメイン_ の横の **+**：`interaction` と名付けます。作成。
2. `tilt` を選択します。インスペクターの**タイミング**セクションで、**更新ドメイン**を _interaction_ に設定します。
3. `brightness` を選択して同じことをします。

`dimByTilt` は _任意のタイミングドメイン_ のままです。純粋な規則であり、それを適用するもののリズムに従います。ドメインの名前が 2 つのノードの右端に控えめに表示されます。

**作ったもの。**「これはいつ更新されるか」が明示的な決定である設計。ドメインは名前であってレートではありません——_interaction_ がどのくらいの頻度でティックするかは、ここではなくシミュレートやデプロイのときに選びます。

## 8. ライトを追加する

Brightness は設計の内部の値です。ランプ自体は**物理出力**、つまり値が設計を離れて世界へ出ていく場所です。

1. _出力_ の横の **+**。_新規出力_ シートで：名前 `light`、**受け取る** Brightness、**更新ドメイン** interaction、**必須**をオン。作成。

キャンバスの右端にシンクノードが破線で現れます。ドメインはありますが、まだ何も駆動していません。ステータス行に _出力が未完了_ と表示されます。

1. `brightness` の出力ソケットからシンクのソケットへドラッグします。（または出力を選択し、そのインスペクターの**接続**ポップアップで `brightness` を選びます。ポップアップは _入力を持つ_ 関係に印を付けます。それらは出力を駆動できません。）

シンクが実線になります。出力を駆動できるのは、_何も読み取らず_、同じドメインで受け入れられるコンセプトを生成する関係だけです。`brightness` は条件を満たします。代わりに `dimByTilt` を接続すると、編集としては受け入れられ、その後出力の下に適合しない接続として報告されます。

**作ったもの。** 完全な設計。ステータス行はもう _出力が未完了_ とは言わず、_未定義_ のものもありません。入力元 `tilt` は数式なしのままでよいもので、ステータス行はそれを _入力元 1 件_ と数えます。**⌘S** を押して保存します。

## 手元にあるもの

![The canvas with the concept rows Tilt and Brightness at the top, the Source node tilt (a bar at its left edge, an entry arrow and the word Source in its header, no input socket), the relationship nodes dimByTilt and brightness below them, and the light sink at the right, joined by links; tilt, brightness and the output carry the domain name interaction at their right edge.](../../../../docs/user-guide/assets/getting-started/complete-lamp.png)

_完成したランプ：入力元 tilt、規則 dimByTilt、値 brightness、駆動される light。_

| オブジェクト | 種類 | 数式 | 更新ドメイン |
| -------------- | -------------------------------------- | ----------------- | ------------- |
| **Tilt** | コンセプト、角度 |  |  |
| **Brightness** | コンセプト、無次元の数 |  |  |
| **tilt** | 何も読み取らない関係：入力元 | _なし_ | _interaction_ |
| **dimByTilt** | Tilt を読み取る関係：規則 | `Tilt / 90 deg` | 任意 |
| **brightness** | 何も読み取らない関係：値 | `dimByTilt(tilt)` | _interaction_ |
| **light** | brightness に駆動される物理出力 |  | _interaction_ |

この設計は _実行可能_ です。出力が依存するすべての関係は定義済みか入力元であり、検査を通り、リズムを持ち、出力にはちょうど 1 つの駆動元があります。

## うまくいかないとき

- **数式の行が赤い。** 読んでください。あなたのコンセプトの言葉で問題を述べています（「これは角度を時間で割っている」）。[型・単位・コンセプト](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md) を参照。
- **`brightness` が出力に接続できない。** その _更新ドメイン_ が出力のものと一致し、何も読み取っていないことを確認してください。[接続](../../../../docs/user-guide/troubleshooting/connection-errors.md) を参照。
- **ステータス行に _未追加の定義 N 件_ とある。** 数式は入力されたが追加されていません。ノードを選択して _定義を追加_ か _元に戻す_ を押します。
- **何も検査されず、最下行に _コンパイラ未接続_ とある。** [インストールと起動](install-and-launch.md) を参照。

## 次へ

[最初のシミュレーション](first-simulation.md)——ランプを傾けて明るさを観察します。
