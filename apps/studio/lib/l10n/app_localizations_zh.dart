// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTitle => 'BDL Studio';

  @override
  String get preferencesTitle => '偏好设置';

  @override
  String get preferencesMenu => '偏好设置…';

  @override
  String get done => '完成';

  @override
  String get languageLabel => '语言';

  @override
  String get languageSystemDefault => '跟随系统';

  @override
  String get languageHelp => '只改变 Studio 界面的用语。BDL 源码、项目文件和编译器不受影响。';

  @override
  String get monitor => '监视';

  @override
  String get liveValuesOnTheCanvasArriveWith => '画布上的实时值将随遥测功能到来（路线图第 S 步）。';

  @override
  String get bdlStudio => 'BDL Studio';

  @override
  String get edited => '已修改';

  @override
  String get save => '保存';

  @override
  String get close => '关闭';

  @override
  String get reloadFromDisk => '从磁盘重新载入';

  @override
  String get overwrite => '覆盖';

  @override
  String get dismiss => '关闭提示';

  @override
  String get noProject => '没有项目';

  @override
  String get saved => '已保存';

  @override
  String get notCausal => '存在瞬时循环';

  @override
  String get readsAcrossDomains => '跨时序域读取';

  @override
  String get outputsIncomplete => '输出不完整';

  @override
  String get compilerNotConnected => '编译器未连接';

  @override
  String get connectingToTheCompiler => '正在连接编译器';

  @override
  String get design => '设计';

  @override
  String get simulate => '仿真';

  @override
  String get deploy => '部署';

  @override
  String get projectManagerClosesThisProject => '项目管理器（会关闭当前项目）';

  @override
  String get start => '开始';

  @override
  String get newProject => '新建项目…';

  @override
  String get openProject => '打开项目…';

  @override
  String get openByPath => '按路径打开…';

  @override
  String get openProjectByPath => '按路径打开项目';

  @override
  String get newAtPath => '在路径处新建…';

  @override
  String get createProjectAtPath => '在路径处创建项目';

  @override
  String get waitingForTheCompilerTheStatusLine => '正在等待编译器。状态栏会显示连接情况。';

  @override
  String get recent => '最近';

  @override
  String get projectsYouOpenWillAppearHere => '你打开过的项目会出现在这里。';

  @override
  String get notFound => '未找到';

  @override
  String get removeFromRecent => '从最近列表移除';

  @override
  String get justNow => '刚刚';

  @override
  String get thisProject => '此项目';

  @override
  String get openAnotherProject => '打开另一个项目';

  @override
  String get createAnotherProject => '新建另一个项目';

  @override
  String get closeIt => '关闭它';

  @override
  String get donTSave => '不保存';

  @override
  String get cancel => '取消';

  @override
  String get folder => '文件夹';

  @override
  String get choose => '选择';

  @override
  String get name => '名称';

  @override
  String get create => '创建';

  @override
  String get newConcept => '新建概念';

  @override
  String get somethingTheProductSensesDecidesOrShows => '产品感知、判断或显示的某样东西。';

  @override
  String get value => '值';

  @override
  String get quantity => '量';

  @override
  String get onOff => '开 / 关';

  @override
  String get count => '计数';

  @override
  String get decideLater => '稍后决定';

  @override
  String get unit => '单位';

  @override
  String get meaning => '含义';

  @override
  String get hollowRingTheValueCanBeDecided => '空心圆环：值形式可以稍后决定；关系现在就可以使用它。';

  @override
  String get roundSocketAMeasuredQuantityItsUnit => '圆形插口：一个可测量的量。它的单位会在每条公式中被检查。';

  @override
  String get diamondSocketOnOrOffActivatesContexts => '菱形插口：开或关——激活上下文、控制行为。';

  @override
  String get squareSocketAWholeNumberOccurrencesSteps => '方形插口：一个整数——次数、步数、件数。';

  @override
  String get newMapping => '新建映射';

  @override
  String get aRelationshipBetweenConceptsItCanExist => '概念之间的一种关系。它可以先存在，再被定义。';

  @override
  String get reads => '读取';

  @override
  String get produces => '产出';

  @override
  String get chooseWhatItProducesTheOutputSocket => '选择它产出什么：输出插口会采用该概念的颜色和形状。';

  @override
  String get newTimingDomain => '新建时序域';

  @override
  String get whenAGroupOfRelationshipsAndOutputs => '一组关系和输出一起更新的时机。这是一个名字，不是频率：多久激活一次在设计运行时才决定。';

  @override
  String get interactionAmbient => 'interaction, ambient, …';

  @override
  String get newOutput => '新建输出';

  @override
  String get whereAValueLeavesTheDesignFor => '值离开设计、进入现实世界的地方：一盏灯、一个电机、一块显示屏。';

  @override
  String get accepts => '接受';

  @override
  String get theConceptThisOutputTakes => '此输出接受的概念';

  @override
  String get updatesIn => '更新于';

  @override
  String get required => '必需';

  @override
  String get theDesignIsIncompleteUntilSomethingDrives => '在有东西驱动它之前，设计是不完整的';

  @override
  String get showingTheLastVersionThatBuiltThe => '显示的是最近一次可构建的版本；文本中的改动尚未通过构建。';

  @override
  String get noProjectOpen => '没有打开的项目。';

  @override
  String get system => '系统';

  @override
  String get notPlacedYet => '尚未放置';

  @override
  String get itsPromiseIsWhatInstancesSeeEdits => '——实例看到的是它的承诺；这里的修改会影响每个实例。';

  @override
  String get code => '代码';

  @override
  String get split => '分栏';

  @override
  String get project => '项目';

  @override
  String get library => '库';

  @override
  String get concepts => '概念';

  @override
  String get mappings => '映射';

  @override
  String get timingDomains => '时序域';

  @override
  String get outputs => '输出';

  @override
  String get contexts => '上下文';

  @override
  String get components => '组件';

  @override
  String get newComponent => '新建组件';

  @override
  String get aReusableBehaviourWithAPromiseIts => '一个可复用的行为：有自己的承诺（端口）和自己的源码。';

  @override
  String get adaptivelamp => 'AdaptiveLamp';

  @override
  String get instances => '实例';

  @override
  String get behaviors => '行为';

  @override
  String get add => '添加';

  @override
  String get stillInUse => '仍在使用中';

  @override
  String get environment => '环境';

  @override
  String get humanInteraction => '人机交互';

  @override
  String get geometryMotion => '几何与运动';

  @override
  String get mechanical => '机械';

  @override
  String get electricalSystem => '电气与系统';

  @override
  String get visualDisplay => '视觉与显示';

  @override
  String get actuation => '执行机构';

  @override
  String get audio => '音频';

  @override
  String get noUnit => '无单位';

  @override
  String get groupedValue => '组合值';

  @override
  String get optionalValue => '可选值';

  @override
  String get searchConcepts => '搜索概念';

  @override
  String get loadingTheConceptLibrary => '正在加载概念库…';

  @override
  String get theConceptLibraryArrivesWithTheCompiler => '概念库随编译器服务一起提供。';

  @override
  String get openAProjectToAddConceptsFrom => '打开一个项目，即可从这里添加概念。';

  @override
  String get selectAConceptARelationshipAnOutput => '选择一个概念、关系、输出、实例或分组。';

  @override
  String get selectAConceptAMappingOrAn => '选择一个概念、映射或输出。';

  @override
  String get inspector => '检查器';

  @override
  String get nothingElseNeedsRechecking => '没有其他内容需要重新检查。';

  @override
  String get thisWillBeCheckedAgain => '这一项会被重新检查。';

  @override
  String get thisChangeAffects => '此改动影响';

  @override
  String get itWillBeCheckedAgain => '——它会被重新检查。';

  @override
  String get theyWillBeCheckedAgain => '——它们会被重新检查。';

  @override
  String get lastChange => '最近一次改动';

  @override
  String get interface => '接口';

  @override
  String get realization => '实现';

  @override
  String get semantic => '语义';

  @override
  String get reactive => '响应';

  @override
  String get clock => '时钟';

  @override
  String get output => '输出';

  @override
  String get deployment => '部署';

  @override
  String get unspecified => '未指定';

  @override
  String get order => '顺序';

  @override
  String get valuesAreMagnitudesSmallestLargestClampIn => '值是可比较大小的量：<、最小、最大、限幅、区间内';

  @override
  String get valuesAreComparedForEqualityOnly => '值只比较是否相等';

  @override
  String get relationships => '关系';

  @override
  String get producedBy => '产出自';

  @override
  String get nothingYet => '暂无';

  @override
  String get usedBy => '被使用于';

  @override
  String get explain => '解释';

  @override
  String get collectionOf => '集合，元素为…';

  @override
  String get optional => '可选…';

  @override
  String get each => '每个元素';

  @override
  String get whenPresent => '存在时';

  @override
  String get first => '第一项';

  @override
  String get second => '第二项';

  @override
  String get place => '位置';

  @override
  String get showGroup => '显示分组';

  @override
  String get relationship => '关系';

  @override
  String get showBinding => '显示绑定';

  @override
  String get disconnect => '断开';

  @override
  String get disconnectItToDefineTheRelationshipYourself => '断开后即可自行定义这个关系。';

  @override
  String get timing => '时序';

  @override
  String get anyDomain => '任意时序域';

  @override
  String get aRelationshipInNoDomainIsPure => '不属于任何时序域的关系是纯的：在哪里被读取，就在哪里求值。';

  @override
  String get drives => '驱动';

  @override
  String get thisValueCanCommitToAPhysical => '此值可以提交到一个物理输出。';

  @override
  String get onlyARelationshipWithoutInputsCanDrive => '规则不能驱动输出——请连接应用此规则的值。';

  @override
  String appliedByValue(String value) {
    return '$value 应用了它。';
  }

  @override
  String recordedAsDriving(String output) {
    return '已记录为驱动 $output。';
  }

  @override
  String noValueOfConceptYet(String concept) {
    return '设计中还没有 $concept 的值——一个不读取任何内容并产生 $concept 的关系可以驱动此输出。';
  }

  @override
  String get noExplicitInputsTheCanonicalDomainIs => '没有显式输入：规范定义域是 ()，即空积；内核把 () -> B 编码为 B';

  @override
  String get pendingAnalysis => '等待分析';

  @override
  String get noTimingDomainYetNotPartOf => '尚无时序域：在选定之前，它不属于设计的承诺。';

  @override
  String get undrivenTheDesignIsIncompleteWithoutA => '未被驱动——没有驱动方，设计就不完整。';

  @override
  String get undriven => '未被驱动。';

  @override
  String get theConnectionDoesNotFitSeeThe => '连接不匹配：请看下方驱动方的发现项。';

  @override
  String get checking => '正在检查…';

  @override
  String get noDomainYet => '尚无时序域';

  @override
  String get theDesignIsIncompleteUntilThisIs => '在它被驱动之前，设计是不完整的';

  @override
  String get driver => '驱动方';

  @override
  String get connect => '连接';

  @override
  String get aValue => '选择一个值…';

  @override
  String get anyTimingDomain => '任意时序域';

  @override
  String get systemInput => '系统输入';

  @override
  String get givenAValue => '已赋值';

  @override
  String get editSource => '编辑源码';

  @override
  String get replaceWith => '替换为';

  @override
  String get anotherComponent => '另一个组件…';

  @override
  String get notAssigned => '未指定';

  @override
  String get ports => '端口';

  @override
  String get noPortsYet => '尚无端口。';

  @override
  String get findings => '发现项';

  @override
  String get remove => '移除';

  @override
  String get disconnectItsPortsFirstTheComponentStays => '请先断开它的端口；组件会保留。';

  @override
  String get port => '端口';

  @override
  String get promise => '承诺';

  @override
  String get implementedBy => '实现于';

  @override
  String get nothingThePromiseHasNoBacking => '无（承诺没有对应的实现）';

  @override
  String get goToSource => '转到源码';

  @override
  String get aConstantEG05 => '一个常量，例如 0.5';

  @override
  String get aClosedConstantInThePortS => '以端口的单位表示的封闭常量；或者改为绑定该端口。';

  @override
  String get connection => '连接';

  @override
  String get openNothingSuppliesItYetDrawA =>
      '开放：还没有任何东西为它供值。从一个提供端口或同一概念的顶层关系画一条连线。存在开放端口的设计仍然可以仿真，此时端口作为输入。';

  @override
  String get binding => '绑定';

  @override
  String get from => '来自';

  @override
  String get to => '到';

  @override
  String get directTheDestinationReadsTheValueAs => '直接：目标按原样读取该值。';

  @override
  String get aBindingConvertsNothingBothEndsCarry => '绑定不做任何转换：两端按身份承载同一个概念。';

  @override
  String get keepsItsPromise => '履行承诺';

  @override
  String get promiseBroken => '承诺已失效';

  @override
  String get backToSystem => '‹ 系统';

  @override
  String get duplicateAsVersion => '复制为新版本';

  @override
  String get placeInstance => '放置实例';

  @override
  String get noPortsYetSelectARelationshipOf => '尚无端口。在源码中选择一个关系，并把它声明为端口。';

  @override
  String get timingParameters => '时序参数';

  @override
  String get aParameterTheSystemAssignsIt => '参数（由系统指定）';

  @override
  String get privateToEachInstance => '每个实例私有';

  @override
  String get sharedConcepts => '共享概念';

  @override
  String get privateFreshPerInstance => '私有（每个实例各自一份）';

  @override
  String get aSharedConceptIsTheSameIdentity => '共享概念在任何地方都是同一个身份；私有概念属于此组件自身，每个实例各有一份。';

  @override
  String get physicalOutputs => '物理输出';

  @override
  String get privateOnePerInstance => '私有（每个实例一个）';

  @override
  String get declareAPort => '声明端口';

  @override
  String get exposeAs => '暴露为…';

  @override
  String get thePromiseIsTakenFromTheRelationship => '承诺取自该关系当前的样子，并保持不变，直到你在这里修改它。';

  @override
  String get deleteItsInstancesFirst => '请先删除它的实例。';

  @override
  String get aRelationshipOfTheSource => '源码中的一个关系';

  @override
  String get retirePort => '撤销端口';

  @override
  String get group => '分组';

  @override
  String get aBehaviorIsAWayOfSeeing => '行为只是看待设计的一种方式：分组、移动、拆分或解散它，都不会改变设计的含义。';

  @override
  String get addRelationship => '+ 添加关系';

  @override
  String get emptyDragRelationshipsInOrAddOne => '空。把关系拖进来，或添加一个。';

  @override
  String removeFromGroup(Object group) {
    return '从 $group 中移除';
  }

  @override
  String get boundary => '边界';

  @override
  String get computedOnceTheAnalysisArrives => '分析结果到达后即可计算。';

  @override
  String get inputs => '输入';

  @override
  String get open => '开放';

  @override
  String get internal => '内部';

  @override
  String get readOffTheDependencyGraphWhatThe => '从依赖图中读出：成员从外部读取什么、外部从成员读取什么。这是切分的一幅图景，本身不是连接。';

  @override
  String get package => '打包';

  @override
  String get packageAsReusableComponent => '打包为可复用组件…';

  @override
  String get turnsTheBehaviorIntoAComponentAnd => '把该行为变成一个组件，并在原位置放一个实例；设计计算出的值不变。';

  @override
  String get aBehaviorInsideAComponentStaysA =>
      '组件内部的行为仍只是看待其源码的一种方式。把它单独打包成组件需要嵌套组件的支持；在那之前，可以自由编辑和移动它。';

  @override
  String get canvas => '画布';

  @override
  String get expand => '展开';

  @override
  String get collapse => '折叠';

  @override
  String get ungroup => '解散分组';

  @override
  String get deleteGroupAndRelationships => '删除分组及其关系';

  @override
  String get ungroupKeepsEveryRelationshipWhereItIs => '解散分组会让每个关系留在原处。';

  @override
  String get behavior => '行为';

  @override
  String get theseRelationshipsReadEachOther => '这些关系相互读取。';

  @override
  String get theseRelationshipsUpdateInOneTimingDomain => '这些关系在同一个时序域中更新。';

  @override
  String get groupAsBehavior => '归为行为';

  @override
  String get replaceTheConnection => '替换这条连接？';

  @override
  String get carryAcrossTimingDomains => '跨时序域传递';

  @override
  String get startsAt => '起始值';

  @override
  String get aConstantInTheDestinationSUnits => '以目标的单位表示的常量，例如 0';

  @override
  String get disconnectAndConnect => '断开并连接';

  @override
  String get workingOutTheBoundary => '正在计算边界…';

  @override
  String get requires => '需要';

  @override
  String get whatTheMembersReadFromOutside => '成员从外部读取的内容';

  @override
  String get nothingTheComponentIsSelfContained => '无——组件是自包含的';

  @override
  String get provides => '提供';

  @override
  String get whatOutsideReadsFromTheMembers => '外部从成员读取的内容';

  @override
  String get nothingYetNothingOutsideReadsTheGroup => '暂无——外部尚未读取该分组';

  @override
  String get openRelationships => '开放关系';

  @override
  String get declaredInsideNotYetDefined => '在内部声明、尚未定义';

  @override
  String get treatAsInput => '视为输入';

  @override
  String get keepInternal => '保持内部';

  @override
  String get drivenByMembersTheDriveStaysWith => '由成员驱动；驱动关系随成员一起';

  @override
  String get staysTheSystemS => '留在系统';

  @override
  String get movesInside => '移入组件';

  @override
  String get everyDomainTheMembersUse => '成员用到的每个时序域';

  @override
  String get nonePure => '无——纯关系';

  @override
  String get readByMembersOnly => '仅被成员读取';

  @override
  String get theDesignComputesTheSameValuesAfterwards => '之后设计计算出的值不变；分组在原位置变成一个实例，组件可以再次放置。';

  @override
  String get expressionWithNoInputs => '没有输入的表达式';

  @override
  String get noDefinitionYetALegalStateOther => '尚无定义。这是合法状态：其他关系可能已经依赖它的签名。';

  @override
  String get nothingToAddYet => '暂无可添加的内容。';

  @override
  String get emptyDetachToRemoveTheDefinition => '为空。分离即可移除定义。';

  @override
  String get saving => '正在保存…';

  @override
  String get addDefinition => '添加定义';

  @override
  String get saveDefinition => '保存定义';

  @override
  String get invalidDefinition => '定义无效。';

  @override
  String get cannotBeRead => '无法读取。';

  @override
  String get openSomethingItNeedsIsNotDecided => '开放：它需要的某项内容尚未决定。';

  @override
  String get validDefinition => '定义有效';

  @override
  String get noDefinition => '没有定义。';

  @override
  String get formula => '公式';

  @override
  String get revert => '还原';

  @override
  String get addingADefinitionIsARefinementNothing => '添加定义是一次细化：不会重新打开其他地方已确立的结论。';

  @override
  String get detachDefinition => '分离定义';

  @override
  String get replacingOrDetachingIsAnEditThis => '替换或分离是一次编辑：此定义和仿真会被重新检查。';

  @override
  String get thisRelationshipSDefinitionChangedWhileYou => '在你编辑期间，这个关系的定义发生了变化。';

  @override
  String get reload => '重新载入';

  @override
  String get showTheDefinitionCommittedMeanwhileDropWhat => '显示期间提交的定义；丢弃你输入的内容';

  @override
  String get keepMine => '保留我的';

  @override
  String get keepWhatYouTypedSaveWillReplace => '保留你输入的内容；保存时将替换已提交的定义';

  @override
  String get looking => '正在查找…';

  @override
  String get waitingForTheCompilerToReadThe => '正在等待编译器读取公式…';

  @override
  String get theTextCannotBeReadAsA => '文本无法作为公式读取。';

  @override
  String get editAsText => '以文本编辑';

  @override
  String get askingWhatFitsHere => '正在询问这里可以放什么…';

  @override
  String get hideDetail => '隐藏详情';

  @override
  String get insert => '插入';

  @override
  String get noUnitIsSuggestedFillTheOther => '没有推荐的单位：请先填写另一侧。';

  @override
  String get references => '引用';

  @override
  String get compare => '比较';

  @override
  String get function => '函数';

  @override
  String get eachElement => '每个元素';

  @override
  String get allSatisfy => 'all … satisfy';

  @override
  String get anySatisfies => 'any … satisfies';

  @override
  String get mapEach => 'map each …';

  @override
  String get filter => 'filter …';

  @override
  String get range => '区间';

  @override
  String get betweenTwoEndsIn => '两端之间：in … .. …';

  @override
  String get thisPartIsEditedAsText => '这一部分以文本方式编辑。';

  @override
  String get composeChoose => '选择';

  @override
  String get aChoiceIfThenElse => '选择：if … then … else …';

  @override
  String get negateThis => '取反：not …';

  @override
  String get readingTheSources => '正在读取源码…';

  @override
  String get show => '显示';

  @override
  String get wholeNumber => '整数';

  @override
  String get noDomainsEveryRelationshipIsEvaluatedAt => '没有时序域：每个关系在每个刻度都会求值。';

  @override
  String get activationPeriodInTicksChangingOneStarts => '激活周期，以刻度计。修改任何一个都会重新开始运行。';

  @override
  String get evaluating => '正在求值…';

  @override
  String get setTheInputsThenStep => '设置输入，然后步进。';

  @override
  String get step => '步进';

  @override
  String get reset => '重置';

  @override
  String get theDesignContainsAnInstantaneousCycle => '设计中存在瞬时循环。';

  @override
  String get noTicksEvaluatedYet => '尚未求值任何刻度。';

  @override
  String get selectAnInputAColumnOrA => '选择一个输入、一列或一个概念。';

  @override
  String get aRelationshipItIsAppliedInsideOther => '一个关系：它在其他关系内部被应用，本身没有可采样的值。';

  @override
  String get appliedBy => '应用于';

  @override
  String addAValueThatApplies(String name) {
    return '添加一个应用 $name 的值';
  }

  @override
  String get noValueYet => '尚无值';

  @override
  String get stateNotApplied => '未应用';

  @override
  String get appliedByNothing => '无任何值应用';

  @override
  String notPossibleYet(String reason) {
    return '暂不可行：$reason';
  }

  @override
  String get now => '当前';

  @override
  String get notSteppedYet => '尚未步进';

  @override
  String get finalTarget => '最终目标';

  @override
  String get noneYet => '暂无';

  @override
  String get probe => '探针';

  @override
  String get overTheRun => '整个运行过程';

  @override
  String get notEvaluatedAtAnyTickYet => '尚未在任何刻度求值。';

  @override
  String get pwmChannel => 'PWM 通道';

  @override
  String get digitalOutput => '数字输出';

  @override
  String get digitalInput => '数字输入';

  @override
  String get hBridgeChannel => 'H 桥通道';

  @override
  String get iCSensor => 'I²C 传感器';

  @override
  String get quadratureEncoder => '正交编码器';

  @override
  String get target => '目标板';

  @override
  String get noBoardsKnown => '没有已知的开发板';

  @override
  String get chooseABoard => '选择开发板…';

  @override
  String get loadingBoards => '正在加载开发板…';

  @override
  String get chooseABoardToSeeWhetherThis => '选择一块开发板，看看这个设计能否放得下。';

  @override
  String get devices => '设备';

  @override
  String get addDevice => '添加设备';

  @override
  String get noDeviceYetADeviceRealisesOne => '尚无设备。一个设备在开发板上实现一个输出；它的种类决定它需要开发板提供什么。';

  @override
  String get realizationProfile => '实现方式';

  @override
  String get noRealization => '无 — 按种类放置';

  @override
  String get doesNotFit => '不匹配';

  @override
  String get chooseABoardToSeeRealizations => '选择一块开发板，查看哪些实现方式适合这个输出。';

  @override
  String get encoderWellFormed => '编码器';

  @override
  String get representationFits => '匹配';

  @override
  String get hardwarePlaced => '已放置';

  @override
  String rawCommand(String ty) {
    return '原始命令 $ty';
  }

  @override
  String get noOutput => '无输出';

  @override
  String get notConnected => '未连接';

  @override
  String sourceItem(String name) {
    return '$name — 来源';
  }

  @override
  String get providerProfile => '提供方式';

  @override
  String get noProvider => '无 — 按种类放置';

  @override
  String get chooseABoardToSeeProviders => '选择一块开发板，查看哪些提供方式适合这个来源。';

  @override
  String get transducerWellFormed => '转换器';

  @override
  String get backendReadable => '可读取';

  @override
  String rawReading(String ty) {
    return '原始读数 $ty';
  }

  @override
  String fromPackage(String origin) {
    return '来自 $origin';
  }

  @override
  String placedBeforeTheDeadEnd(Object placements) {
    return '死路之前已放置：$placements。这只是求解器顺序下的一个冲突，不一定是唯一的冲突。';
  }

  @override
  String get fixes => '修复';

  @override
  String get rename => '重命名';

  @override
  String get addToGroup => '添加到分组';

  @override
  String get addInstance => '添加实例';

  @override
  String get newBehaviorGroup => '新建行为分组';

  @override
  String get input => '输入';

  @override
  String get more => '更多…';

  @override
  String get addConcept => '添加概念';

  @override
  String get addAConceptFromTheLibraryTo => '从库中添加一个概念开始';

  @override
  String get valueNotDecided => '值形式未决定';

  @override
  String get aQuantity => '一个量';

  @override
  String get onOrOff => '开或关';

  @override
  String get aCount => '一个计数';

  @override
  String get aCollection => '一个集合';

  @override
  String get aGroupedValue => '一个组合值';

  @override
  String get anOptionalValue => '一个可选值';

  @override
  String get declaredNotYetDefined => '已声明，尚未定义';

  @override
  String get definitionDoesNotCheck => '定义未通过检查';

  @override
  String get noTimingDomainYet => '尚无时序域';

  @override
  String get drivenByAnIllFormedConnection => '被一条不合规的连接驱动';

  @override
  String get contestedBySeveralDrivers => '被多个驱动方争用';

  @override
  String get noDomain => '无时序域';

  @override
  String get luminousIntensity => '发光强度';

  @override
  String get amountOfSubstance => '物质的量';

  @override
  String backsThePort(Object port) {
    return '支撑端口“$port”：实例看到的是承诺，它保存在组件上，而不是这个定义。';
  }

  @override
  String evaluatedAtEachActivationOf(Object domain) {
    return '在 $domain 的每次激活时求值；读取其他时序域的值需要显式传递。';
  }

  @override
  String eachActivationCommitsThisValueTo(Object output) {
    return '每次激活都把此值提交到 $output。每个输出只有一个驱动方：第二个是冲突，而不是优先级。';
  }

  @override
  String relationshipsWithInputsNeedADefinition(Object names) {
    return '$names：有输入的关系需要先有定义才能运行。';
  }

  @override
  String oneOfNInstancesOf(int count, Object component) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$component 的 $count 个实例之一。这里显示的是它的承诺；源码由所有实例共享。',
    );
    return '$_temp0';
  }

  @override
  String carriedAcrossTimingDomains(Object init) {
    return '跨时序域传递：目标看到的是严格早于自身激活时最后提交的值，起始值为 $init。';
  }

  @override
  String get aBehaviorInAComponentIsAWayOfSeeing => '行为只是看待此组件源码的一种方式：分组、移动、拆分或解散它，都不会改变组件的承诺或行为。';

  @override
  String alreadyTakesItsValueFrom(Object to, Object source) {
    return '$to 已经从 $source 取值。需求端口只能有一个来源；会先移除当前连接。';
  }

  @override
  String transportExplanation(Object from, Object fromDomain, Object to, Object toDomain) {
    return '$from 在 $fromDomain 中更新，$to 在 $toDomain 中更新。该值会跨域传递：$to 看到的是严格早于自身激活时最后提交的值，因此需要一个起始值。';
  }

  @override
  String unsavedChangesExplanation(Object action) {
    return '项目有未保存的改动。保存会保留现在的一切——包括未完成的公式和尚未通过构建的文本。如果不保存就$action，项目会回到上次保存的状态。';
  }

  @override
  String thisFileDoesNotBuildYet(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '此文件尚未通过构建（$count 个问题）：设计显示的是最近一次通过构建的版本。',
    );
    return '$_temp0';
  }

  @override
  String get ofComponent => '所属组件';

  @override
  String get stateDefined => '已定义';

  @override
  String get stateUndriven => '未被驱动';

  @override
  String get stateDriven => '已驱动';

  @override
  String get stateRequired => '必需';

  @override
  String get stateOptional => '可选';

  @override
  String get stateDeclared => '已声明';

  @override
  String get stateContested => '争用';

  @override
  String get stateIllFormed => '不合规';

  @override
  String get formOnOffShort => '开关';

  @override
  String get formCountShort => '计数';

  @override
  String get formCollectionShort => '集合';

  @override
  String expressionOver(Object inputs) {
    return '基于 $inputs 的表达式';
  }

  @override
  String needsAValueForThisStep(Object who) {
    return '$who 在这一步需要一个值。';
  }

  @override
  String dividedByZero(Object who) {
    return '$who 出现了除以零。';
  }

  @override
  String producedAValueThatIsNotANumber(Object who) {
    return '$who 产出了一个不是数字的值。';
  }

  @override
  String updatesInClockParameter(Object clock) {
    return '更新于 $clock（组件的参数）';
  }

  @override
  String updatesInOwnClock(Object clock) {
    return '更新于自己的 $clock';
  }

  @override
  String get portProvided => '已提供';

  @override
  String get portBound => '已绑定';

  @override
  String portBoundTo(Object source) {
    return '绑定到 $source';
  }

  @override
  String get portOpen => '开放';

  @override
  String get dimAngle => '角度';

  @override
  String get dimLength => '长度';

  @override
  String get dimTime => '时间';

  @override
  String get dimMass => '质量';

  @override
  String get dimTemperature => '温度';

  @override
  String get dimCurrent => '电流';

  @override
  String minutesAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 分钟前');
    return '$_temp0';
  }

  @override
  String hoursAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 小时前');
    return '$_temp0';
  }

  @override
  String daysAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 天前');
    return '$_temp0';
  }

  @override
  String get yesterday => '昨天';

  @override
  String undoTooltip(Object shortcut) {
    return '撤销（$shortcut）';
  }

  @override
  String redoTooltip(Object shortcut) {
    return '重做（$shortcut）';
  }

  @override
  String notYetDefinedCount(int count) {
    return '$count 个尚未定义';
  }

  @override
  String definitionsDoNotCheckCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个定义未通过检查');
    return '$_temp0';
  }

  @override
  String definitionsNotAddedCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个定义尚未添加');
    return '$_temp0';
  }

  @override
  String feasibleOnBoard(Object board) {
    return '可部署到 $board';
  }

  @override
  String notFeasibleOnBoard(Object board) {
    return '无法部署到 $board';
  }

  @override
  String incompleteOnBoard(Object board) {
    return '在 $board 上尚不完整';
  }

  @override
  String compilerVersion(Object version) {
    return '编译器 $version';
  }

  @override
  String protocolVersion(Object version) {
    return '协议 $version';
  }

  @override
  String conceptsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个概念');
    return '$_temp0';
  }

  @override
  String mappingsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个映射');
    return '$_temp0';
  }

  @override
  String sourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个来源');
    return '$_temp0';
  }

  @override
  String saveChangesTo(Object name) {
    return '保存对“$name”的改动？';
  }

  @override
  String readConcept(Object concept) {
    return '读取 $concept';
  }

  @override
  String domainAlreadyExists(Object name) {
    return '已存在名为 $name 的时序域。';
  }

  @override
  String instancesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个实例');
    return '$_temp0';
  }

  @override
  String componentsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个组件');
    return '$_temp0';
  }

  @override
  String editingComponent(Object component) {
    return '正在编辑 $component';
  }

  @override
  String usedByInstances(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '被 $count 个实例使用');
    return '$_temp0';
  }

  @override
  String noConceptMatches(Object query) {
    return '没有匹配“$query”的概念。';
  }

  @override
  String deleteNamed(Object name) {
    return '删除 $name';
  }

  @override
  String changingThisRechecks(Object names) {
    return '修改它会重新检查 $names。';
  }

  @override
  String stillUsedBy(Object names) {
    return '仍被 $names 使用。';
  }

  @override
  String inGroup(Object group) {
    return '属于分组 $group。';
  }

  @override
  String takesItsValueFrom(Object source) {
    return '从 $source 取值。';
  }

  @override
  String takesItsValueFromTransported(Object source, Object init) {
    return '从 $source 取值，跨时序域传递，起始值为 $init。';
  }

  @override
  String checkedOnceValueDecided(Object names) {
    return '$names 的值形式决定后再检查。';
  }

  @override
  String drivenBy(Object driver) {
    return '由 $driver 驱动。';
  }

  @override
  String contestedOutput(Object output, Object claimants) {
    return '$output 已有最终目标：$claimants 都在争用它。';
  }

  @override
  String stillDrivenBy(Object names) {
    return '仍由 $names 驱动';
  }

  @override
  String promiseBrokenOpenSource(Object component) {
    return '$component 的源码不再履行它的承诺；打开源码查看原因。';
  }

  @override
  String boundTo(Object source) {
    return '绑定到 $source。';
  }

  @override
  String boundToTransported(Object source, Object init) {
    return '绑定到 $source，跨时序域传递，起始值为 $init。';
  }

  @override
  String usedByInstancesNamed(int count, Object names) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '被 $count 个实例使用：$names。',
    );
    return '$_temp0';
  }

  @override
  String standsFor(Object concept) {
    return '代表 $concept';
  }

  @override
  String theSystemsOutput(Object output) {
    return '系统的 $output';
  }

  @override
  String clockParameterSuffix(Object clock) {
    return '$clock（参数）';
  }

  @override
  String clockPrivateSuffix(Object clock) {
    return '$clock（私有）';
  }

  @override
  String relationshipsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个关系');
    return '$_temp0';
  }

  @override
  String selectedCount(int count) {
    return '已选择 $count 项';
  }

  @override
  String selectionSummary(Object relationships, int others) {
    String _temp0 = intl.Intl.pluralLogic(
      others,
      locale: localeName,
      other: '，另有 $others 项',
      zero: '',
    );
    return '$relationships$_temp0。';
  }

  @override
  String groupNAsBehavior(int count) {
    return '把 $count 个关系归为行为';
  }

  @override
  String alreadyInABehavior(int count) {
    return '$count 个已属于某个行为；请先从那里移出。';
  }

  @override
  String notSaved(Object error) {
    return '未保存：$error';
  }

  @override
  String notChecked(Object reason) {
    return '未检查：$reason';
  }

  @override
  String get compilerServiceUnavailable => '编译器服务不可用';

  @override
  String producesDescription(Object what) {
    return '产出 $what';
  }

  @override
  String get expressionHint => '表达式';

  @override
  String thisIs(Object what) {
    return '这是 $what。';
  }

  @override
  String equationsCount(int count) {
    return '方程（$count）';
  }

  @override
  String insertAfterThis(Object operator) {
    return '在此之后插入 $operator';
  }

  @override
  String paramIsEachElementOf(Object param, Object type) {
    return '$param 是每个元素：$type。';
  }

  @override
  String paramIsEachElement(Object param) {
    return '$param 是集合中的每个元素。';
  }

  @override
  String sourceOf(Object file) {
    return '$file 的源码';
  }

  @override
  String notBuiltSuffix(Object file) {
    return '$file——未构建';
  }

  @override
  String get checkingTheDesign => '正在检查设计…';

  @override
  String dependOnEachOtherInTheSameInstant(String names) {
    return '这些关系在同一瞬间相互依赖：$names。其中一个必须改为读取上一个值。';
  }

  @override
  String hasNoValidDefinition(String name) {
    return '$name 没有有效的定义。';
  }

  @override
  String hasNoDefinitionReadsSomething(String name) {
    return '$name 没有定义。读取内容的关系需要先有定义，设计才能运行。';
  }

  @override
  String needsAValueFormBeforeInput(String concept, String input) {
    return '$concept 需要先有值形式（量、开 / 关或计数），$input 才能被赋值。';
  }

  @override
  String needsAValueBeforeSimulationCanStep(String name) {
    return '仿真前进之前 $name 需要一个值。';
  }

  @override
  String get valueHint => '值';

  @override
  String get onWord => 'on';

  @override
  String get offWord => 'off';

  @override
  String get tickColumn => '刻度';

  @override
  String get activeColumn => '激活';

  @override
  String hasNoValueFormYet(Object concept) {
    return '$concept 尚无值形式。';
  }

  @override
  String ticksEvaluated(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '已求值 $count 个刻度。');
    return '$_temp0';
  }

  @override
  String tickN(int n) {
    return '刻度 $n';
  }

  @override
  String resourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个资源');
    return '$_temp0';
  }

  @override
  String couldNotAnalyse(Object error) {
    return '无法分析：$error';
  }

  @override
  String checkingBoard(Object board) {
    return '正在检查 $board…';
  }

  @override
  String feasibleOnBoardSentence(Object board) {
    return '可部署到 $board。';
  }

  @override
  String fitsBoardSoFar(Object board) {
    return '目前可放入 $board——绑定尚未完成。';
  }

  @override
  String notFeasibleOnBoardSentence(Object board) {
    return '无法部署到 $board。';
  }

  @override
  String placementOn(Object board) {
    return '在 $board 上的放置';
  }

  @override
  String notConnectedToAnOutput(Object devices) {
    return '未连接到输出：$devices。';
  }

  @override
  String noDeviceOnBoardFor(Object board, Object outputs) {
    return '$board 上没有对应的设备：$outputs。';
  }

  @override
  String requirementN(int n) {
    return '需求 $n';
  }

  @override
  String couldNotPlaceOf(Object what, Object who) {
    return '无法放置 $who 的 $what。';
  }

  @override
  String nothingOnBoardCanCarry(Object board, Object what) {
    return '$board 上没有任何资源能承载 $what。';
  }

  @override
  String fixedPinCannotCarry(Object pin, Object what) {
    return '手动选择的引脚 $pin 在这里无法承载 $what。';
  }

  @override
  String groupAsBehaviorCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '归为行为（$count 个关系）');
    return '$_temp0';
  }

  @override
  String readsSocket(Object name) {
    return '读取 $name';
  }

  @override
  String producesSocket(Object name) {
    return '产出 $name';
  }

  @override
  String get removeFromGroupButton => '从分组中移除';

  @override
  String get decideLaterLower => '稍后决定';

  @override
  String get groupedValueTitle => '组合值';

  @override
  String get instanceTitle => '实例';

  @override
  String get componentTitle => '组件';

  @override
  String get notPlacedYetSentence => '尚未放置。';

  @override
  String get packageAsReusableComponentTitle => '打包为可复用组件';

  @override
  String get textMode => '文本';

  @override
  String get stepTen => '步进 ×10';

  @override
  String get aRelationshipCapital => '某个关系';

  @override
  String get delete => '删除';

  @override
  String get diagNotConnected => '编译器服务未连接。';

  @override
  String get diagTransport => '无法联系编译器服务。';

  @override
  String get diagPickerUnavailable =>
      '无法显示系统文件对话框。当 Studio 从受沙盒限制的宿主（例如内嵌终端）启动时会出现这种情况。请从 Finder 或终端启动，或在下方输入路径。';

  @override
  String get diagChangedOnDisk => '项目在打开之后已在磁盘上被修改。';

  @override
  String get diagPersist => '项目无法写入磁盘。';

  @override
  String get diagStaleRevision => '该编辑针对的是项目的旧版本，未被应用。';

  @override
  String get diagNothingToUndo => '没有可撤销的操作。';

  @override
  String get diagNothingToRedo => '没有可重做的操作。';

  @override
  String get diagNoProject => '没有打开的项目。';

  @override
  String get diagAlreadyOpen => '已有一个项目处于打开状态；请先关闭它。';

  @override
  String get diagNotASystem => '这个项目是平面设计，不是行为系统。';

  @override
  String get diagSimulationNotStarted => '请先开始仿真。';

  @override
  String get dialogOpenProject => '打开项目';

  @override
  String get dialogCreateProject => '创建项目';

  @override
  String get dialogUntitledProject => '未命名项目';

  @override
  String get roleSource => '来源';

  @override
  String get role => '角色';

  @override
  String get realizationEnvironment => '由环境提供；尚未绑定设备。';

  @override
  String realizationNoDeviceOn(String board) {
    return '由环境提供；$board 上尚无设备。';
  }

  @override
  String realizationProvidedBy(String device, String board) {
    return '由 $device 在 $board 上提供。';
  }

  @override
  String realizationProvidedByAs(String device, String profile, String board) {
    return '由 $device 以 $profile 在 $board 上提供。';
  }

  @override
  String get sourceExplanation => '由环境进入行为模型的值，每次激活观测一次。这里没有任何缺失：只有当你想改为在模型内部计算它时，才添加定义。';

  @override
  String get categorySources => '来源';

  @override
  String get newSource => '新建来源';

  @override
  String get aSourceAValueTheEnvironmentProvides => '由环境提供的值：不读取任何内容，每次激活观测一次。';

  @override
  String get addSource => '添加来源';

  @override
  String get newSourceEllipsis => '新建来源…';

  @override
  String sourceNodeSemantics(String name, String concept) {
    return '$name，来源：由环境进入行为模型的值，提供 $concept';
  }

  @override
  String sourceOfConcept(String concept) {
    return '$concept 的来源';
  }

  @override
  String get chooseWhatItProvidesTheOutputSocket => '选择它提供什么：其右侧唯一的插槽。';

  @override
  String get sources => '来源';

  @override
  String get noSourcesARelationshipWithNoReads => '没有来源：一个既不读取任何内容也没有定义的关系就是来源。它的值由环境提供，每次激活一个。';

  @override
  String get searchLibrary => '搜索库';

  @override
  String get loadingTheLibrary => '正在载入库…';

  @override
  String get theLibraryArrivesWithTheCompiler => '库随编译器服务一同到达。';

  @override
  String noLibraryMatches(Object query) {
    return '库中没有与“$query”匹配的项。';
  }

  @override
  String get openAProjectToInsertFrom => '打开项目后即可从此处插入。';

  @override
  String get libraryCreates => '将创建';

  @override
  String libraryValueOf(Object name) {
    return '值：$name';
  }

  @override
  String librarySourceOf(Object name) {
    return '来源：$name';
  }

  @override
  String libraryTypeOf(Object type) {
    return '类型：$type';
  }

  @override
  String get libraryGroupExternal => '外部';

  @override
  String get ruleWord => '规则';

  @override
  String get roleRule => '规则';

  @override
  String get roleValue => '值';

  @override
  String get ruleExplanation => '从它读取的概念到它产出的概念的函数。它本身没有值：由某个值的公式来应用它。';

  @override
  String get valueExplanation => '设计中的一个值：它的公式在每次激活时给出一个值。';

  @override
  String get dependsOn => '依赖于';

  @override
  String get namedIn => '被提及于';

  @override
  String ruleNodeSemantics(String reads) {
    return '规则，读取 $reads';
  }

  @override
  String get valueNodeSemantics => '值';

  @override
  String dependsOnList(String names) {
    return '依赖于 $names';
  }

  @override
  String sheetSourceShape(String concept) {
    return '不读取任何概念：一个来源。由环境在每次激活时提供 $concept；之后添加公式则会使它变为计算得到的值。';
  }

  @override
  String sheetRuleShape(String reads, String concept) {
    return '读取 $reads：一条规则，到 $concept 的函数。它本身没有值——由某个值的公式来应用它；在添加公式之前显示为虚线。';
  }

  @override
  String get aRuleNoValueOfItsOwn => '一条规则：它本身没有值。模拟器采样的是应用它的值。';

  @override
  String get noValueAppliesItYet => '尚无值应用它。';

  @override
  String noValueCarries(String concept) {
    return '尚无任何东西承载 $concept：没有值或来源产出它。';
  }

  @override
  String ruleProducesNoValue(String rule, String concept) {
    return '$rule 是一条规则；应用它的值才会承载 $concept。';
  }

  @override
  String get carriedBy => '承载于';

  @override
  String get format => '格式化';

  @override
  String get formatTooltip => '按规范布局整理文件（⌥⇧F）';

  @override
  String nothingNames(String name) {
    return '没有任何地方引用 $name。';
  }

  @override
  String placesNaming(int count, String name) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 处引用了 $name');
    return '$_temp0';
  }

  @override
  String get conceptLabel => '概念';

  @override
  String get existingConcept => '现有概念';

  @override
  String get sourceName => '来源名称';

  @override
  String get createSource => '创建来源';

  @override
  String get noConceptsYetCreateOne => '此设计中还没有概念——请选择“新建概念”。';

  @override
  String get sourceOverExistingConceptCaption => '只会创建来源；概念保持不变。撤销一次即移除该来源。';

  @override
  String get sourceWithNewConceptCaption => '概念与其来源会在一步中一起创建。撤销一次即移除两者。';

  @override
  String get inputForAConcept => '为你选择的概念创建一个输入';

  @override
  String inputForConceptOfKind(String kind) {
    return '为你选择的$kind概念创建一个输入——已有的或新建的';
  }

  @override
  String get chooseConcept => '选择一个概念';

  @override
  String get fixMenu => '修正';

  @override
  String get revealInCode => '在代码中显示';

  @override
  String get editDefinition => '编辑定义';

  @override
  String deleteObjects(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '删除 $count 个对象');
    return '$_temp0';
  }

  @override
  String showDriver(String name) {
    return '显示驱动方：$name';
  }

  @override
  String showEnd(String name) {
    return '显示 $name';
  }

  @override
  String replaceDriver(String current, String candidate) {
    return '用 $candidate 替换 $current';
  }

  @override
  String driveWith(String name) {
    return '由 $name 驱动';
  }

  @override
  String noDriverForConcept(String output, String concept) {
    return '$output 接受 $concept，但当前没有任何关系可以驱动它。';
  }

  @override
  String showInInspector(String name) {
    return '在检查器中显示 $name';
  }

  @override
  String get selectAll => '全选';

  @override
  String get frameAll => '显示全部';

  @override
  String get windowSelection => '窗口选择';

  @override
  String get crossingSelection => '交叉选择';

  @override
  String get activeObject => '当前对象';

  @override
  String get deleteBlockedDetails => '把它们也选上，或先断开连接。';

  @override
  String nothingWasDeleted(String reasons) {
    return '没有删除任何内容：$reasons。';
  }

  @override
  String outputsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个输出');
    return '$_temp0';
  }

  @override
  String othersCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '$count 个其他');
    return '$_temp0';
  }

  @override
  String get firmware => '固件';

  @override
  String get stepDeployment => '部署';

  @override
  String get stepBuild => '构建';

  @override
  String get stepFlash => '烧录';

  @override
  String get stepObserve => '观察';

  @override
  String buildForBoard(String board) {
    return '为 $board 构建';
  }

  @override
  String get buildAgain => '重新构建';

  @override
  String get stopBuild => '停止';

  @override
  String get flash => '烧录';

  @override
  String get flashAgain => '再次烧录';

  @override
  String get lookAgain => '重新查找';

  @override
  String get notReadyToBuild => '尚不能构建';

  @override
  String get fixOnTheDesignPage => '在设计页修正';

  @override
  String get building => '正在构建…';

  @override
  String get stageChecking => '检查部署';

  @override
  String get stageGenerating => '生成代码';

  @override
  String get stagePreparing => '准备工具链';

  @override
  String get stageCompiling => '编译';

  @override
  String get stagePackaging => '写入镜像';

  @override
  String get stageCancelled => '已停止';

  @override
  String cratesCompiled(int count) {
    String _temp0 = intl.Intl.pluralLogic(count, locale: localeName, other: '已编译 $count 个 crate');
    return '$_temp0';
  }

  @override
  String firmwareBuiltAt(String time) {
    return '固件已于 $time 构建';
  }

  @override
  String get firmwareStale => '固件来自更早的设计或部署。';

  @override
  String get buildDidNotComplete => '构建未完成';

  @override
  String imageSize(String bytes) {
    return '$bytes 字节';
  }

  @override
  String get noBoardReachable => '未找到可连接的开发板。';

  @override
  String get chooseTheBoardToFlash => '找到多块开发板——请选择一块。';

  @override
  String get flashing => '正在烧录…';

  @override
  String get flashPreparing => '准备';

  @override
  String get flashWriting => '写入镜像';

  @override
  String get flashRestarting => '重启开发板';

  @override
  String flashedToAt(String device, String time) {
    return '已于 $time 烧录到 $device';
  }

  @override
  String get flashDidNotComplete => '烧录未完成';

  @override
  String tryItOnTheBoard(String sources, String outputs) {
    return '试一试：操作 $sources；$outputs 应按设计响应。';
  }

  @override
  String tryItOutputsOnly(String outputs) {
    return '试一试：$outputs 应按设计运行。';
  }

  @override
  String get boardRunsEarlierDesign => '开发板运行的是更早的设计——重新构建并烧录以更新。';

  @override
  String get details => '详细信息';

  @override
  String get generatedCrate => '生成的 crate';

  @override
  String get buildCommand => '命令';

  @override
  String get rustTarget => '目标';

  @override
  String get imageFile => '镜像';

  @override
  String get compilerOutput => '编译器输出';

  @override
  String get demos => '演示';

  @override
  String readyToBuild(String board) {
    return '已可为 $board 构建。';
  }

  @override
  String get libraryValues => '值';

  @override
  String get libraryQuantities => '物理量';

  @override
  String get sourceRow => '来源';

  @override
  String get sourceRowHint => '环境提供的值——用于你选择的概念，现有的或新建的。';

  @override
  String get conceptSheetSubtitle => '来自库的值类别，以及它在本产品中的名称。';

  @override
  String get valueCategory => '值';

  @override
  String get measuredIn => '单位';

  @override
  String get nameRequired => '需要一个名称。';

  @override
  String nameTaken(String name) {
    return '$name 已被使用。';
  }

  @override
  String get nameNotIdentifier => '名称由字母、数字和 _ 组成，不能以数字开头。';

  @override
  String get createConcept => '创建概念';

  @override
  String get conceptSheetCaption => '创建一个概念，如同代码视图所写。';

  @override
  String get chooseACategory => '选择类别';

  @override
  String get showFormula => '显示公式';

  @override
  String get hideFormula => '隐藏公式';

  @override
  String get editFormula => '编辑公式';

  @override
  String get formulaPreviewUnavailable => '此处无法显示公式。';

  @override
  String get formulaPreviewLoading => '正在读取公式…';

  @override
  String get resultLabel => '结果';

  @override
  String get typeToWrite => '输入以书写，或选择一个部分';

  @override
  String get typeAnOperatorFirst => '在此处添加值之前先输入运算符。';

  @override
  String caretBefore(String what) {
    return '$what 之前';
  }

  @override
  String caretAfter(String what) {
    return '$what 之后';
  }

  @override
  String get caretIn => '在空位中';

  @override
  String caretInside(String what) {
    return '$what 内';
  }

  @override
  String fractionOf(String numerator, String denominator) {
    return '$numerator 除以 $denominator';
  }

  @override
  String choiceSemantics(String condition, String then, String otherwise) {
    return '一个选择：如果 $condition，则 $then，否则 $otherwise';
  }

  @override
  String matchSemantics(String value, int count) {
    return '对 $value 的匹配，共 $count 种情况';
  }

  @override
  String blockSemantics(int count) {
    return '含 $count 个局部绑定的块';
  }

  @override
  String temporalSemantics(String word) {
    return '一个 $word 边界';
  }

  @override
  String get libItem_std_value_boolean_name => '开 / 关';

  @override
  String get libItem_std_value_boolean_description => '非真即假的值：开或关、按下或未按下、有或无。';

  @override
  String get libItem_std_value_boolean_tags => '布尔 开关 真 假 按钮 按下 触摸 状态 标志';

  @override
  String get libItem_std_value_count_name => '计数';

  @override
  String get libItem_std_value_count_description => '事物的整数个数：脉冲、按压、件数、步数。';

  @override
  String get libItem_std_value_count_tags => '计数 整数 个数 脉冲 按压 次数 步数';

  @override
  String get libItem_std_value_level_name => '水平值';

  @override
  String get libItem_std_value_level_description => '无单位的普通数：0 到 1 的水平、比率、系数、百分比。';

  @override
  String get libItem_std_value_level_tags => '标量 比率 系数 百分比 无量纲 亮度 音量 湿度 电量 强度 开度';

  @override
  String get libItem_std_value_open_name => '稍后决定';

  @override
  String get libItem_std_value_open_description => '尚未选定值形式的概念；关系已可使用它，形式在明确后再定。';

  @override
  String get libItem_std_value_open_tags => '未定 稍后 未知 模拟 外部 原始 主机 网络';

  @override
  String get libItem_std_quantity_angle_name => '角度';

  @override
  String get libItem_std_quantity_angle_description => '转动了多少：倾斜、朝向、旋钮、盖子、轴。';

  @override
  String get libItem_std_quantity_angle_tags => '角度 旋转 转 倾斜 朝向 航向 旋钮 舵机 编码器 轴 盖 弧度 度';

  @override
  String get libItem_std_quantity_length_name => '长度';

  @override
  String get libItem_std_quantity_length_description => '沿直线的距离或位置：多远、多长、多高。';

  @override
  String get libItem_std_quantity_length_tags => '长度 距离 位置 范围 高度 宽度 深度 行程 障碍 接近 米 毫米 厘米';

  @override
  String get libItem_std_quantity_time_name => '时间';

  @override
  String get libItem_std_quantity_time_description => '持续时间或间隔：某事花多久、持续多久。';

  @override
  String get libItem_std_quantity_time_tags => '时间 时长 间隔 延迟 周期 超时 已用 秒 毫秒 分 小时';

  @override
  String get libItem_std_quantity_mass_name => '质量';

  @override
  String get libItem_std_quantity_mass_description => '某物有多少物质：负载、秤上的重量。';

  @override
  String get libItem_std_quantity_mass_tags => '质量 重量 负载 秤 千克 克';

  @override
  String get libItem_std_quantity_current_name => '电流';

  @override
  String get libItem_std_quantity_current_description => '电流：流过多少电荷，如电机或电源的消耗。';

  @override
  String get libItem_std_quantity_current_tags => '电流 电 安培 消耗 毫安';

  @override
  String get libItem_std_quantity_temperature_name => '温度';

  @override
  String get libItem_std_quantity_temperature_description => '某物有多热：空气、房间、电机、表面。';

  @override
  String get libItem_std_quantity_temperature_tags => '温度 热 冷 热量 温度计 开尔文 摄氏';

  @override
  String get libItem_std_quantity_amount_name => '物质的量';

  @override
  String get libItem_std_quantity_amount_description => '某种物质有多少，以摩尔计。';

  @override
  String get libItem_std_quantity_amount_tags => '物质的量 物质 摩尔 化学';

  @override
  String get libItem_std_quantity_luminous_intensity_name => '发光强度';

  @override
  String get libItem_std_quantity_luminous_intensity_description => '光源在某一方向上有多亮。';

  @override
  String get libItem_std_quantity_luminous_intensity_tags => '发光强度 光强 坎德拉 光源 灯';

  @override
  String get libItem_std_quantity_speed_name => '速度';

  @override
  String get libItem_std_quantity_speed_description => '沿直线移动多快：车辆、传送带、风。';

  @override
  String get libItem_std_quantity_speed_tags => '速度 速率 快慢 车轮 车辆 传送带 风';

  @override
  String get libItem_std_quantity_acceleration_name => '加速度';

  @override
  String get libItem_std_quantity_acceleration_description => '速度变化多快：冲击、重力感知的倾斜、车辆起步。';

  @override
  String get libItem_std_quantity_acceleration_tags => '加速度 加速度计 冲击 振动 惯性';

  @override
  String get libItem_std_quantity_angular_velocity_name => '角速度';

  @override
  String get libItem_std_quantity_angular_velocity_description => '转动多快：轴、轮、陀螺仪读数。';

  @override
  String get libItem_std_quantity_angular_velocity_tags => '角速度 转速 旋转速度 转每分 陀螺仪 轴 轮 电机转速';

  @override
  String get libItem_std_quantity_frequency_name => '频率';

  @override
  String get libItem_std_quantity_frequency_description => '每秒重复多少次：音高、脉冲率、闪烁。';

  @override
  String get libItem_std_quantity_frequency_tags => '频率 速率 音高 音调 赫兹 脉冲 闪烁 周期';

  @override
  String get libItem_std_quantity_force_name => '力';

  @override
  String get libItem_std_quantity_force_description => '推或拉：称重传感器读数、握力、弹簧。';

  @override
  String get libItem_std_quantity_force_tags => '力 推 拉 称重传感器 握力 弹簧 推力 牛顿';

  @override
  String get libItem_std_quantity_pressure_name => '压强';

  @override
  String get libItem_std_quantity_pressure_description => '单位面积上的力：空气、流体、触摸板上的按压。';

  @override
  String get libItem_std_quantity_pressure_tags => '压强 压力 气压计 大气 海拔 天气 流体 轮胎 帕 千帕';

  @override
  String get libItem_std_quantity_torque_name => '转矩';

  @override
  String get libItem_std_quantity_torque_description => '转动的力：电机输出到轴上的力矩。';

  @override
  String get libItem_std_quantity_torque_tags => '转矩 力矩 扭矩 电机 轴 牛米';

  @override
  String get libItem_std_quantity_power_name => '功率';

  @override
  String get libItem_std_quantity_power_description => '每秒的能量：加热器、电机或灯的消耗或输出。';

  @override
  String get libItem_std_quantity_power_tags => '功率 瓦 千瓦 加热器 消耗 瓦数';

  @override
  String get libItem_std_quantity_voltage_name => '电压';

  @override
  String get libItem_std_quantity_voltage_description => '电势：电池、电源轨、以伏特计的模拟读数。';

  @override
  String get libItem_std_quantity_voltage_tags => '电压 伏特 电势 电池 电源 模拟 毫伏';

  @override
  String get libItem_std_quantity_illuminance_name => '照度';

  @override
  String get libItem_std_quantity_illuminance_description => '落在表面上的光有多少：环境光、日光、光敏电阻读数。';

  @override
  String get libItem_std_quantity_illuminance_tags => '照度 光 勒克斯 环境光 日光 黑暗 光敏 光传感器';
}

/// The translations for Chinese, using the Han script (`zh_Hans`).
class AppLocalizationsZhHans extends AppLocalizationsZh {
  AppLocalizationsZhHans() : super('zh_Hans');
}
