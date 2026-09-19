/// Locale handling for Studio (docs/project/localization-style.md).
///
/// Exactly three locales are a product surface: English (the canonical
/// source), Simplified Chinese and Japanese.  Any other system locale
/// resolves to English; a key missing from a translation falls back to
/// English through gen-l10n's class hierarchy, so a raw key is never shown.
/// Locale is user-environment state (an application preference), never
/// project state: BDL source, protocol identities and diagnostic codes do
/// not change with it.
library;

import 'package:flutter/widgets.dart';

import '../app/preferences.dart';
import 'app_localizations.dart';

export '../app/preferences.dart';
export 'app_localizations.dart';

/// Map any locale the platform reports to one of the three supported
/// locales, or English.  `zh` in any Simplified form (`zh`, `zh-CN`,
/// `zh-SG`, `zh-Hans-*`) is Simplified Chinese; Traditional Chinese is not a
/// supported locale in this milestone and resolves to English, as does
/// everything else.
Locale resolveLocale(Locale? requested, Iterable<Locale> supported) {
  if (requested == null) return kSupportedLocales[0];
  switch (requested.languageCode) {
    case 'en':
      return kSupportedLocales[0];
    case 'ja':
      return kSupportedLocales[2];
    case 'zh':
      final script = requested.scriptCode;
      final country = requested.countryCode;
      final traditional =
          script == 'Hant' || (script == null && const {'TW', 'HK', 'MO'}.contains(country));
      return traditional ? kSupportedLocales[0] : kSupportedLocales[1];
    default:
      return kSupportedLocales[0];
  }
}

/// Localized strings for a build context.  Falls back to English when no
/// `Localizations` ancestor exists (a widget test that pumps a bare widget),
/// so a widget never has to check for `null` and never shows a key.
extension StudioL10n on BuildContext {
  AppLocalizations get l10n =>
      Localizations.of<AppLocalizations>(this, AppLocalizations) ?? kEnglish;
}

/// The English catalog, the canonical fallback.
final AppLocalizations kEnglish = lookupAppLocalizations(const Locale('en'));

/// How each supported locale names itself — shown in the language menu in
/// its own language, whatever the current locale.
String endonym(Locale locale) => switch (locale.languageCode) {
  'zh' => '简体中文',
  'ja' => '日本語',
  _ => 'English',
};
