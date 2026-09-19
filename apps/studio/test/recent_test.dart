@Tags(['filesystem'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/effects/recent_store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/welcome/welcome_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

AppState connected() => AppState(
  connection: Connected(executable: 'bdld', handshake: pb.HandshakeResponse(compatible: true)),
);

pb.ProjectProjection projection(String root, {int revision = 0}) =>
    pb.ProjectProjection(revision: Int64(revision), name: root.split('/').last, rootPath: root);

void main() {
  test('opening a project puts it first in Recent and asks to persist', () {
    final s0 = connected().copyWith(
      recent: [RecentProject(path: '/a', name: 'a', lastOpened: DateTime(2026))],
    );
    final t = reduce(s0, ProjectReceived(projection('/b')));
    expect(t.state.recent.map((r) => r.path), ['/b', '/a']);
    expect(t.effects.whereType<SaveRecentProjects>().length, 1);

    // re-opening an existing one moves it to the top without duplicating
    final t2 = reduce(t.state.copyWith(clearProject: true), ProjectReceived(projection('/a')));
    expect(t2.state.recent.map((r) => r.path), ['/a', '/b']);

    // edits to the open project do not touch Recent
    final t3 = reduce(t2.state, ProjectReceived(projection('/a', revision: 1)));
    expect(t3.effects.whereType<SaveRecentProjects>(), isEmpty);
  });

  test('removing a recent entry persists', () {
    final s = connected().copyWith(
      recent: [RecentProject(path: '/a', name: 'a', lastOpened: DateTime(2026))],
    );
    final t = reduce(s, const RemoveRecentRequested('/a'));
    expect(t.state.recent, isEmpty);
    expect(t.effects.single, isA<SaveRecentProjects>());
  });

  test('recent store round-trips and tolerates garbage', () async {
    final dir = await Directory.systemTemp.createTemp('bdl-recent');
    try {
      final store = RecentStore(dir: dir);
      expect(await store.load(), isEmpty);
      final items = [RecentProject(path: '/x', name: 'x', lastOpened: DateTime.utc(2026, 9, 15))];
      await store.save(items);
      final back = await store.load();
      expect(back.single.path, '/x');
      expect(back.single.lastOpened, DateTime.utc(2026, 9, 15));
      await File('${dir.path}/recent.json').writeAsString('{not json');
      expect(await store.load(), isEmpty);
    } finally {
      await dir.delete(recursive: true);
    }
  });

  test('relative time wording', () {
    final now = DateTime(2026, 9, 15, 12);
    expect(relativeTime(now.subtract(const Duration(seconds: 10)), now: now), 'just now');
    expect(relativeTime(now.subtract(const Duration(hours: 3)), now: now), '3 h ago');
    expect(relativeTime(now.subtract(const Duration(days: 1)), now: now), 'yesterday');
    expect(relativeTime(DateTime(2026, 8, 1), now: now), '2026-08-01');
  });
}
