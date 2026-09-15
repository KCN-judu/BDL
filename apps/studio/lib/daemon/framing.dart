/// Frame = 4-byte big-endian length prefix + protobuf payload.
///
/// Mirrors `crates/bdl-protocol/src/framing.rs` exactly.
library;

import 'dart:typed_data';

const int kHeaderLen = 4;
const int kMaxFrameLen = 64 * 1024 * 1024;

class FrameTooLarge implements Exception {
  const FrameTooLarge(this.length);
  final int length;
  @override
  String toString() => 'frame of $length bytes exceeds $kMaxFrameLen';
}

Uint8List encodeFrame(List<int> payload) {
  if (payload.length > kMaxFrameLen) throw FrameTooLarge(payload.length);
  final out = Uint8List(kHeaderLen + payload.length);
  ByteData.view(out.buffer).setUint32(0, payload.length, Endian.big);
  out.setRange(kHeaderLen, out.length, payload);
  return out;
}

/// Incremental decoder: feed chunks, take complete payloads.
class FrameDecoder {
  final BytesBuilder _buf = BytesBuilder(copy: false);
  Uint8List _pending = Uint8List(0);

  /// Feed a chunk and return every complete payload it completes.
  List<Uint8List> feed(List<int> chunk) {
    _buf.add(_pending);
    _buf.add(chunk);
    var bytes = _buf.takeBytes();
    final frames = <Uint8List>[];
    var offset = 0;
    while (bytes.length - offset >= kHeaderLen) {
      final len = ByteData.view(
        bytes.buffer,
        bytes.offsetInBytes + offset,
        kHeaderLen,
      ).getUint32(0, Endian.big);
      if (len > kMaxFrameLen) throw FrameTooLarge(len);
      if (bytes.length - offset < kHeaderLen + len) break;
      frames.add(Uint8List.sublistView(bytes, offset + kHeaderLen, offset + kHeaderLen + len));
      offset += kHeaderLen + len;
    }
    _pending = Uint8List.sublistView(bytes, offset);
    // Copy so the next feed does not alias the returned frames.
    _pending = Uint8List.fromList(_pending);
    return frames;
  }
}
