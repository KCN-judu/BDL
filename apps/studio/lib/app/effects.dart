/// Side effects the reducer asks for.  Pure data; executed by
/// `effects/effect_executor.dart`.
library;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'state.dart';

@immutable
sealed class Effect {
  const Effect();
}

class ConnectDaemon extends Effect {
  const ConnectDaemon();
}

class LoadRecentProjects extends Effect {
  const LoadRecentProjects();
}

class SaveRecentProjects extends Effect {
  const SaveRecentProjects(this.recent);
  final List<RecentProject> recent;
}

/// Show the OS folder picker; the executor dispatches `OpenProjectRequested`
/// with the choice, or nothing when cancelled.
class PickProjectToOpen extends Effect {
  const PickProjectToOpen();
}

/// Show the OS save dialog to choose where a new project directory goes;
/// the executor dispatches `NewProjectRequested(rootPath, name)`.
class PickNewProjectLocation extends Effect {
  const PickNewProjectLocation();
}

class OpenProject extends Effect {
  const OpenProject(this.rootPath);
  final String rootPath;
}

class InitProject extends Effect {
  const InitProject({required this.rootPath, required this.name});
  final String rootPath;
  final String name;
}

class SaveProject extends Effect {
  const SaveProject();
}

class CloseProject extends Effect {
  const CloseProject();
}

class SubscribeProject extends Effect {
  const SubscribeProject();
}

class ApplyEdit extends Effect {
  const ApplyEdit({required this.baseRevision, required this.op});
  final int baseRevision;
  final pb.EditOp op;
}

class SetLayout extends Effect {
  const SetLayout(this.layout);
  final pb.Layout layout;
}

class Undo extends Effect {
  const Undo();
}

class Redo extends Effect {
  const Redo();
}
