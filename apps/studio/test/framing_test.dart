import 'dart:typed_data';

import 'package:bdl_studio/daemon/framing.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('frames round-trip through byte-at-a-time feeding', () {
    final a = encodeFrame([1, 2, 3]);
    final b = encodeFrame(List.filled(1000, 9));
    final all = Uint8List.fromList([...a, ...b]);
    final decoder = FrameDecoder();
    final out = <Uint8List>[];
    for (final byte in all) {
      out.addAll(decoder.feed([byte]));
    }
    expect(out.length, 2);
    expect(out[0], [1, 2, 3]);
    expect(out[1].length, 1000);
  });

  test('oversized prefix is rejected', () {
    final header = Uint8List(4)..buffer.asByteData().setUint32(0, 0xFFFFFFFF, Endian.big);
    expect(() => FrameDecoder().feed(header), throwsA(isA<FrameTooLarge>()));
  });
}
