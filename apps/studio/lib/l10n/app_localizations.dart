import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_ja.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale) : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate = _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('ja'),
    Locale('zh'),
    Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans'),
  ];

  /// Window title.
  ///
  /// In en, this message translates to:
  /// **'BDL Studio'**
  String get appTitle;

  /// Title of the application preferences sheet (not project settings).
  ///
  /// In en, this message translates to:
  /// **'Preferences'**
  String get preferencesTitle;

  /// Menu/link/tooltip that opens the preferences sheet; the ellipsis means a sheet follows.
  ///
  /// In en, this message translates to:
  /// **'Preferences…'**
  String get preferencesMenu;

  /// No description provided for @done.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get done;

  /// Form label for the display-language menu.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get languageLabel;

  /// Menu entry: follow the operating system's language.
  ///
  /// In en, this message translates to:
  /// **'System Default'**
  String get languageSystemDefault;

  /// Help line under the language menu; states that the language is presentation only.
  ///
  /// In en, this message translates to:
  /// **'Changes the words Studio uses. BDL source, project files and the compiler are unaffected.'**
  String get languageHelp;

  /// Page name: Monitor (live telemetry; a placeholder today).
  ///
  /// In en, this message translates to:
  /// **'Monitor'**
  String get monitor;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Live values on the canvas arrive with telemetry (roadmap step S).'**
  String get liveValuesOnTheCanvasArriveWith;

  /// Product name: keep as is.
  ///
  /// In en, this message translates to:
  /// **'BDL Studio'**
  String get bdlStudio;

  /// Status line word: the project has unsaved changes.
  ///
  /// In en, this message translates to:
  /// **'Edited'**
  String get edited;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Reload from disk'**
  String get reloadFromDisk;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Overwrite'**
  String get overwrite;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Dismiss'**
  String get dismiss;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'No project'**
  String get noProject;

  /// Status line word: everything is saved.
  ///
  /// In en, this message translates to:
  /// **'Saved'**
  String get saved;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'not causal'**
  String get notCausal;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'reads across domains'**
  String get readsAcrossDomains;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'outputs incomplete'**
  String get outputsIncomplete;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Compiler not connected'**
  String get compilerNotConnected;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Connecting to the compiler'**
  String get connectingToTheCompiler;

  /// Page name and view mode: Design (the canvas).
  ///
  /// In en, this message translates to:
  /// **'Design'**
  String get design;

  /// Page name: Simulate.
  ///
  /// In en, this message translates to:
  /// **'Simulate'**
  String get simulate;

  /// Page name: Deploy.
  ///
  /// In en, this message translates to:
  /// **'Deploy'**
  String get deploy;

  /// Studio UI text (shell.dart).
  ///
  /// In en, this message translates to:
  /// **'Project manager (closes this project)'**
  String get projectManagerClosesThisProject;

  /// Welcome page section: start actions.
  ///
  /// In en, this message translates to:
  /// **'Start'**
  String get start;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'New Project…'**
  String get newProject;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Open Project…'**
  String get openProject;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Open by path…'**
  String get openByPath;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Open project by path'**
  String get openProjectByPath;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'New at path…'**
  String get newAtPath;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Create project at path'**
  String get createProjectAtPath;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Waiting for the compiler. The status line shows the connection.'**
  String get waitingForTheCompilerTheStatusLine;

  /// Welcome page: recently opened projects; canvas menu: recently used library items.
  ///
  /// In en, this message translates to:
  /// **'Recent'**
  String get recent;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Projects you open will appear here.'**
  String get projectsYouOpenWillAppearHere;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'not found'**
  String get notFound;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Remove from Recent'**
  String get removeFromRecent;

  /// Studio UI text (welcome_page.dart).
  ///
  /// In en, this message translates to:
  /// **'just now'**
  String get justNow;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'this project'**
  String get thisProject;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'open another project'**
  String get openAnotherProject;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'create another project'**
  String get createAnotherProject;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'close it'**
  String get closeIt;

  /// Studio UI text (close_guard.dart).
  ///
  /// In en, this message translates to:
  /// **'Don’t Save'**
  String get donTSave;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Folder'**
  String get folder;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Choose'**
  String get choose;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get name;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Create'**
  String get create;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'New concept'**
  String get newConcept;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Something the product senses, decides or shows.'**
  String get somethingTheProductSensesDecidesOrShows;

  /// Concept inspector: the value form (quantity, on/off, count, …).
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get value;

  /// Value form: a measured quantity with a unit.
  ///
  /// In en, this message translates to:
  /// **'Quantity'**
  String get quantity;

  /// Value form: on / off (a Boolean).
  ///
  /// In en, this message translates to:
  /// **'On / off'**
  String get onOff;

  /// Value form: a whole number (a count). Noun.
  ///
  /// In en, this message translates to:
  /// **'Count'**
  String get count;

  /// Value form choice: leave the value form undecided for now (a legal state).
  ///
  /// In en, this message translates to:
  /// **'Decide later'**
  String get decideLater;

  /// Physical measurement unit of a quantity (m, s, deg). Never the type-theoretic unit ().
  ///
  /// In en, this message translates to:
  /// **'Unit'**
  String get unit;

  /// Inspector label for the description (free text explaining what a thing means).
  ///
  /// In en, this message translates to:
  /// **'Meaning'**
  String get meaning;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Hollow ring: the value can be decided later; relationships can already use it.'**
  String get hollowRingTheValueCanBeDecided;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Round socket: a measured quantity. Its unit is checked in every formula.'**
  String get roundSocketAMeasuredQuantityItsUnit;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Diamond socket: on or off — activates contexts, gates behaviour.'**
  String get diamondSocketOnOrOffActivatesContexts;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Square socket: a whole number — occurrences, steps, items.'**
  String get squareSocketAWholeNumberOccurrencesSteps;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'New mapping'**
  String get newMapping;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'A relationship between concepts. It can exist before it is defined.'**
  String get aRelationshipBetweenConceptsItCanExist;

  /// Mapping sheet / inspector label: the concepts this relationship reads (its inputs).
  ///
  /// In en, this message translates to:
  /// **'Reads'**
  String get reads;

  /// Mapping sheet / inspector label: the concept this relationship produces (its output).
  ///
  /// In en, this message translates to:
  /// **'Produces'**
  String get produces;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Choose what it produces: the output socket takes that concept’s colour and shape.'**
  String get chooseWhatItProducesTheOutputSocket;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'New timing domain'**
  String get newTimingDomain;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'When a group of relationships and outputs update together. A name, not a rate: how often it activates is decided when the design runs.'**
  String get whenAGroupOfRelationshipsAndOutputs;

  /// Example timing-domain names shown as a placeholder hint. Identifiers: keep as is in every locale.
  ///
  /// In en, this message translates to:
  /// **'interaction, ambient, …'**
  String get interactionAmbient;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'New output'**
  String get newOutput;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'Where a value leaves the design for the world: a light, a motor, a display.'**
  String get whereAValueLeavesTheDesignFor;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Accepts'**
  String get accepts;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'the concept this output takes'**
  String get theConceptThisOutputTakes;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Updates in'**
  String get updatesIn;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Required'**
  String get required;

  /// Studio UI text (dialogs.dart).
  ///
  /// In en, this message translates to:
  /// **'the design is incomplete until something drives it'**
  String get theDesignIsIncompleteUntilSomethingDrives;

  /// Studio UI text (design_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Showing the last version that built; the text has changes that do not build yet.'**
  String get showingTheLastVersionThatBuiltThe;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'No project open.'**
  String get noProjectOpen;

  /// The system: the top level of a system project; also a sidebar section.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get system;

  /// Studio UI text (design_page.dart).
  ///
  /// In en, this message translates to:
  /// **'not placed yet'**
  String get notPlacedYet;

  /// Studio UI text (design_page.dart).
  ///
  /// In en, this message translates to:
  /// **'— its promise is what instances see; edits here reach every instance.'**
  String get itsPromiseIsWhatInstancesSeeEdits;

  /// View mode: Code (the source text).
  ///
  /// In en, this message translates to:
  /// **'Code'**
  String get code;

  /// View mode: canvas and code side by side.
  ///
  /// In en, this message translates to:
  /// **'Split'**
  String get split;

  /// Sidebar tab: this project’s own concepts, relationships, domains and outputs.
  ///
  /// In en, this message translates to:
  /// **'Project'**
  String get project;

  /// Sidebar tab: the Standard Library (Concept items and Source presets).
  ///
  /// In en, this message translates to:
  /// **'Library'**
  String get library;

  /// Sidebar section: concepts.
  ///
  /// In en, this message translates to:
  /// **'Concepts'**
  String get concepts;

  /// Sidebar section: relationships (Studio’s label is Mappings).
  ///
  /// In en, this message translates to:
  /// **'Mappings'**
  String get mappings;

  /// Sidebar section: timing domains.
  ///
  /// In en, this message translates to:
  /// **'Timing domains'**
  String get timingDomains;

  /// Boundary section: what outside reads from the members.
  ///
  /// In en, this message translates to:
  /// **'Outputs'**
  String get outputs;

  /// Sidebar section (reserved): contexts.
  ///
  /// In en, this message translates to:
  /// **'Contexts'**
  String get contexts;

  /// Sidebar section: components.
  ///
  /// In en, this message translates to:
  /// **'Components'**
  String get components;

  /// Studio UI text (library.dart).
  ///
  /// In en, this message translates to:
  /// **'New component'**
  String get newComponent;

  /// Studio UI text (library.dart).
  ///
  /// In en, this message translates to:
  /// **'A reusable behaviour with a promise (its ports) and a source of its own.'**
  String get aReusableBehaviourWithAPromiseIts;

  /// Example component name shown as a placeholder hint. An identifier: keep as is in every locale.
  ///
  /// In en, this message translates to:
  /// **'AdaptiveLamp'**
  String get adaptivelamp;

  /// Sidebar section: instances.
  ///
  /// In en, this message translates to:
  /// **'Instances'**
  String get instances;

  /// Sidebar section: behavior groups.
  ///
  /// In en, this message translates to:
  /// **'Behaviors'**
  String get behaviors;

  /// Button: add a library item to the project.
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get add;

  /// Studio UI text (library.dart).
  ///
  /// In en, this message translates to:
  /// **'Still in use'**
  String get stillInUse;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Environment'**
  String get environment;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Human interaction'**
  String get humanInteraction;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Geometry & motion'**
  String get geometryMotion;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Mechanical'**
  String get mechanical;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Electrical & system'**
  String get electricalSystem;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Visual & display'**
  String get visualDisplay;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Actuation'**
  String get actuation;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Audio'**
  String get audio;

  /// A quantity with no measurement unit (dimensionless). Not the empty product.
  ///
  /// In en, this message translates to:
  /// **'no unit'**
  String get noUnit;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'grouped value'**
  String get groupedValue;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'optional value'**
  String get optionalValue;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Search concepts'**
  String get searchConcepts;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Loading the concept library…'**
  String get loadingTheConceptLibrary;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'The concept library arrives with the compiler service.'**
  String get theConceptLibraryArrivesWithTheCompiler;

  /// Studio UI text (concept_library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Open a project to add concepts from here.'**
  String get openAProjectToAddConceptsFrom;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Select a concept, a relationship, an output, an instance or a group.'**
  String get selectAConceptARelationshipAnOutput;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Select a concept, a mapping or an output.'**
  String get selectAConceptAMappingOrAn;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Inspector'**
  String get inspector;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Nothing else needs rechecking.'**
  String get nothingElseNeedsRechecking;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'This will be checked again.'**
  String get thisWillBeCheckedAgain;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'This change affects'**
  String get thisChangeAffects;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'— it will be checked again.'**
  String get itWillBeCheckedAgain;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'— they will be checked again.'**
  String get theyWillBeCheckedAgain;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Last change'**
  String get lastChange;

  /// Formal word: a declaration’s interface (signature and commitments).
  ///
  /// In en, this message translates to:
  /// **'Interface'**
  String get interface;

  /// Formal word: the definition that realizes an interface; an invalidation category.
  ///
  /// In en, this message translates to:
  /// **'Realization'**
  String get realization;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Semantic'**
  String get semantic;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Reactive'**
  String get reactive;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Clock'**
  String get clock;

  /// Noun: a physical output (a light, a motor) — the thing a value leaves the design through. Never "output" as a computation result.
  ///
  /// In en, this message translates to:
  /// **'Output'**
  String get output;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Deployment'**
  String get deployment;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Unspecified'**
  String get unspecified;

  /// Concept inspector section: whether values of this concept are ordered (compared as magnitudes) — not a sequence.
  ///
  /// In en, this message translates to:
  /// **'Order'**
  String get order;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'values are magnitudes: <, smallest, largest, clamp, in range'**
  String get valuesAreMagnitudesSmallestLargestClampIn;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'values are compared for equality only'**
  String get valuesAreComparedForEqualityOnly;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Relationships'**
  String get relationships;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Produced by'**
  String get producedBy;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'nothing yet'**
  String get nothingYet;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Used by'**
  String get usedBy;

  /// Button: open the Explain disclosure (why the compiler judged this so).
  ///
  /// In en, this message translates to:
  /// **'Explain'**
  String get explain;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Collection of…'**
  String get collectionOf;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Optional…'**
  String get optional;

  /// Collection representation: the element type (each element).
  ///
  /// In en, this message translates to:
  /// **'Each'**
  String get each;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'When present'**
  String get whenPresent;

  /// Pair representation: the first component of a grouped value.
  ///
  /// In en, this message translates to:
  /// **'First'**
  String get first;

  /// Pair representation: the second component of a grouped value.
  ///
  /// In en, this message translates to:
  /// **'Second'**
  String get second;

  /// Inspector section on a relationship inside a component: its place (domain and group). Noun.
  ///
  /// In en, this message translates to:
  /// **'Place'**
  String get place;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Show Group'**
  String get showGroup;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Relationship'**
  String get relationship;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Show Binding'**
  String get showBinding;

  /// Button: remove the binding / driver connection.
  ///
  /// In en, this message translates to:
  /// **'Disconnect'**
  String get disconnect;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Disconnect it to define the relationship yourself.'**
  String get disconnectItToDefineTheRelationshipYourself;

  /// Inspector section: timing (the timing domain the thing updates in).
  ///
  /// In en, this message translates to:
  /// **'Timing'**
  String get timing;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'any domain'**
  String get anyDomain;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A relationship in no domain is pure: it is evaluated wherever it is read.'**
  String get aRelationshipInNoDomainIsPure;

  /// Inspector section label on a relationship: the physical output this relationship is the final driver of.
  ///
  /// In en, this message translates to:
  /// **'Drives'**
  String get drives;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'This value can commit to a physical output.'**
  String get thisValueCanCommitToAPhysical;

  /// Inspector caption under Drives for a relationship with inputs: no Output pop-up is offered; the caption names what would make a connection possible.
  ///
  /// In en, this message translates to:
  /// **'A rule cannot drive an output — connect the value that applies this rule.'**
  String get onlyARelationshipWithoutInputsCanDrive;

  /// Inspector caption under Drives for a rule, when exactly one value of the design applies it; followed by a Show link to that value.
  ///
  /// In en, this message translates to:
  /// **'{value} applies it.'**
  String appliedByValue(String value);

  /// Inspector caption under Drives for a rule that a text-authored design connects to an output; the finding card below says why it is refused.
  ///
  /// In en, this message translates to:
  /// **'Recorded as driving {output}.'**
  String recordedAsDriving(String output);

  /// Output inspector, in place of the Connect pop-up when no value of the accepted concept exists.
  ///
  /// In en, this message translates to:
  /// **'No value of {concept} in the design yet — a relationship that reads nothing and produces {concept} could drive this output.'**
  String noValueOfConceptYet(String concept);

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'no explicit inputs: the canonical domain is (), the empty product; the kernel encodes () -> B as B'**
  String get noExplicitInputsTheCanonicalDomainIs;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'pending analysis'**
  String get pendingAnalysis;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'No timing domain yet: not part of the design\'s commitment until one is chosen.'**
  String get noTimingDomainYetNotPartOf;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Undriven — the design is incomplete without a driver.'**
  String get undrivenTheDesignIsIncompleteWithoutA;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Undriven.'**
  String get undriven;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'The connection does not fit: see the driver\'s findings below.'**
  String get theConnectionDoesNotFitSeeThe;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Checking…'**
  String get checking;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'no domain yet'**
  String get noDomainYet;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'the design is incomplete until this is driven'**
  String get theDesignIsIncompleteUntilThisIs;

  /// Output inspector label: the relationship that drives this physical output.
  ///
  /// In en, this message translates to:
  /// **'Driver'**
  String get driver;

  /// Output inspector: connect a driver.
  ///
  /// In en, this message translates to:
  /// **'Connect'**
  String get connect;

  /// Output inspector Connect menu hint: choose a value (a relationship without inputs) of the accepted concept.
  ///
  /// In en, this message translates to:
  /// **'a value…'**
  String get aValue;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Any timing domain'**
  String get anyTimingDomain;

  /// Port status: exported as an input of the whole system.
  ///
  /// In en, this message translates to:
  /// **'system input'**
  String get systemInput;

  /// Port status: a parameter port given a constant.
  ///
  /// In en, this message translates to:
  /// **'given a value'**
  String get givenAValue;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Edit Source'**
  String get editSource;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Replace with'**
  String get replaceWith;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'another component…'**
  String get anotherComponent;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'not assigned'**
  String get notAssigned;

  /// Instance inspector section: the instance’s ports.
  ///
  /// In en, this message translates to:
  /// **'Ports'**
  String get ports;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'No ports yet.'**
  String get noPortsYet;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Findings'**
  String get findings;

  /// Button: remove (an instance, a device, a group member).
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get remove;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Disconnect its ports first; the component stays.'**
  String get disconnectItsPortsFirstTheComponentStays;

  /// Inspector title for a port.
  ///
  /// In en, this message translates to:
  /// **'Port'**
  String get port;

  /// Component inspector section: the component’s promise (its ports).
  ///
  /// In en, this message translates to:
  /// **'Promise'**
  String get promise;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Implemented by'**
  String get implementedBy;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'nothing (the promise has no backing)'**
  String get nothingThePromiseHasNoBacking;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Go to Source'**
  String get goToSource;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'a constant, e.g. 0.5'**
  String get aConstantEG05;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A closed constant in the port\'s units, or bind the port instead.'**
  String get aClosedConstantInThePortS;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Connection'**
  String get connection;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Open: nothing supplies it yet. Draw a link from a provided port or a top-level relationship of the same concept. A design with open ports still simulates, with the port as an input.'**
  String get openNothingSuppliesItYetDrawA;

  /// A binding: the connection of an instance’s port to a relationship or to another port.
  ///
  /// In en, this message translates to:
  /// **'Binding'**
  String get binding;

  /// Binding inspector: the source end of the binding.
  ///
  /// In en, this message translates to:
  /// **'From'**
  String get from;

  /// Binding inspector: the destination end of the binding.
  ///
  /// In en, this message translates to:
  /// **'To'**
  String get to;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Direct: the destination reads the value as it is.'**
  String get directTheDestinationReadsTheValueAs;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A binding converts nothing: both ends carry the same concept, by identity.'**
  String get aBindingConvertsNothingBothEndsCarry;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'keeps its promise'**
  String get keepsItsPromise;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'promise broken'**
  String get promiseBroken;

  /// Breadcrumb from a component’s source back to the system level; ‹ is a chevron.
  ///
  /// In en, this message translates to:
  /// **'‹ System'**
  String get backToSystem;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Duplicate as Version'**
  String get duplicateAsVersion;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Place Instance'**
  String get placeInstance;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'No ports yet. Select a relationship of the source and declare it a port.'**
  String get noPortsYetSelectARelationshipOf;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Timing parameters'**
  String get timingParameters;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'a parameter (the system assigns it)'**
  String get aParameterTheSystemAssignsIt;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'private to each instance'**
  String get privateToEachInstance;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Shared concepts'**
  String get sharedConcepts;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'private (fresh per instance)'**
  String get privateFreshPerInstance;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A shared concept is the same identity everywhere; a private one is this component\'s own, fresh in every instance.'**
  String get aSharedConceptIsTheSameIdentity;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Physical outputs'**
  String get physicalOutputs;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'private (one per instance)'**
  String get privateOnePerInstance;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Declare a port'**
  String get declareAPort;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'expose as…'**
  String get exposeAs;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'The promise is taken from the relationship as it is now, and kept until you change it here.'**
  String get thePromiseIsTakenFromTheRelationship;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Delete its instances first.'**
  String get deleteItsInstancesFirst;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'a relationship of the source'**
  String get aRelationshipOfTheSource;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Retire port'**
  String get retirePort;

  /// Inspector title for a behavior group.
  ///
  /// In en, this message translates to:
  /// **'Group'**
  String get group;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A behavior is a way of seeing the design: grouping, moving, splitting or dissolving it changes nothing about what the design means.'**
  String get aBehaviorIsAWayOfSeeing;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'+ Add relationship'**
  String get addRelationship;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Empty. Drag relationships in, or add one.'**
  String get emptyDragRelationshipsInOrAddOne;

  /// Canvas context menu.
  ///
  /// In en, this message translates to:
  /// **'Remove from {group}'**
  String removeFromGroup(Object group);

  /// Group inspector section: the boundary of the group (what crosses it).
  ///
  /// In en, this message translates to:
  /// **'Boundary'**
  String get boundary;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Computed once the analysis arrives.'**
  String get computedOnceTheAnalysisArrives;

  /// Simulate section: the inputs (relationships without a definition that take values); boundary section: what members read from outside.
  ///
  /// In en, this message translates to:
  /// **'Inputs'**
  String get inputs;

  /// Boundary section: the open (unbound) members — a legal state, not an error.
  ///
  /// In en, this message translates to:
  /// **'Open'**
  String get open;

  /// Boundary section: edges among members only.
  ///
  /// In en, this message translates to:
  /// **'Internal'**
  String get internal;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Read off the dependency graph: what the members read from outside, what outside reads from them. A picture of the cut, not a connection of its own.'**
  String get readOffTheDependencyGraphWhatThe;

  /// Verb/button: package the behavior as a reusable component.
  ///
  /// In en, this message translates to:
  /// **'Package'**
  String get package;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Package as Reusable Component…'**
  String get packageAsReusableComponent;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Turns the behavior into a component and one instance in its place; the design computes the same values.'**
  String get turnsTheBehaviorIntoAComponentAnd;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'A behavior inside a component stays a way of seeing its source. Packaging it as a component of its own comes with nested components; until then, edit and move it freely.'**
  String get aBehaviorInsideAComponentStaysA;

  /// Group inspector section: canvas presentation.
  ///
  /// In en, this message translates to:
  /// **'Canvas'**
  String get canvas;

  /// Group: expand the collapsed box.
  ///
  /// In en, this message translates to:
  /// **'Expand'**
  String get expand;

  /// Group: collapse the box.
  ///
  /// In en, this message translates to:
  /// **'Collapse'**
  String get collapse;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Ungroup'**
  String get ungroup;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Delete group and relationships'**
  String get deleteGroupAndRelationships;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Ungroup keeps every relationship where it is.'**
  String get ungroupKeepsEveryRelationshipWhereItIs;

  /// A behavior group: a named set of relationships drawn together on the canvas; authoring metadata, not semantics.
  ///
  /// In en, this message translates to:
  /// **'Behavior'**
  String get behavior;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'These relationships read each other.'**
  String get theseRelationshipsReadEachOther;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'These relationships update in one timing domain.'**
  String get theseRelationshipsUpdateInOneTimingDomain;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Group as Behavior'**
  String get groupAsBehavior;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Replace the connection?'**
  String get replaceTheConnection;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Carry across timing domains'**
  String get carryAcrossTimingDomains;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Starts at'**
  String get startsAt;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'a constant in the destination\'s units, e.g. 0'**
  String get aConstantInTheDestinationSUnits;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Disconnect and Connect'**
  String get disconnectAndConnect;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Working out the boundary…'**
  String get workingOutTheBoundary;

  /// Packaging sheet: what the new component requires (its required ports).
  ///
  /// In en, this message translates to:
  /// **'Requires'**
  String get requires;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'what the members read from outside'**
  String get whatTheMembersReadFromOutside;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'nothing — the component is self-contained'**
  String get nothingTheComponentIsSelfContained;

  /// Packaging sheet: what the new component provides (its provided ports).
  ///
  /// In en, this message translates to:
  /// **'Provides'**
  String get provides;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'what outside reads from the members'**
  String get whatOutsideReadsFromTheMembers;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'nothing yet — nothing outside reads the group'**
  String get nothingYetNothingOutsideReadsTheGroup;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Open relationships'**
  String get openRelationships;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'declared inside, not yet defined'**
  String get declaredInsideNotYetDefined;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Treat as input'**
  String get treatAsInput;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Keep internal'**
  String get keepInternal;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'driven by members; the drive stays with them'**
  String get drivenByMembersTheDriveStaysWith;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Stays the system\'s'**
  String get staysTheSystemS;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Moves inside'**
  String get movesInside;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'every domain the members use'**
  String get everyDomainTheMembersUse;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'none — pure'**
  String get nonePure;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'read by members only'**
  String get readByMembersOnly;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'The design computes the same values afterwards; the group becomes one instance in its place, and the component can be placed again.'**
  String get theDesignComputesTheSameValuesAfterwards;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'expression with no inputs'**
  String get expressionWithNoInputs;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'No definition yet. A legal state: other relationships may already depend on the signature.'**
  String get noDefinitionYetALegalStateOther;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Nothing to add yet.'**
  String get nothingToAddYet;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Empty. Detach to remove the definition.'**
  String get emptyDetachToRemoveTheDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Saving…'**
  String get saving;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Add definition'**
  String get addDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Save definition'**
  String get saveDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Invalid definition.'**
  String get invalidDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Cannot be read.'**
  String get cannotBeRead;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Open: something it needs is not decided yet.'**
  String get openSomethingItNeedsIsNotDecided;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Valid definition'**
  String get validDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'No definition.'**
  String get noDefinition;

  /// Definition editor mode switch: the Formula composer (structured editing).
  ///
  /// In en, this message translates to:
  /// **'Formula'**
  String get formula;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Revert'**
  String get revert;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Adding a definition is a refinement: nothing established elsewhere is reopened.'**
  String get addingADefinitionIsARefinementNothing;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Detach definition'**
  String get detachDefinition;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Replacing or detaching is an edit: this definition and the simulation are re-checked.'**
  String get replacingOrDetachingIsAnEditThis;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'This relationship\'s definition changed while you were editing it.'**
  String get thisRelationshipSDefinitionChangedWhileYou;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Reload'**
  String get reload;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Show the definition committed meanwhile; drop what you typed'**
  String get showTheDefinitionCommittedMeanwhileDropWhat;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Keep mine'**
  String get keepMine;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Keep what you typed; save will replace the committed definition'**
  String get keepWhatYouTypedSaveWillReplace;

  /// Studio UI text (definition_editor.dart).
  ///
  /// In en, this message translates to:
  /// **'Looking…'**
  String get looking;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Waiting for the compiler to read the formula…'**
  String get waitingForTheCompilerToReadThe;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'The text cannot be read as a formula.'**
  String get theTextCannotBeReadAsA;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Edit as text'**
  String get editAsText;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Asking what fits here…'**
  String get askingWhatFitsHere;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Hide detail'**
  String get hideDetail;

  /// Composer button: insert the chosen equation or reference.
  ///
  /// In en, this message translates to:
  /// **'Insert'**
  String get insert;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'No unit is suggested: fill the other side first.'**
  String get noUnitIsSuggestedFillTheOther;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'References'**
  String get references;

  /// Composer operator group: comparisons.
  ///
  /// In en, this message translates to:
  /// **'Compare'**
  String get compare;

  /// Composer operator group: apply a relationship.
  ///
  /// In en, this message translates to:
  /// **'Function'**
  String get function;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Each element'**
  String get eachElement;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'all … satisfy'**
  String get allSatisfy;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'any … satisfies'**
  String get anySatisfies;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'map each …'**
  String get mapEach;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'filter …'**
  String get filter;

  /// Composer: a closed range in … .. ….
  ///
  /// In en, this message translates to:
  /// **'Range'**
  String get range;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'Between two ends: in … .. …'**
  String get betweenTwoEndsIn;

  /// Studio UI text (formula_composer.dart).
  ///
  /// In en, this message translates to:
  /// **'This part is edited as text.'**
  String get thisPartIsEditedAsText;

  /// Composer: wrap the component in a choice (if … then … else …), or open one in a slot.
  ///
  /// In en, this message translates to:
  /// **'Choose'**
  String get composeChoose;

  /// Composer: the Choose button's tooltip.
  ///
  /// In en, this message translates to:
  /// **'A choice: if … then … else …'**
  String get aChoiceIfThenElse;

  /// Composer: the not button's tooltip.
  ///
  /// In en, this message translates to:
  /// **'Negate this: not …'**
  String get negateThis;

  /// Studio UI text (code_pane.dart).
  ///
  /// In en, this message translates to:
  /// **'Reading the sources…'**
  String get readingTheSources;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Show'**
  String get show;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'whole number'**
  String get wholeNumber;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'No domains: every relationship is evaluated at every tick.'**
  String get noDomainsEveryRelationshipIsEvaluatedAt;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Activation period, in ticks. Changing one starts the run over.'**
  String get activationPeriodInTicksChangingOneStarts;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Evaluating…'**
  String get evaluating;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Set the inputs, then step.'**
  String get setTheInputsThenStep;

  /// Simulate button: evaluate one tick.
  ///
  /// In en, this message translates to:
  /// **'Step'**
  String get step;

  /// Simulate button: start the run over.
  ///
  /// In en, this message translates to:
  /// **'Reset'**
  String get reset;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'The design contains an instantaneous cycle.'**
  String get theDesignContainsAnInstantaneousCycle;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'No ticks evaluated yet.'**
  String get noTicksEvaluatedYet;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Select an input, a column or a concept.'**
  String get selectAnInputAColumnOrA;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'A relationship: it is applied inside other relationships and has no value of its own to sample.'**
  String get aRelationshipItIsAppliedInsideOther;

  /// Simulate probe, a rule selected: label of the row linking to the values whose formula applies it.
  ///
  /// In en, this message translates to:
  /// **'Applied in'**
  String get appliedBy;

  /// The Fix for a rule nothing applies (bdl-ide action rule.apply), on the Simulate page's readiness area and in the probe; name is the rule's name.
  ///
  /// In en, this message translates to:
  /// **'Add a value that applies {name}'**
  String addAValueThatApplies(String name);

  /// Simulate inputs: a Source not yet given a value — beside the dashed on/off control and as the hint of an empty number field. Not off, not zero: a value is still to be given.
  ///
  /// In en, this message translates to:
  /// **'no value yet'**
  String get noValueYet;

  /// Canvas node header word: a defined rule that no value applies (its output socket is hollow).
  ///
  /// In en, this message translates to:
  /// **'not applied'**
  String get stateNotApplied;

  /// Canvas node accessibility label fragment: a rule no value applies.
  ///
  /// In en, this message translates to:
  /// **'applied by nothing'**
  String get appliedByNothing;

  /// A blocked fix: the service's reason follows.
  ///
  /// In en, this message translates to:
  /// **'Not possible yet: {reason}'**
  String notPossibleYet(String reason);

  /// Simulate probe: the current value.
  ///
  /// In en, this message translates to:
  /// **'Now'**
  String get now;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'not stepped yet'**
  String get notSteppedYet;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Final target'**
  String get finalTarget;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'none yet'**
  String get noneYet;

  /// Simulate: the probe panel (inspect one value over the run).
  ///
  /// In en, this message translates to:
  /// **'Probe'**
  String get probe;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Over the run'**
  String get overTheRun;

  /// Studio UI text (simulate_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Not evaluated at any tick yet.'**
  String get notEvaluatedAtAnyTickYet;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'PWM channel'**
  String get pwmChannel;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Digital output'**
  String get digitalOutput;

  /// Studio UI text (deploy_page.dart): the device kind of one digital input line.
  ///
  /// In en, this message translates to:
  /// **'Digital input'**
  String get digitalInput;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'H-bridge channel'**
  String get hBridgeChannel;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'I²C sensor'**
  String get iCSensor;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Quadrature encoder'**
  String get quadratureEncoder;

  /// Deploy page: the target board.
  ///
  /// In en, this message translates to:
  /// **'Target'**
  String get target;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'no boards known'**
  String get noBoardsKnown;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'choose a board…'**
  String get chooseABoard;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'loading boards…'**
  String get loadingBoards;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Choose a board to see whether this design fits it.'**
  String get chooseABoardToSeeWhetherThis;

  /// Deploy section: the devices on the board.
  ///
  /// In en, this message translates to:
  /// **'Devices'**
  String get devices;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'Add device'**
  String get addDevice;

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'No device yet. A device realises one output on the board; its kind says what it needs from the board.'**
  String get noDeviceYetADeviceRealisesOne;

  /// Deploy page: label of the realization profile dropdown on a device card.
  ///
  /// In en, this message translates to:
  /// **'Realization'**
  String get realizationProfile;

  /// Deploy page: the realization dropdown item that leaves the device without a profile.
  ///
  /// In en, this message translates to:
  /// **'None — place by kind'**
  String get noRealization;

  /// Deploy page: suffix on a realization profile whose encoder does not fit the output's concept.
  ///
  /// In en, this message translates to:
  /// **'does not fit'**
  String get doesNotFit;

  /// Deploy page: shown on a device card while no deployment analysis is available.
  ///
  /// In en, this message translates to:
  /// **'Choose a board to see which realizations fit this output.'**
  String get chooseABoardToSeeRealizations;

  /// Deploy page: the first of the three realization judgments (the encoder is a typed pure function).
  ///
  /// In en, this message translates to:
  /// **'encoder'**
  String get encoderWellFormed;

  /// Deploy page: the second realization judgment (the encoder's representation is the output's).
  ///
  /// In en, this message translates to:
  /// **'fits'**
  String get representationFits;

  /// Deploy page: the third realization judgment (every requirement is placed on the board).
  ///
  /// In en, this message translates to:
  /// **'placed'**
  String get hardwarePlaced;

  /// Deploy page: the raw command type of the chosen realization profile.
  ///
  /// In en, this message translates to:
  /// **'raw command {ty}'**
  String rawCommand(String ty);

  /// Studio UI text (deploy_page.dart).
  ///
  /// In en, this message translates to:
  /// **'no output'**
  String get noOutput;

  /// Deploy page: the device-target dropdown item for a device bound to nothing.
  ///
  /// In en, this message translates to:
  /// **'not connected'**
  String get notConnected;

  /// Deploy page: a Source in the device-target dropdown, marked as one.
  ///
  /// In en, this message translates to:
  /// **'{name} — Source'**
  String sourceItem(String name);

  /// Deploy page: label of the provider profile dropdown on a device bound to a Source.
  ///
  /// In en, this message translates to:
  /// **'Provider'**
  String get providerProfile;

  /// Deploy page: the provider dropdown item that leaves the device without a profile.
  ///
  /// In en, this message translates to:
  /// **'None — place by kind'**
  String get noProvider;

  /// Deploy page: shown on a device card bound to a Source while no deployment analysis is available.
  ///
  /// In en, this message translates to:
  /// **'Choose a board to see which providers fit this Source.'**
  String get chooseABoardToSeeProviders;

  /// Deploy page: the provision judgment that the transducer is a typed pure function.
  ///
  /// In en, this message translates to:
  /// **'transducer'**
  String get transducerWellFormed;

  /// Deploy page: the provision judgment that the chosen board's firmware can read the profile.
  ///
  /// In en, this message translates to:
  /// **'readable'**
  String get backendReadable;

  /// Deploy page: the raw reading type of the chosen provider profile.
  ///
  /// In en, this message translates to:
  /// **'raw reading {ty}'**
  String rawReading(String ty);

  /// Deploy page: where a provider profile comes from when it is not builtin.
  ///
  /// In en, this message translates to:
  /// **'from {origin}'**
  String fromPackage(String origin);

  /// Deploy dead-end explanation; placements is a comma-joined list of "device → resource".
  ///
  /// In en, this message translates to:
  /// **'Placed before the dead end: {placements}. This is one conflict under the solver’s order, not necessarily the only one.'**
  String placedBeforeTheDeadEnd(Object placements);

  /// Inspector section: the fixes the compiler offers for a finding.
  ///
  /// In en, this message translates to:
  /// **'Fixes'**
  String get fixes;

  /// Context menu: rename.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get rename;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Add to Group'**
  String get addToGroup;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Add Instance'**
  String get addInstance;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'New Behavior Group'**
  String get newBehaviorGroup;

  /// Canvas add-menu group: concepts that are inputs (values entering from the environment).
  ///
  /// In en, this message translates to:
  /// **'Input'**
  String get input;

  /// Menu: more…
  ///
  /// In en, this message translates to:
  /// **'More…'**
  String get more;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Add Concept'**
  String get addConcept;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'Add a concept from the Library to start'**
  String get addAConceptFromTheLibraryTo;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'value not decided'**
  String get valueNotDecided;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'a quantity'**
  String get aQuantity;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'on or off'**
  String get onOrOff;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'a count'**
  String get aCount;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'a collection'**
  String get aCollection;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'a grouped value'**
  String get aGroupedValue;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'an optional value'**
  String get anOptionalValue;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'declared, not yet defined'**
  String get declaredNotYetDefined;

  /// Studio UI text (concept_glyphs.dart).
  ///
  /// In en, this message translates to:
  /// **'definition does not check'**
  String get definitionDoesNotCheck;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'no timing domain yet'**
  String get noTimingDomainYet;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'driven by an ill-formed connection'**
  String get drivenByAnIllFormedConnection;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'contested by several drivers'**
  String get contestedBySeveralDrivers;

  /// Studio UI text (node_canvas.dart).
  ///
  /// In en, this message translates to:
  /// **'no domain'**
  String get noDomain;

  /// Studio UI text (units.dart).
  ///
  /// In en, this message translates to:
  /// **'luminous intensity'**
  String get luminousIntensity;

  /// Studio UI text (units.dart).
  ///
  /// In en, this message translates to:
  /// **'amount of substance'**
  String get amountOfSubstance;

  /// Inspector note on a relationship that implements a component port.
  ///
  /// In en, this message translates to:
  /// **'Backs the port \"{port}\": what instances see of it is the promise, kept on the component, not this definition.'**
  String backsThePort(Object port);

  /// Inspector timing note for a relationship in a timing domain.
  ///
  /// In en, this message translates to:
  /// **'Evaluated at each activation of {domain}; a value read from another domain needs an explicit transport.'**
  String evaluatedAtEachActivationOf(Object domain);

  /// Inspector note on a relationship that drives a physical output.
  ///
  /// In en, this message translates to:
  /// **'Each activation commits this value to {output}. One driver per output: a second one is a conflict, never a priority.'**
  String eachActivationCommitsThisValueTo(Object output);

  /// Simulate readiness note; names is a comma-joined list of relationship names.
  ///
  /// In en, this message translates to:
  /// **'{names}: a relationship with inputs needs a definition before it can run.'**
  String relationshipsWithInputsNeedADefinition(Object names);

  /// Instance inspector: this instance is one of count instances of the component.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{One of {count} instance of {component}. What you see here is its promise; its source is shared by all of them.} other{One of {count} instances of {component}. What you see here is its promise; its source is shared by all of them.}}'**
  String oneOfNInstancesOf(int count, Object component);

  /// Binding inspector: a transported binding; init is the starting value.
  ///
  /// In en, this message translates to:
  /// **'Carried across timing domains: the destination sees the last value committed strictly before its own activation, starting at {init}.'**
  String carriedAcrossTimingDomains(Object init);

  /// Group inspector note for a group inside a component.
  ///
  /// In en, this message translates to:
  /// **'A behavior is a way of seeing this component\'s source: grouping, moving, splitting or dissolving it changes nothing about what the component promises or does.'**
  String get aBehaviorInAComponentIsAWayOfSeeing;

  /// Connect sheet: replacing an existing binding.
  ///
  /// In en, this message translates to:
  /// **'{to} already takes its value from {source}. A required port takes one source; the current connection would be removed first.'**
  String alreadyTakesItsValueFrom(Object to, Object source);

  /// Connect sheet: why a binding across timing domains needs a starting value.
  ///
  /// In en, this message translates to:
  /// **'{from} updates in {fromDomain}, {to} in {toDomain}. The value is carried across: {to} sees the last value committed strictly before its own activation, and needs a value to start from.'**
  String transportExplanation(Object from, Object fromDomain, Object to, Object toDomain);

  /// Save-changes sheet body; action is one of the closeAction* phrases (e.g. "close it").
  ///
  /// In en, this message translates to:
  /// **'The project has changes that are not saved. Saving keeps everything as it is now — unfinished formulas and text that does not build yet included. If you don’t save and {action}, the project returns to what was last saved.'**
  String unsavedChangesExplanation(Object action);

  /// Code view banner; count is the number of problems.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{This file does not build yet: the design shows the last version that did.} other{This file does not build yet ({count} problems): the design shows the last version that did.}}'**
  String thisFileDoesNotBuildYet(int count);

  /// Instance inspector label: which component the instance is of.
  ///
  /// In en, this message translates to:
  /// **'Of'**
  String get ofComponent;

  /// Relationship state word (accessibility): has a definition.
  ///
  /// In en, this message translates to:
  /// **'defined'**
  String get stateDefined;

  /// Physical output state: nothing drives it yet.
  ///
  /// In en, this message translates to:
  /// **'undriven'**
  String get stateUndriven;

  /// Physical output state: a relationship drives it.
  ///
  /// In en, this message translates to:
  /// **'driven'**
  String get stateDriven;

  /// Physical output header word: the design is incomplete until it is driven.
  ///
  /// In en, this message translates to:
  /// **'required'**
  String get stateRequired;

  /// Physical output state: may stay undriven.
  ///
  /// In en, this message translates to:
  /// **'optional'**
  String get stateOptional;

  /// Relationship header word on the canvas: declared, not yet defined.
  ///
  /// In en, this message translates to:
  /// **'declared'**
  String get stateDeclared;

  /// Physical output header word: several relationships claim to drive it.
  ///
  /// In en, this message translates to:
  /// **'contested'**
  String get stateContested;

  /// Physical output header word: driven by a connection that does not fit.
  ///
  /// In en, this message translates to:
  /// **'ill-formed'**
  String get stateIllFormed;

  /// Short value-form word in the concept library row: a Boolean value.
  ///
  /// In en, this message translates to:
  /// **'on–off'**
  String get formOnOffShort;

  /// Short value-form word: a whole number.
  ///
  /// In en, this message translates to:
  /// **'count'**
  String get formCountShort;

  /// Short value-form word: a list of values.
  ///
  /// In en, this message translates to:
  /// **'collection'**
  String get formCollectionShort;

  /// Definition editor placeholder hint: the inputs the formula may read, comma-joined.
  ///
  /// In en, this message translates to:
  /// **'expression over {inputs}'**
  String expressionOver(Object inputs);

  /// Simulation stop reason: an input relationship has no value at this tick.
  ///
  /// In en, this message translates to:
  /// **'{who} needs a value for this step.'**
  String needsAValueForThisStep(Object who);

  /// Simulation stop reason.
  ///
  /// In en, this message translates to:
  /// **'{who} divided by zero.'**
  String dividedByZero(Object who);

  /// Simulation stop reason (NaN or infinity).
  ///
  /// In en, this message translates to:
  /// **'{who} produced a value that is not a number.'**
  String producedAValueThatIsNotANumber(Object who);

  /// Port timing: the port updates in a clock parameter the instance assigns.
  ///
  /// In en, this message translates to:
  /// **'Updates in {clock} (a parameter of the component)'**
  String updatesInClockParameter(Object clock);

  /// Port timing: the port updates in a domain private to the component.
  ///
  /// In en, this message translates to:
  /// **'Updates in its own {clock}'**
  String updatesInOwnClock(Object clock);

  /// Port status word: the port is provided by the component.
  ///
  /// In en, this message translates to:
  /// **'provided'**
  String get portProvided;

  /// Port status word: the port is bound (source unknown).
  ///
  /// In en, this message translates to:
  /// **'bound'**
  String get portBound;

  /// Port status word: bound to a relationship or another port.
  ///
  /// In en, this message translates to:
  /// **'bound to {source}'**
  String portBoundTo(Object source);

  /// Port status word: nothing supplies the port yet — a legal state, not an error.
  ///
  /// In en, this message translates to:
  /// **'open'**
  String get portOpen;

  /// Quantity kind (physical dimension) shown in the unit picker: angle.
  ///
  /// In en, this message translates to:
  /// **'angle'**
  String get dimAngle;

  /// Quantity kind (physical dimension) shown in the unit picker: length.
  ///
  /// In en, this message translates to:
  /// **'length'**
  String get dimLength;

  /// Quantity kind (physical dimension) shown in the unit picker: time.
  ///
  /// In en, this message translates to:
  /// **'time'**
  String get dimTime;

  /// Quantity kind (physical dimension) shown in the unit picker: mass.
  ///
  /// In en, this message translates to:
  /// **'mass'**
  String get dimMass;

  /// Quantity kind (physical dimension) shown in the unit picker: temperature.
  ///
  /// In en, this message translates to:
  /// **'temperature'**
  String get dimTemperature;

  /// Quantity kind (physical dimension) shown in the unit picker: current.
  ///
  /// In en, this message translates to:
  /// **'current'**
  String get dimCurrent;

  /// Recent-projects row: how long ago it was opened.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 min ago} other{{count} min ago}}'**
  String minutesAgo(int count);

  /// Recent-projects row.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 h ago} other{{count} h ago}}'**
  String hoursAgo(int count);

  /// Recent-projects row.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 d ago} other{{count} d ago}}'**
  String daysAgo(int count);

  /// Recent-projects row.
  ///
  /// In en, this message translates to:
  /// **'yesterday'**
  String get yesterday;

  /// Toolbar tooltip; shortcut is the platform key combination, e.g. ⌘Z.
  ///
  /// In en, this message translates to:
  /// **'Undo ({shortcut})'**
  String undoTooltip(Object shortcut);

  /// Toolbar tooltip.
  ///
  /// In en, this message translates to:
  /// **'Redo ({shortcut})'**
  String redoTooltip(Object shortcut);

  /// Status line: number of relationships declared without a definition — reading inputs or backing a port, no formula yet (not an error). Sources are not counted.
  ///
  /// In en, this message translates to:
  /// **'{count} not yet defined'**
  String notYetDefinedCount(int count);

  /// Status line: definitions with errors.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 definition does not check} other{{count} definitions do not check}}'**
  String definitionsDoNotCheckCount(int count);

  /// Status line: definition drafts typed but not yet added to the project.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 definition not added} other{{count} definitions not added}}'**
  String definitionsNotAddedCount(int count);

  /// Status line: deployment verdict for the chosen board.
  ///
  /// In en, this message translates to:
  /// **'feasible on {board}'**
  String feasibleOnBoard(Object board);

  /// Status line: deployment verdict.
  ///
  /// In en, this message translates to:
  /// **'not feasible on {board}'**
  String notFeasibleOnBoard(Object board);

  /// Status line: deployment verdict — the design is not finished enough to place.
  ///
  /// In en, this message translates to:
  /// **'incomplete on {board}'**
  String incompleteOnBoard(Object board);

  /// Status line: the connected compiler service and its version.
  ///
  /// In en, this message translates to:
  /// **'Compiler {version}'**
  String compilerVersion(Object version);

  /// Status line: the protocol version (technical; the word "protocol" may stay English).
  ///
  /// In en, this message translates to:
  /// **'protocol {version}'**
  String protocolVersion(Object version);

  /// Status line fact.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 concept} other{{count} concepts}}'**
  String conceptsCount(int count);

  /// Status line fact; Studio calls a relationship a mapping in this row.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 mapping} other{{count} mappings}}'**
  String mappingsCount(int count);

  /// Status line fact: relationships the environment provides (glossary: source), a plain count, never an open item.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 source} other{{count} sources}}'**
  String sourcesCount(int count);

  /// Title of the save-changes sheet; name is the project name.
  ///
  /// In en, this message translates to:
  /// **'Save changes to “{name}”?'**
  String saveChangesTo(Object name);

  /// Chip in the new-mapping sheet: the mapping reads this concept.
  ///
  /// In en, this message translates to:
  /// **'read {concept}'**
  String readConcept(Object concept);

  /// Validation note in the new-timing-domain sheet.
  ///
  /// In en, this message translates to:
  /// **'A domain named {name} already exists.'**
  String domainAlreadyExists(Object name);

  /// Design page system header.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 instance} other{{count} instances}}'**
  String instancesCount(int count);

  /// Design page system header.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 component} other{{count} components}}'**
  String componentsCount(int count);

  /// Design page header while a component’s source is open.
  ///
  /// In en, this message translates to:
  /// **'Editing {component}'**
  String editingComponent(Object component);

  /// Design page: how many instances of the component being edited exist.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{used by 1 instance} other{used by {count} instances}}'**
  String usedByInstances(int count);

  /// Library search empty state.
  ///
  /// In en, this message translates to:
  /// **'No concept matches “{query}”.'**
  String noConceptMatches(Object query);

  /// Tooltip/menu: delete the named thing (a concept, a relationship, an output, a domain, an instance, a component).
  ///
  /// In en, this message translates to:
  /// **'Delete {name}'**
  String deleteNamed(Object name);

  /// Concept inspector: relationships re-checked by a value-form change; names comma-joined.
  ///
  /// In en, this message translates to:
  /// **'Changing this re-checks {names}.'**
  String changingThisRechecks(Object names);

  /// Why a concept cannot be deleted yet.
  ///
  /// In en, this message translates to:
  /// **'Still used by {names}.'**
  String stillUsedBy(Object names);

  /// Relationship inspector: the behavior group it belongs to.
  ///
  /// In en, this message translates to:
  /// **'In group {group}.'**
  String inGroup(Object group);

  /// Relationship inspector: bound to a port or relationship (direct).
  ///
  /// In en, this message translates to:
  /// **'Takes its value from {source}.'**
  String takesItsValueFrom(Object source);

  /// Relationship inspector: bound across domains with a starting value.
  ///
  /// In en, this message translates to:
  /// **'Takes its value from {source}, carried across timing domains starting at {init}.'**
  String takesItsValueFromTransported(Object source, Object init);

  /// Relationship inspector: open because a concept has no value form yet; names joined with "and".
  ///
  /// In en, this message translates to:
  /// **'Checked once {names}’s value is decided.'**
  String checkedOnceValueDecided(Object names);

  /// Output inspector: the relationship that drives the output.
  ///
  /// In en, this message translates to:
  /// **'Driven by {driver}.'**
  String drivenBy(Object driver);

  /// Output inspector: two relationships drive one output; claimants joined with "and".
  ///
  /// In en, this message translates to:
  /// **'{output} already has a final target: {claimants} both claim it.'**
  String contestedOutput(Object output, Object claimants);

  /// Why an output cannot be deleted yet.
  ///
  /// In en, this message translates to:
  /// **'Still driven by {names}'**
  String stillDrivenBy(Object names);

  /// Instance inspector warning.
  ///
  /// In en, this message translates to:
  /// **'{component}’s source no longer keeps its promise; open it to see why.'**
  String promiseBrokenOpenSource(Object component);

  /// Port inspector: what supplies the port (direct).
  ///
  /// In en, this message translates to:
  /// **'Bound to {source}.'**
  String boundTo(Object source);

  /// Port inspector: supplied across domains with a starting value.
  ///
  /// In en, this message translates to:
  /// **'Bound to {source}, carried across timing domains starting at {init}.'**
  String boundToTransported(Object source, Object init);

  /// Component inspector: the instances of this component.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Used by 1 instance: {names}.} other{Used by {count} instances: {names}.}}'**
  String usedByInstancesNamed(int count, Object names);

  /// Component inspector, shared concepts: the system concept a body concept stands for.
  ///
  /// In en, this message translates to:
  /// **'stands for {concept}'**
  String standsFor(Object concept);

  /// Component inspector, physical outputs: a body output that is the system’s.
  ///
  /// In en, this message translates to:
  /// **'the system’s {output}'**
  String theSystemsOutput(Object output);

  /// Port timing menu entry: a clock parameter of the component.
  ///
  /// In en, this message translates to:
  /// **'{clock} (parameter)'**
  String clockParameterSuffix(Object clock);

  /// Port timing menu entry: a clock private to the component.
  ///
  /// In en, this message translates to:
  /// **'{clock} (private)'**
  String clockPrivateSuffix(Object clock);

  /// Count of relationships (group inspector, multi-selection).
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 relationship} other{{count} relationships}}'**
  String relationshipsCount(int count);

  /// Multi-selection inspector title.
  ///
  /// In en, this message translates to:
  /// **'{count} selected'**
  String selectedCount(int count);

  /// Multi-selection: "3 relationships, 2 others." — relationships is the relationshipsCount text.
  ///
  /// In en, this message translates to:
  /// **'{relationships}{others, plural, =0{} =1{, 1 other} other{, {others} others}}.'**
  String selectionSummary(Object relationships, int others);

  /// Multi-selection button when only some of the selected relationships are free.
  ///
  /// In en, this message translates to:
  /// **'Group {count} relationships as Behavior'**
  String groupNAsBehavior(int count);

  /// Multi-selection note.
  ///
  /// In en, this message translates to:
  /// **'{count} already in a behavior; move them from there.'**
  String alreadyInABehavior(int count);

  /// Definition editor status after a failed save; error is the daemon’s message.
  ///
  /// In en, this message translates to:
  /// **'Not saved: {error}'**
  String notSaved(Object error);

  /// Definition editor status when the draft could not be checked.
  ///
  /// In en, this message translates to:
  /// **'Not checked: {reason}'**
  String notChecked(Object reason);

  /// Reason fragment used in notChecked when no other reason is known.
  ///
  /// In en, this message translates to:
  /// **'the compiler service is unavailable'**
  String get compilerServiceUnavailable;

  /// Composer result slot hint: what the whole formula produces.
  ///
  /// In en, this message translates to:
  /// **'produces {what}'**
  String producesDescription(Object what);

  /// Composer slot hint when nothing is known about the result.
  ///
  /// In en, this message translates to:
  /// **'expression'**
  String get expressionHint;

  /// Composer detail line: the type of the selected part.
  ///
  /// In en, this message translates to:
  /// **'This is {what}.'**
  String thisIs(Object what);

  /// Composer section: the equations that fit here.
  ///
  /// In en, this message translates to:
  /// **'Equations ({count})'**
  String equationsCount(int count);

  /// Composer operator button tooltip; operator is a glyph such as +.
  ///
  /// In en, this message translates to:
  /// **'Insert {operator} after this'**
  String insertAfterThis(Object operator);

  /// Composer note on a binder parameter.
  ///
  /// In en, this message translates to:
  /// **'{param} is each element: {type}.'**
  String paramIsEachElementOf(Object param, Object type);

  /// Composer note on a binder parameter of unknown element type.
  ///
  /// In en, this message translates to:
  /// **'{param} is each element of the collection.'**
  String paramIsEachElement(Object param);

  /// Code view accessibility label.
  ///
  /// In en, this message translates to:
  /// **'Source of {file}'**
  String sourceOf(Object file);

  /// Code view file menu: a file whose text does not build yet.
  ///
  /// In en, this message translates to:
  /// **'{file} — not built'**
  String notBuiltSuffix(Object file);

  /// Simulate readiness: the compiler has not answered for this revision yet; nothing is wrong.
  ///
  /// In en, this message translates to:
  /// **'Checking the design…'**
  String get checkingTheDesign;

  /// Simulate readiness: an instantaneous cycle, with the relationships on it as a comma-separated list.
  ///
  /// In en, this message translates to:
  /// **'These relationships depend on each other in the same instant: {names}. One of them must read the previous value instead.'**
  String dependOnEachOtherInTheSameInstant(String names);

  /// Simulate readiness: the relationship's definition does not check.
  ///
  /// In en, this message translates to:
  /// **'{name} has no valid definition.'**
  String hasNoValidDefinition(String name);

  /// Simulate readiness: a declared relationship with inputs (a Source needs no definition).
  ///
  /// In en, this message translates to:
  /// **'{name} has no definition. A relationship that reads something needs one before the design can run.'**
  String hasNoDefinitionReadsSomething(String name);

  /// Simulate readiness: the concept a Source produces is still decide-later. The value form names are the New concept sheet's.
  ///
  /// In en, this message translates to:
  /// **'{concept} needs a value form (Quantity, On / off or Count) before {input} can be given a value.'**
  String needsAValueFormBeforeInput(String concept, String input);

  /// Simulate readiness: a Source without a value for the run.
  ///
  /// In en, this message translates to:
  /// **'{name} needs a value before simulation can step.'**
  String needsAValueBeforeSimulationCanStep(String name);

  /// Simulate inputs: placeholder of an empty quantity field.
  ///
  /// In en, this message translates to:
  /// **'value'**
  String get valueHint;

  /// Simulate inputs: the word beside an On / off switch that is on. Matches bdld's rendering of a truth value.
  ///
  /// In en, this message translates to:
  /// **'on'**
  String get onWord;

  /// Simulate inputs: the word beside an On / off switch that is off. Matches bdld's rendering of a truth value.
  ///
  /// In en, this message translates to:
  /// **'off'**
  String get offWord;

  /// Simulate trace: header of the tick column.
  ///
  /// In en, this message translates to:
  /// **'tick'**
  String get tickColumn;

  /// Simulate trace: header of the column listing the domains that activated at the tick.
  ///
  /// In en, this message translates to:
  /// **'active'**
  String get activeColumn;

  /// Simulate inputs: a concept without a value form cannot take a value.
  ///
  /// In en, this message translates to:
  /// **'{concept} has no value form yet.'**
  String hasNoValueFormYet(Object concept);

  /// Simulate status line.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 tick evaluated.} other{{count} ticks evaluated.}}'**
  String ticksEvaluated(int count);

  /// Simulate: the next tick number.
  ///
  /// In en, this message translates to:
  /// **'tick {n}'**
  String tickN(int n);

  /// Deploy target menu detail.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 resource} other{{count} resources}}'**
  String resourcesCount(int count);

  /// Deploy verdict when the analysis request failed.
  ///
  /// In en, this message translates to:
  /// **'Could not analyse: {error}'**
  String couldNotAnalyse(Object error);

  /// Deploy verdict while the analysis runs.
  ///
  /// In en, this message translates to:
  /// **'Checking {board}…'**
  String checkingBoard(Object board);

  /// Deploy verdict.
  ///
  /// In en, this message translates to:
  /// **'Feasible on {board}.'**
  String feasibleOnBoardSentence(Object board);

  /// Deploy verdict for an incomplete design.
  ///
  /// In en, this message translates to:
  /// **'Fits {board} so far — the binding is not finished.'**
  String fitsBoardSoFar(Object board);

  /// Deploy verdict.
  ///
  /// In en, this message translates to:
  /// **'Not feasible on {board}.'**
  String notFeasibleOnBoardSentence(Object board);

  /// Deploy section title: where each requirement landed.
  ///
  /// In en, this message translates to:
  /// **'Placement on {board}'**
  String placementOn(Object board);

  /// Deploy finding: devices without an output; devices comma-joined.
  ///
  /// In en, this message translates to:
  /// **'Not connected to an output: {devices}.'**
  String notConnectedToAnOutput(Object devices);

  /// Deploy finding: outputs without a device.
  ///
  /// In en, this message translates to:
  /// **'No device on {board} for: {outputs}.'**
  String noDeviceOnBoardFor(Object board, Object outputs);

  /// Deploy dead end: an unnamed requirement by index.
  ///
  /// In en, this message translates to:
  /// **'requirement {n}'**
  String requirementN(int n);

  /// Deploy dead-end headline.
  ///
  /// In en, this message translates to:
  /// **'Could not place {what} of {who}.'**
  String couldNotPlaceOf(Object what, Object who);

  /// Deploy dead-end reason.
  ///
  /// In en, this message translates to:
  /// **'Nothing on {board} can carry {what}.'**
  String nothingOnBoardCanCarry(Object board, Object what);

  /// Deploy dead-end reason.
  ///
  /// In en, this message translates to:
  /// **'The pin chosen by hand, {pin}, cannot carry {what} here.'**
  String fixedPinCannotCarry(Object pin, Object what);

  /// Canvas context menu on a multi-selection.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Group as Behavior (1 relationship)} other{Group as Behavior ({count} relationships)}}'**
  String groupAsBehaviorCount(int count);

  /// Canvas socket tooltip.
  ///
  /// In en, this message translates to:
  /// **'reads {name}'**
  String readsSocket(Object name);

  /// Canvas socket tooltip.
  ///
  /// In en, this message translates to:
  /// **'produces {name}'**
  String producesSocket(Object name);

  /// Group inspector: remove this relationship from the group.
  ///
  /// In en, this message translates to:
  /// **'Remove from group'**
  String get removeFromGroupButton;

  /// Same as decideLater, lowercase, inline in a row.
  ///
  /// In en, this message translates to:
  /// **'decide later'**
  String get decideLaterLower;

  /// Studio UI text (inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Grouped value'**
  String get groupedValueTitle;

  /// Packaging sheet field: the name of the instance to create.
  ///
  /// In en, this message translates to:
  /// **'Instance'**
  String get instanceTitle;

  /// Packaging sheet field: the name of the component to create.
  ///
  /// In en, this message translates to:
  /// **'Component'**
  String get componentTitle;

  /// Studio UI text (system_inspector.dart).
  ///
  /// In en, this message translates to:
  /// **'Not placed yet.'**
  String get notPlacedYetSentence;

  /// Studio UI text (system_sheets.dart).
  ///
  /// In en, this message translates to:
  /// **'Package as Reusable Component'**
  String get packageAsReusableComponentTitle;

  /// Definition editor mode switch: edit as text (versus the Formula composer).
  ///
  /// In en, this message translates to:
  /// **'Text'**
  String get textMode;

  /// Simulate button: evaluate ten ticks.
  ///
  /// In en, this message translates to:
  /// **'Step ×10'**
  String get stepTen;

  /// Simulate error sentence subject when the relationship is unknown.
  ///
  /// In en, this message translates to:
  /// **'A relationship'**
  String get aRelationshipCapital;

  /// Context-menu verb.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get delete;

  /// Error banner: the daemon process is not reachable.
  ///
  /// In en, this message translates to:
  /// **'The compiler service is not connected.'**
  String get diagNotConnected;

  /// Error banner: a request to the daemon failed in transit.
  ///
  /// In en, this message translates to:
  /// **'The compiler service could not be reached.'**
  String get diagTransport;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'The system file dialog could not be shown. This happens when Studio is launched from a sandboxed host (an embedded terminal, for example). Launch it from Finder or Terminal, or enter a path below.'**
  String get diagPickerUnavailable;

  /// Conflict banner; the daemon's file list is shown as detail.
  ///
  /// In en, this message translates to:
  /// **'The project changed on disk since it was opened.'**
  String get diagChangedOnDisk;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'The project could not be written to disk.'**
  String get diagPersist;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'The edit targeted an older revision of the project; it was not applied.'**
  String get diagStaleRevision;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'Nothing to undo.'**
  String get diagNothingToUndo;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'Nothing to redo.'**
  String get diagNothingToRedo;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'No project is open.'**
  String get diagNoProject;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'A project is already open; close it first.'**
  String get diagAlreadyOpen;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'This project is a flat design, not a behavior system.'**
  String get diagNotASystem;

  /// Error banner.
  ///
  /// In en, this message translates to:
  /// **'Start a simulation first.'**
  String get diagSimulationNotStarted;

  /// OS file dialog confirm button.
  ///
  /// In en, this message translates to:
  /// **'Open Project'**
  String get dialogOpenProject;

  /// OS save dialog confirm button.
  ///
  /// In en, this message translates to:
  /// **'Create Project'**
  String get dialogCreateProject;

  /// Suggested name in the OS save dialog for a new project directory.
  ///
  /// In en, this message translates to:
  /// **'Untitled Project'**
  String get dialogUntitledProject;

  /// Canvas header word, inspector role and library category word for a relationship the environment provides (glossary: source). Never sensor, file or source code.
  ///
  /// In en, this message translates to:
  /// **'Source'**
  String get roleSource;

  /// Inspector row label: what a relationship is (Source, Relationship, port).
  ///
  /// In en, this message translates to:
  /// **'Role'**
  String get role;

  /// Inspector realization value for a Source with no deployment realization.
  ///
  /// In en, this message translates to:
  /// **'Provided by the environment; no device is bound yet.'**
  String get realizationEnvironment;

  /// Inspector realization value for a Source no device provides on the chosen board.
  ///
  /// In en, this message translates to:
  /// **'Provided by the environment; no device on {board} yet.'**
  String realizationNoDeviceOn(String board);

  /// Inspector realization value for a Source a device provides on the chosen board.
  ///
  /// In en, this message translates to:
  /// **'Provided by {device} on {board}.'**
  String realizationProvidedBy(String device, String board);

  /// Inspector realization value for a Source a device provides through a chosen profile.
  ///
  /// In en, this message translates to:
  /// **'Provided by {device} as {profile} on {board}.'**
  String realizationProvidedByAs(String device, String profile, String board);

  /// Inspector explanation under a Source's Role row.
  ///
  /// In en, this message translates to:
  /// **'A value that enters the behavior model from the environment, observed once per activation. Nothing is missing: add a definition only to compute it inside the model instead.'**
  String get sourceExplanation;

  /// Library section of the standard Source items (glossary: source). Not all are sensors.
  ///
  /// In en, this message translates to:
  /// **'Sources'**
  String get categorySources;

  /// Sheet title: create a Source.
  ///
  /// In en, this message translates to:
  /// **'New source'**
  String get newSource;

  /// Sheet subtitle for a new Source.
  ///
  /// In en, this message translates to:
  /// **'A value the environment provides: it reads nothing and is observed once per activation.'**
  String get aSourceAValueTheEnvironmentProvides;

  /// Canvas contextual menu: submenu of Source items.
  ///
  /// In en, this message translates to:
  /// **'Add Source'**
  String get addSource;

  /// Canvas contextual menu / sidebar: open the new-source sheet.
  ///
  /// In en, this message translates to:
  /// **'New source…'**
  String get newSourceEllipsis;

  /// Accessibility label of a Source node on the canvas.
  ///
  /// In en, this message translates to:
  /// **'{name}, Source: a value entering the behavior model from the environment, provides {concept}'**
  String sourceNodeSemantics(String name, String concept);

  /// Library row subtitle for a Source item: what the relationship provides.
  ///
  /// In en, this message translates to:
  /// **'Source of {concept}'**
  String sourceOfConcept(String concept);

  /// New-source sheet hint while no concept is chosen.
  ///
  /// In en, this message translates to:
  /// **'Choose what it provides: the one socket on its right.'**
  String get chooseWhatItProvidesTheOutputSocket;

  /// Simulate page: the section of values the environment provides during a run (the Sources).
  ///
  /// In en, this message translates to:
  /// **'Sources'**
  String get sources;

  /// Simulate page, the Sources section while the design has none (glossary: source).
  ///
  /// In en, this message translates to:
  /// **'No Sources: a relationship with no reads and no definition is one. The environment then provides its value, one per activation.'**
  String get noSourcesARelationshipWithNoReads;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Search the library'**
  String get searchLibrary;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Loading the library…'**
  String get loadingTheLibrary;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'The library arrives with the compiler service.'**
  String get theLibraryArrivesWithTheCompiler;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Nothing in the library matches “{query}”.'**
  String noLibraryMatches(Object query);

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Open a project to insert from here.'**
  String get openAProjectToInsertFrom;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'Creates'**
  String get libraryCreates;

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'value: {name}'**
  String libraryValueOf(Object name);

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'source: {name}'**
  String librarySourceOf(Object name);

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'type: {type}'**
  String libraryTypeOf(Object type);

  /// Studio UI text (library_panel.dart).
  ///
  /// In en, this message translates to:
  /// **'External'**
  String get libraryGroupExternal;

  /// Canvas header word for a relationship that reads something (glossary: rule): a function applied by a value's formula, with no value of its own. Shown only when no state word takes the slot.
  ///
  /// In en, this message translates to:
  /// **'rule'**
  String get ruleWord;

  /// Inspector Role row and Simulate probe: a relationship that reads something (glossary: rule).
  ///
  /// In en, this message translates to:
  /// **'Rule'**
  String get roleRule;

  /// Inspector Role row: a relationship that reads nothing and has a formula (glossary: computed value).
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get roleValue;

  /// Inspector explanation under a rule's Role row.
  ///
  /// In en, this message translates to:
  /// **'A function from what it reads to what it produces. It has no value of its own: a value\'s formula applies it.'**
  String get ruleExplanation;

  /// Inspector explanation under a computed value's Role row.
  ///
  /// In en, this message translates to:
  /// **'A value of the design: its formula gives it a value once per activation.'**
  String get valueExplanation;

  /// Inspector row label: the relationships this one's formula references (the reference edges on the canvas).
  ///
  /// In en, this message translates to:
  /// **'Depends on'**
  String get dependsOn;

  /// Inspector row label: the relationships whose formulas reference this one.
  ///
  /// In en, this message translates to:
  /// **'Named in'**
  String get namedIn;

  /// Accessibility fragment of a rule node on the canvas; reads is the list of concept names.
  ///
  /// In en, this message translates to:
  /// **'rule, reads {reads}'**
  String ruleNodeSemantics(String reads);

  /// Accessibility fragment of a computed-value node on the canvas.
  ///
  /// In en, this message translates to:
  /// **'value'**
  String get valueNodeSemantics;

  /// Accessibility fragment of a relationship node: the relationships its formula references.
  ///
  /// In en, this message translates to:
  /// **'depends on {names}'**
  String dependsOnList(String names);

  /// New-relationship / new-source sheet hint once the produced concept is chosen and nothing is read: which shape is being created and its consequence.
  ///
  /// In en, this message translates to:
  /// **'Reads nothing: a Source. The environment provides {concept} once per activation; a formula added later makes it a computed value instead.'**
  String sheetSourceShape(String concept);

  /// New-relationship sheet hint once the produced concept is chosen and something is read: which shape is being created and its consequence.
  ///
  /// In en, this message translates to:
  /// **'Reads {reads}: a rule, a function to {concept}. It has no value of its own — a value\'s formula applies it; dashed until its formula is added.'**
  String sheetRuleShape(String reads, String concept);

  /// Simulate probe: a rule is selected.
  ///
  /// In en, this message translates to:
  /// **'A rule: it has no value of its own. A value whose formula applies it is what the simulator samples.'**
  String get aRuleNoValueOfItsOwn;

  /// Simulate probe: nothing references the selected rule.
  ///
  /// In en, this message translates to:
  /// **'No value applies it yet.'**
  String get noValueAppliesItYet;

  /// Simulate probe: no unit-domain relationship produces this concept, so it has no value per tick.
  ///
  /// In en, this message translates to:
  /// **'Nothing carries {concept} yet: no value or Source produces it.'**
  String noValueCarries(String concept);

  /// Simulate probe: a rule produces the concept by signature, but only a value gives it a value per tick.
  ///
  /// In en, this message translates to:
  /// **'{rule} is a rule; a value whose formula applies it would carry {concept}.'**
  String ruleProducesNoValue(String rule, String concept);

  /// Simulate probe caption: the values and Sources that give this concept a value per tick.
  ///
  /// In en, this message translates to:
  /// **'Carried by'**
  String get carriedBy;

  /// Studio UI text (code_pane.dart): the file-bar button asking the daemon for the canonical layout.
  ///
  /// In en, this message translates to:
  /// **'Format'**
  String get format;

  /// Studio UI text (code_pane.dart).
  ///
  /// In en, this message translates to:
  /// **'Lay the file out the canonical way (⌥⇧F)'**
  String get formatTooltip;

  /// Studio UI text (code_pane.dart): the references list when empty.
  ///
  /// In en, this message translates to:
  /// **'Nothing names {name}.'**
  String nothingNames(String name);

  /// Studio UI text (code_pane.dart): the references list header.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 place names {name}} other{{count} places name {name}}}'**
  String placesNaming(int count, String name);

  /// Source sheet: the row label of the one choice the sheet exists for — which concept the Source provides.
  ///
  /// In en, this message translates to:
  /// **'Concept'**
  String get conceptLabel;

  /// Source sheet segment: bind the Source to a concept already in the design (by identity).
  ///
  /// In en, this message translates to:
  /// **'Existing concept'**
  String get existingConcept;

  /// Source sheet: the field label for the relationship's name (an identifier).
  ///
  /// In en, this message translates to:
  /// **'Source name'**
  String get sourceName;

  /// Source sheet: the primary button; enabled once the concept choice and the name are complete.
  ///
  /// In en, this message translates to:
  /// **'Create Source'**
  String get createSource;

  /// Source sheet: shown in place of the concept pop-up when the design has no concept to bind to.
  ///
  /// In en, this message translates to:
  /// **'No concepts in this design yet — choose New concept.'**
  String get noConceptsYetCreateOne;

  /// Source sheet caption under the preview, existing-concept path: exactly what will be committed.
  ///
  /// In en, this message translates to:
  /// **'Only the Source is created; the concept stays as it is. One Undo removes the Source.'**
  String get sourceOverExistingConceptCaption;

  /// Source sheet caption under the preview, new-concept path: exactly what will be committed.
  ///
  /// In en, this message translates to:
  /// **'The concept and its Source are created together, in one step. One Undo removes both.'**
  String get sourceWithNewConceptCaption;

  /// Library panel, Source row hover: what a Source preset does — it creates an input over a concept the designer picks; it owns no concept.
  ///
  /// In en, this message translates to:
  /// **'An input for a concept you choose'**
  String get inputForAConcept;

  /// Library panel, Source row hover, when the preset suggests a value form (kind: e.g. Temperature, On / off).
  ///
  /// In en, this message translates to:
  /// **'An input for a {kind} concept you choose — existing, or new'**
  String inputForConceptOfKind(String kind);

  /// Library panel, Source row right column: the row creates an input for a concept the designer chooses; no signature is shown before the choice.
  ///
  /// In en, this message translates to:
  /// **'choose a concept'**
  String get chooseConcept;

  /// Contextual menu: the submenu of the compiler's fixes for the object.
  ///
  /// In en, this message translates to:
  /// **'Fix'**
  String get fixMenu;

  /// Contextual menu: open the Split view at the object's declaration.
  ///
  /// In en, this message translates to:
  /// **'Reveal in Code'**
  String get revealInCode;

  /// Contextual menu: focus the relationship's definition editor.
  ///
  /// In en, this message translates to:
  /// **'Edit Definition'**
  String get editDefinition;

  /// Contextual menu on a multi-selection: delete every selected object.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Delete 1 object} other{Delete {count} objects}}'**
  String deleteObjects(int count);

  /// Contextual menu on a sink: select the relationship that drives it.
  ///
  /// In en, this message translates to:
  /// **'Show Driver: {name}'**
  String showDriver(String name);

  /// Contextual menu on a link: select one of its ends.
  ///
  /// In en, this message translates to:
  /// **'Show {name}'**
  String showEnd(String name);

  /// Chooser after dropping a concept on a driven sink: swap the driver.
  ///
  /// In en, this message translates to:
  /// **'Replace {current} with {candidate}'**
  String replaceDriver(String current, String candidate);

  /// Chooser after dropping a concept on a sink: which relationship drives it.
  ///
  /// In en, this message translates to:
  /// **'Drive with {name}'**
  String driveWith(String name);

  /// Chooser notice after dropping a concept on a sink with no eligible driver.
  ///
  /// In en, this message translates to:
  /// **'{output} accepts {concept}, but no current relationship can drive it.'**
  String noDriverForConcept(String output, String concept);

  /// Chooser notice: select the sink so the inspector explains it.
  ///
  /// In en, this message translates to:
  /// **'Show {name} in the Inspector'**
  String showInInspector(String name);

  /// Canvas menu / ⌘A: select every visible object.
  ///
  /// In en, this message translates to:
  /// **'Select All'**
  String get selectAll;

  /// Canvas menu / ⌘0: fit every node in the view.
  ///
  /// In en, this message translates to:
  /// **'Frame All'**
  String get frameAll;

  /// Accessibility name of the marquee dragged left to right.
  ///
  /// In en, this message translates to:
  /// **'Window selection'**
  String get windowSelection;

  /// Accessibility name of the marquee dragged right to left.
  ///
  /// In en, this message translates to:
  /// **'Crossing selection'**
  String get crossingSelection;

  /// Inspector, multi-selection: the row of the active object.
  ///
  /// In en, this message translates to:
  /// **'Active'**
  String get activeObject;

  /// Banner detail when a multi-delete is refused before it starts.
  ///
  /// In en, this message translates to:
  /// **'Select those too, or disconnect them first.'**
  String get deleteBlockedDetails;

  /// Banner when a multi-delete is refused before it starts; {reasons} lists the objects still in use.
  ///
  /// In en, this message translates to:
  /// **'Nothing was deleted: {reasons}.'**
  String nothingWasDeleted(String reasons);

  /// Inspector, multi-selection: a count of one kind of selected object.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 output} other{{count} outputs}}'**
  String outputsCount(int count);

  /// Inspector, multi-selection: a count of one kind of selected object.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 other} other{{count} others}}'**
  String othersCount(int count);

  /// Deploy page: the section that builds the board's firmware and flashes it.
  ///
  /// In en, this message translates to:
  /// **'Firmware'**
  String get firmware;

  /// Deploy page progression: the deployment step (devices, providers, realizations, placement).
  ///
  /// In en, this message translates to:
  /// **'Deployment'**
  String get stepDeployment;

  /// Deploy page progression: the build step.
  ///
  /// In en, this message translates to:
  /// **'Build'**
  String get stepBuild;

  /// Deploy page progression: the flash step.
  ///
  /// In en, this message translates to:
  /// **'Flash'**
  String get stepFlash;

  /// Deploy page progression: the step after flashing — try the product.
  ///
  /// In en, this message translates to:
  /// **'Observe'**
  String get stepObserve;

  /// Primary button: build the firmware for the chosen board.
  ///
  /// In en, this message translates to:
  /// **'Build for {board}'**
  String buildForBoard(String board);

  /// Button: rebuild after a failure or a change.
  ///
  /// In en, this message translates to:
  /// **'Build again'**
  String get buildAgain;

  /// Link: cancel the running build.
  ///
  /// In en, this message translates to:
  /// **'Stop'**
  String get stopBuild;

  /// Primary button: write the firmware to the board.
  ///
  /// In en, this message translates to:
  /// **'Flash'**
  String get flash;

  /// Secondary action after a flash.
  ///
  /// In en, this message translates to:
  /// **'Flash again'**
  String get flashAgain;

  /// Link: look again for reachable boards.
  ///
  /// In en, this message translates to:
  /// **'Look again'**
  String get lookAgain;

  /// Firmware section heading while a blocker stands.
  ///
  /// In en, this message translates to:
  /// **'Not ready to build'**
  String get notReadyToBuild;

  /// Link beside a blocker about the design itself.
  ///
  /// In en, this message translates to:
  /// **'Fix it on the Design page'**
  String get fixOnTheDesignPage;

  /// Firmware section heading while a build runs.
  ///
  /// In en, this message translates to:
  /// **'Building…'**
  String get building;

  /// Build stage label.
  ///
  /// In en, this message translates to:
  /// **'Checking the deployment'**
  String get stageChecking;

  /// Build stage label.
  ///
  /// In en, this message translates to:
  /// **'Generating the crate'**
  String get stageGenerating;

  /// Build stage label.
  ///
  /// In en, this message translates to:
  /// **'Preparing the toolchain'**
  String get stagePreparing;

  /// Build stage label.
  ///
  /// In en, this message translates to:
  /// **'Compiling'**
  String get stageCompiling;

  /// Build stage label.
  ///
  /// In en, this message translates to:
  /// **'Writing the image'**
  String get stagePackaging;

  /// Build outcome: cancelled by the designer.
  ///
  /// In en, this message translates to:
  /// **'Stopped'**
  String get stageCancelled;

  /// Progress while compiling.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 crate compiled} other{{count} crates compiled}}'**
  String cratesCompiled(int count);

  /// Firmware section heading once an image exists and is current.
  ///
  /// In en, this message translates to:
  /// **'Firmware built at {time}'**
  String firmwareBuiltAt(String time);

  /// Firmware section heading when the last image is stale.
  ///
  /// In en, this message translates to:
  /// **'The firmware is from an earlier design or deployment.'**
  String get firmwareStale;

  /// Firmware section heading after a failed build.
  ///
  /// In en, this message translates to:
  /// **'The build did not complete'**
  String get buildDidNotComplete;

  /// The image's size.
  ///
  /// In en, this message translates to:
  /// **'{bytes} bytes'**
  String imageSize(String bytes);

  /// Flash step: nothing to write to.
  ///
  /// In en, this message translates to:
  /// **'No board is reachable.'**
  String get noBoardReachable;

  /// Flash step: a choice is needed.
  ///
  /// In en, this message translates to:
  /// **'Several boards are reachable — choose one.'**
  String get chooseTheBoardToFlash;

  /// Firmware section heading while a flash runs.
  ///
  /// In en, this message translates to:
  /// **'Flashing…'**
  String get flashing;

  /// Flash stage label.
  ///
  /// In en, this message translates to:
  /// **'Preparing'**
  String get flashPreparing;

  /// Flash stage label.
  ///
  /// In en, this message translates to:
  /// **'Writing the image'**
  String get flashWriting;

  /// Flash stage label.
  ///
  /// In en, this message translates to:
  /// **'Restarting the board'**
  String get flashRestarting;

  /// Firmware section heading after a flash.
  ///
  /// In en, this message translates to:
  /// **'Flashed to {device} at {time}'**
  String flashedToAt(String device, String time);

  /// Firmware section heading after a failed flash.
  ///
  /// In en, this message translates to:
  /// **'The flash did not complete'**
  String get flashDidNotComplete;

  /// Next step after a flash, naming the design's Sources and outputs.
  ///
  /// In en, this message translates to:
  /// **'Try it: act on {sources}; {outputs} should follow the design.'**
  String tryItOnTheBoard(String sources, String outputs);

  /// Next step after a flash for a design without Sources.
  ///
  /// In en, this message translates to:
  /// **'Try it: {outputs} should behave as designed.'**
  String tryItOutputsOnly(String outputs);

  /// After a flash, once the design or deployment changed.
  ///
  /// In en, this message translates to:
  /// **'The board runs an earlier design — build and flash again to update it.'**
  String get boardRunsEarlierDesign;

  /// Disclosure: the advanced build details.
  ///
  /// In en, this message translates to:
  /// **'Details'**
  String get details;

  /// Details row label.
  ///
  /// In en, this message translates to:
  /// **'Generated crate'**
  String get generatedCrate;

  /// Details row label.
  ///
  /// In en, this message translates to:
  /// **'Command'**
  String get buildCommand;

  /// Details row label: the Rust target triple.
  ///
  /// In en, this message translates to:
  /// **'Target'**
  String get rustTarget;

  /// Details row label: the flashable image.
  ///
  /// In en, this message translates to:
  /// **'Image'**
  String get imageFile;

  /// Details row label.
  ///
  /// In en, this message translates to:
  /// **'Compiler output'**
  String get compilerOutput;

  /// Welcome page: the section of templates a new project can start from.
  ///
  /// In en, this message translates to:
  /// **'Demos'**
  String get demos;

  /// Firmware section heading when everything before the build holds.
  ///
  /// In en, this message translates to:
  /// **'Ready to build for {board}.'**
  String readyToBuild(String board);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Values'**
  String get libraryValues;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Quantities'**
  String get libraryQuantities;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Source'**
  String get sourceRow;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'A value the environment provides — for a concept you choose, existing or new.'**
  String get sourceRowHint;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'A value category from the Library, and the name it has in this product.'**
  String get conceptSheetSubtitle;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get valueCategory;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Measured in'**
  String get measuredIn;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'A name is required.'**
  String get nameRequired;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'{name} is already in use.'**
  String nameTaken(String name);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'A name is letters, digits and _, not starting with a digit.'**
  String get nameNotIdentifier;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Create Concept'**
  String get createConcept;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Creates one concept, as the Code view will write it.'**
  String get conceptSheetCaption;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'choose a category'**
  String get chooseACategory;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Show Formula'**
  String get showFormula;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Hide Formula'**
  String get hideFormula;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Edit formula'**
  String get editFormula;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'The formula cannot be shown here.'**
  String get formulaPreviewUnavailable;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Reading the formula…'**
  String get formulaPreviewLoading;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'result'**
  String get resultLabel;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Type to write, or choose a part'**
  String get typeToWrite;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'Type an operator before adding a value here.'**
  String get typeAnOperatorFirst;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'before {what}'**
  String caretBefore(String what);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'after {what}'**
  String caretAfter(String what);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'in an empty slot'**
  String get caretIn;

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'inside {what}'**
  String caretInside(String what);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'{numerator} over {denominator}'**
  String fractionOf(String numerator, String denominator);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'a choice: if {condition}, then {then}, else {otherwise}'**
  String choiceSemantics(String condition, String then, String otherwise);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'a match on {value} with {count} cases'**
  String matchSemantics(String value, int count);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'a block with {count} local bindings'**
  String blockSemantics(int count);

  /// Authoring UX (library categories, concept sheet, formula composer, expanded formulas).
  ///
  /// In en, this message translates to:
  /// **'a {word} boundary'**
  String temporalSemantics(String word);

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'On / off'**
  String get libItem_std_value_boolean_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A value that is either true or false: on or off, pressed or not, present or absent.'**
  String get libItem_std_value_boolean_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'bool boolean true false flag toggle switch button pressed held touch contact state'**
  String get libItem_std_value_boolean_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Count'**
  String get libItem_std_value_count_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A whole number of things: pulses, presses, items, steps.'**
  String get libItem_std_value_count_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'count integer number of pulses presses items steps tally'**
  String get libItem_std_value_count_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Level'**
  String get libItem_std_value_level_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A plain number with no unit: a level from 0 to 1, a ratio, a factor, a percentage.'**
  String get libItem_std_value_level_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'scalar ratio factor percent % fraction dimensionless brightness volume humidity battery speed setting intensity opening dimmer fan heater power setting'**
  String get libItem_std_value_level_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Decide later'**
  String get libItem_std_value_open_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A concept whose value form is not chosen yet; relationships can already use it, and the form is decided once known.'**
  String get libItem_std_value_open_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'open unknown later undecided analog external raw host network'**
  String get libItem_std_value_open_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Angle'**
  String get libItem_std_quantity_angle_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How far something is turned: a tilt, a heading, a dial, a lid, a shaft.'**
  String get libItem_std_quantity_angle_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'angle rotation turn tilt orientation heading dial servo encoder shaft lid rad deg degrees radians °'**
  String get libItem_std_quantity_angle_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Length'**
  String get libItem_std_quantity_length_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A distance or a position along a line: how far, how long, how high.'**
  String get libItem_std_quantity_length_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'length distance position range height width depth travel obstacle proximity m mm cm km inch ft metre meter'**
  String get libItem_std_quantity_length_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Time'**
  String get libItem_std_quantity_time_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A duration or an interval: how long something takes or lasts.'**
  String get libItem_std_quantity_time_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'time duration interval delay period timeout elapsed s ms min h seconds minutes hours'**
  String get libItem_std_quantity_time_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Mass'**
  String get libItem_std_quantity_mass_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How much matter something has: a load, a weight on a scale.'**
  String get libItem_std_quantity_mass_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'mass weight load scale kg g gram kilogram'**
  String get libItem_std_quantity_mass_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Current'**
  String get libItem_std_quantity_current_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Electric current: how much charge flows, as a motor or a supply draws.'**
  String get libItem_std_quantity_current_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'current electric amps ampere draw A mA milliamp'**
  String get libItem_std_quantity_current_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Temperature'**
  String get libItem_std_quantity_temperature_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How warm something is: the air, a room, a motor, a surface.'**
  String get libItem_std_quantity_temperature_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'temperature temp thermal heat warm cold hot thermometer K kelvin °C celsius'**
  String get libItem_std_quantity_temperature_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Amount of substance'**
  String get libItem_std_quantity_amount_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How much of a substance, counted in moles.'**
  String get libItem_std_quantity_amount_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'amount substance mol mole moles chemical'**
  String get libItem_std_quantity_amount_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Luminous intensity'**
  String get libItem_std_quantity_luminous_intensity_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How bright a light source is in one direction.'**
  String get libItem_std_quantity_luminous_intensity_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'luminous intensity candela cd light source lamp'**
  String get libItem_std_quantity_luminous_intensity_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Speed'**
  String get libItem_std_quantity_speed_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How fast something moves along a line: a vehicle, a belt, a wind.'**
  String get libItem_std_quantity_speed_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'speed velocity pace rate m/s km/h wheel vehicle conveyor wind'**
  String get libItem_std_quantity_speed_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Acceleration'**
  String get libItem_std_quantity_acceleration_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How quickly speed changes: a shock, a tilt sensed by gravity, a vehicle pulling away.'**
  String get libItem_std_quantity_acceleration_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'acceleration accelerometer g-force shock vibration m/s² imu'**
  String get libItem_std_quantity_acceleration_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Angular velocity'**
  String get libItem_std_quantity_angular_velocity_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How fast something turns: a shaft, a wheel, a gyroscope\'s reading.'**
  String get libItem_std_quantity_angular_velocity_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'angular velocity angular speed rotation speed rpm spin gyro gyroscope shaft wheel rad/s deg/s revolutions motor speed'**
  String get libItem_std_quantity_angular_velocity_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Frequency'**
  String get libItem_std_quantity_frequency_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How often something repeats each second: a pitch, a pulse rate, a blink.'**
  String get libItem_std_quantity_frequency_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'frequency rate pitch tone hertz Hz kHz pulse blink cycles'**
  String get libItem_std_quantity_frequency_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Force'**
  String get libItem_std_quantity_force_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A push or a pull: a load cell\'s reading, a grip, a spring.'**
  String get libItem_std_quantity_force_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'force push pull load cell grip spring thrust N newton'**
  String get libItem_std_quantity_force_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Pressure'**
  String get libItem_std_quantity_pressure_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Force over an area: the air, a fluid, a touch on a pad.'**
  String get libItem_std_quantity_pressure_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'pressure barometer atmospheric altitude weather fluid tyre pad Pa kPa hPa bar psi'**
  String get libItem_std_quantity_pressure_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Torque'**
  String get libItem_std_quantity_torque_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'A turning force: what a motor delivers to a shaft.'**
  String get libItem_std_quantity_torque_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'torque moment turning force motor shaft N·m Nm newton metre'**
  String get libItem_std_quantity_torque_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Power'**
  String get libItem_std_quantity_power_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Energy per second: what a heater, a motor or a lamp draws or delivers.'**
  String get libItem_std_quantity_power_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'power watt W kW heater consumption wattage'**
  String get libItem_std_quantity_power_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Voltage'**
  String get libItem_std_quantity_voltage_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Electric potential: a battery, a supply rail, an analog reading in volts.'**
  String get libItem_std_quantity_voltage_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'voltage volts potential battery supply rail analog adc V mV'**
  String get libItem_std_quantity_voltage_tags;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'Illuminance'**
  String get libItem_std_quantity_illuminance_name;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'How much light falls on a surface: ambient light, daylight, a photocell\'s reading.'**
  String get libItem_std_quantity_illuminance_description;

  /// Standard Library item (library/std/concepts.toml); generated by scripts/gen_library_l10n.py.
  ///
  /// In en, this message translates to:
  /// **'illuminance light lux lx ambient daylight dark photocell light sensor'**
  String get libItem_std_quantity_illuminance_tags;
}

class _AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) => <String>['en', 'ja', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when language+script codes are specified.
  switch (locale.languageCode) {
    case 'zh':
      {
        switch (locale.scriptCode) {
          case 'Hans':
            return AppLocalizationsZhHans();
        }
        break;
      }
  }

  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'ja':
      return AppLocalizationsJa();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
