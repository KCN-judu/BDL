<!-- scripts/docs_l10n.py が docs/user-guide/getting-started/first-behavior.md から生成しました。locale/user-guide/ja/user-guide.po を編集してください。このファイルは編集しないでください。 -->

> 言語: [English](../../../../docs/user-guide/getting-started/first-behavior.md) · [简体中文](../../zh_Hans/getting-started/first-behavior.md) · 日本語

# 最初の振る舞い

**ゴール。** ヘッドの傾きに明るさが追従する卓上ランプです。まっすぐ立てると消え、水平に倒すと最も明るくなります。終わるころには、2 つのコンセプト、それらから作った 2 つのブロック、1 つのルール、1 つのタイミングドメイン、1 つの物理出力からなる設計ができています——チェック済み、保存済みで、[シミュレーション](first-simulation.md)できる状態です。

**所要時間。** 約 20 分。

各概念は設計がそれを必要とするときに登場します。その後 [コンセプト](../../../../docs/user-guide/concepts/concepts.md) のページがそれぞれをきちんと説明します。

## 1. プロジェクトを作る

1. Studio を起動します（[インストールと起動](install-and-launch.md)）。
2. **新規プロジェクト…** をクリックし、場所を選び、フォルダに `lamp` と名前を付けます。

ワークスペースは**デザイン**ページで開きます。左にサイドバー、中央に空のキャンバス、右にインスペクタ、下部にステータス行とページバーがあります。

## 2. 2 つのコンセプトを追加する

**コンセプト**は意味を持つ値です——そしてテンプレートでもあります。キャンバス上のブロックはそこから作られます。このランプには 2 つあります。どれだけ傾いているか、そしてどれだけ明るいかです。

1. サイドバーの**プロジェクト**タブで、_コンセプト_ の横の **+** をクリックします。
2. _新規コンセプト_ シートで：名前は `Tilt`。_値_ の下で**量**を選び、単位に**角度**を選びます（記号 `rad` が独自の列に表示されます）。**作成**をクリックします。
3. もう一度 _コンセプト_ の横の **+**：名前 `Brightness`、**量**、単位は**単位なし**。**作成**をクリックします。

2 つのコンセプトはキャンバスではなくサイドバーに現れます。コンセプトはテンプレートで、その**ブロック**が値を持つものです。シートのプレビューにある丸い形は _物理量_ を意味します。色はそのコンセプト固有のもので、_その_ コンセプトを運ぶすべてのソケットに付きます。Brightness は単位のない物理量、0 から 1 のレベルです。

> ソケットが塗りつぶしではなく中空の輪なら、値の形がまだ選ばれていないことを意味します——それは許されており、ガイドでは後で戻ってきます。

**作ったもの。** 名前の付いた 2 つの意味です。数値についてはまだ何もありません。サイドバーの**ライブラリ**タブには既製のコンセプトがあります（_Tilt_ も _Brightness_ もそこにあります）。

## 3. それぞれのブロックをキャンバスに置く

**ブロック**はコンセプトの 1 つの値で、ティックごとに一度更新されます。このランプには傾きが 1 つ、明るさが 1 つ必要です。

1. 空のキャンバスを右クリック → **ブロックを追加 ▸** → **Tilt のブロック**。`tilt` という名前のブロックがクリックした場所に置かれます。
2. もう一度右クリック → **ブロックを追加 ▸** → **Brightness のブロック**：ブロック `brightness` ができます。

各ブロックの右側には、そのコンセプトの色の出力ソケットが 1 つあり、ヘッダーに _入力元_ という語、左端に縦のバーがあります。式のないブロックは**外部から提供**されます——環境やセンサーから——どう計算するかを指定するまでそのままです。これはエラーではありません。ここで止めて保存し、明日戻ってきても構いません。

![Two blocks one above the other, each with a bar at its left edge, an entry arrow and the word Source in its header and one output socket on the right: tilt with a socket labelled Tilt, and brightness with a socket labelled Brightness; no link joins them yet.](../../../../docs/user-guide/assets/getting-started/declared-relationship.png)

_式を書く前の 2 つのブロック：tilt と、まだ外部から提供されている brightness。_

（キャンバスを右クリックして**ブロックを追加 ▸ 新しいコンセプト ▸** を選ぶと、クリックした場所にコンセプト _と_ そのブロックが一度に作られます。ライブラリの行をキャンバスへドラッグしても同じです。）

## 4. ルールを書く

明るさは傾きからどう決まるのか。それが**ルール**です。あるコンセプトを読み取り、別のコンセプトを生成する関係です。ルールもテンプレートです——ブロックの式の中で適用され、それ自体はキャンバス上のブロックではありません。

