/// The syntax theme: what a token *class* looks like in Studio's text.
///
/// The IDE service says what each span is, by the LSP semantic-token
/// names (`docs/architecture/syntax-highlighting.md`); this file, and
/// only this file, says what that looks like.  No colour crosses the
/// wire, and no class is decided here — an unknown name is left plain.
///
/// The palette is restrained and derived from the canvas's own encoding
/// (`semantic-ui.md`, channel table): a name's ink carries the *category*
/// of the thing it names, in the same hue family as that category's node
/// header tint — concept, relationship, Source, output, instance — so the
/// text and the graph agree without a legend.  Keywords and operators are
/// structure and recede into the secondary ink; comments into the
/// tertiary; a unit is the secondary ink beside its number.  Weight marks
/// a declaration (the title of its item — the "what to look for"
/// channel); italic marks a name bound in the formula itself (a parameter
/// or binder), which belongs to no project object.  A slot `?` is the
/// *open* colour, the text form of the dashed hollow chip: a decision
/// still to be made.  Every colour has its non-colour partner: the
/// keyword that declares the thing (`concept`, `mapping`, `output`…), the
/// shape of its use, the hover card's word, the weight or slant.
/// Contrast is checked for both appearances.
library;

import 'package:flutter/material.dart';

import '../../app/state.dart';
import '../mac/tokens.dart';

class SyntaxTheme {
  const SyntaxTheme({
    required this.concept,
    required this.relationship,
    required this.source,
    required this.output,
    required this.instance,
    required this.structure,
    required this.comment,
    required this.slot,
    required this.plain,
  });

  /// A concept's name (`type`).
  final Color concept;

  /// A relationship's name: a rule (`function`) or a value (`variable`),
  /// a clock (`namespace`), a component (`class`), a port (`property`).
  final Color relationship;

  /// A Source (`variable` + `source`): the environment's side.
  final Color source;

  /// An output or a device (`variable` + `output` / `device`): the
  /// physical boundary.
  final Color output;

  /// An instance (`variable` + `instance`).
  final Color instance;

  /// Keywords, operators, units.
  final Color structure;
  final Color comment;

  /// The `?` slot: a decision still to be made.
  final Color slot;

  /// Everything else: numbers, parameters, library functions, punctuation.
  final Color plain;

  /// The theme for the current appearance, from the window's tokens: the
  /// category hues are the node header tints' hues at an ink lightness.
  factory SyntaxTheme.of(MacTokens t) {
    Color ink(Color tint) {
      final h = HSLColor.fromColor(tint);
      return h
          .withSaturation(t.isDark ? 0.45 : 0.55)
          .withLightness(t.isDark ? 0.74 : 0.30)
          .toColor();
    }

    return SyntaxTheme(
      concept: ink(t.isDark ? const Color(0xFF3A4556) : const Color(0xFFDCE3EE)),
      relationship: ink(t.isDark ? const Color(0xFF2E4A6B) : const Color(0xFFCFE0F5)),
      source: ink(t.isDark ? const Color(0xFF2F5A4A) : const Color(0xFFD2ECDD)),
      output: ink(t.isDark ? const Color(0xFF4A4030) : const Color(0xFFEFE3CF)),
      instance: ink(t.isDark ? const Color(0xFF2F4F4A) : const Color(0xFFD0E6E1)),
      structure: t.textSecondary,
      comment: t.textTertiary,
      slot: t.open,
      plain: t.textPrimary,
    );
  }

  /// The style of one span over [base] (the editor's own style).  A name
  /// the theme does not know is [base] unchanged.
  TextStyle styleOf(HighlightSpan span, TextStyle base) {
    final m = span.modifiers;
    final declaration = m.contains('declaration');
    TextStyle named(Color color) =>
        base.copyWith(color: color, fontWeight: declaration ? FontWeight.w600 : base.fontWeight);
    return switch (span.type) {
      'type' => named(concept),
      'function' || 'namespace' || 'class' || 'property' || 'enumMember' =>
        m.contains('defaultLibrary') ? base.copyWith(color: plain) : named(relationship),
      'variable' when m.contains('source') => named(source),
      'variable' when m.contains('output') || m.contains('device') => named(output),
      'variable' when m.contains('instance') => named(instance),
      'variable' => named(relationship),
      'parameter' => base.copyWith(
        color: plain,
        fontStyle: FontStyle.italic,
        fontWeight: declaration ? FontWeight.w600 : base.fontWeight,
      ),
      'keyword' || 'operator' || 'unit' => base.copyWith(color: structure),
      'comment' => base.copyWith(color: comment),
      'number' => base.copyWith(color: plain),
      'slot' => base.copyWith(color: slot, fontWeight: FontWeight.w600),
      _ => base,
    };
  }
}
