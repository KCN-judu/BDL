/// The Deploy page's last section: from a deployment that fits to a board
/// that runs it.
///
/// Task: "build this design for the board and put it on the board".  The
/// facts, ranked: (1) what the next thing to do is — one primary action
/// per state, or the one blocker that stands in its way; (2) whether the
/// last image is still this design (the daemon's `artifact_fresh`; a stale
/// image is named, never offered); (3) what a build or flash is doing now;
/// (4) which board a flash would reach, and that a choice between several
/// is the designer's; (5) the advanced facts — the crate, the command, the
/// compiler's words — behind a disclosure.  Nothing here judges: readiness
/// is the daemon's `build_ready` and its ordered `build_blockers`, the
/// stages are its events, the devices are its list.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../../l10n/l10n.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../mac/controls.dart';
import '../mac/interactive.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';

/// Where the designer stands on the way to a running board.
enum FirmwareStep { deployment, build, flash, observe }

/// The step the page is at, from the daemon's facts alone.
FirmwareStep firmwareStep(pb.DeploymentAnalysis? analysis, FirmwareState fw) {
  if (analysis == null || !analysis.buildReady) return FirmwareStep.deployment;
  final flashedNow =
      fw.flashed != null &&
      fw.flashed!.stage == pb.FlashStage.FLASH_STAGE_COMPLETED &&
      fw.artifactFresh &&
      fw.flashed!.hasArtifact() &&
      fw.flashed!.artifact.identity == fw.artifact?.identity;
  if (flashedNow) return FirmwareStep.observe;
  if (fw.artifact != null && fw.artifactFresh) return FirmwareStep.flash;
  return FirmwareStep.build;
}

String _clock(Int64 secs) {
  final t = DateTime.fromMillisecondsSinceEpoch(secs.toInt() * 1000).toLocal();
  String two(int n) => n.toString().padLeft(2, '0');
  return '${two(t.hour)}:${two(t.minute)}';
}

class FirmwareSection extends StatelessWidget {
  const FirmwareSection({
    super.key,
    required this.state,
    required this.analysis,
    required this.dispatch,
  });
  final AppState state;

  /// The deployment analysis for the current revision, when one exists.
  final pb.DeploymentAnalysis? analysis;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final d = state.editor.deploy;
    final board = d.target?.name;
    final fw = d.firmware;
    // Nothing to say until the board is chosen and judged — the verdict
    // line above already says the judging is under way — unless a build
    // or a flash is running through the judging.
    if (board == null || (analysis == null && !fw.isBuilding && !fw.isFlashing)) {
      return const SizedBox.shrink();
    }
    final step = firmwareStep(analysis, fw);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gap,
      children: [
        Row(
          children: [
            Text(context.l10n.firmware, style: Theme.of(context).textTheme.titleSmall),
            const Spacer(),
            _Progression(step: step),
          ],
        ),
        Container(
          key: const ValueKey('firmware-card'),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: t.content,
            borderRadius: BorderRadius.circular(MacMetrics.radius),
            border: Border.all(color: t.hairline),
          ),
          child: _Body(
            state: state,
            analysis: analysis,
            board: board,
            step: step,
            dispatch: dispatch,
          ),
        ),
        _Details(fw: fw, open: fw.status?.hasFailure() == true && !fw.isBuilding),
      ],
    );
  }
}

/// Deployment · Build · Flash · Observe — the steps done with a check, the
/// current one in the primary ink, the rest to come in the tertiary.
class _Progression extends StatelessWidget {
  const _Progression({required this.step});
  final FirmwareStep step;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final labels = [
      (FirmwareStep.deployment, l10n.stepDeployment),
      (FirmwareStep.build, l10n.stepBuild),
      (FirmwareStep.flash, l10n.stepFlash),
      (FirmwareStep.observe, l10n.stepObserve),
    ];
    final cells = <Widget>[];
    for (final (s, label) in labels) {
      final done = s.index < step.index;
      final current = s == step;
      if (cells.isNotEmpty) {
        cells.add(
          Text(
            ' · ',
            style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
          ),
        );
      }
      cells.add(
        Text(
          done ? '✓ $label' : label,
          key: ValueKey('firmware-step-${s.name}${current ? '-current' : ''}'),
          style: TextStyle(
            fontSize: MacType.secondary,
            fontWeight: current ? FontWeight.w600 : FontWeight.w400,
            color: done
                ? t.settled
                : current
                ? t.textPrimary
                : t.textTertiary,
          ),
        ),
      );
    }
    return Row(mainAxisSize: MainAxisSize.min, children: cells);
  }
}

