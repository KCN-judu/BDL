/// Application preferences: the user's environment, not a project's state.
/// Persisted by the effect executor (`effects/preferences_store.dart`);
/// pure data here so the reducer and its tests need no I/O.
library;

import 'dart:ui' show Locale;

import 'package:flutter/foundation.dart';

/// The locales Studio offers.  Order is the order of the language menu.
const List<Locale> kSupportedLocales = [
  Locale('en'),
  Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans'),
  Locale('ja'),
];

/// The language preference as the user chooses it: follow the system, or
/// one of the three locales.  Persisted in `preferences.json`.
enum LanguagePreference {
  system,
  en,
  zhHans,
  ja;

  /// The locale to hand `MaterialApp`; `null` follows the system.
  Locale? get locale => switch (this) {
    LanguagePreference.system => null,
    LanguagePreference.en => kSupportedLocales[0],
    LanguagePreference.zhHans => kSupportedLocales[1],
    LanguagePreference.ja => kSupportedLocales[2],
  };

  /// The stored spelling (BCP 47 for a locale, `system` otherwise).
  String get tag => switch (this) {
    LanguagePreference.system => 'system',
    LanguagePreference.en => 'en',
    LanguagePreference.zhHans => 'zh-Hans',
    LanguagePreference.ja => 'ja',
  };

  static LanguagePreference fromTag(String? tag) => switch (tag) {
    'en' => LanguagePreference.en,
    'zh-Hans' => LanguagePreference.zhHans,
    'ja' => LanguagePreference.ja,
    _ => LanguagePreference.system,
  };
}

@immutable
class AppPreferences {
  const AppPreferences({this.language = LanguagePreference.system});
  final LanguagePreference language;

  AppPreferences copyWith({LanguagePreference? language}) =>
      AppPreferences(language: language ?? this.language);

  @override
  bool operator ==(Object other) => other is AppPreferences && other.language == language;

  @override
  int get hashCode => language.hashCode;
}
