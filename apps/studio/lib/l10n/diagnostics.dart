/// Localized presentation of diagnostics and errors, by their stable code.
///
/// A code is identity and never changes with the locale; the sentence a
/// person reads is presentation.  Studio renders a sentence itself only
/// for a code whose meaning the code alone determines — nothing in it
/// names an entity Studio would have to pick out of the daemon's English
/// text.  Every other diagnostic keeps the message the daemon composed
/// (ISS-0011 records that remainder); the code stays beside it either way.
library;

import 'dart:ui' show Locale;

import 'l10n.dart';

/// Former spellings of diagnostic codes and their current names — the
/// `bdl_diagnostics::CODE_ALIASES` table (ADR-0043): an old log or fixture
/// still names the same finding.
const Map<String, String> kDiagnosticCodeAliases = {
  'semantic.concept_mismatch': 'concept.mismatch',
  'semantic.no_order': 'concept.no_order',
  'semantic.unbound_representation': 'concept.unbound_representation',
  'semantic.construction_not_granted': 'concept.construction_not_granted',
  'type.rep_of_non_semantic': 'type.rep_of_non_concept',
};

/// The current name of [code]: itself, or the name an old spelling maps to.
String canonicalDiagnosticCode(String code) => kDiagnosticCodeAliases[code] ?? code;

/// The sentence to show for [code], or [fallback] (the daemon's message)
/// when the code is not one Studio words itself.
String localizedMessage(AppLocalizations l10n, String code, String fallback) =>
    switch (canonicalDiagnosticCode(code)) {
      'studio.not_connected' => l10n.diagNotConnected,
      'studio.transport' => l10n.diagTransport,
      'studio.picker_unavailable' => l10n.diagPickerUnavailable,
      'project.changed_on_disk' => l10n.diagChangedOnDisk,
      'project.persist' => l10n.diagPersist,
      'edit.stale_revision' => l10n.diagStaleRevision,
      'group_edit.stale_generation' => l10n.diagStaleRevision,
      'edit.nothing_to_undo' => l10n.diagNothingToUndo,
      'edit.nothing_to_redo' => l10n.diagNothingToRedo,
      'session.no_project' => l10n.diagNoProject,
      'session.already_open' => l10n.diagAlreadyOpen,
      'system.not_a_system' => l10n.diagNotASystem,
      'simulation.not_started' => l10n.diagSimulationNotStarted,
      'simulation.not_causal' => l10n.theDesignContainsAnInstantaneousCycle,
      _ => fallback,
    };

/// Whether [code] is one Studio words itself (so the daemon's text is a
/// detail, not the headline).
bool isStudioWorded(String code) => localizedMessage(kEnglish, code, '') != '';

/// A finding about the mapping's place in the design rather than about
/// its definition text — a rule nothing applies — which no text typed into
/// the definition can change: the definition editor leaves it to the
/// inspector's Relationship section, whatever draft is being tried.
bool isPlacementNote(String code) => code == 'reactive.rule_unapplied';

/// The catalog for a language preference outside the widget tree (the
/// effect executor's OS dialogs): the chosen locale, or the platform's
/// resolved to a supported one.
AppLocalizations catalogFor(LanguagePreference language, Locale platform) =>
    lookupAppLocalizations(resolveLocale(language.locale ?? platform, kSupportedLocales));