class _Body extends StatelessWidget {
  const _Body({
    required this.state,
    required this.analysis,
    required this.board,
    required this.step,
    required this.dispatch,
  });
  final AppState state;
  final pb.DeploymentAnalysis? analysis;
  final String board;
  final FirmwareStep step;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final t = MacTokens.of(context);
    final fw = state.editor.deploy.firmware;
    final a = analysis;

    if (fw.isBuilding) {
      final b = fw.building!;
      return _Running(
        heading: l10n.building,
        stage: _buildStageLabel(l10n, b.stage),
        message: b.hasDone() ? l10n.cratesCompiled(b.done) : b.message,
        trailing: MacLink(
          label: l10n.stopBuild,
          onTap: () => dispatch(const CancelBuildRequested()),
        ),
      );
    }
    if (fw.isFlashing) {
      final f = fw.flashing!;
      return _Running(
        heading: l10n.flashing,
        stage: _flashStageLabel(l10n, f.stage),
        message: f.message,
      );
    }
    if (a == null) {
      return _Line(color: t.textTertiary, heading: l10n.checkingBoard(board));
    }
    if (fw.error case final message?) {
      return _Line(
        color: t.error,
        heading: message,
        action: step == FirmwareStep.flash
            ? MacButton.primary(
                label: l10n.flash,
                onPressed: () => dispatch(const FlashRequested()),
              )
            : MacButton.primary(
                label: l10n.buildForBoard(board),
                onPressed: () => dispatch(const BuildRequested()),
              ),
      );
    }
    if (!a.buildReady) {
      final blocker = a.buildBlockers.firstOrNull;
      final aboutTheDesign = blocker != null && _isSemantic(blocker.code);
      return _Line(
        color: t.open,
        heading: l10n.notReadyToBuild,
        headingKey: const ValueKey('firmware-blocked'),
        body: blocker?.message,
        detail: blocker?.explanation,
        action: aboutTheDesign
            ? MacLink(
                label: l10n.fixOnTheDesignPage,
                onTap: () => dispatch(const PageSelected(StudioPage.design)),
              )
            : null,
      );
    }
    final status = fw.status;
    final failure = status?.hasFailure() == true ? status!.failure : null;
    if (failure != null && status!.stage == pb.BuildStage.BUILD_STAGE_FAILED) {
      return _Line(
        color: t.error,
        heading: l10n.buildDidNotComplete,
        headingKey: const ValueKey('firmware-failed'),
        body: failure.message,
        detail: failure.explanation,
        action: MacButton.primary(
          label: l10n.buildAgain,
          onPressed: () => dispatch(const BuildRequested()),
        ),
      );
    }
    final artifact = fw.artifact;
    if (artifact == null || status?.stage == pb.BuildStage.BUILD_STAGE_CANCELLED) {
      final cancelled = status?.stage == pb.BuildStage.BUILD_STAGE_CANCELLED;
      return _Line(
        color: cancelled ? t.textSecondary : t.settled,
        heading: cancelled ? l10n.stageCancelled : l10n.readyToBuild(board),
        headingKey: const ValueKey('firmware-ready'),
        action: MacButton.primary(
          key: const ValueKey('firmware-build'),
          label: l10n.buildForBoard(board),
          onPressed: () => dispatch(const BuildRequested()),
        ),
      );
    }
    if (!fw.artifactFresh) {
      return _Line(
        color: t.open,
        heading: l10n.firmwareStale,
        headingKey: const ValueKey('firmware-stale'),
        body: fw.flashed?.stage == pb.FlashStage.FLASH_STAGE_COMPLETED
            ? l10n.boardRunsEarlierDesign
            : null,
        action: MacButton.primary(
          key: const ValueKey('firmware-build'),
          label: l10n.buildAgain,
          onPressed: () => dispatch(const BuildRequested()),
        ),
      );
    }
    // A fresh image: the flash is the next step, then the trial.
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gap,
      children: [
        Row(
          spacing: MacMetrics.gap,
          children: [
            Icon(Icons.circle, size: 8, color: t.settled),
            Text(
              l10n.firmwareBuiltAt(_clock(artifact.builtAt)),
              key: const ValueKey('firmware-built'),
              style: TextStyle(
                fontSize: MacType.body,
                fontWeight: FontWeight.w500,
                color: t.textPrimary,
              ),
            ),
            Text(
              l10n.imageSize(artifact.sizeBytes.toString()),
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            ),
            const Spacer(),
            MacLink(label: l10n.buildAgain, onTap: () => dispatch(const BuildRequested())),
          ],
        ),
        _FlashRow(fw: fw, dispatch: dispatch),
        if (fw.flashed case final f?) _Flashed(flashed: f, project: state.flat, dispatch: dispatch),
      ],
    );
  }

  bool _isSemantic(String code) => const {
    'relationship_not_checking',
    'not_causal',
    'not_clock_consistent',
    'output_no_domain',
    'output_no_driver',
    'output_connection_invalid',
  }.contains(code);
}

