// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'BDL Studio';

  @override
  String get preferencesTitle => 'Preferences';

  @override
  String get preferencesMenu => 'Preferences…';

  @override
  String get done => 'Done';

  @override
  String get languageLabel => 'Language';

  @override
  String get languageSystemDefault => 'System Default';

  @override
  String get languageHelp =>
      'Changes the words Studio uses. BDL source, project files and the compiler are unaffected.';

  @override
  String get monitor => 'Monitor';

  @override
  String get liveValuesOnTheCanvasArriveWith =>
      'Live values on the canvas arrive with telemetry (roadmap step S).';

  @override
  String get bdlStudio => 'BDL Studio';

  @override
  String get edited => 'Edited';

  @override
  String get save => 'Save';

  @override
  String get close => 'Close';

  @override
  String get reloadFromDisk => 'Reload from disk';

  @override
  String get overwrite => 'Overwrite';

  @override
  String get dismiss => 'Dismiss';

  @override
  String get noProject => 'No project';

  @override
  String get saved => 'Saved';

  @override
  String get notCausal => 'not causal';

  @override
  String get readsAcrossDomains => 'reads across domains';

  @override
  String get outputsIncomplete => 'outputs incomplete';

  @override
  String get compilerNotConnected => 'Compiler not connected';

  @override
  String get connectingToTheCompiler => 'Connecting to the compiler';

  @override
  String get design => 'Design';

  @override
  String get simulate => 'Simulate';

  @override
  String get deploy => 'Deploy';

  @override
  String get projectManagerClosesThisProject => 'Project manager (closes this project)';

  @override
  String get start => 'Start';

  @override
  String get newProject => 'New Project…';

  @override
  String get openProject => 'Open Project…';

  @override
  String get openByPath => 'Open by path…';

  @override
  String get openProjectByPath => 'Open project by path';

  @override
  String get newAtPath => 'New at path…';

  @override
  String get createProjectAtPath => 'Create project at path';

  @override
  String get waitingForTheCompilerTheStatusLine =>
      'Waiting for the compiler. The status line shows the connection.';

  @override
  String get recent => 'Recent';

  @override
  String get projectsYouOpenWillAppearHere => 'Projects you open will appear here.';

  @override
  String get notFound => 'not found';

  @override
  String get removeFromRecent => 'Remove from Recent';

  @override
  String get justNow => 'just now';

  @override
  String get thisProject => 'this project';

  @override
  String get openAnotherProject => 'open another project';

  @override
  String get createAnotherProject => 'create another project';

  @override
  String get closeIt => 'close it';

  @override
  String get donTSave => 'Don’t Save';

  @override
  String get cancel => 'Cancel';

  @override
  String get folder => 'Folder';

  @override
  String get choose => 'Choose';

  @override
  String get name => 'Name';

  @override
  String get create => 'Create';

  @override
  String get newConcept => 'New concept';

  @override
  String get somethingTheProductSensesDecidesOrShows =>
      'Something the product senses, decides or shows.';

  @override
  String get value => 'Value';

  @override
  String get quantity => 'Quantity';

  @override
  String get onOff => 'On / off';

  @override
  String get count => 'Count';

  @override
  String get decideLater => 'Decide later';

  @override
  String get unit => 'Unit';

  @override
  String get meaning => 'Meaning';

  @override
  String get hollowRingTheValueCanBeDecided =>
      'Hollow ring: the value can be decided later; relationships can already use it.';

  @override
  String get roundSocketAMeasuredQuantityItsUnit =>
      'Round socket: a measured quantity. Its unit is checked in every formula.';

  @override
  String get diamondSocketOnOrOffActivatesContexts =>
      'Diamond socket: on or off — activates contexts, gates behaviour.';

  @override
  String get squareSocketAWholeNumberOccurrencesSteps =>
      'Square socket: a whole number — occurrences, steps, items.';

  @override
  String get newMapping => 'New mapping';

  @override
  String get aRelationshipBetweenConceptsItCanExist =>
      'A relationship between concepts. It can exist before it is defined.';

  @override
  String get reads => 'Reads';

  @override
  String get produces => 'Produces';

  @override
  String get chooseWhatItProducesTheOutputSocket =>
      'Choose what it produces: the output socket takes that concept’s colour and shape.';

  @override
  String get newTimingDomain => 'New timing domain';

  @override
  String get whenAGroupOfRelationshipsAndOutputs =>
      'When a group of relationships and outputs update together. A name, not a rate: how often it activates is decided when the design runs.';

  @override
  String get interactionAmbient => 'interaction, ambient, …';

  @override
  String get newOutput => 'New output';

  @override
  String get whereAValueLeavesTheDesignFor =>
      'Where a value leaves the design for the world: a light, a motor, a display.';

  @override
  String get accepts => 'Accepts';

  @override
  String get theConceptThisOutputTakes => 'the concept this output takes';

  @override
  String get updatesIn => 'Updates in';

  @override
  String get required => 'Required';

  @override
  String get theDesignIsIncompleteUntilSomethingDrives =>
      'the design is incomplete until something drives it';

  @override
  String get showingTheLastVersionThatBuiltThe =>
      'Showing the last version that built; the text has changes that do not build yet.';

  @override
  String get noProjectOpen => 'No project open.';

  @override
  String get system => 'System';

  @override
  String get notPlacedYet => 'not placed yet';

  @override
  String get itsPromiseIsWhatInstancesSeeEdits =>
      '— its promise is what instances see; edits here reach every instance.';

  @override
  String get code => 'Code';

  @override
  String get split => 'Split';

  @override
  String get project => 'Project';

  @override
  String get library => 'Library';

  @override
  String get concepts => 'Concepts';

  @override
  String get mappings => 'Mappings';

  @override
  String get timingDomains => 'Timing domains';

  @override
  String get outputs => 'Outputs';

  @override
  String get contexts => 'Contexts';

  @override
  String get components => 'Components';

  @override
  String get newComponent => 'New component';

  @override
  String get aReusableBehaviourWithAPromiseIts =>
      'A reusable behaviour with a promise (its ports) and a source of its own.';

  @override
  String get adaptivelamp => 'AdaptiveLamp';

  @override
  String get instances => 'Instances';

  @override
  String get behaviors => 'Behaviors';

  @override
  String get add => 'Add';

  @override
  String get stillInUse => 'Still in use';

  @override
  String get environment => 'Environment';

  @override
  String get humanInteraction => 'Human interaction';

  @override
  String get geometryMotion => 'Geometry & motion';

  @override
  String get mechanical => 'Mechanical';

  @override
  String get electricalSystem => 'Electrical & system';

  @override
  String get visualDisplay => 'Visual & display';

  @override
  String get actuation => 'Actuation';

  @override
  String get audio => 'Audio';

  @override
  String get noUnit => 'no unit';

  @override
  String get groupedValue => 'grouped value';

  @override
  String get optionalValue => 'optional value';

  @override
  String get searchConcepts => 'Search concepts';

  @override
  String get loadingTheConceptLibrary => 'Loading the concept library…';

  @override
  String get theConceptLibraryArrivesWithTheCompiler =>
      'The concept library arrives with the compiler service.';

  @override
  String get openAProjectToAddConceptsFrom => 'Open a project to add concepts from here.';

  @override
  String get selectAConceptARelationshipAnOutput =>
      'Select a concept, a relationship, an output, an instance or a group.';

  @override
  String get selectAConceptAMappingOrAn => 'Select a concept, a mapping or an output.';

  @override
  String get inspector => 'Inspector';

  @override
  String get nothingElseNeedsRechecking => 'Nothing else needs rechecking.';

  @override
  String get thisWillBeCheckedAgain => 'This will be checked again.';

  @override
  String get thisChangeAffects => 'This change affects';

  @override
  String get itWillBeCheckedAgain => '— it will be checked again.';

  @override
  String get theyWillBeCheckedAgain => '— they will be checked again.';

  @override
  String get lastChange => 'Last change';

  @override
  String get interface => 'Interface';

  @override
  String get realization => 'Realization';

  @override
  String get semantic => 'Semantic';

  @override
  String get reactive => 'Reactive';

  @override
  String get clock => 'Clock';

  @override
  String get output => 'Output';

  @override
  String get deployment => 'Deployment';

  @override
  String get unspecified => 'Unspecified';

  @override
  String get order => 'Order';

  @override
  String get valuesAreMagnitudesSmallestLargestClampIn =>
      'values are magnitudes: <, smallest, largest, clamp, in range';

  @override
  String get valuesAreComparedForEqualityOnly => 'values are compared for equality only';

  @override
  String get relationships => 'Relationships';

  @override
  String get producedBy => 'Produced by';

  @override
  String get nothingYet => 'nothing yet';

  @override
  String get usedBy => 'Used by';

  @override
  String get explain => 'Explain';

  @override
  String get collectionOf => 'Collection of…';

  @override
  String get optional => 'Optional…';

  @override
  String get each => 'Each';

  @override
  String get whenPresent => 'When present';

  @override
  String get first => 'First';

  @override
  String get second => 'Second';

  @override
  String get place => 'Place';

  @override
  String get showGroup => 'Show Group';

  @override
  String get relationship => 'Relationship';

  @override
  String get showBinding => 'Show Binding';

  @override
  String get disconnect => 'Disconnect';

  @override
  String get disconnectItToDefineTheRelationshipYourself =>
      'Disconnect it to define the relationship yourself.';

  @override
  String get timing => 'Timing';

  @override
  String get anyDomain => 'any domain';

  @override
  String get aRelationshipInNoDomainIsPure =>
      'A relationship in no domain is pure: it is evaluated wherever it is read.';

  @override
  String get drives => 'Drives';

  @override
  String get thisValueCanCommitToAPhysical => 'This value can commit to a physical output.';

  @override
  String get onlyARelationshipWithoutInputsCanDrive =>
      'A rule cannot drive an output — connect the value that applies this rule.';

  @override
  String appliedByValue(String value) {
    return '$value applies it.';
  }

  @override
  String recordedAsDriving(String output) {
    return 'Recorded as driving $output.';
  }

  @override
  String noValueOfConceptYet(String concept) {
    return 'No value of $concept in the design yet — a relationship that reads nothing and produces $concept could drive this output.';
  }

  @override
  String get noExplicitInputsTheCanonicalDomainIs =>
      'no explicit inputs: the canonical domain is (), the empty product; the kernel encodes () -> B as B';

  @override
  String get pendingAnalysis => 'pending analysis';

  @override
  String get noTimingDomainYetNotPartOf =>
      'No timing domain yet: not part of the design\'s commitment until one is chosen.';

  @override
  String get undrivenTheDesignIsIncompleteWithoutA =>
      'Undriven — the design is incomplete without a driver.';

  @override
  String get undriven => 'Undriven.';

  @override
  String get theConnectionDoesNotFitSeeThe =>
      'The connection does not fit: see the driver\'s findings below.';

  @override
  String get checking => 'Checking…';

  @override
  String get noDomainYet => 'no domain yet';

  @override
  String get theDesignIsIncompleteUntilThisIs => 'the design is incomplete until this is driven';

  @override
  String get driver => 'Driver';

  @override
  String get connect => 'Connect';

  @override
  String get aValue => 'a value…';

  @override
  String get anyTimingDomain => 'Any timing domain';

  @override
  String get systemInput => 'system input';

  @override
  String get givenAValue => 'given a value';

  @override
  String get editSource => 'Edit Source';

  @override
  String get replaceWith => 'Replace with';

  @override
  String get anotherComponent => 'another component…';

  @override
  String get notAssigned => 'not assigned';

  @override
  String get ports => 'Ports';

  @override
  String get noPortsYet => 'No ports yet.';

  @override
  String get findings => 'Findings';

  @override
  String get remove => 'Remove';

  @override
  String get disconnectItsPortsFirstTheComponentStays =>
      'Disconnect its ports first; the component stays.';

  @override
  String get port => 'Port';

  @override
  String get promise => 'Promise';

  @override
  String get implementedBy => 'Implemented by';

  @override
  String get nothingThePromiseHasNoBacking => 'nothing (the promise has no backing)';

  @override
  String get goToSource => 'Go to Source';

  @override
  String get aConstantEG05 => 'a constant, e.g. 0.5';

  @override
  String get aClosedConstantInThePortS =>
      'A closed constant in the port\'s units, or bind the port instead.';

  @override
  String get connection => 'Connection';

  @override
  String get openNothingSuppliesItYetDrawA =>
      'Open: nothing supplies it yet. Draw a link from a provided port or a top-level relationship of the same concept. A design with open ports still simulates, with the port as an input.';

  @override
  String get binding => 'Binding';

  @override
  String get from => 'From';

  @override
  String get to => 'To';

  @override
  String get directTheDestinationReadsTheValueAs =>
      'Direct: the destination reads the value as it is.';

  @override
  String get aBindingConvertsNothingBothEndsCarry =>
      'A binding converts nothing: both ends carry the same concept, by identity.';

  @override
  String get keepsItsPromise => 'keeps its promise';

  @override
  String get promiseBroken => 'promise broken';

  @override
  String get backToSystem => '‹ System';

  @override
  String get duplicateAsVersion => 'Duplicate as Version';

  @override
  String get placeInstance => 'Place Instance';

  @override
  String get noPortsYetSelectARelationshipOf =>
      'No ports yet. Select a relationship of the source and declare it a port.';

  @override
  String get timingParameters => 'Timing parameters';

  @override
  String get aParameterTheSystemAssignsIt => 'a parameter (the system assigns it)';

  @override
  String get privateToEachInstance => 'private to each instance';

  @override
  String get sharedConcepts => 'Shared concepts';

  @override
  String get privateFreshPerInstance => 'private (fresh per instance)';

  @override
  String get aSharedConceptIsTheSameIdentity =>
      'A shared concept is the same identity everywhere; a private one is this component\'s own, fresh in every instance.';

  @override
  String get physicalOutputs => 'Physical outputs';

  @override
  String get privateOnePerInstance => 'private (one per instance)';

  @override
  String get declareAPort => 'Declare a port';

  @override
  String get exposeAs => 'expose as…';

  @override
  String get thePromiseIsTakenFromTheRelationship =>
      'The promise is taken from the relationship as it is now, and kept until you change it here.';

  @override
  String get deleteItsInstancesFirst => 'Delete its instances first.';

  @override
  String get aRelationshipOfTheSource => 'a relationship of the source';

  @override
  String get retirePort => 'Retire port';

  @override
  String get group => 'Group';

  @override
  String get aBehaviorIsAWayOfSeeing =>
      'A behavior is a way of seeing the design: grouping, moving, splitting or dissolving it changes nothing about what the design means.';

  @override
  String get addRelationship => '+ Add relationship';

  @override
  String get emptyDragRelationshipsInOrAddOne => 'Empty. Drag relationships in, or add one.';

  @override
  String removeFromGroup(Object group) {
    return 'Remove from $group';
  }

  @override
  String get boundary => 'Boundary';

  @override
  String get computedOnceTheAnalysisArrives => 'Computed once the analysis arrives.';

  @override
  String get inputs => 'Inputs';

  @override
  String get open => 'Open';

  @override
  String get internal => 'Internal';

  @override
  String get readOffTheDependencyGraphWhatThe =>
      'Read off the dependency graph: what the members read from outside, what outside reads from them. A picture of the cut, not a connection of its own.';

  @override
  String get package => 'Package';

  @override
  String get packageAsReusableComponent => 'Package as Reusable Component…';

  @override
  String get turnsTheBehaviorIntoAComponentAnd =>
      'Turns the behavior into a component and one instance in its place; the design computes the same values.';

  @override
  String get aBehaviorInsideAComponentStaysA =>
      'A behavior inside a component stays a way of seeing its source. Packaging it as a component of its own comes with nested components; until then, edit and move it freely.';

  @override
  String get canvas => 'Canvas';

  @override
  String get expand => 'Expand';

  @override
  String get collapse => 'Collapse';

  @override
  String get ungroup => 'Ungroup';

  @override
  String get deleteGroupAndRelationships => 'Delete group and relationships';

  @override
  String get ungroupKeepsEveryRelationshipWhereItIs =>
      'Ungroup keeps every relationship where it is.';

  @override
  String get behavior => 'Behavior';

  @override
  String get theseRelationshipsReadEachOther => 'These relationships read each other.';

  @override
  String get theseRelationshipsUpdateInOneTimingDomain =>
      'These relationships update in one timing domain.';

  @override
  String get groupAsBehavior => 'Group as Behavior';

  @override
  String get replaceTheConnection => 'Replace the connection?';

  @override
  String get carryAcrossTimingDomains => 'Carry across timing domains';

  @override
  String get startsAt => 'Starts at';

  @override
  String get aConstantInTheDestinationSUnits => 'a constant in the destination\'s units, e.g. 0';

  @override
  String get disconnectAndConnect => 'Disconnect and Connect';

  @override
  String get workingOutTheBoundary => 'Working out the boundary…';

  @override
  String get requires => 'Requires';

  @override
  String get whatTheMembersReadFromOutside => 'what the members read from outside';

  @override
  String get nothingTheComponentIsSelfContained => 'nothing — the component is self-contained';

  @override
  String get provides => 'Provides';

  @override
  String get whatOutsideReadsFromTheMembers => 'what outside reads from the members';

  @override
  String get nothingYetNothingOutsideReadsTheGroup =>
      'nothing yet — nothing outside reads the group';

  @override
  String get openRelationships => 'Open relationships';

  @override
  String get declaredInsideNotYetDefined => 'declared inside, not yet defined';

  @override
  String get treatAsInput => 'Treat as input';

  @override
  String get keepInternal => 'Keep internal';

  @override
  String get drivenByMembersTheDriveStaysWith => 'driven by members; the drive stays with them';

  @override
  String get staysTheSystemS => 'Stays the system\'s';

  @override
  String get movesInside => 'Moves inside';

  @override
  String get everyDomainTheMembersUse => 'every domain the members use';

  @override
  String get nonePure => 'none — pure';

  @override
  String get readByMembersOnly => 'read by members only';

  @override
  String get theDesignComputesTheSameValuesAfterwards =>
      'The design computes the same values afterwards; the group becomes one instance in its place, and the component can be placed again.';

  @override
  String get expressionWithNoInputs => 'expression with no inputs';

  @override
  String get noDefinitionYetALegalStateOther =>
      'No definition yet. A legal state: other relationships may already depend on the signature.';

  @override
  String get nothingToAddYet => 'Nothing to add yet.';

  @override
  String get emptyDetachToRemoveTheDefinition => 'Empty. Detach to remove the definition.';

  @override
  String get saving => 'Saving…';

  @override
  String get addDefinition => 'Add definition';

  @override
  String get saveDefinition => 'Save definition';

  @override
  String get invalidDefinition => 'Invalid definition.';

  @override
  String get cannotBeRead => 'Cannot be read.';

  @override
  String get openSomethingItNeedsIsNotDecided => 'Open: something it needs is not decided yet.';

  @override
  String get validDefinition => 'Valid definition';

  @override
  String get noDefinition => 'No definition.';

  @override
  String get formula => 'Formula';

  @override
  String get revert => 'Revert';

  @override
  String get addingADefinitionIsARefinementNothing =>
      'Adding a definition is a refinement: nothing established elsewhere is reopened.';

  @override
  String get detachDefinition => 'Detach definition';

  @override
  String get replacingOrDetachingIsAnEditThis =>
      'Replacing or detaching is an edit: this definition and the simulation are re-checked.';

  @override
  String get thisRelationshipSDefinitionChangedWhileYou =>
      'This relationship\'s definition changed while you were editing it.';

  @override
  String get reload => 'Reload';

  @override
  String get showTheDefinitionCommittedMeanwhileDropWhat =>
      'Show the definition committed meanwhile; drop what you typed';

  @override
  String get keepMine => 'Keep mine';

  @override
  String get keepWhatYouTypedSaveWillReplace =>
      'Keep what you typed; save will replace the committed definition';

  @override
  String get looking => 'Looking…';

  @override
  String get waitingForTheCompilerToReadThe => 'Waiting for the compiler to read the formula…';

  @override
  String get theTextCannotBeReadAsA => 'The text cannot be read as a formula.';

  @override
  String get editAsText => 'Edit as text';

  @override
  String get askingWhatFitsHere => 'Asking what fits here…';

  @override
  String get hideDetail => 'Hide detail';

  @override
  String get insert => 'Insert';

  @override
  String get noUnitIsSuggestedFillTheOther => 'No unit is suggested: fill the other side first.';

  @override
  String get references => 'References';

  @override
  String get compare => 'Compare';

  @override
  String get function => 'Function';

  @override
  String get eachElement => 'Each element';

  @override
  String get allSatisfy => 'all … satisfy';

  @override
  String get anySatisfies => 'any … satisfies';

  @override
  String get mapEach => 'map each …';

  @override
  String get filter => 'filter …';

  @override
  String get range => 'Range';

  @override
  String get betweenTwoEndsIn => 'Between two ends: in … .. …';

  @override
  String get thisPartIsEditedAsText => 'This part is edited as text.';

  @override
  String get composeChoose => 'Choose';

  @override
  String get aChoiceIfThenElse => 'A choice: if … then … else …';

  @override
  String get negateThis => 'Negate this: not …';

  @override
  String get readingTheSources => 'Reading the sources…';

  @override
  String get show => 'Show';

  @override
  String get wholeNumber => 'whole number';

  @override
  String get noDomainsEveryRelationshipIsEvaluatedAt =>
      'No domains: every relationship is evaluated at every tick.';

  @override
  String get activationPeriodInTicksChangingOneStarts =>
      'Activation period, in ticks. Changing one starts the run over.';

  @override
  String get evaluating => 'Evaluating…';

  @override
  String get setTheInputsThenStep => 'Set the inputs, then step.';

  @override
  String get step => 'Step';

  @override
  String get reset => 'Reset';

  @override
  String get theDesignContainsAnInstantaneousCycle => 'The design contains an instantaneous cycle.';

  @override
  String get noTicksEvaluatedYet => 'No ticks evaluated yet.';

  @override
  String get selectAnInputAColumnOrA => 'Select an input, a column or a concept.';

  @override
  String get aRelationshipItIsAppliedInsideOther =>
      'A relationship: it is applied inside other relationships and has no value of its own to sample.';

  @override
  String get appliedBy => 'Applied in';

  @override
  String addAValueThatApplies(String name) {
    return 'Add a value that applies $name';
  }

  @override
  String get noValueYet => 'no value yet';

  @override
  String notPossibleYet(String reason) {
    return 'Not possible yet: $reason';
  }

  @override
  String get now => 'Now';

  @override
  String get notSteppedYet => 'not stepped yet';

  @override
  String get finalTarget => 'Final target';

  @override
  String get noneYet => 'none yet';

  @override
  String get probe => 'Probe';

  @override
  String get overTheRun => 'Over the run';

  @override
  String get notEvaluatedAtAnyTickYet => 'Not evaluated at any tick yet.';

  @override
  String get pwmChannel => 'PWM channel';

  @override
  String get digitalOutput => 'Digital output';

  @override
  String get digitalInput => 'Digital input';

  @override
  String get hBridgeChannel => 'H-bridge channel';

  @override
  String get iCSensor => 'I²C sensor';

  @override
  String get quadratureEncoder => 'Quadrature encoder';

  @override
  String get target => 'Target';

  @override
  String get noBoardsKnown => 'no boards known';

  @override
  String get chooseABoard => 'choose a board…';

  @override
  String get loadingBoards => 'loading boards…';

  @override
  String get chooseABoardToSeeWhetherThis => 'Choose a board to see whether this design fits it.';

  @override
  String get devices => 'Devices';

  @override
  String get addDevice => 'Add device';

  @override
  String get noDeviceYetADeviceRealisesOne =>
      'No device yet. A device realises one output on the board; its kind says what it needs from the board.';

  @override
  String get realizationProfile => 'Realization';

  @override
  String get noRealization => 'None — place by kind';

  @override
  String get doesNotFit => 'does not fit';

  @override
  String get chooseABoardToSeeRealizations =>
      'Choose a board to see which realizations fit this output.';

  @override
  String get encoderWellFormed => 'encoder';

  @override
  String get representationFits => 'fits';

  @override
  String get hardwarePlaced => 'placed';

  @override
  String rawCommand(String ty) {
    return 'raw command $ty';
  }

  @override
  String get noOutput => 'no output';

  @override
  String get notConnected => 'not connected';

  @override
  String sourceItem(String name) {
    return '$name — Source';
  }

  @override
  String get providerProfile => 'Provider';

  @override
  String get noProvider => 'None — place by kind';

  @override
  String get chooseABoardToSeeProviders => 'Choose a board to see which providers fit this Source.';

  @override
  String get transducerWellFormed => 'transducer';

  @override
  String get backendReadable => 'readable';

  @override
  String rawReading(String ty) {
    return 'raw reading $ty';
  }

  @override
  String fromPackage(String origin) {
    return 'from $origin';
  }

  @override
  String placedBeforeTheDeadEnd(Object placements) {
    return 'Placed before the dead end: $placements. This is one conflict under the solver’s order, not necessarily the only one.';
  }

  @override
  String get fixes => 'Fixes';

  @override
  String get rename => 'Rename';

  @override
  String get addToGroup => 'Add to Group';

  @override
  String get addInstance => 'Add Instance';

  @override
  String get newBehaviorGroup => 'New Behavior Group';

  @override
  String get input => 'Input';

  @override
  String get more => 'More…';

  @override
  String get addConcept => 'Add Concept';

  @override
  String get addAConceptFromTheLibraryTo => 'Add a concept from the Library to start';

  @override
  String get valueNotDecided => 'value not decided';

  @override
  String get onOrOff => 'on or off';

  @override
  String get declaredNotYetDefined => 'declared, not yet defined';

  @override
  String get definitionDoesNotCheck => 'definition does not check';

  @override
  String get noTimingDomainYet => 'no timing domain yet';

  @override
  String get drivenByAnIllFormedConnection => 'driven by an ill-formed connection';

  @override
  String get contestedBySeveralDrivers => 'contested by several drivers';

  @override
  String get noDomain => 'no domain';

  @override
  String get luminousIntensity => 'luminous intensity';

  @override
  String get amountOfSubstance => 'amount of substance';

  @override
  String backsThePort(Object port) {
    return 'Backs the port \"$port\": what instances see of it is the promise, kept on the component, not this definition.';
  }

  @override
  String evaluatedAtEachActivationOf(Object domain) {
    return 'Evaluated at each activation of $domain; a value read from another domain needs an explicit transport.';
  }

  @override
  String eachActivationCommitsThisValueTo(Object output) {
    return 'Each activation commits this value to $output. One driver per output: a second one is a conflict, never a priority.';
  }

  @override
  String relationshipsWithInputsNeedADefinition(Object names) {
    return '$names: a relationship with inputs needs a definition before it can run.';
  }

  @override
  String oneOfNInstancesOf(int count, Object component) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          'One of $count instances of $component. What you see here is its promise; its source is shared by all of them.',
      one:
          'One of $count instance of $component. What you see here is its promise; its source is shared by all of them.',
    );
    return '$_temp0';
  }

  @override
  String carriedAcrossTimingDomains(Object init) {
    return 'Carried across timing domains: the destination sees the last value committed strictly before its own activation, starting at $init.';
  }

  @override
  String get aBehaviorInAComponentIsAWayOfSeeing =>
      'A behavior is a way of seeing this component\'s source: grouping, moving, splitting or dissolving it changes nothing about what the component promises or does.';

  @override
  String alreadyTakesItsValueFrom(Object to, Object source) {
    return '$to already takes its value from $source. A required port takes one source; the current connection would be removed first.';
  }

  @override
  String transportExplanation(Object from, Object fromDomain, Object to, Object toDomain) {
    return '$from updates in $fromDomain, $to in $toDomain. The value is carried across: $to sees the last value committed strictly before its own activation, and needs a value to start from.';
  }

  @override
  String unsavedChangesExplanation(Object action) {
    return 'The project has changes that are not saved. Saving keeps everything as it is now — unfinished formulas and text that does not build yet included. If you don’t save and $action, the project returns to what was last saved.';
  }

  @override
  String thisFileDoesNotBuildYet(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          'This file does not build yet ($count problems): the design shows the last version that did.',
      one: 'This file does not build yet: the design shows the last version that did.',
    );
    return '$_temp0';
  }

  @override
  String get ofComponent => 'Of';

  @override
  String get stateDefined => 'defined';

  @override
  String get stateUndriven => 'undriven';

  @override
  String get stateDriven => 'driven';

  @override
  String get stateRequired => 'required';

  @override
  String get stateOptional => 'optional';

  @override
  String get stateContested => 'contested';

  @override
  String get stateIllFormed => 'ill-formed';

  @override
  String get formOnOffShort => 'on–off';

  @override
  String get formCountShort => 'count';

  @override
  String get formCollectionShort => 'collection';

  @override
  String expressionOver(Object inputs) {
    return 'expression over $inputs';
  }

  @override
  String needsAValueForThisStep(Object who) {
    return '$who needs a value for this step.';
  }

  @override
  String dividedByZero(Object who) {
    return '$who divided by zero.';
  }

  @override
  String producedAValueThatIsNotANumber(Object who) {
    return '$who produced a value that is not a number.';
  }

  @override
  String updatesInClockParameter(Object clock) {
    return 'Updates in $clock (a parameter of the component)';
  }

  @override
  String updatesInOwnClock(Object clock) {
    return 'Updates in its own $clock';
  }

  @override
  String get portProvided => 'provided';

  @override
  String get portBound => 'bound';

  @override
  String portBoundTo(Object source) {
    return 'bound to $source';
  }

  @override
  String get portOpen => 'open';

  @override
  String get dimAngle => 'angle';

  @override
  String get dimLength => 'length';

  @override
  String get dimTime => 'time';

  @override
  String get dimMass => 'mass';

  @override
  String get dimTemperature => 'temperature';

  @override
  String get dimCurrent => 'current';

  @override
  String minutesAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count min ago',
      one: '1 min ago',
    );
    return '$_temp0';
  }

  @override
  String hoursAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count h ago',
      one: '1 h ago',
    );
    return '$_temp0';
  }

  @override
  String daysAgo(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count d ago',
      one: '1 d ago',
    );
    return '$_temp0';
  }

  @override
  String get yesterday => 'yesterday';

  @override
  String undoTooltip(Object shortcut) {
    return 'Undo ($shortcut)';
  }

  @override
  String redoTooltip(Object shortcut) {
    return 'Redo ($shortcut)';
  }

  @override
  String notYetDefinedCount(int count) {
    return '$count not yet defined';
  }

  @override
  String definitionsDoNotCheckCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count definitions do not check',
      one: '1 definition does not check',
    );
    return '$_temp0';
  }

  @override
  String definitionsNotAddedCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count definitions not added',
      one: '1 definition not added',
    );
    return '$_temp0';
  }

  @override
  String feasibleOnBoard(Object board) {
    return 'feasible on $board';
  }

  @override
  String notFeasibleOnBoard(Object board) {
    return 'not feasible on $board';
  }

  @override
  String incompleteOnBoard(Object board) {
    return 'incomplete on $board';
  }

  @override
  String compilerVersion(Object version) {
    return 'Compiler $version';
  }

  @override
  String protocolVersion(Object version) {
    return 'protocol $version';
  }

  @override
  String conceptsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count concepts',
      one: '1 concept',
    );
    return '$_temp0';
  }

  @override
  String mappingsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count mappings',
      one: '1 mapping',
    );
    return '$_temp0';
  }

  @override
  String sourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count sources',
      one: '1 source',
    );
    return '$_temp0';
  }

  @override
  String saveChangesTo(Object name) {
    return 'Save changes to “$name”?';
  }

  @override
  String readConcept(Object concept) {
    return 'read $concept';
  }

  @override
  String domainAlreadyExists(Object name) {
    return 'A domain named $name already exists.';
  }

  @override
  String instancesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count instances',
      one: '1 instance',
    );
    return '$_temp0';
  }

  @override
  String componentsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count components',
      one: '1 component',
    );
    return '$_temp0';
  }

  @override
  String editingComponent(Object component) {
    return 'Editing $component';
  }

  @override
  String usedByInstances(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'used by $count instances',
      one: 'used by 1 instance',
    );
    return '$_temp0';
  }

  @override
  String noConceptMatches(Object query) {
    return 'No concept matches “$query”.';
  }

  @override
  String deleteNamed(Object name) {
    return 'Delete $name';
  }

  @override
  String changingThisRechecks(Object names) {
    return 'Changing this re-checks $names.';
  }

  @override
  String stillUsedBy(Object names) {
    return 'Still used by $names.';
  }

  @override
  String inGroup(Object group) {
    return 'In group $group.';
  }

  @override
  String takesItsValueFrom(Object source) {
    return 'Takes its value from $source.';
  }

  @override
  String takesItsValueFromTransported(Object source, Object init) {
    return 'Takes its value from $source, carried across timing domains starting at $init.';
  }

  @override
  String checkedOnceValueDecided(Object names) {
    return 'Checked once $names’s value is decided.';
  }

  @override
  String drivenBy(Object driver) {
    return 'Driven by $driver.';
  }

  @override
  String contestedOutput(Object output, Object claimants) {
    return '$output already has a final target: $claimants both claim it.';
  }

  @override
  String stillDrivenBy(Object names) {
    return 'Still driven by $names';
  }

  @override
  String promiseBrokenOpenSource(Object component) {
    return '$component’s source no longer keeps its promise; open it to see why.';
  }

  @override
  String boundTo(Object source) {
    return 'Bound to $source.';
  }

  @override
  String boundToTransported(Object source, Object init) {
    return 'Bound to $source, carried across timing domains starting at $init.';
  }

  @override
  String usedByInstancesNamed(int count, Object names) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Used by $count instances: $names.',
      one: 'Used by 1 instance: $names.',
    );
    return '$_temp0';
  }

  @override
  String standsFor(Object concept) {
    return 'stands for $concept';
  }

  @override
  String theSystemsOutput(Object output) {
    return 'the system’s $output';
  }

  @override
  String clockParameterSuffix(Object clock) {
    return '$clock (parameter)';
  }

  @override
  String clockPrivateSuffix(Object clock) {
    return '$clock (private)';
  }

  @override
  String relationshipsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count relationships',
      one: '1 relationship',
    );
    return '$_temp0';
  }

  @override
  String selectedCount(int count) {
    return '$count selected';
  }

  @override
  String selectionSummary(Object relationships, int others) {
    String _temp0 = intl.Intl.pluralLogic(
      others,
      locale: localeName,
      other: ', $others others',
      one: ', 1 other',
      zero: '',
    );
    return '$relationships$_temp0.';
  }

  @override
  String groupNAsBehavior(int count) {
    return 'Group $count relationships as Behavior';
  }

  @override
  String alreadyInABehavior(int count) {
    return '$count already in a behavior; move them from there.';
  }

  @override
  String notSaved(Object error) {
    return 'Not saved: $error';
  }

  @override
  String notChecked(Object reason) {
    return 'Not checked: $reason';
  }

  @override
  String get compilerServiceUnavailable => 'the compiler service is unavailable';

  @override
  String producesDescription(Object what) {
    return 'produces $what';
  }

  @override
  String get expressionHint => 'expression';

  @override
  String thisIs(Object what) {
    return 'This is $what.';
  }

  @override
  String equationsCount(int count) {
    return 'Equations ($count)';
  }

  @override
  String insertAfterThis(Object operator) {
    return 'Insert $operator after this';
  }

  @override
  String paramIsEachElementOf(Object param, Object type) {
    return '$param is each element: $type.';
  }

  @override
  String paramIsEachElement(Object param) {
    return '$param is each element of the collection.';
  }

  @override
  String sourceOf(Object file) {
    return 'Source of $file';
  }

  @override
  String notBuiltSuffix(Object file) {
    return '$file — not built';
  }

  @override
  String get checkingTheDesign => 'Checking the design…';

  @override
  String dependOnEachOtherInTheSameInstant(String names) {
    return 'These relationships depend on each other in the same instant: $names. One of them must read the previous value instead.';
  }

  @override
  String hasNoValidDefinition(String name) {
    return '$name has no valid definition.';
  }

  @override
  String hasNoDefinitionReadsSomething(String name) {
    return '$name has no definition. A relationship that reads something needs one before the design can run.';
  }

  @override
  String needsAValueFormBeforeInput(String concept, String input) {
    return '$concept needs a value form (Quantity, On / off or Count) before $input can be given a value.';
  }

  @override
  String needsAValueBeforeSimulationCanStep(String name) {
    return '$name needs a value before simulation can step.';
  }

  @override
  String get valueHint => 'value';

  @override
  String get onWord => 'on';

  @override
  String get offWord => 'off';

  @override
  String get tickColumn => 'tick';

  @override
  String get activeColumn => 'active';

  @override
  String hasNoValueFormYet(Object concept) {
    return '$concept has no value form yet.';
  }

  @override
  String ticksEvaluated(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count ticks evaluated.',
      one: '1 tick evaluated.',
    );
    return '$_temp0';
  }

  @override
  String tickN(int n) {
    return 'tick $n';
  }

  @override
  String resourcesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count resources',
      one: '1 resource',
    );
    return '$_temp0';
  }

  @override
  String couldNotAnalyse(Object error) {
    return 'Could not analyse: $error';
  }

  @override
  String checkingBoard(Object board) {
    return 'Checking $board…';
  }

  @override
  String feasibleOnBoardSentence(Object board) {
    return 'Feasible on $board.';
  }

  @override
  String fitsBoardSoFar(Object board) {
    return 'Fits $board so far — the binding is not finished.';
  }

  @override
  String notFeasibleOnBoardSentence(Object board) {
    return 'Not feasible on $board.';
  }

  @override
  String placementOn(Object board) {
    return 'Placement on $board';
  }

  @override
  String notConnectedToAnOutput(Object devices) {
    return 'Not connected to an output: $devices.';
  }

  @override
  String noDeviceOnBoardFor(Object board, Object outputs) {
    return 'No device on $board for: $outputs.';
  }

  @override
  String requirementN(int n) {
    return 'requirement $n';
  }

  @override
  String couldNotPlaceOf(Object what, Object who) {
    return 'Could not place $what of $who.';
  }

  @override
  String nothingOnBoardCanCarry(Object board, Object what) {
    return 'Nothing on $board can carry $what.';
  }

  @override
  String fixedPinCannotCarry(Object pin, Object what) {
    return 'The pin chosen by hand, $pin, cannot carry $what here.';
  }

  @override
  String groupAsBehaviorCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Group as Behavior ($count relationships)',
      one: 'Group as Behavior (1 relationship)',
    );
    return '$_temp0';
  }

  @override
  String readsSocket(Object name) {
    return 'reads $name';
  }

  @override
  String producesSocket(Object name) {
    return 'produces $name';
  }

  @override
  String get removeFromGroupButton => 'Remove from group';

  @override
  String get decideLaterLower => 'decide later';

  @override
  String get groupedValueTitle => 'Grouped value';

  @override
  String get instanceTitle => 'Instance';

  @override
  String get componentTitle => 'Component';

  @override
  String get notPlacedYetSentence => 'Not placed yet.';

  @override
  String get packageAsReusableComponentTitle => 'Package as Reusable Component';

  @override
  String get textMode => 'Text';

  @override
  String get stepTen => 'Step ×10';

  @override
  String get aRelationshipCapital => 'A relationship';

  @override
  String get delete => 'Delete';

  @override
  String get diagNotConnected => 'The compiler service is not connected.';

  @override
  String get diagTransport => 'The compiler service could not be reached.';

  @override
  String get diagPickerUnavailable =>
      'The system file dialog could not be shown. This happens when Studio is launched from a sandboxed host (an embedded terminal, for example). Launch it from Finder or Terminal, or enter a path below.';

  @override
  String get diagChangedOnDisk => 'The project changed on disk since it was opened.';

  @override
  String get diagPersist => 'The project could not be written to disk.';

  @override
  String get diagStaleRevision =>
      'The edit targeted an older revision of the project; it was not applied.';

  @override
  String get diagNothingToUndo => 'Nothing to undo.';

  @override
  String get diagNothingToRedo => 'Nothing to redo.';

  @override
  String get diagNoProject => 'No project is open.';

  @override
  String get diagAlreadyOpen => 'A project is already open; close it first.';

  @override
  String get diagNotASystem => 'This project is a flat design, not a behavior system.';

  @override
  String get diagSimulationNotStarted => 'Start a simulation first.';

  @override
  String get dialogOpenProject => 'Open Project';

  @override
  String get dialogCreateProject => 'Create Project';

  @override
  String get dialogUntitledProject => 'Untitled Project';

  @override
  String get roleSource => 'Source';

  @override
  String get role => 'Role';

  @override
  String get realizationEnvironment => 'Provided by the environment; no device is bound yet.';

  @override
  String realizationNoDeviceOn(String board) {
    return 'Provided by the environment; no device on $board yet.';
  }

  @override
  String realizationProvidedBy(String device, String board) {
    return 'Provided by $device on $board.';
  }

  @override
  String realizationProvidedByAs(String device, String profile, String board) {
    return 'Provided by $device as $profile on $board.';
  }

  @override
  String get sourceExplanation =>
      'A value that enters the behavior model from the environment, observed once per activation. Nothing is missing: add a definition only to compute it inside the model instead.';

  @override
  String get categorySources => 'Sources';

  @override
  String get newSource => 'New source';

  @override
  String get aSourceAValueTheEnvironmentProvides =>
      'A value the environment provides: it reads nothing and is observed once per activation.';

  @override
  String get addSource => 'Add Source';

  @override
  String get newSourceEllipsis => 'New source…';

  @override
  String sourceNodeSemantics(String name, String concept) {
    return '$name, Source: a value entering the behavior model from the environment, provides $concept';
  }

  @override
  String sourceOfConcept(String concept) {
    return 'Source of $concept';
  }

  @override
  String get chooseWhatItProvidesTheOutputSocket =>
      'Choose what it provides: the one socket on its right.';

  @override
  String get sources => 'Sources';

  @override
  String get noSourcesARelationshipWithNoReads =>
      'No Sources: a relationship with no reads and no definition is one. The environment then provides its value, one per activation.';

  @override
  String get searchLibrary => 'Search the library';

  @override
  String get loadingTheLibrary => 'Loading the library…';

  @override
  String get theLibraryArrivesWithTheCompiler => 'The library arrives with the compiler service.';

  @override
  String noLibraryMatches(Object query) {
    return 'Nothing in the library matches “$query”.';
  }

  @override
  String get openAProjectToInsertFrom => 'Open a project to insert from here.';

  @override
  String get libraryCreates => 'Creates';

  @override
  String libraryValueOf(Object name) {
    return 'value: $name';
  }

  @override
  String librarySourceOf(Object name) {
    return 'source: $name';
  }

  @override
  String libraryTypeOf(Object type) {
    return 'type: $type';
  }

  @override
  String get libraryGroupExternal => 'External';

  @override
  String get roleRule => 'Rule';

  @override
  String get roleValue => 'Value';

  @override
  String get ruleExplanation =>
      'A function from what it reads to what it produces. It has no value of its own: a value\'s formula applies it.';

  @override
  String get valueExplanation =>
      'A value of the design: its formula gives it a value once per activation.';

  @override
  String get dependsOn => 'Depends on';

  @override
  String get namedIn => 'Named in';

  @override
  String get valueNodeSemantics => 'value';

  @override
  String mappingBlockSemantics(String name) {
    return 'mapping block of $name: its definition';
  }

  @override
  String dependsOnList(String names) {
    return 'depends on $names';
  }

  @override
  String sheetSourceShape(String concept) {
    return 'Reads nothing: a Source. The environment provides $concept once per activation; a formula added later makes it a computed value instead.';
  }

  @override
  String sheetRuleShape(String reads, String concept) {
    return 'Reads $reads: a rule, a function to $concept. It has no value of its own — a value\'s formula applies it; dashed until its formula is added.';
  }

  @override
  String get aRuleNoValueOfItsOwn =>
      'A rule: it has no value of its own. A value whose formula applies it is what the simulator samples.';

  @override
  String get noValueAppliesItYet => 'No value applies it yet.';

  @override
  String get format => 'Format';

  @override
  String get formatTooltip => 'Lay the file out the canonical way (⌥⇧F)';

  @override
  String nothingNames(String name) {
    return 'Nothing names $name.';
  }

  @override
  String placesNaming(int count, String name) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count places name $name',
      one: '1 place names $name',
    );
    return '$_temp0';
  }

  @override
  String get conceptLabel => 'Concept';

  @override
  String get existingConcept => 'Existing concept';

  @override
  String get sourceName => 'Source name';

  @override
  String get createSource => 'Create Source';

  @override
  String get noConceptsYetCreateOne => 'No concepts in this design yet — choose New concept.';

  @override
  String get sourceOverExistingConceptCaption =>
      'Only the Source is created; the concept stays as it is. One Undo removes the Source.';

  @override
  String get sourceWithNewConceptCaption =>
      'The concept and its Source are created together, in one step. One Undo removes both.';

  @override
  String get inputForAConcept => 'An input for a concept you choose';

  @override
  String inputForConceptOfKind(String kind) {
    return 'An input for a $kind concept you choose — existing, or new';
  }

  @override
  String get chooseConcept => 'choose a concept';

  @override
  String get fixMenu => 'Fix';

  @override
  String get revealInCode => 'Reveal in Code';

  @override
  String get editDefinition => 'Edit Definition';

  @override
  String deleteObjects(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Delete $count objects',
      one: 'Delete 1 object',
    );
    return '$_temp0';
  }

  @override
  String showDriver(String name) {
    return 'Show Driver: $name';
  }

  @override
  String showEnd(String name) {
    return 'Show $name';
  }

  @override
  String replaceDriver(String current, String candidate) {
    return 'Replace $current with $candidate';
  }

  @override
  String driveWith(String name) {
    return 'Drive with $name';
  }

  @override
  String noDriverForConcept(String output, String concept) {
    return '$output accepts $concept, but no current relationship can drive it.';
  }

  @override
  String showInInspector(String name) {
    return 'Show $name in the Inspector';
  }

  @override
  String get selectAll => 'Select All';

  @override
  String get frameAll => 'Frame All';

  @override
  String get windowSelection => 'Window selection';

  @override
  String get crossingSelection => 'Crossing selection';

  @override
  String get activeObject => 'Active';

  @override
  String get deleteBlockedDetails => 'Select those too, or disconnect them first.';

  @override
  String nothingWasDeleted(String reasons) {
    return 'Nothing was deleted: $reasons.';
  }

  @override
  String outputsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count outputs',
      one: '1 output',
    );
    return '$_temp0';
  }

  @override
  String othersCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count others',
      one: '1 other',
    );
    return '$_temp0';
  }

  @override
  String get firmware => 'Firmware';

  @override
  String get stepDeployment => 'Deployment';

  @override
  String get stepBuild => 'Build';

  @override
  String get stepFlash => 'Flash';

  @override
  String get stepObserve => 'Observe';

  @override
  String buildForBoard(String board) {
    return 'Build for $board';
  }

  @override
  String get buildAgain => 'Build again';

  @override
  String get stopBuild => 'Stop';

  @override
  String get flash => 'Flash';

  @override
  String get flashAgain => 'Flash again';

  @override
  String get lookAgain => 'Look again';

  @override
  String get notReadyToBuild => 'Not ready to build';

  @override
  String get fixOnTheDesignPage => 'Fix it on the Design page';

  @override
  String get building => 'Building…';

  @override
  String get stageChecking => 'Checking the deployment';

  @override
  String get stageGenerating => 'Generating the crate';

  @override
  String get stagePreparing => 'Preparing the toolchain';

  @override
  String get stageCompiling => 'Compiling';

  @override
  String get stagePackaging => 'Writing the image';

  @override
  String get stageCancelled => 'Stopped';

  @override
  String cratesCompiled(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count crates compiled',
      one: '1 crate compiled',
    );
    return '$_temp0';
  }

  @override
  String firmwareBuiltAt(String time) {
    return 'Firmware built at $time';
  }

  @override
  String get firmwareStale => 'The firmware is from an earlier design or deployment.';

  @override
  String get buildDidNotComplete => 'The build did not complete';

  @override
  String imageSize(String bytes) {
    return '$bytes bytes';
  }

  @override
  String get noBoardReachable => 'No board is reachable.';

  @override
  String get chooseTheBoardToFlash => 'Several boards are reachable — choose one.';

  @override
  String get flashing => 'Flashing…';

  @override
  String get flashPreparing => 'Preparing';

  @override
  String get flashWriting => 'Writing the image';

  @override
  String get flashRestarting => 'Restarting the board';

  @override
  String flashedToAt(String device, String time) {
    return 'Flashed to $device at $time';
  }

  @override
  String get flashDidNotComplete => 'The flash did not complete';

  @override
  String tryItOnTheBoard(String sources, String outputs) {
    return 'Try it: act on $sources; $outputs should follow the design.';
  }

  @override
  String tryItOutputsOnly(String outputs) {
    return 'Try it: $outputs should behave as designed.';
  }

  @override
  String get boardRunsEarlierDesign =>
      'The board runs an earlier design — build and flash again to update it.';

  @override
  String get details => 'Details';

  @override
  String get generatedCrate => 'Generated crate';

  @override
  String get buildCommand => 'Command';

  @override
  String get rustTarget => 'Target';

  @override
  String get imageFile => 'Image';

  @override
  String get compilerOutput => 'Compiler output';

  @override
  String get demos => 'Demos';

  @override
  String readyToBuild(String board) {
    return 'Ready to build for $board.';
  }

  @override
  String get libraryValues => 'Values';

  @override
  String get libraryQuantities => 'Quantities';

  @override
  String get sourceRow => 'Source';

  @override
  String get sourceRowHint =>
      'A value the environment provides — for a concept you choose, existing or new.';

  @override
  String get conceptSheetSubtitle =>
      'A value category from the Library, and the name it has in this product.';

  @override
  String get valueCategory => 'Value';

  @override
  String get measuredIn => 'Measured in';

  @override
  String get nameRequired => 'A name is required.';

  @override
  String nameTaken(String name) {
    return '$name is already in use.';
  }

  @override
  String get nameNotIdentifier => 'A name is letters, digits and _, not starting with a digit.';

  @override
  String get createConcept => 'Create Concept';

  @override
  String get conceptSheetCaption => 'Creates one concept, as the Code view will write it.';

  @override
  String get chooseACategory => 'choose a category';

  @override
  String get showFormula => 'Show Formula';

  @override
  String get hideFormula => 'Hide Formula';

  @override
  String get editFormula => 'Edit formula';

  @override
  String get formulaPreviewUnavailable => 'The formula cannot be shown here.';

  @override
  String get formulaPreviewLoading => 'Reading the formula…';

  @override
  String get resultLabel => 'result';

  @override
  String get typeToWrite => 'Type to write, or choose a part';

  @override
  String get typeAnOperatorFirst => 'Type an operator before adding a value here.';

  @override
  String caretBefore(String what) {
    return 'before $what';
  }

  @override
  String caretAfter(String what) {
    return 'after $what';
  }

  @override
  String get caretIn => 'in an empty slot';

  @override
  String caretInside(String what) {
    return 'inside $what';
  }

  @override
  String fractionOf(String numerator, String denominator) {
    return '$numerator over $denominator';
  }

  @override
  String choiceSemantics(String condition, String then, String otherwise) {
    return 'a choice: if $condition, then $then, else $otherwise';
  }

  @override
  String matchSemantics(String value, int count) {
    return 'a match on $value with $count cases';
  }

  @override
  String blockSemantics(int count) {
    return 'a block with $count local bindings';
  }

  @override
  String temporalSemantics(String word) {
    return 'a $word boundary';
  }

  @override
  String get connectionMenu => 'Connection menu';

  @override
  String get nodeMenu => 'Node menu';

  @override
  String get groupMenu => 'Group menu';

  @override
  String get arrangeAutomatically => 'Arrange Automatically';

  @override
  String get undoArrange => 'Undo Arrange';

  @override
  String connectionDrives(String from, String to) {
    return '$from drives $to';
  }

  @override
  String connectionAggregate(String group) {
    return 'An edge of the collapsed group $group: it stands for a member\'s connection.';
  }

  @override
  String get addBlock => 'Add Block';

  @override
  String blockOf(String concept) {
    return 'of $concept';
  }

  @override
  String get newConceptSubmenu => 'New Concept';

  @override
  String appliesList(String names) {
    return 'applies $names';
  }

  @override
  String connectionReadEdge(String to, String from) {
    return '$to reads $from';
  }

  @override
  String get readEdgeIsAName =>
      'A read is a name in the block\'s formula: edit the formula to change it.';

  @override
  String get producedByEnvironment => 'the environment (a Source)';

  @override
  String get appliesRules => 'Applies';

  @override
  String get blocksOfConcept => 'Blocks';

  @override
  String noBlockOfConcept(String name) {
    return 'No block of $name yet: add one from the canvas menu (Add Block ▸ of $name).';
  }

  @override
  String get rulesOverConcept => 'Rules';

  @override
  String get readsLabel => 'Reads';

  @override
  String get readBy => 'Read by';

  @override
  String get libItem_std_value_boolean_name => 'On / off';

  @override
  String get libItem_std_value_boolean_description =>
      'A value that is either true or false: on or off, pressed or not, present or absent.';

  @override
  String get libItem_std_value_boolean_tags =>
      'bool boolean true false flag toggle switch button pressed held touch contact state';

  @override
  String get libItem_std_value_count_name => 'Count';

  @override
  String get libItem_std_value_count_description =>
      'A whole number of things: pulses, presses, items, steps.';

  @override
  String get libItem_std_value_count_tags =>
      'count integer number of pulses presses items steps tally';

  @override
  String get libItem_std_value_level_name => 'Level';

  @override
  String get libItem_std_value_level_description =>
      'A plain number with no unit: a level from 0 to 1, a ratio, a factor, a percentage.';

  @override
  String get libItem_std_value_level_tags =>
      'scalar ratio factor percent % fraction dimensionless brightness volume humidity battery speed setting intensity opening dimmer fan heater power setting';

  @override
  String get libItem_std_value_open_name => 'Decide later';

  @override
  String get libItem_std_value_open_description =>
      'A concept whose value form is not chosen yet; relationships can already use it, and the form is decided once known.';

  @override
  String get libItem_std_value_open_tags =>
      'open unknown later undecided analog external raw host network';

  @override
  String get libItem_std_quantity_angle_name => 'Angle';

  @override
  String get libItem_std_quantity_angle_description =>
      'How far something is turned: a tilt, a heading, a dial, a lid, a shaft.';

  @override
  String get libItem_std_quantity_angle_tags =>
      'angle rotation turn tilt orientation heading dial servo encoder shaft lid rad deg degrees radians °';

  @override
  String get libItem_std_quantity_length_name => 'Length';

  @override
  String get libItem_std_quantity_length_description =>
      'A distance or a position along a line: how far, how long, how high.';

  @override
  String get libItem_std_quantity_length_tags =>
      'length distance position range height width depth travel obstacle proximity m mm cm km inch ft metre meter';

  @override
  String get libItem_std_quantity_time_name => 'Time';

  @override
  String get libItem_std_quantity_time_description =>
      'A duration or an interval: how long something takes or lasts.';

  @override
  String get libItem_std_quantity_time_tags =>
      'time duration interval delay period timeout elapsed s ms min h seconds minutes hours';

  @override
  String get libItem_std_quantity_mass_name => 'Mass';

  @override
  String get libItem_std_quantity_mass_description =>
      'How much matter something has: a load, a weight on a scale.';

  @override
  String get libItem_std_quantity_mass_tags => 'mass weight load scale kg g gram kilogram';

  @override
  String get libItem_std_quantity_current_name => 'Current';

  @override
  String get libItem_std_quantity_current_description =>
      'Electric current: how much charge flows, as a motor or a supply draws.';

  @override
  String get libItem_std_quantity_current_tags => 'current electric amps ampere draw A mA milliamp';

  @override
  String get libItem_std_quantity_temperature_name => 'Temperature';

  @override
  String get libItem_std_quantity_temperature_description =>
      'How warm something is: the air, a room, a motor, a surface.';

  @override
  String get libItem_std_quantity_temperature_tags =>
      'temperature temp thermal heat warm cold hot thermometer K kelvin °C celsius';

  @override
  String get libItem_std_quantity_amount_name => 'Amount of substance';

  @override
  String get libItem_std_quantity_amount_description =>
      'How much of a substance, counted in moles.';

  @override
  String get libItem_std_quantity_amount_tags => 'amount substance mol mole moles chemical';

  @override
  String get libItem_std_quantity_luminous_intensity_name => 'Luminous intensity';

  @override
  String get libItem_std_quantity_luminous_intensity_description =>
      'How bright a light source is in one direction.';

  @override
  String get libItem_std_quantity_luminous_intensity_tags =>
      'luminous intensity candela cd light source lamp';

  @override
  String get libItem_std_quantity_speed_name => 'Speed';

  @override
  String get libItem_std_quantity_speed_description =>
      'How fast something moves along a line: a vehicle, a belt, a wind.';

  @override
  String get libItem_std_quantity_speed_tags =>
      'speed velocity pace rate m/s km/h wheel vehicle conveyor wind';

  @override
  String get libItem_std_quantity_acceleration_name => 'Acceleration';

  @override
  String get libItem_std_quantity_acceleration_description =>
      'How quickly speed changes: a shock, a tilt sensed by gravity, a vehicle pulling away.';

  @override
  String get libItem_std_quantity_acceleration_tags =>
      'acceleration accelerometer g-force shock vibration m/s² imu';

  @override
  String get libItem_std_quantity_angular_velocity_name => 'Angular velocity';

  @override
  String get libItem_std_quantity_angular_velocity_description =>
      'How fast something turns: a shaft, a wheel, a gyroscope\'s reading.';

  @override
  String get libItem_std_quantity_angular_velocity_tags =>
      'angular velocity angular speed rotation speed rpm spin gyro gyroscope shaft wheel rad/s deg/s revolutions motor speed';

  @override
  String get libItem_std_quantity_frequency_name => 'Frequency';

  @override
  String get libItem_std_quantity_frequency_description =>
      'How often something repeats each second: a pitch, a pulse rate, a blink.';

  @override
  String get libItem_std_quantity_frequency_tags =>
      'frequency rate pitch tone hertz Hz kHz pulse blink cycles';

  @override
  String get libItem_std_quantity_force_name => 'Force';

  @override
  String get libItem_std_quantity_force_description =>
      'A push or a pull: a load cell\'s reading, a grip, a spring.';

  @override
  String get libItem_std_quantity_force_tags =>
      'force push pull load cell grip spring thrust N newton';

  @override
  String get libItem_std_quantity_pressure_name => 'Pressure';

  @override
  String get libItem_std_quantity_pressure_description =>
      'Force over an area: the air, a fluid, a touch on a pad.';

  @override
  String get libItem_std_quantity_pressure_tags =>
      'pressure barometer atmospheric altitude weather fluid tyre pad Pa kPa hPa bar psi';

  @override
  String get libItem_std_quantity_torque_name => 'Torque';

  @override
  String get libItem_std_quantity_torque_description =>
      'A turning force: what a motor delivers to a shaft.';

  @override
  String get libItem_std_quantity_torque_tags =>
      'torque moment turning force motor shaft N·m Nm newton metre';

  @override
  String get libItem_std_quantity_power_name => 'Power';

  @override
  String get libItem_std_quantity_power_description =>
      'Energy per second: what a heater, a motor or a lamp draws or delivers.';

  @override
  String get libItem_std_quantity_power_tags => 'power watt W kW heater consumption wattage';

  @override
  String get libItem_std_quantity_voltage_name => 'Voltage';

  @override
  String get libItem_std_quantity_voltage_description =>
      'Electric potential: a battery, a supply rail, an analog reading in volts.';

  @override
  String get libItem_std_quantity_voltage_tags =>
      'voltage volts potential battery supply rail analog adc V mV';

  @override
  String get libItem_std_quantity_illuminance_name => 'Illuminance';

  @override
  String get libItem_std_quantity_illuminance_description =>
      'How much light falls on a surface: ambient light, daylight, a photocell\'s reading.';

  @override
  String get libItem_std_quantity_illuminance_tags =>
      'illuminance light lux lx ambient daylight dark photocell light sensor';
}
