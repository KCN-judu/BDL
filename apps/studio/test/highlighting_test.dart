/// Semantic highlighting: the reducer (generations, stale answers, byte
/// ranges to code units), the shift of spans across an edit, the theme's
/// mapping of the LSP names, the controller's rendering, and the
/// definition editor asking for its tokens.  No test here tokenizes any
/// text: every span is the service's.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/highlighting.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/code/highlighting_controller.dart';
import 'package:bdl_studio/ui/code/syntax_theme.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

const tilt = 0;
const brightness = 1;
const dim = 0;

pb.ProjectProjection lamp({int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(tilt), name: 'Tilt'),
        pb.ConceptView(id: Int64(brightness), name: 'Brightness'),
      ])
      ..mappings.add(
        mappingView(
          id: Int64(dim),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
        ),
      );

AppState connected(pb.ProjectProjection project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: const EditorState(
    selection: MappingSelected(dim),
    composer: ComposerState(formulaMode: false),
  ),
);

/// The legend as bdl-ide states it (`LEGEND_VERSION` 1).
pb.SemanticTokenLegend legend() => pb.SemanticTokenLegend(
  version: 1,
  types: const [
    'namespace',
    'type',
    'class',
    'function',
    'variable',
    'parameter',
    'property',
    'enumMember',
    'keyword',
    'number',
    'operator',
    'comment',
    'unit',
    'slot',
  ],
  modifiers: const [
    'declaration',
    'defaultLibrary',
    'source',
    'output',
    'device',
    'instance',
    'unresolved',
  ],
);

int typeIndex(String name) => legend().types.indexOf(name);
int modifierBits(List<String> names) =>
    names.fold(0, (bits, n) => bits | (1 << legend().modifiers.indexOf(n)));

pb.SemanticToken token(int start, int end, String type, [List<String> mods = const []]) =>
    pb.SemanticToken(
      start: start,
      end: end,
      tokenType: typeIndex(type),
      tokenModifiers: modifierBits(mods),
    );

pb.SemanticTokensResponse answer(
  String text,
  int generation,
  List<pb.SemanticToken> tokens, {
  int revision = 1,
}) => pb.SemanticTokensResponse(
  revision: Int64(revision),
  generation: Int64(generation),
  legend: legend(),
  textLen: text.codeUnits.length == text.length ? utf8Length(text) : utf8Length(text),
  tokens: tokens,
);

int utf8Length(String s) => s.runes.fold(0, (n, r) {
  if (r < 0x80) return n + 1;
  if (r < 0x800) return n + 2;
  if (r < 0x10000) return n + 3;
  return n + 4;
});

Finder get definitionField => find.descendant(
  of: find.byKey(const ValueKey('definition-field')),
  matching: find.byType(TextField),
);

HighlightSpan span(int start, int end, String type, [Set<String> mods = const {}]) =>
    HighlightSpan(start, end, type, mods);

