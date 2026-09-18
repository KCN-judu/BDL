/// The localization layer (docs/project/localization-style.md): exactly
/// three locales, English fallback, a persisted preference, live UI
/// updates, complete catalogs, and — the acceptance principle — a locale
/// that changes presentation only: never BDL source, project files,
/// formal notation or diagnostic identity.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/effects/effect_executor.dart';
import 'package:bdl_studio/effects/preferences_store.dart';
import 'package:bdl_studio/l10n/diagnostics.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/app.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/preferences_sheet.dart';
import 'package:bdl_studio/ui/shell.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

/// A store that reduces for real but runs no effects (no daemon, no disk).
class _PureStore extends AppStore {
  _PureStore([this.initial = const AppState()]);
  final AppState initial;
  final List<Effect> effects = [];

  @override
  AppState build() => initial;

  @override
  void dispatch(AppAction action) {
    final t = reduce(state, action);
    state = t.state;
    effects.addAll(t.effects);
  }
}

Widget _app(_PureStore store, {Widget? home}) => ProviderScope(
  overrides: [appStoreProvider.overrideWith(() => store)],
  child: Consumer(
    builder: (context, ref, _) {
      final language = ref.watch(appStoreProvider.select((s) => s.preferences.language));
      return MaterialApp(
        theme: macTheme(Brightness.light),
        locale: language.locale,
        supportedLocales: kSupportedLocales,
        localizationsDelegates: const [
          ...AppLocalizations.localizationsDelegates,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        localeResolutionCallback: resolveLocale,
        builder: (context, child) => LanguageScope(language: language, child: child!),
        home: home ?? const StudioShell(),
      );
    },
  ),
);

AppState _connected() => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(
      compatible: true,
      compilerVersion: '1.0.0',
      protocolVersion: pb.Version(major: 0, minor: 9, patch: 0),
    ),
  ),
);

pb.ProjectProjection _lamp() =>
    pb.ProjectProjection(revision: Int64(1), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(0), name: 'Tilt'),
        pb.ConceptView(id: Int64(1), name: 'Brightness'),
      ])
      ..mappings.add(
        pb.MappingView(
          id: Int64(0),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED,
          definition: pb.Definition(formula: 'Tilt / 90 deg'),
        ),
      );

