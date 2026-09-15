/// Persistence of the recent-projects list in the per-user application
/// support directory.  App preference, never project data.
library;

import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../app/state.dart';

const int _schemaVersion = 1;

/// `~/Library/Application Support/BDL Studio` on macOS, `%APPDATA%\BDL Studio`
/// on Windows, `$XDG_CONFIG_HOME/bdl-studio` (or `~/.config/bdl-studio`) elsewhere.
Directory appSupportDir() {
  final env = Platform.environment;
  final home = env['HOME'] ?? env['USERPROFILE'] ?? '.';
  if (Platform.isMacOS) {
    return Directory(p.join(home, 'Library', 'Application Support', 'BDL Studio'));
  }
  if (Platform.isWindows) return Directory(p.join(env['APPDATA'] ?? home, 'BDL Studio'));
  return Directory(p.join(env['XDG_CONFIG_HOME'] ?? p.join(home, '.config'), 'bdl-studio'));
}

class RecentStore {
  RecentStore({Directory? dir})
    : _file = File(p.join((dir ?? appSupportDir()).path, 'recent.json'));
  final File _file;

  Future<List<RecentProject>> load() async {
    try {
      if (!await _file.exists()) return const [];
      final json = jsonDecode(await _file.readAsString());
      if (json is! Map || json['schema_version'] != _schemaVersion) return const [];
      final items = json['projects'];
      if (items is! List) return const [];
      return items.map(RecentProject.fromJson).whereType<RecentProject>().toList();
    } catch (_) {
      // A corrupt preference file must never stop the app from starting.
      return const [];
    }
  }

  Future<void> save(List<RecentProject> recent) async {
    await _file.parent.create(recursive: true);
    final tmp = File('${_file.path}.tmp');
    await tmp.writeAsString(
      const JsonEncoder.withIndent('  ').convert({
        'schema_version': _schemaVersion,
        'projects': recent.map((r) => r.toJson()).toList(),
      }),
      flush: true,
    );
    await tmp.rename(_file.path);
  }
}