String _buildStageLabel(AppLocalizations l10n, pb.BuildStage s) => switch (s) {
  pb.BuildStage.BUILD_STAGE_CHECKING => l10n.stageChecking,
  pb.BuildStage.BUILD_STAGE_GENERATING => l10n.stageGenerating,
  pb.BuildStage.BUILD_STAGE_PREPARING => l10n.stagePreparing,
  pb.BuildStage.BUILD_STAGE_COMPILING => l10n.stageCompiling,
  pb.BuildStage.BUILD_STAGE_PACKAGING => l10n.stagePackaging,
  pb.BuildStage.BUILD_STAGE_CANCELLED => l10n.stageCancelled,
  _ => '',
};

String _flashStageLabel(AppLocalizations l10n, pb.FlashStage s) => switch (s) {
  pb.FlashStage.FLASH_STAGE_PREPARING => l10n.flashPreparing,
  pb.FlashStage.FLASH_STAGE_WRITING => l10n.flashWriting,
  pb.FlashStage.FLASH_STAGE_RESTARTING => l10n.flashRestarting,
  _ => '',
};

/// A heading in the state's colour, an optional body and explanation, and
/// the one action of the state on the right.
class _Line extends StatelessWidget {
  const _Line({
    required this.color,
    required this.heading,
    this.headingKey,
    this.body,
    this.detail,
    this.action,
  });
  final Color color;
  final String heading;
  final Key? headingKey;
  final String? body;
  final String? detail;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: MacMetrics.gap,
      children: [
        if (heading.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Icon(Icons.circle, size: 8, color: color),
          ),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 2,
            children: [
              if (heading.isNotEmpty)
                Text(
                  heading,
                  key: headingKey,
                  style: TextStyle(
                    fontSize: MacType.body,
                    fontWeight: FontWeight.w500,
                    color: color == t.textSecondary ? t.textPrimary : color,
                  ),
                ),
              if (body case final b? when b.isNotEmpty)
                Text(
                  b,
                  key: const ValueKey('firmware-blocker'),
                  style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
                ),
              if (detail case final x? when x.isNotEmpty)
                Text(
                  x,
                  style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                ),
            ],
          ),
        ),
        ?action,
      ],
    );
  }
}