1. サイドバーの _マッピング_ の横の **+** をクリックします（シートのタイトルは _新規マッピング_ です。ガイドでは _関係_ と言いますが、同じものです）。
2. 名前は `dimByTilt`。_読み取り_ の下で **Tilt** をオンにします。_生成_ の下で **Brightness** を選びます。**作成**をクリックします。

`dimByTilt` がサイドバーの _マッピング_ リストに選択された状態で現れ、インスペクタに表示されます。_ルール_、Tilt を読み取り、Brightness を生成、式はまだありません。

1. **関係**セクションで、エディタは**数式**ビューで開きます。空のスロット `?` と、_Brightness を生成_ という語です。スロットをクリックします。その下の _参照_ に **Tilt** が並びます——クリックします。スロットが `Tilt` になります。
2. `Tilt` をクリックして **÷** を押します。数式は `Tilt ÷ ?` になり、新しいスロットが選択されます。_期待：角度。角度 ÷ 角度 = 無次元量だから。_
3. 数値入力に `90` と入力し、単位ポップアップから **deg** を選び（角度の単位だけが提示されます）、Return を押します。数式は `Tilt ÷ 90 deg` になり、フィールドの下の行に _有効な定義_ と表示されます。

タイプする方がよければ、**テキスト**に切り替えてテキストで書きます：

   ```text
   Tilt / 90 deg
   ```

どちらのビューも同じ数式を編集します。**⌘↩** を押すか、**定義を追加**をクリックします。

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../../../../docs/user-guide/assets/studio/formula-composer.png)

_分母のスロットを選択した式ビュー：商は分数として描かれ、コンパイラはそのスロットが角度を期待する理由を示し、角度の単位付きの数値、適合する参照、結果が適合する方程式を提示します。_

**作ったもの。** ルールです。明るさは傾きを 90 度で割ったものです。**テキスト**ビューで試してみましょう。`90 deg` を `90 s` に変えると、フィールドの下の行が赤くなります。Brightness が何であり、式が代わりに何を生成しているかを示します——角度を時間で割ったものは普通の数ではありません。`deg` に戻します。すべての式は、入力中も組み立て中も、このように単位と意味がチェックされます——そして _定義を追加_ を押すまで、設計には何も保存されません。

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../../../../docs/user-guide/assets/studio/formula-verdict.png)

_検査を通らない下書きのある数式フィールド：赤い判定行は Brightness が何であり、数式が代わりに何を生成するかを述べる。_

> テキストビューでは **⌃Space** で補完が開きます。ここで使える名前（_Tilt_）、数値の後の単位、キーワードです。名前に少しホバーすると、それが何かが表示されます。数式ビューは何も求めません。各スロットが適合するものを列挙します。

## 5. 傾きは環境から入ってくる

傾きはどこから来るのでしょうか。環境から——設計の外にあるセンサーからです。BDL では、環境が提供する値は**入力元**、つまり**式のない**ブロックです。`tilt` はすでにそうなっています。ヘッダーに _入力元_ とあり、左端にバーがあり、入力ソケットはありません。何も欠けていません。シミュレータでは値を入力し、デバイスではセンサーが提供します。

## 6. ランプの明るさを計算する

ランプが実際に示す値は、傾きにルールを適用したものです。それが `brightness` の式です。

1. `brightness` ブロックをクリックします。インスペクタに表示されます。
2. 式 `dimByTilt(tilt)` を入力します——式ビューではスロットをクリックし、_参照_ の下で **dimByTilt** を選び（`dimByTilt(?)` になります）、新しいスロットをクリックして **tilt** を選びます。テキストビューで入力しても構いません。_定義を追加。_

補完は `dimByTilt(` を提示します。規則だからです。また `tilt` も提示します。ブロックだからです。キャンバスでは `brightness` の隣に**マッピングブロック**——式をノードとして描いたもの——が現れます。ヘッダーには適用する規則（`dimByTilt`）の名が書かれ、左には `tilt` と書かれた入力ソケット——式が読むブロックごとに一つ——があり、`tilt` の出力ソケットからそこへリンクが走ります。短いリンクが、それが定義するブロック `brightness` へつなぎます。`brightness` のヘッダーにはもう _Source_ とは書かれません。この値は計算されるからです。規則の名は式の中、それが適用される場所にあり、規則そのものはサイドバーに残ります。

> ドラッグで配線することもできます。`tilt` の出力ソケットを、まだ式のないブロックの上か、空き位置のあるマッピングブロックの中空の `?` ソケットの上に落とすと、式がその名を得ます。キャンバスが描くのは式が言っていることそのもの——式こそがブロックを組み合わせる唯一の場所です。

**作ったもの。** 2 つのブロック——入力元（`tilt`）と計算された値（`brightness`）——と、その値が適用するルール（`dimByTilt`）です。下部のステータス行は 3 つの関係と 1 つの入力元を数えます。

