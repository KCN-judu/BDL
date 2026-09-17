/// The Deploy page: which board, whether this design fits it, and where
/// each device lands.
///
/// Task: "choose a target and see whether the design can be placed on it".
/// The facts, ranked: (1) the verdict for the chosen board — feasible,
/// incomplete or not feasible *on that board*, never "design invalid";
/// (2) the placement, device → requirement → pin; (3) what stopped a
/// placement (one dead end, as the solver found it — not a minimal core);
/// (4) what is not bound yet (a device without an output, an output without
/// a device); (5) the devices themselves, editable.  Boards come from bdld;
/// nothing here allocates a pin.
library;

import 'package:flutter/material.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../mac/controls.dart';
import '../mac/interactive.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';

/// Product words for a device kind (presentation of the protocol enum).
String deviceKindLabel(pb.DeviceKind k) => switch (k) {
  pb.DeviceKind.DEVICE_KIND_PWM_CHANNEL => 'PWM channel',
  pb.DeviceKind.DEVICE_KIND_DIGITAL_OUTPUT => 'Digital output',
  pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL => 'H-bridge channel',
  pb.DeviceKind.DEVICE_KIND_I2C_SENSOR => 'I²C sensor',
  pb.DeviceKind.DEVICE_KIND_QUADRATURE_ENCODER => 'Quadrature encoder',
  pb.DeviceKind.DEVICE_KIND_UART => 'UART',
  _ => 'unspecified',
};

const List<pb.DeviceKind> deviceKinds = [
  pb.DeviceKind.DEVICE_KIND_PWM_CHANNEL,
  pb.DeviceKind.DEVICE_KIND_DIGITAL_OUTPUT,
  pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL,
  pb.DeviceKind.DEVICE_KIND_I2C_SENSOR,
  pb.DeviceKind.DEVICE_KIND_QUADRATURE_ENCODER,
  pb.DeviceKind.DEVICE_KIND_UART,
];

class DeployPage extends StatelessWidget {
  const DeployPage({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.flat;
    if (p == null) {
      return Container(
        color: t.canvas,
        alignment: Alignment.center,
        child: Text('No project open.', style: TextStyle(color: t.textTertiary)),
      );
    }
    final d = state.editor.deploy;
    return Container(
      color: t.canvas,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(MacMetrics.gapSection),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 760),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            spacing: MacMetrics.gapSection,
            children: [
              _TargetRow(deploy: d, dispatch: dispatch),
              _Verdict(deploy: d, project: p),
              _Devices(project: p, dispatch: dispatch),
              if (d.analysis case final a? when a.revision.toInt() == state.revision)
                _Result(analysis: a, project: p, dispatch: dispatch),
            ],
          ),
        ),
      ),
    );
  }
}

class _TargetRow extends StatelessWidget {
  const _TargetRow({required this.deploy, required this.dispatch});
  final DeployState deploy;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Row(
      spacing: MacMetrics.gapGroup,
      children: [
        Text('Target', style: Theme.of(context).textTheme.titleSmall),
        SizedBox(
          width: 260,
          child: MacDropdown<String>(
            key: const ValueKey('target-picker'),
            value: deploy.targetId,
            hint: deploy.targetsLoaded
                ? (deploy.targets.isEmpty ? 'no boards known' : 'choose a board…')
                : 'loading boards…',
            items: [for (final x in deploy.targets) x.id],
            labelOf: (id) => deploy.targets.firstWhere((x) => x.id == id).name,
            detailOf: (id) =>
                '${deploy.targets.firstWhere((x) => x.id == id).resourceCount} resources',
            onChanged: (id) => dispatch(TargetSelected(id)),
          ),
        ),
        if (deploy.pending)
          SizedBox(
            width: 10,
            height: 10,
            child: CircularProgressIndicator(strokeWidth: 1.5, color: t.textTertiary),
          ),
      ],
    );
  }
}

/// One line: the verdict *for this board*.  Feasible is settled, incomplete
/// is open, infeasible is wrong-on-this-board — the design's own validity
/// is the Design page's business and is never restated here.
class _Verdict extends StatelessWidget {
  const _Verdict({required this.deploy, required this.project});
  final DeployState deploy;
  final pb.ProjectProjection project;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final a = deploy.analysis;
    final board = deploy.target?.name;
    final (String text, Color color) = deploy.error != null
        ? ('Could not analyse: ${deploy.error}', t.error)
        : board == null
        ? ('Choose a board to see whether this design fits it.', t.textSecondary)
        : a == null || a.revision.toInt() != project.revision.toInt()
        ? ('Checking $board…', t.textTertiary)
        : switch (a.status) {
            pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE => ('Feasible on $board.', t.settled),
            pb.DeploymentStatus.DEPLOYMENT_STATUS_INCOMPLETE => (
              'Fits $board so far — the binding is not finished.',
              t.open,
            ),
            pb.DeploymentStatus.DEPLOYMENT_STATUS_INFEASIBLE => (
              'Not feasible on $board.',
              t.error,
            ),
            _ => ('Checking $board…', t.textTertiary),
          };
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: MacMetrics.gap,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Icon(Icons.circle, size: 8, color: color),
        ),
        Expanded(
          child: Text(
            text,
            key: const ValueKey('deploy-verdict'),
            style: TextStyle(fontSize: 13, fontWeight: FontWeight.w500, color: color),
          ),
        ),
      ],
    );
  }
}

