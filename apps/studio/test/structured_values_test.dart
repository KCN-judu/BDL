import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  final temp = pb.Representation(quantity: pb.Dim(temperature: 1));
  final readings = pb.Representation(list: temp);
  final climate = pb.Representation(
    pair: pb.PairRepresentation(
      first: temp,
      second: pb.Representation(boolean: pb.Unit()),
    ),
  );
  final maybe = pb.Representation(optional: temp);

  test('structured value forms have their own socket shapes and words', () {
    expect(socketKindOf(readings), SocketKind.collection);
    expect(socketKindOf(climate), SocketKind.grouped);
    expect(socketKindOf(maybe), SocketKind.optional);
    expect(socketKindOf(temp), SocketKind.quantity);
    expect(representationWords(readings), 'a collection of values (a quantity)');
    expect(representationWords(climate), 'a grouped value (a quantity and on or off)');
    expect(representationWords(maybe), 'an optional value (a quantity)');
  });

  test('simulation inputs for structured forms read and render the same text', () {
    final list = parseValue(readings, '[20, 25.5, 28]')!;
    expect(list.list.items.map((v) => v.quantity.value), [20.0, 25.5, 28.0]);
    expect(renderValue(list), '[20, 25.5, 28]');
    expect(parseValue(readings, '[]')!.list.items, isEmpty);
    final pair = parseValue(climate, '(21, true)')!;
    expect(pair.pair.fst.quantity.value, 21.0);
    expect(pair.pair.snd.boolean, isTrue);
    expect(renderValue(pair), '(21, true)');
    expect(parseValue(maybe, 'none')!.hasNone(), isTrue);
    expect(renderValue(parseValue(maybe, 'some(3)')!), 'some(3)');
    // the wrong shape is refused, never guessed
    expect(parseValue(readings, '20'), isNull);
    expect(parseValue(climate, '(21)'), isNull);
    expect(parseValue(climate, '(21, maybe)'), isNull);
    expect(parseValue(maybe, 'some()'), isNull);
    // nesting follows the form
    final nested = pb.Representation(list: climate);
    expect(renderValue(parseValue(nested, '[(1, true), (2, false)]')!), '[(1, true), (2, false)]');
  });
}
