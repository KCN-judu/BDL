/// Timing domains and physical outputs in Studio: reducer transitions,
/// canvas geometry (sink nodes, drive links, link rules), the output and
/// mapping inspectors, semantic-action plumbing.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/system.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/library.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const speed = 0;
const cruise = 0;
const boost = 1;
const motor = 0;
const main_ = 0;

pb.ProjectProjection rover({
  int revision = 1,
  bool withOutput = true,
  int? outputClock = main_,
  List<int> drivers = const [cruise],
  bool required = true,
}) {
  final p = pb.ProjectProjection(revision: Int64(revision), name: 'rover', rootPath: '/r')
    ..concepts.add(
      pb.ConceptView(
        id: Int64(speed),
        name: 'Speed',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    )
    ..clocks.add(pb.ClockView(id: Int64(main_), name: 'main'))
    ..mappings.addAll([
      pb.MappingView(
        id: Int64(cruise),
        name: 'cruise',
        signature: pb.Signature(inputs: [], output: Int64(speed)),
        definition: pb.Definition(formula: '0.5'),
        clockId: Int64(main_),
        drivesOutputId: drivers.contains(cruise) ? Int64(motor) : null,
      ),
      pb.MappingView(
        id: Int64(boost),
        name: 'boost',
        signature: pb.Signature(inputs: [], output: Int64(speed)),
        definition: pb.Definition(formula: '1'),
        drivesOutputId: drivers.contains(boost) ? Int64(motor) : null,
      ),
    ]);
  if (withOutput) {
    p.outputs.add(
      pb.OutputView(
        id: Int64(motor),
        name: 'motor',
        accepts: Int64(speed),
        clockId: outputClock == null ? null : Int64(outputClock),
        required: required,
      ),
    );
  }
  return p;
}

pb.ProjectAnalysis analysisWith(pb.OutputState state, {int? driver, int revision = 1}) =>
    pb.ProjectAnalysis(revision: Int64(revision))
      ..outputs.add(
        pb.OutputAnalysis(
          id: Int64(motor),
          state: state,
          driver: driver == null ? null : Int64(driver),
        ),
      );

AppState connected(pb.ProjectProjection project, {Selection selection = const NoSelection()}) =>
    AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
      ),
      project: project,
      editor: EditorState(selection: selection),
    );

pb.EditOp opOf(Transition t) => (t.effects.whereType<ApplyEdit>().single).op;

class Harness extends StatefulWidget {
  const Harness({super.key, required this.initial, required this.child});
  final AppState initial;
  final Widget Function(AppState, void Function(AppAction)) child;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late AppState state = widget.initial;
  final List<Effect> effects = [];
  void dispatch(AppAction a) => setState(() {
    final t = reduce(state, a);
    state = t.state;
    effects.addAll(t.effects);
  });

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox(width: 300, height: 900, child: widget.child(state, dispatch)),
      ),
    ),
  );
}