void main() {
  group('the reducer', () {
    test('a request tags a generation per document and asks the service with the text', () {
      final t = reduce(
        connected(lamp()),
        const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt / 90 deg'),
      );
      final key = HighlightState.formulaKey(dim, null);
      final h = t.state.editor.highlights[key]!;
      expect(h.generation, 1);
      expect(h.pending, isTrue);
      expect(h.requested, 'Tilt / 90 deg');
      expect(h.spans, isEmpty);
      final e = t.effects.single as FetchSemanticTokens;
      expect(e.key, key);
      expect(e.generation, 1);
      expect(e.mappingId, dim);
      expect(e.path, isNull);
      expect(e.text, 'Tilt / 90 deg');
      // a file document has its own key
      final f = reduce(
        t.state,
        const SemanticTokensRequested.file(path: 'src/main.bdl', text: 'concept Tilt : Angle'),
      );
      expect(f.state.editor.highlights.keys, containsAll([key, 'file:src/main.bdl']));
      expect((f.effects.single as FetchSemanticTokens).path, 'src/main.bdl');
    });

    test('the same text again is not asked twice', () {
      var s = connected(lamp());
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).state;
      final again = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt'));
      expect(again.effects, isEmpty);
      // answered, and asked once more with the same text: still nothing
      final key = HighlightState.formulaKey(dim, null);
      s = reduce(
        s,
        SemanticTokensReceived(key: key, result: answer('Tilt', 1, [token(0, 4, 'type')])),
      ).state;
      expect(
        reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).effects,
        isEmpty,
      );
    });

    test('only the latest generation lands; an older answer is dropped', () {
      var s = connected(lamp());
      const key = 'formula:-:$dim';
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Ti')).state;
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).state;
      expect(s.editor.highlights[key]!.generation, 2);
      // the answer to 'Ti' arrives late
      s = reduce(
        s,
        SemanticTokensReceived(key: key, result: answer('Ti', 1, [token(0, 2, 'type')])),
      ).state;
      expect(s.editor.highlights[key]!.spans, isEmpty);
      expect(s.editor.highlights[key]!.pending, isTrue);
      // the answer to 'Tilt' lands
      s = reduce(
        s,
        SemanticTokensReceived(key: key, result: answer('Tilt', 2, [token(0, 4, 'type')])),
      ).state;
      final h = s.editor.highlights[key]!;
      expect(h.pending, isFalse);
      expect(h.text, 'Tilt');
      expect(h.spans, [span(0, 4, 'type')]);
      expect(h.legendVersion, 1);
    });

    test('an answer over a different length than asked is not applied', () {
      var s = connected(lamp());
      const key = 'formula:-:$dim';
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).state;
      final wrong = answer('Tilt', 1, [token(0, 4, 'type')])..textLen = 3;
      s = reduce(s, SemanticTokensReceived(key: key, result: wrong)).state;
      expect(s.editor.highlights[key]!.spans, isEmpty);
      expect(s.editor.highlights[key]!.pending, isFalse);
    });

    test('a failure keeps what is on show and clears pending for its generation only', () {
      var s = connected(lamp());
      const key = 'formula:-:$dim';
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).state;
      s = reduce(
        s,
        SemanticTokensReceived(key: key, result: answer('Tilt', 1, [token(0, 4, 'type')])),
      ).state;
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt /')).state;
      final stale = reduce(s, const SemanticTokensFailed(key: key, generation: 1));
      expect(stale.state.editor.highlights[key]!.pending, isTrue);
      final now = reduce(s, const SemanticTokensFailed(key: key, generation: 2));
      final h = now.state.editor.highlights[key]!;
      expect(h.pending, isFalse);
      expect(h.spans, [span(0, 4, 'type')]);
      expect(h.text, 'Tilt');
    });

    test('byte ranges become code-unit ranges and indices become names', () {
      // ångström is 8 chars, 10 bytes; 𝛼 is 2 code units, 4 bytes
      const text = 'ångström + 𝛼 ?';
      final r = answer(text, 1, [
        token(0, 10, 'variable', ['source']),
        token(11, 12, 'operator'),
        token(13, 17, 'parameter', ['declaration']),
        token(18, 19, 'slot'),
        pb.SemanticToken(start: 0, end: 1, tokenType: 99), // unknown type: skipped
      ]);
      final spans = spansOf(text, r);
      expect(spans, [
        span(0, 8, 'variable', {'source'}),
        span(9, 10, 'operator'),
        span(11, 13, 'parameter', {'declaration'}),
        span(14, 15, 'slot'),
      ]);
      expect(text.substring(11, 13), '𝛼');
    });

    test('highlights survive a pushed projection and a new revision', () {
      var s = connected(lamp());
      const key = 'formula:-:$dim';
      s = reduce(s, const SemanticTokensRequested.formula(mappingId: dim, text: 'Tilt')).state;
      s = reduce(
        s,
        SemanticTokensReceived(key: key, result: answer('Tilt', 1, [token(0, 4, 'type')])),
      ).state;
      s = reduce(s, ProjectReceived(lamp(revision: 2), fromRequest: false)).state;
      expect(s.editor.highlights[key]!.spans, [span(0, 4, 'type')]);
    });
  });

  group('shifting spans across an edit', () {
    final spans = [
      span(0, 4, 'type'),
      span(5, 6, 'operator'),
      span(7, 9, 'number'),
      span(10, 13, 'unit'),
    ];
    const from = 'Tilt / 90 deg';

    test(
      'an insertion in the middle keeps the prefix, moves the suffix, drops the touched span',
      () {
        // 'Tilt / 900 deg'
        final out = shiftSpans(from, 'Tilt / 900 deg', spans);
        expect(out, [span(0, 4, 'type'), span(5, 6, 'operator'), span(11, 14, 'unit')]);
      },
    );

    test('typing at the end keeps everything but the word being typed into', () {
      final out = shiftSpans(from, '$from + 1', spans);
      expect(out, spans.sublist(0, 3));
      // a space first: the unit is whole again once the service answers;
      // until then it is plain, never wrongly extended
      expect(shiftSpans(from, '$from ', spans), spans.sublist(0, 3));
    });

    test('typing at the start moves everything but the word being typed into', () {
      final out = shiftSpans(from, '- $from', spans);
      expect(out, [for (final s in spans.sublist(1)) s.shifted(2)]);
    });

    test('a deletion never leaves a span past the end', () {
      expect(shiftSpans(from, 'Tilt', spans), isEmpty);
      final out = shiftSpans(from, 'Tilt /', spans);
      expect(out, [span(0, 4, 'type')]);
      for (final s in out) {
        expect(s.end, lessThanOrEqualTo(6));
      }
    });

    test('a replacement of the whole text leaves nothing', () {
      expect(shiftSpans(from, 'Brightness', spans), isEmpty);
    });

    test('a repeated character at the edit point is not double-counted', () {
      // 'aa' -> 'aaa': prefix 2, suffix bounded to 0
      final out = shiftSpans('aa', 'aaa', [span(0, 2, 'type')]);
      expect(out, isEmpty);
    });

    test('the same text is the same spans', () {
      expect(identical(shiftSpans(from, from, spans), spans), isTrue);
    });
  });

  group('the theme', () {
    final light = SyntaxTheme.of(MacTokens.light);
    final dark = SyntaxTheme.of(MacTokens.dark);
    const base = TextStyle(fontSize: 12, fontFamily: 'Menlo');

    test('names carry their category; structure recedes; unknown names stay plain', () {
      for (final theme in [light, dark]) {
        final concept = theme.styleOf(span(0, 1, 'type'), base);
        final rule = theme.styleOf(span(0, 1, 'function'), base);
        final value = theme.styleOf(span(0, 1, 'variable'), base);
        final source = theme.styleOf(span(0, 1, 'variable', {'source'}), base);
        final output = theme.styleOf(span(0, 1, 'variable', {'output'}), base);
        final instance = theme.styleOf(span(0, 1, 'variable', {'instance'}), base);
        expect(concept.color, theme.concept);
        expect(rule.color, theme.relationship);
        expect(value.color, theme.relationship);
        expect(source.color, theme.source);
        expect(output.color, theme.output);
        expect(instance.color, theme.instance);
        expect(
          {theme.concept, theme.relationship, theme.source, theme.output, theme.instance}.length,
          5,
        );
        expect(theme.styleOf(span(0, 1, 'keyword'), base).color, theme.structure);
        expect(theme.styleOf(span(0, 1, 'operator'), base).color, theme.structure);
        expect(theme.styleOf(span(0, 1, 'unit'), base).color, theme.structure);
        expect(theme.styleOf(span(0, 1, 'comment'), base).color, theme.comment);
        expect(theme.styleOf(span(0, 1, 'number'), base).color, theme.plain);
        expect(theme.styleOf(span(0, 1, 'slot'), base).color, theme.slot);
        expect(theme.styleOf(span(0, 1, 'something-new'), base), base);
      }
    });

    test('a declaration is weight, a parameter is slant, a library function is plain', () {
      final decl = light.styleOf(span(0, 1, 'type', {'declaration'}), base);
      expect(decl.fontWeight, FontWeight.w600);
      expect(light.styleOf(span(0, 1, 'type'), base).fontWeight, isNull);
      final param = light.styleOf(span(0, 1, 'parameter'), base);
      expect(param.fontStyle, FontStyle.italic);
      expect(param.color, light.plain);
      final lib = light.styleOf(span(0, 1, 'function', {'defaultLibrary'}), base);
      expect(lib.color, light.plain);
      expect(lib.fontWeight, isNull);
    });

    test('the category inks read on both appearances', () {
      double contrast(Color a, Color b) {
        final la = a.computeLuminance();
        final lb = b.computeLuminance();
        final hi = la > lb ? la : lb;
        final lo = la > lb ? lb : la;
        return (hi + 0.05) / (lo + 0.05);
      }

      for (final (theme, tokens) in [(light, MacTokens.light), (dark, MacTokens.dark)]) {
        for (final c in [
          theme.concept,
          theme.relationship,
          theme.source,
          theme.output,
          theme.instance,
        ]) {
          expect(contrast(c, tokens.content), greaterThanOrEqualTo(4.5), reason: '$c');
        }
      }
    });
  });

  group('the controller', () {
    testWidgets('draws the spans through the theme and the marks on top, shifted while typing', (
      t,
    ) async {
      final c = HighlightingController(text: 'Tilt / 90 deg');
      c.theme = SyntaxTheme.of(MacTokens.light);
      c.setHighlight(
        HighlightState(
          key: 'k',
          generation: 1,
          requested: 'Tilt / 90 deg',
          text: 'Tilt / 90 deg',
          spans: [span(0, 4, 'type'), span(7, 9, 'number'), span(10, 13, 'unit')],
        ),
      );
      c.marks = [(const TextRange(start: 10, end: 13), Colors.red)];
      late BuildContext ctx;
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              ctx = context;
              return const SizedBox();
            },
          ),
        ),
      );
      const base = TextStyle(fontSize: 12);
      final s = c.buildTextSpan(context: ctx, style: base, withComposing: false);
      final pieces = s.children!.cast<TextSpan>();
      expect(pieces.map((p) => p.text).join(), 'Tilt / 90 deg');
      final tiltPiece = pieces.firstWhere((p) => p.text == 'Tilt');
      expect(tiltPiece.style!.color, SyntaxTheme.of(MacTokens.light).concept);
      final unit = pieces.firstWhere((p) => p.text == 'deg');
      expect(unit.style!.decoration, TextDecoration.underline);
      expect(unit.style!.color, SyntaxTheme.of(MacTokens.light).structure);
      // the designer types in the middle: the spans shift, nothing stale
      c.value = const TextEditingValue(text: 'Tilt / 900 deg');
      c.marks = const [];
      final shifted = c.buildTextSpan(context: ctx, style: base, withComposing: false);
      final after = shifted.children!.cast<TextSpan>();
      expect(after.map((p) => p.text).join(), 'Tilt / 900 deg');
      expect(after.firstWhere((p) => p.text == 'Tilt').style!.color, isNotNull);
      expect(after.firstWhere((p) => p.text == 'deg').style!.color, isNotNull);
      final middle = after.firstWhere((p) => p.text!.contains('900'));
      expect(middle.style!.color, isNull);
    });

    testWidgets('the definition editor asks for its tokens and colours the answer', (t) async {
      final effects = <Effect>[];
      var state = connected(lamp());
      late StateSetter refresh;
      await t.pumpWidget(
        MaterialApp(
          theme: macTheme(Brightness.light),
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                refresh = setState;
                return SizedBox(
                  width: 320,
                  height: 900,
                  child: Inspector(
                    state: state,
                    dispatch: (a) {
                      final tr = reduce(state, a);
                      state = tr.state;
                      effects.addAll(tr.effects);
                      setState(() {});
                    },
                  ),
                );
              },
            ),
          ),
        ),
      );
      await t.pump();
      final asked = effects.whereType<FetchSemanticTokens>().toList();
      expect(asked, hasLength(1));
      expect(asked.single.mappingId, dim);
      expect(asked.single.text, '');
      // the designer types; after the pause the new text is asked about
      await t.enterText(definitionField, 'Tilt / 90 deg');
      await t.pump(kHighlightPause + const Duration(milliseconds: 10));
      final again = effects.whereType<FetchSemanticTokens>().toList();
      expect(again.last.text, 'Tilt / 90 deg');
      expect(again.last.generation, 2);
      // the service answers: the field is coloured
      final key = HighlightState.formulaKey(dim, null);
      state = reduce(
        state,
        SemanticTokensReceived(
          key: key,
          result: answer('Tilt / 90 deg', 2, [token(0, 4, 'type'), token(10, 13, 'unit')]),
        ),
      ).state;
      refresh(() {});
      await t.pump();
      final field = t.widget<TextField>(definitionField);
      final c = field.controller as HighlightingController;
      expect(c.spans, [span(0, 4, 'type'), span(10, 13, 'unit')]);
    });
  });
}
