/// Wires the pure reducer to the effect executor.  Riverpod owns the
/// lifecycle; the semantics of every transition live in `reducer.dart`.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../effects/effect_executor.dart';
import 'actions.dart';
import 'reducer.dart';
import 'state.dart';

typedef ExecutorFactory = EffectExecutor Function(void Function(AppAction) dispatch);

class AppStore extends Notifier<AppState> {
  AppStore([this._executorFactory]);

  /// How the executor is made — the real one, or a test's (a fake daemon).
  final ExecutorFactory? _executorFactory;
  late final EffectExecutor _executor;

  @override
  AppState build() {
    _executor = (_executorFactory ?? EffectExecutor.new)(dispatch);
    ref.onDispose(_executor.dispose);
    return const AppState();
  }

  void dispatch(AppAction action) {
    final t = reduce(state, action);
    state = t.state;
    for (final effect in t.effects) {
      // Effects run after the state change is visible; results come back as
      // ResponseActions through dispatch.
      _executor.run(effect);
    }
  }
}

final appStoreProvider = NotifierProvider<AppStore, AppState>(AppStore.new);
