import 'dart:io';

import 'package:path/path.dart' as p;

import '../platform/desktop.dart';

/// Where to find the `bdld` binary.
///
/// Resolution order: `--dart-define=BDLD_PATH=…`, the `BDLD_PATH` environment
/// variable, a `bdld` next to the Studio executable, then the workspace
/// `target/debug/bdld` for development (`just studio` builds it first).
String locateDaemon() {
  const defined = String.fromEnvironment('BDLD_PATH');
  if (defined.isNotEmpty) return defined;
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && env.isNotEmpty) return env;
  final beside = p.join(p.dirname(Platform.resolvedExecutable), daemonExecutableName);
  if (File(beside).existsSync()) return beside;
  // apps/studio → workspace root
  final dev = p.normalize(
    p.join(Directory.current.path, '..', '..', 'target', 'debug', daemonExecutableName),
  );
  if (File(dev).existsSync()) return dev;
  return daemonExecutableName;
}