/// The devices: what carries each output on the board.  Kind decides the
/// requirements (the pin table is the kind's, from the projection); a pin
/// may be fixed by hand — a constraint the placement must honour.
class _Devices extends StatelessWidget {
  const _Devices({required this.project, required this.dispatch});
  final pb.ProjectProjection project;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gap,
      children: [
        Row(
          children: [
            Text('Devices', style: Theme.of(context).textTheme.titleSmall),
            const Spacer(),
            MacButton(
              label: 'Add device',
              onPressed: () => dispatch(
                CreateDeviceRequested(
                  name: _freshName(),
                  kind: pb.DeviceKind.DEVICE_KIND_PWM_CHANNEL,
                  outputId: project.outputs.length == 1 ? project.outputs.single.id.toInt() : null,
                ),
              ),
            ),
          ],
        ),
        if (project.devices.isEmpty)
          Text(
            'No device yet. A device realises one output on the board; its kind says what it '
            'needs from the board.',
            style: small,
          ),
        for (final dv in project.devices)
          _DeviceCard(device: dv, project: project, dispatch: dispatch),
      ],
    );
  }

  String _freshName() {
    var n = project.devices.length + 1;
    while (project.devices.any((d) => d.name == 'device $n')) {
      n++;
    }
    return 'device $n';
  }
}

