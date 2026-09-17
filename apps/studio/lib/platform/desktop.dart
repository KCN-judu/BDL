/// Platform facts the UI adapts to.  Studio targets macOS and Windows
/// (Linux builds too); everything here is a *detail* of one design, not a
/// second design per platform (docs/architecture/studio-ui.md §3).
library;

import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

final bool isMacOS = Platform.isMacOS;
final bool isWindows = Platform.isWindows;

/// The primary shortcut modifier: ⌘ on macOS, Ctrl elsewhere.
SingleActivator primary(LogicalKeyboardKey key, {bool shift = false}) =>
    SingleActivator(key, meta: isMacOS, control: !isMacOS, shift: shift);

/// How the modifier is written in tooltips.
String shortcut(String key, {bool shift = false}) =>
    isMacOS ? '${shift ? '⇧' : ''}⌘$key' : 'Ctrl+${shift ? 'Shift+' : ''}$key';

/// Name of the daemon executable on this platform.
String get daemonExecutableName => isWindows ? 'bdld.exe' : 'bdld';
