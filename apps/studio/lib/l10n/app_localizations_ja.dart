// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get appTitle => 'BDL Studio';

  @override
  String get preferencesTitle => '環境設定';

  @override
  String get preferencesMenu => '環境設定…';

  @override
  String get done => '完了';

  @override
  String get languageLabel => '言語';

  @override
  String get languageSystemDefault => 'システムのデフォルト';

  @override
  String get languageHelp => 'Studio の表示言語だけを変更します。BDL のソース、プロジェクトファイル、コンパイラには影響しません。';

  @override
  String get monitor => 'モニター';

  @override
  String get liveValuesOnTheCanvasArriveWith => 'キャンバス上のライブ値はテレメトリ機能とともに提供されます（ロードマップ ステップ S）。';

  @override
  String get bdlStudio => 'BDL Studio';

  @override
  String get edited => '編集済み';

  @override
  String get save => '保存';

  @override
  String get close => '閉じる';

  @override
  String get reloadFromDisk => 'ディスクから再読み込み';

  @override
  String get overwrite => '上書き';

  @override
  String get dismiss => '閉じる';

  @override
  String get noProject => 'プロジェクトなし';

  @override
  String get saved => '保存済み';

  @override
  String get notCausal => '瞬時サイクルあり';

  @override
  String get readsAcrossDomains => 'ドメインをまたぐ読み取り';

  @override
  String get outputsIncomplete => '出力が未完了';

  @override
  String get compilerNotConnected => 'コンパイラ未接続';

  @override
  String get connectingToTheCompiler => 'コンパイラに接続中';

  @override
  String get design => '設計';

  @override
  String get simulate => 'シミュレート';

  @override
  String get deploy => 'デプロイ';

  @override
  String get projectManagerClosesThisProject => 'プロジェクトマネージャー（このプロジェクトを閉じます）';

  @override
  String get start => '開始';

  @override
  String get newProject => '新規プロジェクト…';

  @override
  String get openProject => 'プロジェクトを開く…';

  @override
  String get openByPath => 'パスを指定して開く…';

  @override
  String get openProjectByPath => 'パスを指定してプロジェクトを開く';

  @override
  String get newAtPath => 'パスを指定して新規作成…';

  @override
  String get createProjectAtPath => 'パスを指定してプロジェクトを作成';

  @override
  String get waitingForTheCompilerTheStatusLine => 'コンパイラを待っています。接続状態はステータス行に表示されます。';

  @override
  String get recent => '最近';

  @override
  String get projectsYouOpenWillAppearHere => '開いたプロジェクトがここに表示されます。';

  @override
  String get notFound => '見つかりません';

  @override
  String get removeFromRecent => '最近の一覧から削除';

  @override
  String get justNow => 'たった今';

  @override
  String get thisProject => 'このプロジェクト';

  @override
  String get openAnotherProject => '別のプロジェクトを開く';

  @override
  String get createAnotherProject => '別のプロジェクトを作成する';

  @override
  String get closeIt => '閉じる';

  @override
  String get donTSave => '保存しない';

  @override
  String get cancel => 'キャンセル';

  @override
  String get folder => 'フォルダ';

  @override
  String get choose => '選択';

  @override
  String get name => '名前';

  @override
  String get create => '作成';

  @override
  String get newConcept => '新規コンセプト';

  @override
  String get somethingTheProductSensesDecidesOrShows => '製品が感知・判断・表示するもの。';

  @override
  String get value => '値';

  @override
  String get quantity => '量';

  @override
  String get onOff => 'オン／オフ';

  @override
  String get count => '個数';

  @override
  String get decideLater => '後で決める';

  @override
  String get unit => '単位';

  @override
  String get meaning => '意味';

  @override
  String get hollowRingTheValueCanBeDecided => '中空の輪：値の形式は後で決められます。関係は今から使えます。';

  @override
  String get roundSocketAMeasuredQuantityItsUnit => '丸いソケット：計測される量。単位はすべての数式で検査されます。';

  @override
  String get diamondSocketOnOrOffActivatesContexts => 'ひし形のソケット：オンかオフ — コンテキストを起動し、振る舞いを切り替えます。';

  @override
  String get squareSocketAWholeNumberOccurrencesSteps => '四角いソケット：整数 — 回数、ステップ数、個数。';

  @override
  String get newMapping => '新規マッピング';

  @override
  String get aRelationshipBetweenConceptsItCanExist => 'コンセプト間の関係。定義される前から存在できます。';

  @override
  String get reads => '読み取り';

  @override
  String get produces => '生成';

  @override
  String get chooseWhatItProducesTheOutputSocket => '何を生成するかを選びます。出力ソケットはそのコンセプトの色と形になります。';

  @override
  String get dashedDeclaredNotYetDefinedAttachA => '破線：宣言済み、未定義。準備ができたらインスペクターから数式を付けてください。';

  @override
  String get newTimingDomain => '新規タイミングドメイン';

  @override
  String get whenAGroupOfRelationshipsAndOutputs =>
      '一群の関係と出力が一緒に更新されるタイミング。名前であってレートではありません。起動の頻度は設計の実行時に決まります。';

  @override
  String get interactionAmbient => 'interaction, ambient, …';

  @override
  String get newOutput => '新規出力';

  @override
  String get whereAValueLeavesTheDesignFor => '値が設計から外の世界へ出ていく場所：ライト、モーター、ディスプレイ。';

  @override
  String get accepts => '受け取る';

  @override
  String get theConceptThisOutputTakes => 'この出力が受け取るコンセプト';

  @override
  String get updatesIn => '更新ドメイン';

  @override
  String get required => '必須';

  @override
  String get theDesignIsIncompleteUntilSomethingDrives => '何かが駆動するまで設計は未完了です';

  @override
  String get showingTheLastVersionThatBuiltThe => '最後にビルドできたバージョンを表示しています。テキストにはまだビルドできない変更があります。';

  @override
  String get noProjectOpen => 'プロジェクトが開かれていません。';

  @override
  String get system => 'システム';

  @override
  String get notPlacedYet => '未配置';

  @override
  String get itsPromiseIsWhatInstancesSeeEdits => '— インスタンスに見えるのはその約束です。ここでの編集はすべてのインスタンスに及びます。';

  @override
  String get code => 'コード';

  @override
  String get split => '分割';

  @override
  String get project => 'プロジェクト';

  @override
  String get library => 'ライブラリ';

  @override
  String get concepts => 'コンセプト';

  @override
  String get mappings => 'マッピング';

  @override
  String get timingDomains => 'タイミングドメイン';

  @override
  String get outputs => '出力';

  @override
  String get contexts => 'コンテキスト';

  @override
  String get components => 'コンポーネント';

  @override
  String get newComponent => '新規コンポーネント';

  @override
  String get aReusableBehaviourWithAPromiseIts => '約束（ポート）と独自のソースを持つ、再利用可能な振る舞い。';

  @override
  String get adaptivelamp => 'AdaptiveLamp';

  @override
  String get instances => 'インスタンス';

  @override
  String get behaviors => '振る舞い';

  @override
  String get add => '追加';

  @override
  String get stillInUse => 'まだ使用中';

  @override
  String get environment => '環境';

  @override
  String get humanInteraction => '人の操作';

  @override
  String get geometryMotion => '形状と動き';

  @override
  String get mechanical => '機械';

  @override
  String get electricalSystem => '電気とシステム';

  @override
  String get visualDisplay => '表示';

  @override
  String get actuation => '駆動機構';

  @override
  String get audio => '音声';

  @override
  String get noUnit => '単位なし';

  @override
  String get groupedValue => '組の値';

  @override
  String get optionalValue => '省略可能な値';

  @override
  String get searchConcepts => 'コンセプトを検索';

  @override
  String get loadingTheConceptLibrary => 'コンセプトライブラリを読み込み中…';

  @override
  String get theConceptLibraryArrivesWithTheCompiler => 'コンセプトライブラリはコンパイラサービスとともに提供されます。';

  @override
  String get openAProjectToAddConceptsFrom => 'プロジェクトを開くと、ここからコンセプトを追加できます。';

  @override
  String get selectAConceptARelationshipAnOutput => 'コンセプト、関係、出力、インスタンス、グループのいずれかを選択してください。';

  @override
  String get selectAConceptAMappingOrAn => 'コンセプト、マッピング、出力のいずれかを選択してください。';

  @override
  String get inspector => 'インスペクター';

  @override
  String get nothingElseNeedsRechecking => '他に再検査が必要なものはありません。';

  @override
  String get thisWillBeCheckedAgain => 'これは再検査されます。';

  @override
  String get thisChangeAffects => 'この変更が影響するもの';

  @override
  String get itWillBeCheckedAgain => '— 再検査されます。';

  @override
  String get theyWillBeCheckedAgain => '— それらは再検査されます。';

  @override
  String get lastChange => '最後の変更';

  @override
  String get interface => 'インターフェース';

  @override
  String get realization => '実現';

  @override
  String get semantic => '意味';

  @override
  String get reactive => 'リアクティブ';

  @override
  String get clock => 'クロック';

  @override
  String get output => '出力';

  @override
  String get deployment => 'デプロイ';

  @override
  String get unspecified => '未指定';

  @override
  String get order => '順序';

  @override
  String get valuesAreMagnitudesSmallestLargestClampIn => '値は大小のある量：<、最小、最大、クランプ、範囲内';

  @override
  String get valuesAreComparedForEqualityOnly => '値は等しいかどうかだけを比較します';

  @override
  String get relationships => '関係';

  @override
  String get producedBy => '生成元';

  @override
  String get nothingYet => 'まだありません';

  @override
  String get usedBy => '使用元';

  @override
  String get explain => '説明';

  @override
  String get collectionOf => 'コレクション…';

  @override
  String get optional => '省略可能…';

  @override
  String get each => '各要素';

  @override
  String get whenPresent => '存在するとき';

  @override
  String get first => '第 1 要素';

  @override
  String get second => '第 2 要素';

  @override
  String get place => '配置';

  @override
  String get showGroup => 'グループを表示';

  @override
  String get relationship => '関係';

  @override
  String get showBinding => 'バインディングを表示';

  @override
  String get disconnect => '切断';

  @override
  String get disconnectItToDefineTheRelationshipYourself => '切断すると、この関係を自分で定義できます。';

  @override
  String get timing => 'タイミング';

  @override
  String get anyDomain => '任意のドメイン';

  @override
  String get aRelationshipInNoDomainIsPure => 'ドメインに属さない関係は純粋です。読み取られる場所で評価されます。';

  @override
  String get drives => '駆動先';

  @override
  String get thisValueCanCommitToAPhysical => 'この値は物理出力にコミットできます。';

  @override
  String get onlyARelationshipWithoutInputsCanDrive =>
      '出力を駆動できるのは入力を持たない関係だけです。各入力元をまとめる関係を接続してください。';

  @override
  String get noExplicitInputsTheCanonicalDomainIs =>
      '明示的な入力なし：正規のドメインは空積 () で、カーネルは () -> B を B として符号化します';

  @override
  String get pendingAnalysis => '解析待ち';

  @override
  String get noTimingDomainYetNotPartOf => 'タイミングドメインが未設定：選ばれるまで設計の約束には含まれません。';

  @override
  String get undrivenTheDesignIsIncompleteWithoutA => '未駆動 — 駆動元がないと設計は未完了です。';

  @override
  String get undriven => '未駆動。';

  @override
  String get theConnectionDoesNotFitSeeThe => '接続が適合しません。下の駆動元の検出項目を参照してください。';

  @override
  String get checking => '検査中…';

  @override
  String get noDomainYet => 'ドメイン未設定';

  @override
  String get theDesignIsIncompleteUntilThisIs => 'これが駆動されるまで設計は未完了です';

  @override
  String get driver => '駆動元';

  @override
  String get connect => '接続';

  @override
  String get aRelationship => '関係を選択…';

  @override
  String get hasInputs => '入力あり';

  @override
  String get anyTimingDomain => '任意のタイミングドメイン';

  @override
  String get systemInput => 'システム入力';

  @override
  String get givenAValue => '値が与えられています';

  @override
  String get editSource => 'ソースを編集';

  @override
  String get replaceWith => '置き換え';

  @override
  String get anotherComponent => '別のコンポーネント…';

  @override
  String get notAssigned => '未割り当て';

  @override
  String get ports => 'ポート';

  @override
  String get noPortsYet => 'ポートはまだありません。';

  @override
  String get findings => '検出項目';

  @override
  String get remove => '削除';

  @override
  String get disconnectItsPortsFirstTheComponentStays => '先にポートを切断してください。コンポーネントは残ります。';

  @override
  String get port => 'ポート';

  @override
  String get promise => '約束';

  @override
  String get implementedBy => '実装';

  @override
  String get nothingThePromiseHasNoBacking => 'なし（約束を裏付ける実装がありません）';

  @override
  String get goToSource => 'ソースへ移動';

  @override
  String get aConstantEG05 => '定数（例：0.5）';

  @override
  String get aClosedConstantInThePortS => 'ポートの単位で表した閉じた定数。または代わりにポートをバインドしてください。';

  @override
  String get connection => '接続';

  @override
  String get openNothingSuppliesItYetDrawA =>
      'オープン：まだ何も供給していません。提供ポートか、同じコンセプトのトップレベルの関係からリンクを引いてください。オープンなポートがあっても設計はシミュレートできます（ポートは入力になります）。';

  @override
  String get binding => 'バインディング';

  @override
  String get from => '元';

  @override
  String get to => '先';

  @override
  String get directTheDestinationReadsTheValueAs => '直接：宛先は値をそのまま読み取ります。';

  @override
  String get aBindingConvertsNothingBothEndsCarry => 'バインディングは何も変換しません。両端は同一のコンセプトを持ちます。';

  @override
  String get keepsItsPromise => '約束を守っています';

  @override
  String get promiseBroken => '約束が破られています';

  @override
  String get backToSystem => '‹ システム';

  @override
  String get duplicateAsVersion => 'バージョンとして複製';

  @override
  String get placeInstance => 'インスタンスを配置';

  @override
  String get noPortsYetSelectARelationshipOf => 'ポートはまだありません。ソースの関係を選び、ポートとして宣言してください。';

  @override
  String get timingParameters => 'タイミングパラメータ';

  @override
  String get aParameterTheSystemAssignsIt => 'パラメータ（システムが割り当てます）';

  @override
  String get privateToEachInstance => '各インスタンスに固有';

  @override
  String get sharedConcepts => '共有コンセプト';

  @override
  String get privateFreshPerInstance => '固有（インスタンスごとに新規）';

  @override
  String get aSharedConceptIsTheSameIdentity =>
      '共有コンセプトはどこでも同一の識別子です。固有コンセプトはこのコンポーネント自身のもので、インスタンスごとに新しく作られます。';

  @override
  String get physicalOutputs => '物理出力';

  @override
  String get privateOnePerInstance => '固有（インスタンスごとに 1 つ）';

  @override
  String get declareAPort => 'ポートを宣言';

  @override
  String get exposeAs => '公開…';

  @override
  String get thePromiseIsTakenFromTheRelationship => '約束は現在の関係から取られ、ここで変更するまで保持されます。';

  @override
  String get deleteItsInstancesFirst => '先にインスタンスを削除してください。';

  @override
  String get aRelationshipOfTheSource => 'ソースの関係';

  @override
  String get retirePort => 'ポートを廃止';

  @override
  String get group => 'グループ';

  @override
  String get aBehaviorIsAWayOfSeeing => '振る舞いは設計の見方の一つです。グループ化、移動、分割、解除をしても、設計の意味は変わりません。';

  @override
  String get addRelationship => '+ 関係を追加';

  @override
  String get emptyDragRelationshipsInOrAddOne => '空です。関係をドラッグして入れるか、追加してください。';

  @override
  String removeFromGroup(Object group) {
    return '$group から外す';
  }

  @override
  String get boundary => '境界';

  @override
  String get computedOnceTheAnalysisArrives => '解析結果が届いたら計算されます。';

  @override
  String get inputs => '入力';

  @override
  String get open => 'オープン';

  @override
  String get internal => '内部';

  @override
  String get readOffTheDependencyGraphWhatThe =>
      '依存グラフから読み取ったもの：メンバーが外部から読むもの、外部がメンバーから読むもの。切り口の図であって、接続そのものではありません。';

  @override
  String get package => 'パッケージ化';

  @override
  String get packageAsReusableComponent => '再利用可能なコンポーネントとしてパッケージ化…';

  @override
  String get turnsTheBehaviorIntoAComponentAnd =>
      '振る舞いをコンポーネントに変え、その場所に 1 つのインスタンスを置きます。設計が計算する値は変わりません。';

  @override
  String get aBehaviorInsideAComponentStaysA =>
      'コンポーネント内の振る舞いは、そのソースの見方の一つのままです。単独のコンポーネントとしてパッケージ化するには入れ子のコンポーネントが必要です。それまでは自由に編集・移動できます。';

  @override
  String get canvas => 'キャンバス';

  @override
  String get expand => '展開';

  @override
  String get collapse => '折りたたむ';

  @override
  String get ungroup => 'グループ解除';

  @override
  String get deleteGroupAndRelationships => 'グループと関係を削除';

  @override
  String get ungroupKeepsEveryRelationshipWhereItIs => 'グループ解除しても各関係はその場に残ります。';

  @override
  String get behavior => '振る舞い';

  @override
  String get theseRelationshipsReadEachOther => 'これらの関係は互いを読み取っています。';

  @override
  String get theseRelationshipsUpdateInOneTimingDomain => 'これらの関係は同じタイミングドメインで更新されます。';

  @override
  String get groupAsBehavior => '振る舞いとしてグループ化';

  @override
  String get replaceTheConnection => '接続を置き換えますか？';

  @override
  String get carryAcrossTimingDomains => 'タイミングドメインをまたいで運ぶ';

  @override
  String get startsAt => '開始値';

  @override
  String get aConstantInTheDestinationSUnits => '宛先の単位で表した定数（例：0）';

  @override
  String get disconnectAndConnect => '切断して接続';

  @override
  String get workingOutTheBoundary => '境界を計算中…';

  @override
  String get requires => '要求';

  @override
  String get whatTheMembersReadFromOutside => 'メンバーが外部から読み取るもの';

  @override
  String get nothingTheComponentIsSelfContained => 'なし — コンポーネントは自己完結しています';

  @override
  String get provides => '提供';

  @override
  String get whatOutsideReadsFromTheMembers => '外部がメンバーから読み取るもの';

  @override
  String get nothingYetNothingOutsideReadsTheGroup => 'まだなし — 外部はこのグループを読み取っていません';

  @override
  String get openRelationships => 'オープンな関係';

  @override
  String get declaredInsideNotYetDefined => '内部で宣言済み、未定義';

  @override
  String get treatAsInput => '入力として扱う';

  @override
  String get keepInternal => '内部に保つ';

  @override
  String get drivenByMembersTheDriveStaysWith => 'メンバーが駆動。駆動はメンバーに残ります';

  @override
  String get staysTheSystemS => 'システムに残す';

  @override
  String get movesInside => '内部へ移す';

  @override
  String get everyDomainTheMembersUse => 'メンバーが使うすべてのドメイン';

  @override
  String get nonePure => 'なし — 純粋';

  @override
  String get readByMembersOnly => 'メンバーだけが読み取る';

  @override
  String get theDesignComputesTheSameValuesAfterwards =>
      'その後も設計は同じ値を計算します。グループはその場所で 1 つのインスタンスになり、コンポーネントは再び配置できます。';

  @override
  String get expressionWithNoInputs => '入力のない式';

  @override
  String get noDefinitionYetALegalStateOther => '定義はまだありません。正当な状態です。他の関係がすでにこのシグネチャに依存しているかもしれません。';

  @override
  String get nothingToAddYet => '追加するものはまだありません。';

  @override
  String get emptyDetachToRemoveTheDefinition => '空です。定義を外すには「定義を外す」を選んでください。';

  @override
  String get saving => '保存中…';

  @override
  String get addDefinition => '定義を追加';

  @override
  String get saveDefinition => '定義を保存';

  @override
  String get invalidDefinition => '定義が無効です。';

  @override
  String get cannotBeRead => '読み取れません。';

  @override
  String get openSomethingItNeedsIsNotDecided => 'オープン：必要なものがまだ決まっていません。';

  @override
  String get validDefinition => '有効な定義';

  @override
  String get noDefinition => '定義がありません。';

  @override
  String get formula => '数式';

  @override
  String get revert => '元に戻す';

  @override
  String get addingADefinitionIsARefinementNothing => '定義の追加は精緻化です。他で確立されたものは再検査されません。';

  @override
  String get detachDefinition => '定義を外す';

  @override
  String get replacingOrDetachingIsAnEditThis => '置き換えや取り外しは編集です。この定義とシミュレーションが再検査されます。';

  @override
  String get thisRelationshipSDefinitionChangedWhileYou => '編集中にこの関係の定義が変更されました。';

  @override
  String get reload => '再読み込み';

  @override
  String get showTheDefinitionCommittedMeanwhileDropWhat => 'その間にコミットされた定義を表示し、入力した内容を破棄します';

  @override
  String get keepMine => '自分の内容を保持';

  @override
  String get keepWhatYouTypedSaveWillReplace => '入力した内容を保持します。保存するとコミット済みの定義が置き換わります';

  @override
  String get looking => '検索中…';

  @override
  String get waitingForTheCompilerToReadThe => 'コンパイラが数式を読み取るのを待っています…';

  @override
  String get theTextCannotBeReadAsA => 'テキストを数式として読み取れません。';

  @override
  String get editAsText => 'テキストとして編集';

  @override
  String get askingWhatFitsHere => 'ここに何が入るかを問い合わせ中…';

  @override
  String get hideDetail => '詳細を隠す';

  @override
  String get insert => '挿入';

  @override
  String get noUnitIsSuggestedFillTheOther => '単位の候補がありません。先に反対側を埋めてください。';

  @override
  String get references => '参照';

  @override
  String get compare => '比較';

  @override
  String get function => '関数';

  @override
  String get eachElement => '各要素';

  @override
  String get allSatisfy => 'all … satisfy';

  @override
  String get anySatisfies => 'any … satisfies';

  @override
  String get mapEach => 'map each …';

  @override
  String get filter => 'filter …';

  @override
  String get range => '範囲';

  @override
  String get betweenTwoEndsIn => '両端の間：in … .. …';

  @override
  String get thisPartIsEditedAsText => 'この部分はテキストとして編集されています。';

  @override
  String get composeChoose => '選択';

  @override
  String get aChoiceIfThenElse => '選択：if … then … else …';

  @override
  String get negateThis => '否定：not …';

  @override
  String get readingTheSources => 'ソースを読み込み中…';

  @override
  String get show => '表示';

  @override
  String get wholeNumber => '整数';

  @override
  String get noDomainsEveryRelationshipIsEvaluatedAt => 'ドメインなし：すべての関係が毎ティック評価されます。';

  @override
  String get activationPeriodInTicksChangingOneStarts => '起動周期（ティック単位）。変更すると実行が最初からやり直されます。';

  @override
  String get evaluating => '評価中…';

  @override
  String get setTheInputsThenStep => '入力を設定してからステップしてください。';

  @override
  String get step => 'ステップ';

  @override
  String get reset => 'リセット';

  @override
  String get theDesignContainsAnInstantaneousCycle => '設計に瞬時サイクルがあります。';

  @override
  String get noTicksEvaluatedYet => 'まだティックが評価されていません。';

  @override
  String get selectAnInputAColumnOrA => '入力、列、コンセプトのいずれかを選択してください。';

  @override
  String get aRelationshipItIsAppliedInsideOther => '関係：他の関係の内部で適用されるため、サンプルできる固有の値を持ちません。';

  @override
  String get now => '現在';

  @override
  String get notSteppedYet => '未ステップ';

  @override
  String get finalTarget => '最終ターゲット';

  @override
  String get noneYet => 'まだなし';

  @override
  String get probe => 'プローブ';

  @override
  String get overTheRun => '実行全体';

  @override
  String get notEvaluatedAtAnyTickYet => 'まだどのティックでも評価されていません。';

  @override
  String get pwmChannel => 'PWM チャネル';

  @override
  String get digitalOutput => 'デジタル出力';

  @override
  String get hBridgeChannel => 'H ブリッジチャネル';

  @override
  String get iCSensor => 'I²C センサー';

  @override
  String get quadratureEncoder => '直交エンコーダー';

  @override
  String get target => 'ターゲット';

  @override
  String get noBoardsKnown => '既知のボードがありません';

  @override
  String get chooseABoard => 'ボードを選択…';

  @override
  String get loadingBoards => 'ボードを読み込み中…';

  @override
  String get chooseABoardToSeeWhetherThis => 'ボードを選ぶと、この設計が収まるかどうかを確認できます。';

  @override
  String get devices => 'デバイス';

  @override
  String get addDevice => 'デバイスを追加';

  @override
  String get noDeviceYetADeviceRealisesOne =>
      'デバイスはまだありません。デバイスはボード上で 1 つの出力を実現します。その種類が、ボードに必要とするものを決めます。';

  @override
  String get noOutput => '出力なし';

  @override
  String placedBeforeTheDeadEnd(Object placements) {
    return '行き止まりの前に配置済み：$placements。これはソルバーの順序における 1 つの衝突であり、唯一のものとは限りません。';
  }

  @override
  String get fixes => '修正';

  @override
  String get rename => '名前を変更';

  @override
  String get addToGroup => 'グループに追加';

  @override
  String get addInstance => 'インスタンスを追加';

  @override
  String get newBehaviorGroup => '新規振る舞いグループ';

  @override
  String get input => '入力';

  @override
  String get more => 'その他…';

  @override
  String get addConcept => 'コンセプトを追加';

  @override
  String get addAConceptFromTheLibraryTo => 'ライブラリからコンセプトを追加して始めましょう';

  @override
  String get valueNotDecided => '値の形式は未決定';

  @override
  String get aQuantity => '量';

  @override
  String get onOrOff => 'オンまたはオフ';

  @override
  String get aCount => '個数';

  @override
  String get aCollection => 'コレクション';

  @override
  String get aGroupedValue => '組の値';

  @override
  String get anOptionalValue => '省略可能な値';

  @override
  String get declaredNotYetDefined => '宣言済み、未定義';

  @override
  String get definitionDoesNotCheck => '定義が検査を通りません';

  @override
  String get noTimingDomainYet => 'タイミングドメイン未設定';

  @override
  String get drivenByAnIllFormedConnection => '不正な接続に駆動されています';

  @override
  String get contestedBySeveralDrivers => '複数の駆動元が競合しています';

  @override
  String get noDomain => 'ドメインなし';

  @override
  String get luminousIntensity => '光度';

  @override
  String get amountOfSubstance => '物質量';

  @override
  String backsThePort(Object port) {
    return 'ポート「$port」を裏付けます。インスタンスに見えるのは約束であり、それはこの定義ではなくコンポーネントに保持されます。';
  }

  @override
  String evaluatedAtEachActivationOf(Object domain) {
    return '$domain の起動ごとに評価されます。別のドメインの値を読むには明示的な転送が必要です。';
  }

  @override
  String eachActivationCommitsThisValueTo(Object output) {
    return '起動ごとにこの値を $output にコミットします。出力の駆動元は 1 つだけです。2 つ目は優先順位ではなく衝突になります。';
  }

  @override
  String relationshipsWithInputsNeedADefinition(Object names) {
    return '$names：入力を持つ関係は、実行する前に定義が必要です。';
  }

  @override
  String oneOfNInstancesOf(int count, Object component) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$component の $count 個のインスタンスの 1 つ。ここに見えるのはその約束で、ソースはすべてのインスタンスで共有されます。',
    );
    return '$_temp0';
  }

  @override
  String carriedAcrossTimingDomains(Object init) {
    return 'タイミングドメインをまたいで運ばれます。宛先は自身の起動より厳密に前にコミットされた最後の値を見ます。開始値は $init です。';
  }

  @override
  String get aBehaviorInAComponentIsAWayOfSeeing =>
      '振る舞いはこのコンポーネントのソースの見方の一つです。グループ化、移動、分割、解除をしても、コンポーネントの約束や動作は変わりません。';

  @override
  String alreadyTakesItsValueFrom(Object to, Object source) {
    return '$to はすでに $source から値を取っています。要求ポートの入力元は 1 つだけです。現在の接続が先に削除されます。';
  }

  @override
  String transportExplanation(Object from, Object fromDomain, Object to, Object toDomain) {
    return '$from は $fromDomain で、$to は $toDomain で更新されます。値はドメインをまたいで運ばれ、$to は自身の起動より厳密に前にコミットされた最後の値を見るため、開始値が必要です。';
  }

  @override
  String unsavedChangesExplanation(Object action) {
    return 'プロジェクトに未保存の変更があります。保存すると、未完成の数式やまだビルドできないテキストも含め、今の状態がそのまま保持されます。保存せずに$actionと、プロジェクトは最後に保存した状態に戻ります。';
  }

  @override
  String thisFileDoesNotBuildYet(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'このファイルはまだビルドできません（問題 $count 件）。設計には最後にビルドできたバージョンが表示されています。',
    );
    return '$_temp0';
  }

  @override
  String get ofComponent => 'コンポーネント';

  @override
  String get stateDefined => '定義済み';

  @override
  String get stateUndriven => '未駆動';

  @override
  String get stateDriven => '駆動済み';

  @override
  String get stateRequired => '必須';

  @override
  String get stateOptional => '省略可';

  @override
  String get stateDeclared => '宣言済み';

  @override
  String get stateContested => '競合';

  @override
  String get stateIllFormed => '不正';

  @override
  String get formOnOffShort => 'オン/オフ';

  @override
  String get formCountShort => '個数';

  @override
  String get formCollectionShort => 'コレクション';

  @override
  String expressionOver(Object inputs) {
    return '$inputs を使う式';
  }

  @override
  String needsAValueForThisStep(Object who) {
    return '$who にはこのステップの値が必要です。';
  }

  @override
  String dividedByZero(Object who) {
    return '$who でゼロ除算が発生しました。';
  }

  @override
  String producedAValueThatIsNotANumber(Object who) {
    return '$who が数値でない値を生成しました。';
  }

  @override
  String updatesInClockParameter(Object clock) {
    return '$clock で更新（コンポーネントのパラメータ）';
  }

  @override
  String updatesInOwnClock(Object clock) {
    return '固有の $clock で更新';
  }

  @override
  String get portProvided => '提供';

  @override
  String get portBound => 'バインド済み';

  @override
  String portBoundTo(Object source) {
    return '$source にバインド';
  }

  @override
  String get portOpen => 'オープン';

  @override
  String get dimAngle => '角度';

  @override
  String get dimLength => '長さ';

  @override
  String get dimTime => '時間';

  @override
  String get dimMass => '質量';

  @override
  String get dimTemperature => '温度';

  @override
  String get dimCurrent => '電流';

  @override
  String minutesAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 分前');
    return '$_temp0';
  }

  @override
  String hoursAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 時間前');
    return '$_temp0';
  }

  @override
  String daysAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 日前');
    return '$_temp0';
  }

  @override
  String get yesterday => '昨日';

  @override
  String undoTooltip(Object shortcut) {
    return '取り消す（$shortcut）';
  }

  @override
  String redoTooltip(Object shortcut) {
    return 'やり直す（$shortcut）';
  }

  @override
  String notYetDefinedCount(int count) {
    return '未定義 $count 件';
  }

  @override
  String definitionsDoNotCheckCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '検査を通らない定義 $count 件');
    return '$_temp0';
  }

  @override
  String definitionsNotAddedCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '未追加の定義 $count 件');
    return '$_temp0';
  }

  @override
  String feasibleOnBoard(Object board) {
    return '$board に配置可能';
  }

  @override
  String notFeasibleOnBoard(Object board) {
    return '$board に配置不可';
  }

  @override
  String incompleteOnBoard(Object board) {
    return '$board 上で未完了';
  }

  @override
  String compilerVersion(Object version) {
    return 'コンパイラ $version';
  }

  @override
  String protocolVersion(Object version) {
    return 'プロトコル $version';
  }

  @override
  String conceptsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: 'コンセプト $count 件');
    return '$_temp0';
  }

  @override
  String mappingsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: 'マッピング $count 件');
    return '$_temp0';
  }

  @override
  String sourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '入力元 $count 件');
    return '$_temp0';
  }

  @override
  String saveChangesTo(Object name) {
    return '「$name」への変更を保存しますか？';
  }

  @override
  String readConcept(Object concept) {
    return '$concept を読み取る';
  }

  @override
  String domainAlreadyExists(Object name) {
    return '$name という名前のドメインはすでに存在します。';
  }

  @override
  String instancesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: 'インスタンス $count 件');
    return '$_temp0';
  }

  @override
  String componentsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: 'コンポーネント $count 件');
    return '$_temp0';
  }

  @override
  String editingComponent(Object component) {
    return '$component を編集中';
  }

  @override
  String usedByInstances(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 個のインスタンスが使用');
    return '$_temp0';
  }

  @override
  String noConceptMatches(Object query) {
    return '「$query」に一致するコンセプトはありません。';
  }

  @override
  String deleteNamed(Object name) {
    return '$name を削除';
  }

  @override
  String changingThisRechecks(Object names) {
    return 'これを変更すると $names が再検査されます。';
  }

  @override
  String stillUsedBy(Object names) {
    return 'まだ $names が使用しています。';
  }

  @override
  String inGroup(Object group) {
    return 'グループ $group に属しています。';
  }

  @override
  String takesItsValueFrom(Object source) {
    return '$source から値を取ります。';
  }

  @override
  String takesItsValueFromTransported(Object source, Object init) {
    return '$source から値を取り、タイミングドメインをまたいで運びます。開始値は $init です。';
  }

  @override
  String checkedOnceValueDecided(Object names) {
    return '$names の値の形式が決まったら検査されます。';
  }

  @override
  String drivenBy(Object driver) {
    return '$driver が駆動しています。';
  }

  @override
  String contestedOutput(Object output, Object claimants) {
    return '$output にはすでに最終ターゲットがあります。$claimants の両方が駆動を主張しています。';
  }

  @override
  String stillDrivenBy(Object names) {
    return 'まだ $names が駆動しています';
  }

  @override
  String promiseBrokenOpenSource(Object component) {
    return '$component のソースはもう約束を守っていません。理由を見るにはソースを開いてください。';
  }

  @override
  String boundTo(Object source) {
    return '$source にバインドされています。';
  }

  @override
  String boundToTransported(Object source, Object init) {
    return '$source にバインドされ、タイミングドメインをまたいで運ばれます。開始値は $init です。';
  }

  @override
  String usedByInstancesNamed(int count, Object names) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 個のインスタンスが使用：$names。',
    );
    return '$_temp0';
  }

  @override
  String standsFor(Object concept) {
    return '$concept を表します';
  }

  @override
  String theSystemsOutput(Object output) {
    return 'システムの $output';
  }

  @override
  String clockParameterSuffix(Object clock) {
    return '$clock（パラメータ）';
  }

  @override
  String clockPrivateSuffix(Object clock) {
    return '$clock（固有）';
  }

  @override
  String relationshipsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '関係 $count 件');
    return '$_temp0';
  }

  @override
  String selectedCount(int count) {
    return '$count 件を選択中';
  }

  @override
  String selectionSummary(Object relationships, int others) {
    String _temp0 = intl.Intl.pluralLogic(
      others,
      locale: localeName,
      other: '、その他 $others 件',
      zero: '',
    );
    return '$relationships$_temp0。';
  }

  @override
  String groupNAsBehavior(int count) {
    return '$count 件の関係を振る舞いとしてグループ化';
  }

  @override
  String alreadyInABehavior(int count) {
    return '$count 件はすでに別の振る舞いに属しています。そちらから移動してください。';
  }

  @override
  String notSaved(Object error) {
    return '未保存：$error';
  }

  @override
  String notChecked(Object reason) {
    return '未検査：$reason';
  }

  @override
  String get compilerServiceUnavailable => 'コンパイラサービスを利用できません';

  @override
  String producesDescription(Object what) {
    return '$what を生成';
  }

  @override
  String get expressionHint => '式';

  @override
  String thisIs(Object what) {
    return 'これは $what です。';
  }

  @override
  String equationsCount(int count) {
    return '方程式（$count）';
  }

  @override
  String insertAfterThis(Object operator) {
    return 'この後に $operator を挿入';
  }

  @override
  String paramIsEachElementOf(Object param, Object type) {
    return '$param は各要素です：$type。';
  }

  @override
  String paramIsEachElement(Object param) {
    return '$param はコレクションの各要素です。';
  }

  @override
  String sourceOf(Object file) {
    return '$file のソース';
  }

  @override
  String notBuiltSuffix(Object file) {
    return '$file — 未ビルド';
  }

  @override
  String get checkingTheDesign => '設計を検査しています…';

  @override
  String dependOnEachOtherInTheSameInstant(String names) {
    return 'これらの関係は同じ瞬間に互いに依存しています：$names。いずれかが代わりに前の値を読み取る必要があります。';
  }

  @override
  String hasNoValidDefinition(String name) {
    return '$name には有効な定義がありません。';
  }

  @override
  String hasNoDefinitionReadsSomething(String name) {
    return '$name には定義がありません。何かを読み取る関係には、設計を実行する前に定義が必要です。';
  }

  @override
  String needsAValueFormBeforeInput(String concept, String input) {
    return '$input に値を与えるには、先に $concept の値の形式（量、オン／オフ、個数）が必要です。';
  }

  @override
  String needsAValueBeforeSimulationCanStep(String name) {
    return 'シミュレーションを進めるには $name の値が必要です。';
  }

  @override
  String get valueHint => '値';

  @override
  String get onWord => 'on';

  @override
  String get offWord => 'off';

  @override
  String get tickColumn => 'ティック';

  @override
  String get activeColumn => 'アクティブ';

  @override
  String hasNoValueFormYet(Object concept) {
    return '$concept の値の形式はまだ決まっていません。';
  }

  @override
  String ticksEvaluated(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count ティックを評価しました。');
    return '$_temp0';
  }

  @override
  String tickN(int n) {
    return 'ティック $n';
  }

  @override
  String noValueDeclarationProduces(Object concept) {
    return '$concept を生成する値の宣言がありません。';
  }

  @override
  String resourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: 'リソース $count 件');
    return '$_temp0';
  }

  @override
  String couldNotAnalyse(Object error) {
    return '解析できませんでした：$error';
  }

  @override
  String checkingBoard(Object board) {
    return '$board を確認中…';
  }

  @override
  String feasibleOnBoardSentence(Object board) {
    return '$board に配置可能です。';
  }

  @override
  String fitsBoardSoFar(Object board) {
    return '今のところ $board に収まります — バインディングは未完了です。';
  }

  @override
  String notFeasibleOnBoardSentence(Object board) {
    return '$board には配置できません。';
  }

  @override
  String placementOn(Object board) {
    return '$board 上の配置';
  }

  @override
  String notConnectedToAnOutput(Object devices) {
    return '出力に接続されていません：$devices。';
  }

  @override
  String noDeviceOnBoardFor(Object board, Object outputs) {
    return '$board 上に対応するデバイスがありません：$outputs。';
  }

  @override
  String requirementN(int n) {
    return '要件 $n';
  }

  @override
  String couldNotPlaceOf(Object what, Object who) {
    return '$who の $what を配置できませんでした。';
  }

  @override
  String nothingOnBoardCanCarry(Object board, Object what) {
    return '$board 上に $what を担えるものがありません。';
  }

  @override
  String fixedPinCannotCarry(Object pin, Object what) {
    return '手動で選んだピン $pin は、ここでは $what を担えません。';
  }

  @override
  String groupAsBehaviorCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '振る舞いとしてグループ化（関係 $count 件）',
    );
    return '$_temp0';
  }

  @override
  String readsSocket(Object name) {
    return '$name を読み取る';
  }

  @override
  String producesSocket(Object name) {
    return '$name を生成';
  }

  @override
  String get removeFromGroupButton => 'グループから外す';

  @override
  String get decideLaterLower => '後で決める';

  @override
  String get groupedValueTitle => '組の値';

  @override
  String get instanceTitle => 'インスタンス';

  @override
  String get componentTitle => 'コンポーネント';

  @override
  String get notPlacedYetSentence => 'まだ配置されていません。';

  @override
  String get packageAsReusableComponentTitle => '再利用可能なコンポーネントとしてパッケージ化';

  @override
  String get textMode => 'テキスト';

  @override
  String get stepTen => 'ステップ ×10';

  @override
  String get aRelationshipCapital => 'ある関係';

  @override
  String get delete => '削除';

  @override
  String get diagNotConnected => 'コンパイラサービスに接続されていません。';

  @override
  String get diagTransport => 'コンパイラサービスに到達できませんでした。';

  @override
  String get diagPickerUnavailable =>
      'システムのファイルダイアログを表示できませんでした。Studio をサンドボックス化されたホスト（埋め込みターミナルなど）から起動するとこうなります。Finder かターミナルから起動するか、下にパスを入力してください。';

  @override
  String get diagChangedOnDisk => 'プロジェクトを開いた後にディスク上で変更されました。';

  @override
  String get diagPersist => 'プロジェクトをディスクに書き込めませんでした。';

  @override
  String get diagStaleRevision => 'この編集はプロジェクトの古いリビジョンを対象にしていたため、適用されませんでした。';

  @override
  String get diagNothingToUndo => '取り消す操作はありません。';

  @override
  String get diagNothingToRedo => 'やり直す操作はありません。';

  @override
  String get diagNoProject => 'プロジェクトが開かれていません。';

  @override
  String get diagAlreadyOpen => 'すでにプロジェクトが開かれています。先に閉じてください。';

  @override
  String get diagNotASystem => 'このプロジェクトはフラットな設計で、振る舞いシステムではありません。';

  @override
  String get diagSimulationNotStarted => '先にシミュレーションを開始してください。';

  @override
  String get dialogOpenProject => 'プロジェクトを開く';

  @override
  String get dialogCreateProject => 'プロジェクトを作成';

  @override
  String get dialogUntitledProject => '名称未設定プロジェクト';

  @override
  String get roleSource => '入力元';

  @override
  String get role => '役割';

  @override
  String get realizationEnvironment => '環境から与えられます。デバイスはまだ割り当てられていません。';

  @override
  String get sourceExplanation =>
      '環境から振る舞いモデルに入る値で、アクティベーションごとに一度観測されます。欠けているものはありません。モデル内で計算したい場合にだけ定義を追加してください。';

  @override
  String get categorySources => '入力元';

  @override
  String get newSource => '新しい入力元';

  @override
  String get aSourceAValueTheEnvironmentProvides => '環境から与えられる値。何も読み取らず、アクティベーションごとに一度観測されます。';

  @override
  String get addSource => '入力元を追加';

  @override
  String get newSourceEllipsis => '新しい入力元…';

  @override
  String sourceNodeSemantics(String name, String concept) {
    return '$name、入力元：環境から振る舞いモデルに入る値。$concept を提供';
  }

  @override
  String sourceOfConcept(String concept) {
    return '$concept の入力元';
  }

  @override
  String get chooseWhatItProvidesTheOutputSocket => '何を提供するかを選んでください。右側にある唯一のソケットです。';

  @override
  String get theEnvironmentProvidesItNoInputSockets => '環境から与えられます。入力ソケットはなく、アクティベーションごとに一つの値です。';

  @override
  String get sources => '入力元';

  @override
  String get noSourcesARelationshipWithNoReads =>
      '入力元はありません。何も読み取らず定義もない関係が入力元です。その値は環境から与えられ、アクティベーションごとに一つです。';

  @override
  String get searchLibrary => 'ライブラリを検索';

  @override
  String get loadingTheLibrary => 'ライブラリを読み込み中…';

  @override
  String get theLibraryArrivesWithTheCompiler => 'ライブラリはコンパイラサービスとともに届きます。';

  @override
  String noLibraryMatches(Object query) {
    return 'ライブラリに「$query」に一致する項目はありません。';
  }

  @override
  String get openAProjectToInsertFrom => 'プロジェクトを開くと、ここから挿入できます。';

  @override
  String get libraryCreates => '作成されるもの';

  @override
  String libraryValueOf(Object name) {
    return '値: $name';
  }

  @override
  String librarySourceOf(Object name) {
    return '入力元: $name';
  }

  @override
  String libraryTypeOf(Object type) {
    return '型: $type';
  }

  @override
  String get libraryGroupExternal => '外部';

  @override
  String get libItem_std_environment_temperature_name => '温度';

  @override
  String get libItem_std_environment_temperature_description => '空気・部屋・モーターなど、何かがどれだけ温かいか。';

  @override
  String get libItem_std_environment_temperature_tags => '温度 熱 冷たさ';

  @override
  String get libItem_std_environment_ambient_light_name => '環境光';

  @override
  String get libItem_std_environment_ambient_light_description => '周囲から製品に当たる光の量。';

  @override
  String get libItem_std_environment_ambient_light_tags => '光 照度 明るさ 日光 暗さ';

  @override
  String get libItem_std_environment_humidity_name => '湿度';

  @override
  String get libItem_std_environment_humidity_description => '空気の相対湿度。0（乾燥）から 1（飽和）。';

  @override
  String get libItem_std_environment_humidity_tags => '湿度 湿気 水分';

  @override
  String get libItem_std_environment_air_pressure_name => '気圧';

  @override
  String get libItem_std_environment_air_pressure_description => '大気圧。高度で下がり、天候で変わる。';

  @override
  String get libItem_std_environment_air_pressure_tags => '気圧 圧力 大気 高度 天気';

  @override
  String get libItem_std_environment_sound_level_name => '音の大きさ';

  @override
  String get libItem_std_environment_sound_level_description => '周囲がどれだけ騒がしいか。0（無音）から 1 のレベル。';

  @override
  String get libItem_std_environment_sound_level_tags => '音 騒音 音量 マイク';

  @override
  String get libItem_std_human_button_pressed_name => 'ボタン押下';

  @override
  String get libItem_std_human_button_pressed_description => 'いまボタンが押されているかどうか。';

  @override
  String get libItem_std_human_button_pressed_tags => 'ボタン 押下 キー クリック';

  @override
  String get libItem_std_human_touch_name => 'タッチ';

  @override
  String get libItem_std_human_touch_description => '表面が触れられているかどうか。';

  @override
  String get libItem_std_human_touch_tags => 'タッチ 触れる 静電容量 指';

  @override
  String get libItem_std_human_switch_state_name => 'スイッチ状態';

  @override
  String get libItem_std_human_switch_state_description => 'トグルスイッチがオンかどうか。';

  @override
  String get libItem_std_human_switch_state_tags => 'スイッチ トグル オン オフ';

  @override
  String get libItem_std_human_dial_position_name => 'ダイヤル位置';

  @override
  String get libItem_std_human_dial_position_description => 'つまみがどれだけ回されたか。';

  @override
  String get libItem_std_human_dial_position_tags => 'ダイヤル つまみ 回転 ポテンショメータ';

  @override
  String get libItem_std_human_slider_position_name => 'スライダー位置';

  @override
  String get libItem_std_human_slider_position_description => 'スライダーが両端の間のどこにあるか。0 から 1。';

  @override
  String get libItem_std_human_slider_position_tags => 'スライダー フェーダー 位置';

  @override
  String get libItem_std_motion_distance_name => '距離';

  @override
  String get libItem_std_motion_distance_description => '何かがどれだけ離れているか。';

  @override
  String get libItem_std_motion_distance_tags => '距離 遠近 測距 近接 超音波';

  @override
  String get libItem_std_motion_position_name => '位置';

  @override
  String get libItem_std_motion_position_description => '何かが一つの軸に沿ってどこにあるか。';

  @override
  String get libItem_std_motion_position_tags => '位置 変位 軸 ストローク';

  @override
  String get libItem_std_motion_angle_name => '角度';

  @override
  String get libItem_std_motion_angle_description => '何かがどれだけ回転しているか。';

  @override
  String get libItem_std_motion_angle_tags => '角度 回転 度 ラジアン 方位';

  @override
  String get libItem_std_motion_tilt_name => '傾き';

  @override
  String get libItem_std_motion_tilt_description => '何かが水平からどれだけ傾いているか。';

  @override
  String get libItem_std_motion_tilt_tags => '傾き 傾斜 水平 加速度センサー';

  @override
  String get libItem_std_motion_speed_name => '速度';

  @override
  String get libItem_std_motion_speed_description => '何かがどれだけ速く動いているか。';

  @override
  String get libItem_std_motion_speed_tags => '速度 速さ';

  @override
  String get libItem_std_motion_acceleration_name => '加速度';

  @override
  String get libItem_std_motion_acceleration_description => '速度がどれだけ速く変わるか。衝撃や衝突はここにピークとして現れる。';

  @override
  String get libItem_std_motion_acceleration_tags => '加速度 衝撃 衝突 振動';

  @override
  String get libItem_std_motion_angular_velocity_name => '角速度';

  @override
  String get libItem_std_motion_angular_velocity_description => '何かがどれだけ速く回っているか。';

  @override
  String get libItem_std_motion_angular_velocity_tags => '角速度 回転速度 ジャイロ';

  @override
  String get libItem_std_motion_orientation_name => '向き';

  @override
  String get libItem_std_motion_orientation_description => '何かがどちらを向いているか。方位として。';

  @override
  String get libItem_std_motion_orientation_tags => '向き 方位 進行方向 コンパス';

  @override
  String get libItem_std_mechanical_force_name => '力';

  @override
  String get libItem_std_mechanical_force_description => '何かがどれだけ強く押す・引くか。';

  @override
  String get libItem_std_mechanical_force_tags => '力 押す力 引く力 荷重';

  @override
  String get libItem_std_mechanical_pressure_name => '圧力';

  @override
  String get libItem_std_mechanical_pressure_description => '面積あたりの力。タイヤ、配管、握力。';

  @override
  String get libItem_std_mechanical_pressure_tags => '圧力 タイヤ 配管';

  @override
  String get libItem_std_mechanical_torque_name => 'トルク';

  @override
  String get libItem_std_mechanical_torque_description => '軸にかかる回転力。';

  @override
  String get libItem_std_mechanical_torque_tags => 'トルク 軸 モーター';

  @override
  String get libItem_std_electrical_voltage_name => '電圧';

  @override
  String get libItem_std_electrical_voltage_description => 'ある点の電位。電源レール、セル、センス線。';

  @override
  String get libItem_std_electrical_voltage_tags => '電圧 電位 ボルト 電源';

  @override
  String get libItem_std_electrical_current_name => '電流';

  @override
  String get libItem_std_electrical_current_description => 'ある経路を流れる電流。';

  @override
  String get libItem_std_electrical_current_tags => '電流 アンペア 負荷';

  @override
  String get libItem_std_electrical_battery_level_name => 'バッテリー残量';

  @override
  String get libItem_std_electrical_battery_level_description => 'バッテリーがどれだけ満ちているか。0（空）から 1（満充電）。';

  @override
  String get libItem_std_electrical_battery_level_tags => 'バッテリー 残量 充電 電池';

  @override
  String get libItem_std_output_brightness_name => '明るさ';

  @override
  String get libItem_std_output_brightness_description => '光をどれだけ明るくするか。0（消灯）から 1（最大）。';

  @override
  String get libItem_std_output_brightness_tags => '明るさ 照明 調光 LED';

  @override
  String get libItem_std_output_color_name => '色';

  @override
  String get libItem_std_output_color_description => '光が示す色。色相環上の色相として。';

  @override
  String get libItem_std_output_color_tags => '色 色相 RGB 照明';

  @override
  String get libItem_std_output_display_value_name => '表示値';

  @override
  String get libItem_std_output_display_value_description => 'ディスプレイに表示する数値。';

  @override
  String get libItem_std_output_display_value_tags => '表示 数値 画面 読み';

  @override
  String get libItem_std_actuator_motor_speed_name => 'モーター速度';

  @override
  String get libItem_std_actuator_motor_speed_description => 'モーターをどれだけ速く回すか。最高速度に対する割合、−1 から 1。';

  @override
  String get libItem_std_actuator_motor_speed_tags => 'モーター 速度 PWM';

  @override
  String get libItem_std_actuator_motor_angle_name => 'モーター角度';

  @override
  String get libItem_std_actuator_motor_angle_description => 'モーターやステッパーが保つべき角度。';

  @override
  String get libItem_std_actuator_motor_angle_tags => 'モーター 角度 ステッパー';

  @override
  String get libItem_std_actuator_servo_position_name => 'サーボ位置';

  @override
  String get libItem_std_actuator_servo_position_description => 'サーボアームが動くべき角度。';

  @override
  String get libItem_std_actuator_servo_position_tags => 'サーボ 角度 位置';

  @override
  String get libItem_std_actuator_vibration_intensity_name => '振動強度';

  @override
  String get libItem_std_actuator_vibration_intensity_description =>
      'ハプティックモーターをどれだけ強く振動させるか。0 から 1。';

  @override
  String get libItem_std_actuator_vibration_intensity_tags => '振動 ハプティック バイブ';

  @override
  String get libItem_std_actuator_heater_power_name => 'ヒーター出力';

  @override
  String get libItem_std_actuator_heater_power_description => 'ヒーターをどれだけ働かせるか。0 から 1。';

  @override
  String get libItem_std_actuator_heater_power_tags => 'ヒーター 加熱 出力';

  @override
  String get libItem_std_actuator_fan_speed_name => 'ファン速度';

  @override
  String get libItem_std_actuator_fan_speed_description => 'ファンをどれだけ速く回すか。0（停止）から 1（最大）。';

  @override
  String get libItem_std_actuator_fan_speed_tags => 'ファン 速度 冷却';

  @override
  String get libItem_std_actuator_valve_opening_name => 'バルブ開度';

  @override
  String get libItem_std_actuator_valve_opening_description => 'バルブがどれだけ開いているか。0（閉）から 1（全開）。';

  @override
  String get libItem_std_actuator_valve_opening_tags => 'バルブ 開度 流量';

  @override
  String get libItem_std_audio_volume_name => '音量';

  @override
  String get libItem_std_audio_volume_description => '製品の再生音量。0（無音）から 1（最大）。';

  @override
  String get libItem_std_audio_volume_tags => '音量 スピーカー 音';

  @override
  String get libItem_std_audio_pitch_name => 'ピッチ';

  @override
  String get libItem_std_audio_pitch_description => '鳴らす音の周波数。';

  @override
  String get libItem_std_audio_pitch_tags => 'ピッチ 周波数 音程 ブザー';

  @override
  String get libItem_std_source_temperature_name => '温度センサー';

  @override
  String get libItem_std_source_temperature_description => '製品が測る温度（部屋、筐体、モーター）を、振る舞いが読む値として。';

  @override
  String get libItem_std_source_temperature_tags => '温度 センサー 熱 サーミスタ';

  @override
  String get libItem_std_source_tilt_name => '傾きセンサー';

  @override
  String get libItem_std_source_tilt_description =>
      '製品が水平からどれだけ傾いているかを、振る舞いが読む値として。ランプヘッド、ハンドヘルド、車体。';

  @override
  String get libItem_std_source_tilt_tags => '傾き センサー 傾斜 加速度センサー 水平';

  @override
  String get libItem_std_source_distance_name => '距離センサー';

  @override
  String get libItem_std_source_distance_description =>
      '何かがどれだけ離れているかを、振る舞いが読む値として。障害物、手、液面。どのセンサーで測るかはデプロイ時の選択。';

  @override
  String get libItem_std_source_distance_tags => '距離 センサー 測距 近接 超音波 レーザー';

  @override
  String get libItem_std_source_ambient_light_name => '環境光センサー';

  @override
  String get libItem_std_source_ambient_light_description =>
      '周囲から製品に当たる光の量を、振る舞いが読む値として。調光、画面の起動、日光の検出に。';

  @override
  String get libItem_std_source_ambient_light_tags => '環境光 センサー 照度 光 日光 明るさ';

  @override
  String get libItem_std_source_button_name => 'ボタン状態';

  @override
  String get libItem_std_source_button_description =>
      'いまボタンが押されているかを、振る舞いが活性化ごとに読む値として。イベントではなく安定した状態。';

  @override
  String get libItem_std_source_button_tags => 'ボタン 状態 押下 キー 入力';

  @override
  String get libItem_std_source_encoder_name => 'エンコーダー位置';

  @override
  String get libItem_std_source_encoder_description =>
      'ロータリーエンコーダーが報告する角度位置を、振る舞いが読む値として。つまみ、軸、ホイール。パルスを数えるのはデプロイの仕事。';

  @override
  String get libItem_std_source_encoder_tags => 'エンコーダー 位置 つまみ 軸 角度 入力';

  @override
  String get libItem_std_source_analog_name => 'アナログ入力';

  @override
  String get libItem_std_source_analog_description =>
      '振る舞いが解釈する汎用のアナログ読み値。値の種類はあなたに委ねる。入力が何を測るか分かったら、コンセプトに量を選ぶ。';

  @override
  String get libItem_std_source_analog_tags => 'アナログ 入力 ADC 電圧 読み値 汎用';

  @override
  String get libItem_std_source_external_name => '外部値';

  @override
  String get libItem_std_source_external_description =>
      '製品の外から与えられる値（ホスト、ネットワーク、シミュレーション）を、振る舞いが読む値として。入力元はセンサーとは限らない。';

  @override
  String get libItem_std_source_external_tags => '外部 入力元 ホスト ネットワーク シミュレーション 入力';
}