/// A build or a flash under way: the stage, the daemon's words, a bar
/// that moves (the count of crates is the only measure cargo gives).
class _Running extends StatelessWidget {
  const _Running({
    required this.heading,
    required this.stage,
    required this.message,
    this.trailing,
  });
  final String heading;
  final String stage;
  final String message;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gap,
      children: [
        Row(
          spacing: MacMetrics.gap,
          children: [
            SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(strokeWidth: 1.5, color: t.textTertiary),
            ),
            Text(
              heading,
              key: const ValueKey('firmware-running'),
              style: TextStyle(
                fontSize: MacType.body,
                fontWeight: FontWeight.w500,
                color: t.textPrimary,
              ),
            ),
            Text(
              stage,
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            ),
            if (message.isNotEmpty)
              Expanded(
                child: Text(
                  message,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
                ),
              )
            else
              const Spacer(),
            ?trailing,
          ],
        ),
        ClipRRect(
          borderRadius: BorderRadius.circular(2),
          child: LinearProgressIndicator(
            minHeight: 3,
            color: t.accent,
            backgroundColor: t.hairline,
          ),
        ),
      ],
    );
  }
}

/// The board a flash would reach — none (what to do), one (named), several
/// (a choice, never a guess) — and the Flash button.
class _FlashRow extends StatelessWidget {
  const _FlashRow({required this.fw, required this.dispatch});
  final FirmwareState fw;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final devices = fw.devices;
    if (!fw.devicesLoaded) {
      return Row(
        spacing: MacMetrics.gap,
        children: [
          SizedBox(
            width: 10,
            height: 10,
            child: CircularProgressIndicator(strokeWidth: 1.5, color: t.textTertiary),
          ),
          Text(l10n.lookAgain, style: small),
        ],
      );
    }
    if (devices.isEmpty) {
      final usb = fw.methods
          .where((m) => m.method == pb.FlashMethod.FLASH_METHOD_UF2_VOLUME && m.available)
          .firstOrNull;
      return Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gap,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Icon(Icons.circle, size: 8, color: t.open),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 2,
              children: [
                Text(
                  l10n.noBoardReachable,
                  key: const ValueKey('firmware-no-device'),
                  style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
                ),
                if (usb != null) Text(usb.hint, style: small),
              ],
            ),
          ),
          MacLink(label: l10n.lookAgain, onTap: () => dispatch(const FlashDevicesRequested())),
        ],
      );
    }
    final target = fw.flashTarget;
    final actions = [
      MacLink(label: l10n.lookAgain, onTap: () => dispatch(const FlashDevicesRequested())),
      if (target != null)
        MacButton.primary(
          key: const ValueKey('firmware-flash'),
          label: l10n.flash,
          onPressed: () => dispatch(const FlashRequested()),
        ),
    ];
    if (devices.length == 1) {
      return Row(
        spacing: MacMetrics.gap,
        children: [
          Icon(Icons.circle, size: 8, color: t.settled),
          Expanded(
            child: Text(
              '${devices.single.label} — ${devices.single.detail}',
              overflow: TextOverflow.ellipsis,
              style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
            ),
          ),
          ...actions,
        ],
      );
    }
    // Several: the sentence, then the choice with the actions beside it.
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gap,
      children: [
        Row(
          spacing: MacMetrics.gap,
          children: [
            Icon(Icons.circle, size: 8, color: t.open),
            Expanded(
              child: Text(
                l10n.chooseTheBoardToFlash,
                style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
              ),
            ),
          ],
        ),
        Row(
          spacing: MacMetrics.gap,
          children: [
            Expanded(
              child: MacDropdown<String>(
                key: const ValueKey('flash-device-picker'),
                value: fw.chosenDevice,
                hint: l10n.chooseABoard,
                items: [for (final x in devices) x.id],
                labelOf: (id) => devices.firstWhere((x) => x.id == id).label,
                detailOf: (id) => devices.firstWhere((x) => x.id == id).detail,
                onChanged: (id) => dispatch(FlashDeviceChosen(id)),
              ),
            ),
            ...actions,
          ],
        ),
      ],
    );
  }
}

