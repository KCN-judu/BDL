/// State-driven sheets of a system project: the pending-bind question and
/// the "Package as reusable component" sheet.  Both are rendered from
/// [EditorState] (not pushed as dialogs) so every answer is a dispatched
/// action and the reducer decides what is sent.
library;

import 'package:flutter/material.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'system_inspector.dart' show endLabel;

/// Modal chrome over the canvas: a dimmed backdrop and a sheet.
class SheetScrim extends StatelessWidget {
  const SheetScrim({
    super.key,
    required this.title,
    this.subtitle,
    required this.child,
    this.width = 520,
  });
  final String title;
  final String? subtitle;
  final Widget child;
  final double width;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Positioned.fill(
      child: Container(
        color: Colors.black.withValues(alpha: 0.25),
        alignment: Alignment.center,
        child: Container(
          // Keyed for the documentation screenshots (docs/user-guide/screenshots).
          key: const ValueKey('sheet'),
          width: width,
          constraints: const BoxConstraints(maxHeight: 640),
          decoration: BoxDecoration(
            color: t.window,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: t.hairline),
          ),
          padding: const EdgeInsets.fromLTRB(20, 18, 20, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleMedium),
              if (subtitle != null) ...[
                const SizedBox(height: 4),
                Text(subtitle!, style: TextStyle(fontSize: 12, color: t.textSecondary)),
              ],
              const SizedBox(height: 14),
              Flexible(child: child),
            ],
          ),
        ),
      ),
    );
  }
}

/// "tiltValue already takes lampA.tiltValue" / "carry across timing
/// domains": the two things a link cannot do silently.
class PendingBindSheet extends StatefulWidget {
  const PendingBindSheet({
    super.key,
    required this.state,
    required this.bind,
    required this.dispatch,
  });
  final AppState state;
  final PendingBind bind;
  final void Function(AppAction) dispatch;

  @override
  State<PendingBindSheet> createState() => _PendingBindSheetState();
}

class _PendingBindSheetState extends State<PendingBindSheet> {
  final TextEditingController _init = TextEditingController();