## 7. 値にリズムを与える

外から来る値にはリズムがあります。センサーは一定間隔で報告します。BDL ではそのリズムは名前付きの**タイミングドメイン**であり、自分で更新されるすべての値はいずれかに属します。

1. _タイミングドメイン_ の横の **+**：`interaction` と名付けます。作成。
2. `tilt` を選択します。インスペクターの**タイミング**セクションで、**更新ドメイン**を _interaction_ に設定します。
3. `brightness` を選択して同じことをします。

`dimByTilt` は _任意のタイミングドメイン_ のままです。純粋なルールなので、それを適用するもののリズムに従います。ドメイン名は 2 つのブロックの右端に控えめに表示されます。

**作ったもの。**「これはいつ更新されるか」が明示的な決定である設計。ドメインは名前であってレートではありません——_interaction_ がどのくらいの頻度でティックするかは、ここではなくシミュレートやデプロイのときに選びます。

## 8. ライトを追加する

Brightness は設計の内部の値です。ランプ自体は**物理出力**、つまり値が設計を離れて世界へ出ていく場所です。

1. _出力_ の横の **+**。_新規出力_ シートで：名前 `light`、**受け取る** Brightness、**更新ドメイン** interaction、**必須**をオン。作成。

キャンバスの右端にシンクノードが破線で現れます。ドメインはありますが、まだ何も駆動していません。ステータス行に _出力が未完了_ と表示されます。

1. `brightness` の出力ソケットからシンクのソケットへドラッグします。（または出力を選択し、そのインスペクタの**接続**ポップアップで `brightness` を選びます。ポップアップはルール——_入力を持つ_ 関係——に印を付けます。ルールは出力を駆動できません。）

シンクは実線になります。同じドメインで受け入れるコンセプトを生成するブロックだけが出力を駆動できます。`brightness` は該当し、`tilt` は該当せず（コンセプトが違い、ソケットが拒否します）、ルールには与える値がありません。

**作ったもの。** 完全な設計。ステータス行はもう _出力が未完了_ とは言わず、_未定義_ のものもありません。入力元 `tilt` は数式なしのままでよいもので、ステータス行はそれを _入力元 1 件_ と数えます。**⌘S** を押して保存します。

## 手元にあるもの

![The canvas left to right: the Source block tilt (a bar at its left edge, an entry arrow and the word Source in its header, one output socket labelled Tilt), a link from it into the mapping block dimByTilt, whose left socket is labelled tilt and whose formula line reads dimByTilt(tilt), a short link from its output socket into the block brightness, whose socket is labelled Brightness, and a link from brightness to the light sink at the right; tilt, brightness and the output carry the domain name interaction.](../../../../docs/user-guide/assets/getting-started/complete-lamp.png)

_完成したランプ：入力元 tilt、規則 dimByTilt を適用するマッピングブロック、それが定義するブロック brightness、そして駆動される light。_

| オブジェクト | 種類 | 数式 | 更新ドメイン |
| -------------- | ----------------------------------------------- | ----------------- | ------------- |
| **Tilt** | コンセプト（テンプレート）、角度 |  |  |
| **Brightness** | コンセプト（テンプレート）、単なる数 |  |  |
| **tilt** | 式のない Tilt のブロック：入力元 | _なし_ | _interaction_ |
| **dimByTilt** | Tilt を読み取り Brightness を生成するルール | `Tilt / 90 deg` | 任意 |
| **brightness** | Brightness のブロック、計算値：ルールを適用 | `dimByTilt(tilt)` | _interaction_ |
| **light** | brightness に駆動される物理出力 |  | _interaction_ |

この設計は _実行可能_ です。出力が依存するすべてのブロックは計算値か入力元であり、チェックを通り、リズムを持ち、出力にはちょうど 1 つのドライバがあります。

## うまくいかないとき

- **数式の行が赤い。** 読んでください。あなたのコンセプトの言葉で問題を述べています（「これは角度を時間で割っている」）。[型・単位・コンセプト](../../../../docs/user-guide/troubleshooting/type-and-concept-errors.md) を参照。
- **`brightness` が出力に接続できない。** その _更新ドメイン_ が出力のものと一致し、何も読み取っていないことを確認してください。[接続](../../../../docs/user-guide/troubleshooting/connection-errors.md) を参照。
- **ステータス行に _N 個の定義が未追加_ とある。** 式は入力されたが追加されていません。ブロックを選択して _定義を追加_ か _元に戻す_ を押します。
- **何も検査されず、最下行に _コンパイラ未接続_ とある。** [インストールと起動](install-and-launch.md) を参照。

## 次へ

[最初のシミュレーション](first-simulation.md)——ランプを傾けて明るさを観察します。
