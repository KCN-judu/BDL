/// Deployment transitions: the board list, the chosen board, and the
/// compiler's target-relative analysis of the revision on screen.
///
/// Feasibility is a fact about (design, board), never about the design
/// alone: an analysis is kept only while its revision is the project's and
/// its target the chosen one; any new revision asks again.  Studio never
/// allocates a pin.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'state.dart';

Transition targetsRequested(AppState s) {
  if (s.editor.deploy.targetsLoaded) return Transition(s);
  return Transition(s, const [ListTargets()]);
}

Transition targetsReceived(AppState s, List<pb.TargetView> targets) {
  final d = s.editor.deploy.copyWith(targets: targets, targetsLoaded: true);
  // A chosen board that is no longer listed is forgotten.
  final keep = d.targetId != null && targets.any((t) => t.id == d.targetId);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: keep ? d : d.copyWith(clearTarget: true, clearAnalysis: true),
      ),
    ),
  );
}

Transition targetSelected(AppState s, String? targetId) {
  if (targetId == null) {
    return Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          deploy: s.editor.deploy.copyWith(clearTarget: true, clearAnalysis: true, pending: false),
        ),
      ),
    );
  }
  return deploymentRequested(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: s.editor.deploy.copyWith(targetId: targetId, clearAnalysis: true, clearError: true),
      ),
    ),
  );
}

/// Ask for the chosen board at the held revision; the previous answer, if
/// any, is already gone.
Transition deploymentRequested(AppState s) {
  final d = s.editor.deploy;
  final target = d.targetId;
  if (s.project == null || target == null) return Transition(s);
  final generation = d.generation + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: d.copyWith(pending: true, generation: generation, clearError: true),
      ),
    ),
    [AnalyzeDeployment(targetId: target, generation: generation)],
  );
}

Transition deploymentReceived(AppState s, int generation, pb.DeploymentAnalysis a) {
  final d = s.editor.deploy;
  if (generation != d.generation) return Transition(s);
  // The answer is about a revision; only the one on screen counts.
  if (a.revision.toInt() != s.revision) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(analysis: a, pending: false)),
    ),
  );
}

Transition deploymentFailed(AppState s, int generation, String message) {
  final d = s.editor.deploy;
  if (generation != d.generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: d.copyWith(pending: false, clearAnalysis: true, error: message),
      ),
    ),
  );
}

/// The design changed: whatever was known about a board is about an
/// earlier design.  Dropped, and asked again if a board is chosen.
Transition deploymentAfterRevision(AppState s) {
  final d = s.editor.deploy;
  if (d.analysis == null && !d.pending) {
    return d.targetId == null ? Transition(s) : deploymentRequested(s);
  }
  return deploymentRequested(
    s.copyWith(editor: s.editor.copyWith(deploy: d.copyWith(clearAnalysis: true))),
  );
}

/// The project closed: the board choice stays (a session preference), the
/// answer goes.
DeployState deployWithoutProject(DeployState d) =>
    d.copyWith(clearAnalysis: true, pending: false, clearError: true);