  @override
  void dispose() {
    _init.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final b = widget.bind;
    final s = widget.state;
    final small = TextStyle(fontSize: 12, color: t.textSecondary);
    final from = endLabel(s, b.source);
    final to = endLabel(s, b.destination);
    final existing = b.replaces == null ? null : s.binding(b.replaces!);
    final ready = !b.needsTransport || _init.text.trim().isNotEmpty;
    return SheetScrim(
      title: b.replaces != null
          ? context.l10n.replaceTheConnection
          : context.l10n.carryAcrossTimingDomains,
      subtitle: '$from → $to',
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (existing != null)
            Text(
              context.l10n.alreadyTakesItsValueFrom(to, endLabel(s, existing.source)),
              style: small,
            ),
          if (b.needsTransport) ...[
            if (existing != null) const SizedBox(height: 8),
            Text(
              context.l10n.transportExplanation(from, b.sourceDomain, to, b.destinationDomain),
              style: small,
            ),
            const SizedBox(height: 10),
            FormRow(
              label: context.l10n.startsAt,
              child: MacTextField(
                controller: _init,
                hint: context.l10n.aConstantInTheDestinationSUnits,
                monospace: true,
                onChanged: (_) => setState(() {}),
              ),
            ),
          ],
          const SizedBox(height: 14),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              MacButton(
                label: context.l10n.cancel,
                onPressed: () => widget.dispatch(const PendingBindCancelled()),
              ),
              const SizedBox(width: 8),
              MacButton.primary(
                label: existing != null ? context.l10n.disconnectAndConnect : context.l10n.connect,
                onPressed: ready
                    ? () => widget.dispatch(
                        PendingBindConfirmed(
                          transportInit: b.needsTransport ? _init.text.trim() : null,
                        ),
                      )
                    : null,
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// The packaging sheet: the compiler's preview of the component the group
/// would become, and the four choices it cannot make for the designer —
/// the name, the instance's name, which open members are inputs, which
/// sinks move inside.  The floor is not negotiable: what crosses in is
/// required, what crosses out is provided.
class ExtractionSheet extends StatelessWidget {
  const ExtractionSheet({
    super.key,
    required this.state,
    required this.extraction,
    required this.dispatch,
  });
  final AppState state;
  final ExtractionState extraction;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final x = extraction;
    final p = x.preview;
    final small = TextStyle(fontSize: 12, color: t.textSecondary);
    final body = TextStyle(fontSize: 12, color: t.textPrimary);
    final project = state.project;
    String conceptName(pb.PreviewPortView port) =>
        project?.concepts.where((c) => c.id == port.concept).firstOrNull?.name ?? '';
    String clockName(int id) =>
        project?.clocks.where((c) => c.id.toInt() == id).firstOrNull?.name ?? '?';
    String declName(int id) =>
        project?.mappings.where((m) => m.id.toInt() == id).firstOrNull?.name ?? '?';
    return SheetScrim(
      title: context.l10n.packageAsReusableComponentTitle,
      subtitle: state.group(x.groupId)?.name,
      width: 560,
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            FormRow(
              label: context.l10n.componentTitle,
              child: CommitTextField(
                value: x.name,
                onCommit: (v) => dispatch(ExtractionChoicesChanged(name: v)),
              ),
            ),
            FormRow(
              label: context.l10n.instanceTitle,
              child: CommitTextField(
                value: x.instanceName.isEmpty ? (p?.instanceName ?? '') : x.instanceName,
                hint: p?.instanceName,
                onCommit: (v) => dispatch(ExtractionChoicesChanged(instanceName: v)),
              ),
            ),
            if (x.error != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(x.error!, style: TextStyle(fontSize: 12, color: t.error)),
              ),
            if (p == null && x.error == null)
              Text(context.l10n.workingOutTheBoundary, style: small)
            else if (p != null) ...[
              _Heading(context.l10n.requires, context.l10n.whatTheMembersReadFromOutside),
              if (p.required.isEmpty)
                Text(context.l10n.nothingTheComponentIsSelfContained, style: small)
              else
                for (final port in p.required)
                  _PortLine(
                    name: port.name,
                    concept: conceptName(port),
                    color: t.conceptColor(port.concept.toInt()),
                  ),
              const SizedBox(height: 10),
              _Heading(context.l10n.provides, context.l10n.whatOutsideReadsFromTheMembers),
              if (p.provided.isEmpty)
                Text(context.l10n.nothingYetNothingOutsideReadsTheGroup, style: small)
              else
                for (final port in p.provided)
                  _PortLine(
                    name: port.name,
                    concept: conceptName(port),
                    color: t.conceptColor(port.concept.toInt()),
                  ),
              if (p.openMembers.isNotEmpty) ...[
                const SizedBox(height: 10),
                _Heading(context.l10n.openRelationships, context.l10n.declaredInsideNotYetDefined),
                for (final o in p.openMembers)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Row(
                      children: [
                        Expanded(child: Text(o.name, style: body)),
                        SizedBox(
                          width: 250,
                          child: MacSegmented<bool>(
                            value: o.asInput,
                            options: {
                              true: context.l10n.treatAsInput,
                              false: context.l10n.keepInternal,
                            },
                            onChanged: (v) {
                              final keep = {...x.keepInternal};
                              if (v) {
                                keep.remove(o.decl.toInt());
                              } else {
                                keep.add(o.decl.toInt());
                              }
                              dispatch(ExtractionChoicesChanged(keepInternal: keep));
                            },
                          ),
                        ),
                      ],
                    ),
                  ),
              ],
              if (p.sinks.isNotEmpty) ...[
                const SizedBox(height: 10),
                _Heading(
                  context.l10n.physicalOutputs,
                  context.l10n.drivenByMembersTheDriveStaysWith,
                ),
                for (final d in p.sinks)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Row(
                      children: [
                        Expanded(
                          child: Text(
                            '${d.name} ← ${d.drivers.map((m) => declName(m.toInt())).join(', ')}',
                            style: body,
                          ),
                        ),
                        SizedBox(
                          width: 250,
                          child: MacSegmented<bool>(
                            value: d.internal,
                            options: {
                              false: context.l10n.staysTheSystemS,
                              true: context.l10n.movesInside,
                            },
                            onChanged: (v) {
                              final inside = {...x.internalizeSinks};
                              if (v) {
                                inside.add(d.output.toInt());
                              } else {
                                inside.remove(d.output.toInt());
                              }
                              dispatch(ExtractionChoicesChanged(internalizeSinks: inside));
                            },
                          ),
                        ),
                      ],
                    ),
                  ),
              ],
              const SizedBox(height: 10),
              _Heading(context.l10n.timingParameters, context.l10n.everyDomainTheMembersUse),
              Text(
                p.clocks.isEmpty
                    ? context.l10n.nonePure
                    : p.clocks.map((c) => clockName(c.toInt())).join(', '),
                style: body,
              ),
              if (p.private.isNotEmpty) ...[
                const SizedBox(height: 10),
                _Heading(context.l10n.internal, context.l10n.readByMembersOnly),
                Text(p.private.map((d) => declName(d.toInt())).join(', '), style: body),
              ],
              for (final w in p.warnings)
                Padding(
                  padding: const EdgeInsets.only(top: MacMetrics.gap),
                  child: DiagnosticCard(diagnostic: w, source: ''),
                ),
            ],
            const SizedBox(height: 14),
            Text(context.l10n.theDesignComputesTheSameValuesAfterwards, style: small),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                MacButton(
                  label: context.l10n.cancel,
                  onPressed: () => dispatch(const ExtractionSheetClosed()),
                ),
                const SizedBox(width: 8),
                MacButton.primary(
                  label: context.l10n.package,
                  onPressed: p == null || x.pending || x.error != null
                      ? null
                      : () => dispatch(const ExtractionConfirmed()),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _Heading extends StatelessWidget {
  const _Heading(this.title, this.note);
  final String title;
  final String note;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          Text(title, style: Theme.of(context).textTheme.titleSmall),
          const SizedBox(width: 8),
          Expanded(
            child: Text(note, style: TextStyle(fontSize: 11, color: t.textTertiary)),
          ),
        ],
      ),
    );
  }
}

class _PortLine extends StatelessWidget {
  const _PortLine({required this.name, required this.concept, required this.color});
  final String name;
  final String concept;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 3),
      child: Row(
        children: [
          Container(
            width: 9,
            height: 9,
            decoration: BoxDecoration(color: color, shape: BoxShape.circle),
          ),
          const SizedBox(width: 8),
          Text(name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          const SizedBox(width: 8),
          Text(concept, style: TextStyle(fontSize: 11, color: t.textSecondary)),
        ],
      ),
    );
  }
}
