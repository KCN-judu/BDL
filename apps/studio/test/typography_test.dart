/// The type scale is the design system's (docs/architecture/studio-ui.md §3,
/// `MacType`): every text in `lib/ui` takes one of its roles.  A literal font
/// size in a widget is a fifth size waiting to happen, so none may appear
/// outside the files that own the scale and the welcome hero.
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test("no literal font size outside the scale's owners", () {
    const owners = {
      'lib/ui/mac/tokens.dart',
      'lib/ui/mac/theme.dart',
      'lib/ui/welcome/hero_mark.dart',
    };
    final literal = RegExp(r'fontSize:\s*[0-9]');
    final offenders = <String>[];
    for (final f in Directory('lib/ui').listSync(recursive: true).whereType<File>()) {
      final path = f.path.replaceAll('\\', '/');
      if (!path.endsWith('.dart') || owners.contains(path)) continue;
      final lines = f.readAsLinesSync();
      for (var i = 0; i < lines.length; i++) {
        if (literal.hasMatch(lines[i])) offenders.add('$path:${i + 1}: ${lines[i].trim()}');
      }
    }
    expect(offenders, isEmpty, reason: 'use MacType.body / secondary / caption / code / nodeTitle');
  });
}
