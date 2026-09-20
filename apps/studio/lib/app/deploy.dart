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
  // Another board: whatever was known about the last one's firmware is
  // about that board.
  return deploymentRequested(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: s.editor.deploy.copyWith(
          targetId: targetId,
          clearAnalysis: true,
          clearError: true,
          firmware: const FirmwareState(),
        ),
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
  // The firmware's freshness is judged against this very revision: ask.
  return buildStatusRequested(
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
/// answer goes, and so does everything known about its firmware.
DeployState deployWithoutProject(DeployState d) => d.copyWith(
  clearAnalysis: true,
  pending: false,
  clearError: true,
  firmware: const FirmwareState(),
);

// ---- firmware ---------------------------------------------------------------
//
// The daemon owns the build and the flash; these transitions ask, keep
// what it answered, and forget what a new board or a new revision makes
// meaningless.  Freshness is the daemon's word (`artifact_fresh`): Studio
// never decides whether an image matches the design.

/// The last status is about a revision and a board: after any revision
/// it is asked again (the daemon judges freshness), after a board change
/// it is dropped.
Transition buildStatusRequested(AppState s) {
  final d = s.editor.deploy;
  final target = d.targetId;
  if (s.project == null || target == null) return Transition(s);
  final generation = d.firmware.statusGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        deploy: d.copyWith(firmware: d.firmware.copyWith(statusGeneration: generation)),
      ),
    ),
    [GetBuildStatus(targetId: target, generation: generation)],
  );
}

Transition buildStatusReceived(AppState s, int generation, pb.BuildStatus status) {
  final d = s.editor.deploy;
  if (generation != d.firmware.statusGeneration) return Transition(s);
  if (status.targetId != d.targetId) return Transition(s);
  var fw = d.firmware.copyWith(status: status);
  // A running build reported by the daemon (after a reopen, say) is shown
  // as running until its final event.
  if (!status.running) fw = fw.copyWith(clearBuilding: true);
  final effects = <Effect>[];
  // With a fresh image the flash is the next step: look for the board.
  if (status.hasArtifact() && status.artifactFresh && !fw.devicesLoaded) {
    effects.add(ListFlashDevices(status.targetId));
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
    effects,
  );
}

Transition buildRequested(AppState s) {
  final d = s.editor.deploy;
  final target = d.targetId;
  final p = s.project;
  if (p == null || target == null || d.firmware.isBuilding) return Transition(s);
  final fw = d.firmware.copyWith(
    building: pb.BuildProgress(
      targetId: target,
      stage: pb.BuildStage.BUILD_STAGE_CHECKING,
      message: '',
    ),
    output: const [],
    clearError: true,
    clearFlashed: true,
  );
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
    [BuildFirmware(targetId: target, revision: s.revision)],
  );
}

Transition cancelBuildRequested(AppState s) {
  final d = s.editor.deploy;
  if (!d.firmware.isBuilding) return Transition(s);
  return Transition(s, const [CancelBuild()]);
}

Transition buildProgressReceived(AppState s, pb.BuildProgress p) {
  final d = s.editor.deploy;
  if (p.targetId != d.targetId) return Transition(s);
  var fw = d.firmware;
  if (p.detail.isNotEmpty) {
    final lines = [...fw.output, ...p.detail.split('\n')];
    fw = fw.copyWith(output: lines.length > 200 ? lines.sublist(lines.length - 200) : lines);
  }
  if (p.hasStatus()) {
    // The final event: the status it carries is the truth from here on.
    fw = fw.copyWith(status: p.status, clearBuilding: true, clearFlashed: true);
    final effects = <Effect>[];
    if (p.status.hasArtifact() && p.status.artifactFresh) {
      effects.add(ListFlashDevices(p.targetId));
    }
    return Transition(
      s.copyWith(
        editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
      ),
      effects,
    );
  }
  // Keep the last message when an event carries only a detail line.
  final shown = p.message.isEmpty && fw.building != null
      ? (p.deepCopy()..message = fw.building!.message)
      : p;
  fw = fw.copyWith(building: shown);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
  );
}

Transition flashDevicesRequested(AppState s) {
  final d = s.editor.deploy;
  final target = d.targetId;
  if (target == null) return Transition(s);
  return Transition(s, [ListFlashDevices(target)]);
}

Transition flashDevicesReceived(
  AppState s,
  String targetId,
  List<pb.FlashDevice> devices,
  List<pb.FlashMethodView> methods,
) {
  final d = s.editor.deploy;
  if (targetId != d.targetId) return Transition(s);
  var fw = d.firmware.copyWith(devices: devices, methods: methods, devicesLoaded: true);
  // A choice that is no longer reachable is forgotten.
  if (fw.chosenDevice != null && !devices.any((x) => x.id == fw.chosenDevice)) {
    fw = fw.copyWith(clearChosenDevice: true);
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
  );
}

Transition flashDeviceChosen(AppState s, String? deviceId) {
  final d = s.editor.deploy;
  final fw = deviceId == null
      ? d.firmware.copyWith(clearChosenDevice: true)
      : d.firmware.copyWith(chosenDevice: deviceId);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
  );
}

/// Flash the one reachable device, or the chosen one.  With several and
/// none chosen the request is not made: the page asks for the choice.
Transition flashRequested(AppState s) {
  final d = s.editor.deploy;
  final target = d.targetId;
  final fw = d.firmware;
  if (target == null || fw.isFlashing || fw.isBuilding) return Transition(s);
  final device = fw.flashTarget;
  if (device == null && fw.devices.length > 1) return Transition(s);
  final started = fw.copyWith(
    flashing: pb.FlashProgress(
      targetId: target,
      stage: pb.FlashStage.FLASH_STAGE_PREPARING,
      message: '',
    ),
    clearFlashed: true,
    clearError: true,
  );
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: started)),
    ),
    [FlashFirmware(targetId: target, deviceId: device?.id)],
  );
}

Transition flashProgressReceived(AppState s, pb.FlashProgress p) {
  final d = s.editor.deploy;
  if (p.targetId != d.targetId) return Transition(s);
  final done =
      p.stage == pb.FlashStage.FLASH_STAGE_COMPLETED || p.stage == pb.FlashStage.FLASH_STAGE_FAILED;
  final fw = done
      ? d.firmware.copyWith(clearFlashing: true, flashed: p)
      : d.firmware.copyWith(flashing: p);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
    // The board left bootloader mode: what was reachable is not any more.
    done ? [ListFlashDevices(p.targetId)] : const [],
  );
}

Transition firmwareRequestFailed(AppState s, String message) {
  final d = s.editor.deploy;
  final fw = d.firmware.copyWith(clearBuilding: true, clearFlashing: true, error: message);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(deploy: d.copyWith(firmware: fw)),
    ),
  );
}

Transition templatesRequested(AppState s) {
  if (s.editor.deploy.templatesLoaded) return Transition(s);
  return Transition(s, const [ListTemplates()]);
}

Transition templatesReceived(AppState s, List<pb.TemplateView> templates) {
  final d = s.editor.deploy.copyWith(templates: templates, templatesLoaded: true);
  return Transition(s.copyWith(editor: s.editor.copyWith(deploy: d)));
}