void main() {
  group('reducer', () {
    test('domain and output edits are the model ops, sent against the held revision', () {
      final s = connected(rover());
      expect(opOf(reduce(s, const CreateClockDomainRequested('ui'))).createClockDomain.name, 'ui');
      expect(
        opOf(reduce(s, const RenameClockDomainRequested(id: main_, name: 'loop')))
            .renameClockDomain
            .name,
        'loop',
      );
      expect(
        opOf(reduce(s, const DeleteClockDomainRequested(main_))).hasDeleteClockDomain(),
        isTrue,
      );
      final agnostic = opOf(
        reduce(s, const SetMappingClockRequested(mappingId: cruise, clockId: null)),
      );
      expect(agnostic.setMappingClock.hasClockId(), isFalse);
      final assigned = opOf(
        reduce(s, const SetMappingClockRequested(mappingId: boost, clockId: main_)),
      );
      expect(assigned.setMappingClock.clockId.toInt(), main_);
      final drive = opOf(
        reduce(s, const SetMappingDriveRequested(mappingId: boost, outputId: motor)),
      );
      expect(drive.setMappingDrive.outputId.toInt(), motor);
      final undrive = opOf(
        reduce(s, const SetMappingDriveRequested(mappingId: cruise, outputId: null)),
      );
      expect(undrive.setMappingDrive.hasOutputId(), isFalse);
      expect(
        opOf(reduce(s, const SetOutputRequiredRequested(id: motor, required: false)))
            .setOutputRequired
            .required,
        isFalse,
      );
      expect(
        opOf(reduce(s, const SetOutputClockRequested(id: motor, clockId: null))).setOutputClock
            .hasClockId(),
        isFalse,
      );
      expect(opOf(reduce(s, const DeleteOutputRequested(motor))).hasDeleteOutput(), isTrue);
      final dev = opOf(
        reduce(
          s,
          const CreateDeviceRequested(
            name: 'drive',
            kind: pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL,
            outputId: motor,
          ),
        ),
      );
      expect(dev.createDevice.kind, pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL);
      expect(dev.createDevice.outputId.toInt(), motor);
    });

    test('a required output is two steps: create, then mark required on the created id', () {
      final s = connected(rover(withOutput: false));
      final t = reduce(
        s,
        const CreateOutputRequested(name: 'motor', accepts: speed, clockId: main_, required: true),
      );
      expect(opOf(t).createOutput.name, 'motor');
      expect(t.state.editor.queuedEdits, hasLength(1));
      // the confirming projection carries the created id; the second step follows
      final confirmed = reduce(
        t.state,
        ProjectReceived(
          rover(revision: 2, required: false),
          outcome: pb.EditOutcome(
            kind: pb.EditKind.EDIT_KIND_REFINEMENT,
            createdOutput: Int64(motor),
          ),
        ),
      );
      final second = confirmed.effects.whereType<ApplyEdit>().single;
      expect(second.baseRevision, 2);
      expect(second.op.setOutputRequired.id.toInt(), motor);
      expect(second.op.setOutputRequired.required, isTrue);
      expect(confirmed.state.editor.queuedEdits, isEmpty);
      // an optional output is one step
      final one = reduce(
        s,
        const CreateOutputRequested(name: 'lamp', accepts: speed, required: false),
      );
      expect(one.state.editor.queuedEdits, isEmpty);
    });

    test('a failed step ends its plan', () {
      var s = connected(rover(withOutput: false));
      s = reduce(
        s,
        const CreateOutputRequested(name: 'motor', accepts: speed, required: true),
      ).state;
      s = reduce(s, const RequestFailed(code: 'edit.duplicate_output_name', message: 'x')).state;
      expect(s.editor.queuedEdits, isEmpty);
    });

    test('selecting an object asks the service for its fixes; a new revision re-asks', () {
      final t = reduce(connected(rover()), const SelectionChanged(OutputSelected(motor)));
      final e = t.effects.whereType<ListSemanticActions>().single;
      expect(e.entity.outputId.toInt(), motor);
      expect(t.state.editor.actions!.pending, isTrue);
      final again = reduce(t.state, ProjectReceived(rover(revision: 2), fromRequest: false));
      expect(again.effects.whereType<ListSemanticActions>().single.revision, 2);
      // deselecting clears
      expect(reduce(t.state, const SelectionChanged(NoSelection())).state.editor.actions, isNull);
    });

    test('a ready action applies its edits one revision at a time; a choice needs its option', () {
      var s = reduce(connected(rover()), const SelectionChanged(OutputSelected(motor))).state;
      final g = s.editor.actions!.generation;
      s = reduce(
        s,
        SemanticActionsReceived(
          generation: g,
          result: pb.SemanticActionsResponse(
            revision: Int64(1),
            actions: [
              pb.SemanticActionView(
                id: 'detach:1',
                title: 'Disconnect boost',
                kind: 'quick_fix',
                applicability: pb.ActionApplicability.ACTION_APPLICABILITY_READY,
                edits: [
                  pb.EditOp(setMappingDrive: pb.SetMappingDrive(id: Int64(boost))),
                  pb.EditOp(
                    renameOutput: pb.RenameOutput(id: Int64(motor), name: 'm'),
                  ),
                ],
              ),
              pb.SemanticActionView(
                id: 'connect:0',
                title: 'Connect a driver',
                kind: 'quick_fix',
                applicability: pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE,
                options: [
                  pb.ActionChoiceView(
                    label: 'cruise',
                    edit: pb.EditOp(
                      setMappingDrive: pb.SetMappingDrive(
                        id: Int64(cruise),
                        outputId: Int64(motor),
                      ),
                    ),
                  ),
                ],
              ),
              pb.SemanticActionView(
                id: 'sync:0',
                title: 'Insert explicit sync',
                kind: 'quick_fix',
                applicability: pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED,
                reason: 'no phrase yet',
              ),
            ],
          ),
        ),
      ).state;
      expect(s.editor.actions!.actions, hasLength(3));
      // ready: first edit now, second queued
      final ready = reduce(s, const SemanticActionApplied(actionId: 'detach:1'));
      expect(opOf(ready).hasSetMappingDrive(), isTrue);
      expect(ready.state.editor.queuedEdits.single.hasRenameOutput(), isTrue);
      expect(ready.state.editor.actions, isNull, reason: 'stale once an edit is sent');
      // needs choice: nothing without an option, the option's edit with one
      expect(reduce(s, const SemanticActionApplied(actionId: 'connect:0')).effects, isEmpty);
      final chosen = reduce(s, const SemanticActionApplied(actionId: 'connect:0', option: 0));
      expect(opOf(chosen).setMappingDrive.id.toInt(), cruise);
      // blocked: nothing
      expect(reduce(s, const SemanticActionApplied(actionId: 'sync:0')).effects, isEmpty);
      // an answer for another generation or revision is ignored
      final stale = reduce(
        s,
        SemanticActionsReceived(
          generation: g + 5,
          result: pb.SemanticActionsResponse(revision: Int64(1)),
        ),
      ).state;
      expect(stale.editor.actions!.actions, hasLength(3));
    });

    test('an output selection survives only while the output exists', () {
      var s = connected(rover(), selection: const OutputSelected(motor));
      s = reduce(s, ProjectReceived(rover(revision: 2, withOutput: false))).state;
      expect(s.editor.selection, isA<NoSelection>());
    });

    test('layout round-trips output positions', () {
      final layout = {
        const NodeRef.concept(0): const Offset(1, 2),
        const NodeRef.mapping(0): const Offset(3, 4),
        const NodeRef.output(0): const Offset(5, 6),
      };
      final back = layoutFromPb(layoutToPb(CanvasLayout(system: ContextLayout(nodes: layout))));
      expect(back.system.nodes, layout);
    });
  });

  group('geometry', () {
    test('an output is a sink node with one socket typed by what it accepts', () {
      final scene = buildScene(
        rover(),
        const {},
        outputStates: {motor: pb.OutputState.OUTPUT_STATE_DRIVEN},
      );
      final sink = scene.nodes.singleWhere((n) => n.ref == const NodeRef.output(motor));
      expect(sink.sockets, hasLength(1));
      expect(sink.sockets.single.ref.side, SocketSide.input);
      expect(sink.sockets.single.ref.concept, speed);
      expect(sink.sink, SinkState.driven);
      expect(sink.required, isTrue);
      expect(sink.timing, 'main');
      expect(sink.title, 'motor');
      // a sink sits to the right of mappings by default
      final mapping = scene.nodes.singleWhere((n) => n.ref == const NodeRef.mapping(cruise));
      expect(sink.rect.left, greaterThan(mapping.rect.right));
      expect(mapping.timing, 'main');
    });

    test(
      'the drive edge is a link from the driver into the sink; states read off the analysis',
      () {
        final scene = buildScene(rover(), const {});
        final drive = scene.links.where((l) => l.to.node == const NodeRef.output(motor)).toList();
        expect(drive, hasLength(1));
        expect(drive.single.from.node, const NodeRef.mapping(cruise));
        expect(drive.single.concept, speed);
        expect(
          buildScene(
            rover(outputClock: null),
            const {},
          ).nodes.singleWhere((n) => n.ref == const NodeRef.output(motor)).sink,
          SinkState.open,
        );
        expect(
          buildScene(
            rover(),
            const {},
            outputStates: {motor: pb.OutputState.OUTPUT_STATE_CONFLICT},
          ).nodes.singleWhere((n) => n.ref == const NodeRef.output(motor)).sink,
          SinkState.contested,
        );
      },
    );

    test('only a mapping output may land on a sink; a sink never feeds anything', () {
      const sinkIn = SocketRef(node: NodeRef.output(motor), side: SocketSide.input, concept: speed);
      const mapOut = SocketRef(
        node: NodeRef.mapping(boost),
        side: SocketSide.output,
        concept: speed,
      );
      const conceptOut = SocketRef(
        node: NodeRef.concept(speed),
        side: SocketSide.output,
        concept: speed,
      );
      const mapIn = SocketRef(node: NodeRef.mapping(boost), side: SocketSide.input, concept: speed);
      expect(canLink(mapOut, sinkIn), isTrue);
      expect(canLink(sinkIn, mapOut), isTrue);
      expect(canLink(conceptOut, sinkIn), isFalse);
      expect(canLink(sinkIn, mapIn), isFalse, reason: 'same side');
      // a different concept never links, sink or not
      const other = SocketRef(node: NodeRef.mapping(boost), side: SocketSide.output, concept: 9);
      expect(canLink(other, sinkIn), isFalse);
    });
  });

  group('inspector', () {
    testWidgets('the output inspector states the sink, lists claimants, connects and disconnects', (
      t,
    ) async {
      final key = GlobalKey<HarnessState>();
      await t.pumpWidget(
        Harness(
          key: key,
          initial: connected(
            rover(drivers: const [cruise, boost]),
            selection: const OutputSelected(motor),
          ).copyWith(analysis: analysisWith(pb.OutputState.OUTPUT_STATE_CONFLICT)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      final h = key.currentState!;
      final stateText = t.widget<Text>(find.byKey(const ValueKey('output-state'))).data!;
      expect(stateText, contains('already has a final target'));
      expect(stateText, contains('cruise and boost'));
      expect(find.text('disconnect'), findsNWidgets(2));
      await t.tap(find.text('disconnect').last);
      await t.pump();
      final op = h.effects.whereType<ApplyEdit>().single.op;
      expect(op.setMappingDrive.id.toInt(), boost);
      expect(op.setMappingDrive.hasOutputId(), isFalse);
      // the required checkbox is a model edit; delete is refused while driven
      await t.tap(find.byType(Checkbox));
      await t.pump();
      expect(h.effects.whereType<ApplyEdit>().last.op.setOutputRequired.required, isFalse);
      expect(
        t
            .widget<Tooltip>(
              find.ancestor(of: find.text('Delete motor'), matching: find.byType(Tooltip)),
            )
            .message,
        contains('Still driven by'),
      );
    });

    testWidgets('a driven sink is settled; an undriven required one is incomplete, not wrong', (
      t,
    ) async {
      await t.pumpWidget(
        Harness(
          key: UniqueKey(),
          initial: connected(
            rover(),
            selection: const OutputSelected(motor),
          ).copyWith(analysis: analysisWith(pb.OutputState.OUTPUT_STATE_DRIVEN, driver: cruise)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      var text = t.widget<Text>(find.byKey(const ValueKey('output-state')));
      expect(text.data, 'Driven by cruise.');
      final tokens = MacTokens.of(t.element(find.byKey(const ValueKey('output-state'))));
      expect(text.style!.color, tokens.settled);

      await t.pumpWidget(
        Harness(
          key: UniqueKey(),
          initial: connected(
            rover(drivers: const []),
            selection: const OutputSelected(motor),
          ).copyWith(analysis: analysisWith(pb.OutputState.OUTPUT_STATE_UNDRIVEN)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      text = t.widget<Text>(find.byKey(const ValueKey('output-state')));
      expect(text.data, contains('incomplete'));
      expect(text.style!.color, tokens.open);
      expect(text.style!.color, isNot(tokens.error));
    });

    testWidgets('the mapping inspector assigns a domain and a drive in product words', (t) async {
      final key = GlobalKey<HarnessState>();
      await t.pumpWidget(
        Harness(
          key: key,
          initial: connected(rover(), selection: const MappingSelected(boost)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Updates in'), findsOneWidget);
      expect(find.text('any domain'), findsOneWidget);
      expect(find.text('Drives'), findsOneWidget);
      expect(find.text('nothing'), findsOneWidget);
      expect(find.textContaining('ClockId'), findsNothing, reason: 'ids live in Explain only');
      await t.tap(find.text('any domain'));
      await t.pumpAndSettle();
      await t.tap(find.text('main').last);
      await t.pumpAndSettle();
      final op = key.currentState!.effects.whereType<ApplyEdit>().single.op;
      expect(op.setMappingClock.clockId.toInt(), main_);
    });

    testWidgets('the fixes list renders ready, choice and blocked actions from the service', (
      t,
    ) async {
      final key = GlobalKey<HarnessState>();
      var s = reduce(connected(rover()), const SelectionChanged(OutputSelected(motor))).state;
      s = reduce(
        s,
        SemanticActionsReceived(
          generation: s.editor.actions!.generation,
          result: pb.SemanticActionsResponse(
            revision: Int64(1),
            actions: [
              pb.SemanticActionView(
                id: 'a',
                title: 'Disconnect boost',
                kind: 'quick_fix',
                applicability: pb.ActionApplicability.ACTION_APPLICABILITY_READY,
                explanation: 'The sink keeps one driver.',
                edits: [pb.EditOp(setMappingDrive: pb.SetMappingDrive(id: Int64(boost)))],
              ),
              pb.SemanticActionView(
                id: 'b',
                title: 'Insert explicit sync',
                kind: 'quick_fix',
                applicability: pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED,
                reason: 'the surface language has no phrase for sync yet',
              ),
            ],
          ),
        ),
      ).state;
      await t.pumpWidget(
        Harness(
          key: key,
          initial: s,
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Fixes'), findsOneWidget);
      expect(find.text('The sink keeps one driver.'), findsOneWidget);
      expect(find.textContaining('Not possible yet'), findsOneWidget);
      await t.ensureVisible(find.text('Disconnect boost'));
      await t.tap(find.text('Disconnect boost'));
      await t.pump();
      expect(
        key.currentState!.effects.whereType<ApplyEdit>().single.op.hasSetMappingDrive(),
        isTrue,
      );
    });

    testWidgets('the library lists domains and outputs with their state glyphs', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(rover()),
          child: (s, d) => Library(state: s, dispatch: d),
        ),
      );
      expect(find.text('Timing domains'), findsOneWidget);
      expect(find.text('Outputs'), findsOneWidget);
      expect(find.text('motor'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget, reason: 'the domain name, renamed in place');
    });
  });
}