void main() {
  group('locales', () {
    test('exactly en, zh-Hans and ja are supported, in menu order', () {
      expect(kSupportedLocales.map((l) => l.toLanguageTag()), ['en', 'zh-Hans', 'ja']);
      expect(LanguagePreference.values.map((p) => p.tag), ['system', 'en', 'zh-Hans', 'ja']);
    });

    test('an unsupported system locale resolves to English', () {
      for (final tag in ['fr', 'de-DE', 'ko', 'zh-Hant', 'zh-TW', 'zh-HK', 'pt-BR']) {
        final parts = tag.split('-');
        final locale = parts.length == 1
            ? Locale(parts[0])
            : (parts[1].length == 4
                  ? Locale.fromSubtags(languageCode: parts[0], scriptCode: parts[1])
                  : Locale(parts[0], parts[1]));
        expect(resolveLocale(locale, kSupportedLocales), const Locale('en'), reason: tag);
      }
      expect(resolveLocale(null, kSupportedLocales), const Locale('en'));
    });

    test('supported locales and their regional variants resolve to themselves', () {
      expect(resolveLocale(const Locale('zh', 'CN'), kSupportedLocales), kSupportedLocales[1]);
      expect(resolveLocale(const Locale('zh'), kSupportedLocales), kSupportedLocales[1]);
      expect(resolveLocale(const Locale('ja', 'JP'), kSupportedLocales), kSupportedLocales[2]);
      expect(resolveLocale(const Locale('en', 'GB'), kSupportedLocales), kSupportedLocales[0]);
    });

    test('every supported locale has a catalog and English is the fallback', () {
      for (final l in kSupportedLocales) {
        expect(lookupAppLocalizations(l).preferencesTitle, isNotEmpty);
      }
      expect(kEnglish.localeName, 'en');
      expect(lookupAppLocalizations(kSupportedLocales[1]).localeName, 'zh_Hans');
    });

    test('the catalogs differ where they should and agree where they must', () {
      final zh = lookupAppLocalizations(kSupportedLocales[1]);
      final ja = lookupAppLocalizations(kSupportedLocales[2]);
      expect(zh.preferencesTitle, isNot(kEnglish.preferencesTitle));
      expect(ja.preferencesTitle, isNot(kEnglish.preferencesTitle));
      expect(zh.preferencesTitle, isNot(ja.preferencesTitle));
      // Identifiers are never localized.
      for (final c in [kEnglish, zh, ja]) {
        expect(c.appTitle, 'BDL Studio');
        expect(c.adaptivelamp, 'AdaptiveLamp');
      }
    });

    test('plurals and placeholders are filled in every locale', () {
      for (final c in kSupportedLocales.map(lookupAppLocalizations)) {
        expect(c.conceptsCount(1), contains('1'));
        expect(c.conceptsCount(3), contains('3'));
        expect(c.saveChangesTo('lamp'), contains('lamp'));
        expect(c.undoTooltip('⌘Z'), contains('⌘Z'));
        expect(
          c.transportExplanation('a', 'fast', 'b', 'slow'),
          allOf(contains('a'), contains('b')),
        );
        expect(c.minutesAgo(2), contains('2'));
        expect(c.selectionSummary('x', 0), isNot(contains('{')));
        expect(c.conceptsCount(2), isNot(contains('{')));
      }
      expect(kEnglish.conceptsCount(1), '1 concept');
      expect(kEnglish.conceptsCount(2), '2 concepts');
    });
  });

  group('preference persistence', () {
    test('round-trips through preferences.json outside the project', () async {
      final dir = await Directory.systemTemp.createTemp('bdl-prefs');
      addTearDown(() => dir.delete(recursive: true));
      final store = PreferencesStore(dir: dir);
      expect((await store.load()).language, LanguagePreference.system);
      await store.save(const AppPreferences(language: LanguagePreference.ja));
      expect((await store.load()).language, LanguagePreference.ja);
      expect(await File('${dir.path}/preferences.json').readAsString(), contains('"ja"'));
      await store.save(const AppPreferences(language: LanguagePreference.zhHans));
      expect((await store.load()).language, LanguagePreference.zhHans);
    });

    test('a corrupt or foreign file means defaults', () async {
      final dir = await Directory.systemTemp.createTemp('bdl-prefs');
      addTearDown(() => dir.delete(recursive: true));
      final store = PreferencesStore(dir: dir);
      await File('${dir.path}/preferences.json').writeAsString('{not json');
      expect((await store.load()).language, LanguagePreference.system);
      await File('${dir.path}/preferences.json')
          .writeAsString('{"schema_version": 1, "language": "fr"}');
      expect((await store.load()).language, LanguagePreference.system);
    });

    test('the reducer records the choice and asks for it to be saved', () {
      final t = reduce(const AppState(), const LanguageChanged(LanguagePreference.zhHans));
      expect(t.state.preferences.language, LanguagePreference.zhHans);
      expect(t.effects, hasLength(1));
      expect((t.effects.single as SavePreferences).preferences.language, LanguagePreference.zhHans);
      final loaded = reduce(
        t.state,
        const PreferencesLoaded(AppPreferences(language: LanguagePreference.ja)),
      );
      expect(loaded.state.preferences.language, LanguagePreference.ja);
      expect(loaded.effects, isEmpty);
    });

    test('the executor loads preferences at startup and saves a change', () async {
      final dir = await Directory.systemTemp.createTemp('bdl-prefs');
      addTearDown(() => dir.delete(recursive: true));
      await PreferencesStore(dir: dir).save(const AppPreferences(language: LanguagePreference.ja));
      final actions = <AppAction>[];
      final e = EffectExecutor(actions.add, preferences: PreferencesStore(dir: dir));
      await e.run(const LoadPreferences());
      expect(actions, hasLength(1));
      expect((actions.single as PreferencesLoaded).preferences.language, LanguagePreference.ja);
      await e.run(const SavePreferences(AppPreferences(language: LanguagePreference.en)));
      expect((await PreferencesStore(dir: dir).load()).language, LanguagePreference.en);
    });
  });

  group('live UI', () {
    testWidgets('changing the language re-renders the project manager in place', (tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final store = _PureStore(_connected());
      await tester.pumpWidget(_app(store));
      expect(find.text('Open Project…'), findsOneWidget);
      store.dispatch(const LanguageChanged(LanguagePreference.zhHans));
      await tester.pumpAndSettle();
      expect(find.text('Open Project…'), findsNothing);
      expect(find.text('打开项目…'), findsOneWidget);
      store.dispatch(const LanguageChanged(LanguagePreference.ja));
      await tester.pumpAndSettle();
      expect(find.text('プロジェクトを開く…'), findsOneWidget);
      store.dispatch(const LanguageChanged(LanguagePreference.en));
      await tester.pumpAndSettle();
      expect(find.text('Open Project…'), findsOneWidget);
    });

    testWidgets('the preferences sheet offers System Default and the three endonyms', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final store = _PureStore(_connected());
      await tester.pumpWidget(_app(store));
      await tester.tap(find.text('Preferences…'));
      await tester.pumpAndSettle();
      expect(find.text('Preferences'), findsOneWidget);
      await tester.tap(find.text('System Default'));
      await tester.pumpAndSettle();
      for (final label in ['System Default', 'English', '简体中文', '日本語']) {
        expect(find.text(label), findsWidgets, reason: label);
      }
      await tester.tap(find.text('日本語').last);
      await tester.pumpAndSettle();
      expect(store.state.preferences.language, LanguagePreference.ja);
      expect(store.effects.whereType<SavePreferences>(), isNotEmpty);
      // The sheet itself re-rendered: its title is now Japanese.
      expect(find.text('環境設定'), findsOneWidget);
    });

    testWidgets('the workspace, toolbar and status line speak the chosen language', (tester) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final store = _PureStore(_connected().copyWith(project: _lamp()));
      await tester.pumpWidget(_app(store));
      expect(find.text('Design'), findsWidgets);
      expect(find.text('Save'), findsOneWidget);
      store.dispatch(const LanguageChanged(LanguagePreference.zhHans));
      await tester.pumpAndSettle();
      expect(find.text('设计'), findsWidgets);
      expect(find.text('仿真'), findsOneWidget);
      expect(find.text('部署'), findsOneWidget);
      expect(find.text('保存'), findsOneWidget);
      expect(find.textContaining('编译器 1.0.0'), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
  });

  group('locale is presentation only', () {
    testWidgets('BDL source, names, () and diagnostic codes are the same in every locale', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(1400, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final project = _lamp();
      final store = _PureStore(_connected().copyWith(project: project));
      await tester.pumpWidget(_app(store));
      for (final lang in [
        LanguagePreference.zhHans,
        LanguagePreference.ja,
        LanguagePreference.en,
      ]) {
        store.dispatch(LanguageChanged(lang));
        await tester.pumpAndSettle();
        // The project the UI renders is byte-identical: nothing rewrote it.
        expect(store.state.project, same(project));
        expect(store.state.project!.mappings.first.definition.formula, 'Tilt / 90 deg');
        expect(find.text('dimByTilt'), findsWidgets, reason: lang.tag);
        expect(find.text('Tilt'), findsWidgets, reason: lang.tag);
      }
      // No effect other than persisting the preference was ever requested.
      expect(store.effects.every((e) => e is SavePreferences), isTrue);
    });

    test('prose may name the empty product; the notation () is never localized', () {
      for (final c in kSupportedLocales.map(lookupAppLocalizations)) {
        expect(c.noExplicitInputsTheCanonicalDomainIs, contains('()'));
        expect(c.noExplicitInputsTheCanonicalDomainIs, contains('() -> B'));
      }
      expect(
        lookupAppLocalizations(kSupportedLocales[1]).noExplicitInputsTheCanonicalDomainIs,
        contains('空积'),
      );
      expect(
        lookupAppLocalizations(kSupportedLocales[2]).noExplicitInputsTheCanonicalDomainIs,
        contains('空積'),
      );
    });

    test('a diagnostic code is identity; only its sentence changes', () {
      const code = 'studio.not_connected';
      final en = localizedMessage(kEnglish, code, 'fallback');
      final zh = localizedMessage(lookupAppLocalizations(kSupportedLocales[1]), code, 'fallback');
      final ja = localizedMessage(lookupAppLocalizations(kSupportedLocales[2]), code, 'fallback');
      expect({en, zh, ja}.length, 3);
      expect(isStudioWorded(code), isTrue);
      // A daemon-owned code keeps the daemon's English text in every locale.
      for (final c in kSupportedLocales.map(lookupAppLocalizations)) {
        expect(
          localizedMessage(c, 'output.missing_driver', 'Output light has no driver'),
          'Output light has no driver',
        );
      }
      expect(isStudioWorded('output.missing_driver'), isFalse);
    });

    test('OS dialog labels follow the preference outside the widget tree', () {
      expect(catalogFor(LanguagePreference.ja, const Locale('en')).dialogOpenProject, 'プロジェクトを開く');
      expect(
        catalogFor(LanguagePreference.system, const Locale('zh', 'CN')).dialogOpenProject,
        '打开项目',
      );
      expect(
        catalogFor(LanguagePreference.system, const Locale('fr')).dialogOpenProject,
        'Open Project',
      );
    });
  });

  group('layout', () {
    for (final lang in [LanguagePreference.zhHans, LanguagePreference.ja]) {
      testWidgets('${lang.tag}: project manager, workspace and sheet render without overflow', (
        tester,
      ) async {
        tester.view.physicalSize = const Size(1100, 700);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        final store = _PureStore(
          _connected().copyWith(preferences: AppPreferences(language: lang)),
        );
        await tester.pumpWidget(_app(store));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: 'project manager');
        await tester.tap(find.byIcon(Icons.language_outlined));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: 'preferences sheet');
        await tester.tap(find.text(lookupAppLocalizations(lang.locale!).done));
        await tester.pumpAndSettle();
        store.dispatch(ProjectReceived(_lamp()));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull, reason: 'workspace');
        // The page bar, the toolbar and the status line show the translated
        // words whole: no label of theirs was truncated to fit.
        final l10n = lookupAppLocalizations(lang.locale!);
        for (final word in [l10n.design, l10n.simulate, l10n.deploy, l10n.save, l10n.close]) {
          for (final e in find.text(word).evaluate()) {
            final rp = e.findRenderObject();
            if (rp is RenderParagraph) {
              expect(rp.didExceedMaxLines, isFalse, reason: '$word is truncated');
            }
          }
        }
      });
    }
  });

  testWidgets('StudioApp wires the delegates, the supported locales and the resolver', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final store = _PureStore();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [appStoreProvider.overrideWith(() => store)],
        child: const StudioApp(),
      ),
    );
    await tester.pump();
    final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
    expect(app.supportedLocales, kSupportedLocales);
    expect(app.localeResolutionCallback, resolveLocale);
    expect(app.localizationsDelegates, contains(AppLocalizations.delegate));
    expect(app.locale, isNull, reason: 'System Default follows the platform');
  });
}
