/// Preferences: application-level choices that belong to the person, not
/// to a project.  Today: the display language.  Changing it re-renders
/// the UI in place; the project, its files and the compiler are untouched.
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../l10n/l10n.dart';
import 'dialogs.dart' show showMacSheet;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

Future<void> showPreferencesSheet(
  BuildContext context, {
  required void Function(AppAction) dispatch,
}) {
  // Title and button are built per frame: choosing a language re-renders
  // the sheet that offered it, chrome included.
  return showMacSheet<void>(
    context,
    title: context.l10n.preferencesTitle,
    titleOf: (ctx) => ctx.l10n.preferencesTitle,
    width: 420,
    content: _PreferencesBody(dispatch: dispatch),
    actions: const [],
    actionsOf: (ctx) => [
      MacButton.primary(label: ctx.l10n.done, onPressed: () => Navigator.pop(ctx)),
    ],
  );
}

class _PreferencesBody extends StatefulWidget {
  const _PreferencesBody({required this.dispatch});
  final void Function(AppAction) dispatch;

  @override
  State<_PreferencesBody> createState() => _PreferencesBodyState();
}

class _PreferencesBodyState extends State<_PreferencesBody> {
  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final t = MacTokens.of(context);
    // The sheet reads the store through its own context so the menu shows
    // the value just chosen even before the surrounding app re-renders.
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        FormRow(
          label: l10n.languageLabel,
          child: LanguageDropdown(dispatch: widget.dispatch),
        ),
        const SizedBox(height: 8),
        Text(
          l10n.languageHelp,
          style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
        ),
      ],
    );
  }
}

/// The language menu: *System default* plus the three locales, each named
/// in its own language so it can be found whatever the current one.
class LanguageDropdown extends StatelessWidget {
  const LanguageDropdown({super.key, required this.dispatch, this.value});
  final void Function(AppAction) dispatch;

  /// The preference to show; when null, read from the nearest store.
  final LanguagePreference? value;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final current = value ?? _LanguageScope.of(context);
    String label(LanguagePreference p) => switch (p) {
      LanguagePreference.system => l10n.languageSystemDefault,
      _ => endonym(p.locale!),
    };
    return MacDropdown<LanguagePreference>(
      value: current,
      items: LanguagePreference.values,
      labelOf: label,
      onChanged: (p) => dispatch(LanguageChanged(p)),
    );
  }
}

/// Where the language dropdown reads its current value when not given one:
/// the app root provides it.
class _LanguageScope extends InheritedWidget {
  const _LanguageScope({required this.language, required super.child});
  final LanguagePreference language;

  static LanguagePreference of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<_LanguageScope>()?.language ??
      LanguagePreference.system;

  @override
  bool updateShouldNotify(_LanguageScope old) => old.language != language;
}

/// Wrap the app so the preferences sheet can read the current language.
class LanguageScope extends StatelessWidget {
  const LanguageScope({super.key, required this.language, required this.child});
  final LanguagePreference language;
  final Widget child;

  @override
  Widget build(BuildContext context) => _LanguageScope(language: language, child: child);
}
