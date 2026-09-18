import 'dart:ui' show AppExitResponse;

import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/store.dart';
import '../l10n/l10n.dart';
import 'mac/theme.dart';
import 'preferences_sheet.dart';
import 'shell.dart';

class StudioApp extends ConsumerStatefulWidget {
  const StudioApp({super.key});

  @override
  ConsumerState<StudioApp> createState() => _StudioAppState();
}

class _StudioAppState extends ConsumerState<StudioApp> with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    // The app's first action; everything after is state transitions.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(appStoreProvider.notifier).dispatch(const AppStarted());
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  /// The desktop asked to quit — the menu's Quit, ⌘Q, the window's close
  /// button (the runner turns it into a terminate request).  With a project
  /// open the exit is declined here and asked for again through the
  /// project's own guard (app/lifecycle.dart), which quits once the project
  /// has been unloaded; nothing is torn down while that question is open.
  @override
  Future<AppExitResponse> didRequestAppExit() async {
    final state = ref.read(appStoreProvider);
    if (state.project == null && !state.editor.unloadInProgress) {
      return AppExitResponse.exit;
    }
    ref.read(appStoreProvider.notifier).dispatch(const QuitRequested());
    return AppExitResponse.cancel;
  }

  @override
  Widget build(BuildContext context) {
    // The language is an application preference: `null` follows the system
    // locale, resolved to one of the three supported locales or English.
    // Locale is presentation only — the project, its files and the
    // compiler never see it.
    final language = ref.watch(appStoreProvider.select((s) => s.preferences.language));
    return MaterialApp(
      onGenerateTitle: (context) => context.l10n.appTitle,
      debugShowCheckedModeBanner: false,
      theme: macTheme(Brightness.light),
      darkTheme: macTheme(Brightness.dark),
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
      home: const StudioShell(),
    );
  }
}