class _DeviceCard extends StatelessWidget {
  const _DeviceCard({required this.device, required this.project, required this.dispatch});
  final pb.DeviceView device;
  final pb.ProjectProjection project;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = device.id.toInt();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final fixed = {for (final f in device.fixedPins) f.index: f.resource};
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: t.content,
        borderRadius: BorderRadius.circular(MacMetrics.radius),
        border: Border.all(color: t.hairline),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        spacing: MacMetrics.gap,
        children: [
          Row(
            spacing: MacMetrics.gap,
            children: [
              Expanded(
                child: CommitTextField(
                  value: device.name,
                  onCommit: (v) => dispatch(RenameDeviceRequested(id: id, name: v)),
                ),
              ),
              SizedBox(
                width: 170,
                child: MacDropdown<pb.DeviceKind>(
                  value: device.kind,
                  items: deviceKinds,
                  labelOf: deviceKindLabel,
                  onChanged: (k) => dispatch(SetDeviceKindRequested(id: id, kind: k)),
                ),
              ),
              SizedBox(
                width: 170,
                child: MacDropdown<int>(
                  value: device.hasOutputId() ? device.outputId.toInt() : -1,
                  items: [-1, for (final o in project.outputs) o.id.toInt()],
                  labelOf: (o) => o < 0
                      ? 'no output'
                      : project.outputs.firstWhere((x) => x.id.toInt() == o).name,
                  onChanged: (o) =>
                      dispatch(SetDeviceOutputRequested(id: id, outputId: o < 0 ? null : o)),
                ),
              ),
              MacLink(label: 'Remove', onTap: () => dispatch(DeleteDeviceRequested(id))),
            ],
          ),
          if (device.requirements.isNotEmpty)
            MacTable(
              columns: const [MacColumn(width: 160), MacColumn(width: 120), MacColumn()],
              rows: [
                for (final r in device.requirements)
                  [
                    Text(r.label, style: TextStyle(fontSize: 12, color: t.textPrimary)),
                    Text(r.capability, style: small),
                    Row(
                      spacing: MacMetrics.gap,
                      children: [
                        Text('pin', style: small),
                        SizedBox(
                          width: 72,
                          child: CommitTextField(
                            value: fixed[r.index] ?? '',
                            hint: 'any',
                            onCommit: (v) => dispatch(
                              SetDevicePinRequested(
                                id: id,
                                index: r.index,
                                resource: v.trim().isEmpty ? null : v.trim(),
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ],
              ],
            ),
        ],
      ),
    );
  }
}

/// The compiler's answer, readable: placement as device → requirement →
/// pin; the dead end as the solver found it; the loose ends.
class _Result extends StatelessWidget {
  const _Result({required this.analysis, required this.project, required this.dispatch});
  final pb.DeploymentAnalysis analysis;
  final pb.ProjectProjection project;
  final void Function(AppAction) dispatch;

  String _device(int id) =>
      project.devices.where((d) => d.id.toInt() == id).map((d) => d.name).firstOrNull ?? '?';
  String _requirement(int device, int index) =>
      analysis.requirements
          .where((r) => r.deviceId.toInt() == device && r.index == index)
          .map((r) => r.label)
          .firstOrNull ??
      '#$index';
  String _output(int id) =>
      project.outputs.where((o) => o.id.toInt() == id).map((o) => o.name).firstOrNull ?? '?';

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final a = analysis;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: MacMetrics.gapGroup,
      children: [
        if (a.assignment.isNotEmpty) ...[
          Text('Placement on ${a.target}', style: Theme.of(context).textTheme.titleSmall),
          MacTable(
            columns: const [MacColumn(width: 160), MacColumn(width: 200), MacColumn()],
            rows: [
              for (final pl in a.assignment)
                [
                  Text(_device(pl.deviceId.toInt())),
                  Text(_requirement(pl.deviceId.toInt(), pl.index), style: small),
                  Text(
                    '→ ${pl.resource}',
                    key: ValueKey('placement-${pl.deviceId}-${pl.index}'),
                    style: TextStyle(fontFamily: 'Menlo', fontSize: 12, color: t.textPrimary),
                  ),
                ],
            ],
          ),
        ],
        if (a.hasDeadEnd()) _DeadEndView(deadEnd: a.deadEnd, analysis: a, project: project),
        for (final d in a.diagnostics)
          if (d.code != 'deploy.infeasible') DiagnosticCard(diagnostic: d, source: ''),
        if (a.unboundDevices.isNotEmpty)
          Text(
            'Not connected to an output: ${a.unboundDevices.map((d) => _device(d.toInt())).join(', ')}.',
            style: small,
          ),
        if (a.unrealisedOutputs.isNotEmpty)
          Text(
            'No device on ${a.target} for: ${a.unrealisedOutputs.map((o) => _output(o.toInt())).join(', ')}.',
            style: small,
          ),
      ],
    );
  }
}

/// One dead end, in the solver's own terms: the first requirement greedy
/// placement could not place, and what blocked each candidate.  Stated as
/// *a* conflict, never *the* conflict (DI-21).
class _DeadEndView extends StatelessWidget {
  const _DeadEndView({required this.deadEnd, required this.analysis, required this.project});
  final pb.DeadEnd deadEnd;
  final pb.DeploymentAnalysis analysis;
  final pb.ProjectProjection project;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    String device(int id) =>
        project.devices.where((d) => d.id.toInt() == id).map((d) => d.name).firstOrNull ?? '?';
    final req = analysis.requirements
        .where((r) => r.deviceId == deadEnd.deviceId && r.index == deadEnd.index)
        .firstOrNull;
    final what = req?.label ?? 'requirement ${deadEnd.index}';
    final who = device(deadEnd.deviceId.toInt());
    final headline = analysis.diagnostics
        .where((d) => d.code == 'deploy.infeasible')
        .map((d) => d.message)
        .firstOrNull;
    final explanation = analysis.diagnostics
        .where((d) => d.code == 'deploy.infeasible')
        .map((d) => d.explanation)
        .firstOrNull;
    return Container(
      key: const ValueKey('dead-end'),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: t.error.withValues(alpha: 0.06),
        borderRadius: BorderRadius.circular(MacMetrics.radius),
        border: Border.all(color: t.error.withValues(alpha: 0.4)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gap,
        children: [
          Text(
            headline ?? 'Could not place $what of $who.',
            key: const ValueKey('dead-end-headline'),
            style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: t.textPrimary),
          ),
          if (explanation != null && explanation.isNotEmpty) Text(explanation, style: small),
          switch (deadEnd.whichReason()) {
            pb.DeadEnd_Reason.noCapableResource => Text(
              'Nothing on ${analysis.target} can carry $what.',
              style: small,
            ),
            pb.DeadEnd_Reason.fixedUnavailable => Text(
              'The pin chosen by hand, ${deadEnd.fixedUnavailable}, cannot carry $what here.',
              style: small,
            ),
            pb.DeadEnd_Reason.blocked => MacTable(
              columns: const [MacColumn(width: 80), MacColumn()],
              rows: [
                for (final c in deadEnd.blocked.candidates)
                  [
                    Text(c.resource, style: TextStyle(fontFamily: 'Menlo', fontSize: 12)),
                    Text(
                      'held by ${device(c.heldByDeviceId.toInt())} '
                      '(${analysis.requirements.where((r) => r.deviceId == c.heldByDeviceId && r.index == c.heldByIndex).map((r) => r.label).firstOrNull ?? '#${c.heldByIndex}'})',
                      style: small,
                    ),
                  ],
              ],
            ),
            pb.DeadEnd_Reason.notSet => const SizedBox.shrink(),
          },
          if (deadEnd.placed.isNotEmpty)
            Text(
              'Placed before the dead end: '
              '${deadEnd.placed.map((p) => '${device(p.deviceId.toInt())} → ${p.resource}').join(', ')}. '
              'This is one conflict under the solver\'s order, not necessarily the only one.',
              style: small,
            ),
        ],
      ),
    );
  }
}