/// The last flash: done, with the trial to make — or not done, with why.
class _Flashed extends StatelessWidget {
  const _Flashed({required this.flashed, required this.project, required this.dispatch});
  final pb.FlashProgress flashed;
  final pb.ProjectProjection? project;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    if (flashed.stage == pb.FlashStage.FLASH_STAGE_FAILED) {
      final f = flashed.hasFailure() ? flashed.failure : null;
      return Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gap,
        children: [
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Icon(Icons.circle, size: 8, color: t.error),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 2,
              children: [
                Text(
                  l10n.flashDidNotComplete,
                  key: const ValueKey('firmware-flash-failed'),
                  style: TextStyle(
                    fontSize: MacType.body,
                    fontWeight: FontWeight.w500,
                    color: t.error,
                  ),
                ),
                Text(
                  f?.message ?? flashed.message,
                  style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
                ),
                if (f != null && f.explanation.isNotEmpty) Text(f.explanation, style: small),
              ],
            ),
          ),
        ],
      );
    }
    final sources =
        project?.mappings
            .where((m) => relationshipRole(m) == RelationshipRole.source)
            .map((m) => m.name)
            .toList() ??
        const <String>[];
    final outputs = project?.outputs.map((o) => o.name).toList() ?? const <String>[];
    final when = _clock(flashed.at);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: MacMetrics.gap,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Icon(Icons.circle, size: 8, color: t.settled),
        ),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 2,
            children: [
              Text(
                l10n.flashedToAt(flashed.device.label, when),
                key: const ValueKey('firmware-flashed'),
                style: TextStyle(
                  fontSize: MacType.body,
                  fontWeight: FontWeight.w500,
                  color: t.textPrimary,
                ),
              ),
              Text(flashed.message, style: small),
              Text(
                outputs.isEmpty
                    ? ''
                    : sources.isEmpty
                    ? l10n.tryItOutputsOnly(outputs.join(', '))
                    : l10n.tryItOnTheBoard(sources.join(', '), outputs.join(', ')),
                key: const ValueKey('firmware-observe'),
                style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
              ),
            ],
          ),
        ),
        MacLink(label: l10n.flashAgain, onTap: () => dispatch(const FlashRequested())),
      ],
    );
  }
}

/// The advanced facts, closed unless a build failed: where the crate is,
/// what is run, for which triple, the image, and the compiler's words.
class _Details extends StatelessWidget {
  const _Details({required this.fw, required this.open});
  final FirmwareState fw;
  final bool open;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final s = fw.status;
    final mono = TextStyle(fontFamily: 'Menlo', fontSize: MacType.code, color: t.textPrimary);
    final label = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final output = fw.isBuilding
        ? fw.output
        : (s?.hasFailure() == true && s!.failure.output.isNotEmpty
              ? s.failure.output
              : s?.output ?? const []);
    Widget row(String name, String value) => Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(width: 120, child: Text(name, style: label)),
        Expanded(child: SelectableText(value.isEmpty ? l10n.nothingYet : value, style: mono)),
      ],
    );
    return MacDisclosure(
      key: ValueKey('firmware-details-${open ? 'open' : 'closed'}'),
      title: l10n.details,
      initiallyOpen: open,
      children: [
        row(l10n.generatedCrate, s?.generatedDir ?? ''),
        row(l10n.buildCommand, s?.command ?? ''),
        row(l10n.rustTarget, s?.triple ?? ''),
        row(l10n.imageFile, fw.artifact?.path ?? ''),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(width: 120, child: Text(l10n.compilerOutput, style: label)),
            Expanded(
              child: Container(
                constraints: const BoxConstraints(maxHeight: 200),
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: t.canvas,
                  borderRadius: BorderRadius.circular(MacMetrics.radius),
                  border: Border.all(color: t.hairline),
                ),
                child: SingleChildScrollView(
                  child: SelectableText(
                    output.isEmpty ? l10n.nothingYet : output.join('\n'),
                    style: mono.copyWith(fontSize: MacType.secondary),
                  ),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
