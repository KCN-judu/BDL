import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../app/preferences.dart';
import 'recent_store.dart' show appSupportDir;

const _schemaVersion = 1;

/// Application preferences that are the user's, not a project's:
/// today the display language.  `preferences.json` beside `recent.json`
/// in the app-support directory; a corrupt or missing file means defaults.
class PreferencesStore {
  PreferencesStore({Directory? dir})
    : _file = File(p.join((dir ?? appSupportDir()).path, 'preferences.json'));
  final File _file;

  Future<AppPreferences> load() async {
    try {
      if (!await _file.exists()) return const AppPreferences();
      final json = jsonDecode(await _file.readAsString());
      if (json is! Map || json['schema_version'] != _schemaVersion) return const AppPreferences();
      return AppPreferences(language: LanguagePreference.fromTag(json['language'] as String?));
    } catch (_) {
      return const AppPreferences();
    }
  }

  Future<void> save(AppPreferences prefs) async {
    await _file.parent.create(recursive: true);
    final tmp = File('${_file.path}.tmp');
    await tmp.writeAsString(
      const JsonEncoder.withIndent('  ')
          .convert({'schema_version': _schemaVersion, 'language': prefs.language.tag}),
      flush: true,
    );
    await tmp.rename(_file.path);
  }
}
