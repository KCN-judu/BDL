// This is a generated file - do not edit.
//
// Generated from bdl/v1/bdl.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'bdl.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'bdl.pbenum.dart';

enum ClientMessage_Payload {
  handshake,
  openProject,
  initProject,
  saveProject,
  closeProject,
  getProject,
  applyEdit,
  undo,
  redo,
  setLayout,
  subscribeProject,
  shutdown,
  runAnalysis,
  startSimulation,
  stepSimulation,
  resetSimulation,
  listTargets,
  analyzeDeployment,
  analyzeDefinitionDraft,
  discardDefinitionDraft,
  completeDefinitionDraft,
  hoverDefinitionDraft,
  hoverEntity,
  listSemanticActions,
  notSet
}

class ClientMessage extends $pb.GeneratedMessage {
  factory ClientMessage({
    $fixnum.Int64? requestId,
    HandshakeRequest? handshake,
    OpenProjectRequest? openProject,
    InitProjectRequest? initProject,
    SaveProjectRequest? saveProject,
    CloseProjectRequest? closeProject,
    GetProjectRequest? getProject,
    ApplyEditRequest? applyEdit,
    UndoRequest? undo,
    RedoRequest? redo,
    SetLayoutRequest? setLayout,
    SubscribeProjectRequest? subscribeProject,
    ShutdownRequest? shutdown,
    RunAnalysisRequest? runAnalysis,
    StartSimulationRequest? startSimulation,
    StepSimulationRequest? stepSimulation,
    ResetSimulationRequest? resetSimulation,
    ListTargetsRequest? listTargets,
    AnalyzeDeploymentRequest? analyzeDeployment,
    AnalyzeDefinitionDraftRequest? analyzeDefinitionDraft,
    DiscardDefinitionDraftRequest? discardDefinitionDraft,
    CompleteDefinitionDraftRequest? completeDefinitionDraft,
    HoverDefinitionDraftRequest? hoverDefinitionDraft,
    HoverEntityRequest? hoverEntity,
    ListSemanticActionsRequest? listSemanticActions,
  }) {
    final result = ClientMessage._();
    if (requestId != null) result.requestId = requestId;
    if (handshake != null) result.handshake = handshake;
    if (openProject != null) result.openProject = openProject;
    if (initProject != null) result.initProject = initProject;
    if (saveProject != null) result.saveProject = saveProject;
    if (closeProject != null) result.closeProject = closeProject;
    if (getProject != null) result.getProject = getProject;
    if (applyEdit != null) result.applyEdit = applyEdit;
    if (undo != null) result.undo = undo;
    if (redo != null) result.redo = redo;
    if (setLayout != null) result.setLayout = setLayout;
    if (subscribeProject != null) result.subscribeProject = subscribeProject;
    if (shutdown != null) result.shutdown = shutdown;
    if (runAnalysis != null) result.runAnalysis = runAnalysis;
    if (startSimulation != null) result.startSimulation = startSimulation;
    if (stepSimulation != null) result.stepSimulation = stepSimulation;
    if (resetSimulation != null) result.resetSimulation = resetSimulation;
    if (listTargets != null) result.listTargets = listTargets;
    if (analyzeDeployment != null) result.analyzeDeployment = analyzeDeployment;
    if (analyzeDefinitionDraft != null) result.analyzeDefinitionDraft = analyzeDefinitionDraft;
    if (discardDefinitionDraft != null) result.discardDefinitionDraft = discardDefinitionDraft;
    if (completeDefinitionDraft != null) result.completeDefinitionDraft = completeDefinitionDraft;
    if (hoverDefinitionDraft != null) result.hoverDefinitionDraft = hoverDefinitionDraft;
    if (hoverEntity != null) result.hoverEntity = hoverEntity;
    if (listSemanticActions != null) result.listSemanticActions = listSemanticActions;
    return result;
  }

  ClientMessage._();

  factory ClientMessage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClientMessage()..mergeFromBuffer(data, registry);
  factory ClientMessage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClientMessage()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ClientMessage_Payload> _ClientMessage_PayloadByTag = {
    10: ClientMessage_Payload.handshake,
    11: ClientMessage_Payload.openProject,
    12: ClientMessage_Payload.initProject,
    13: ClientMessage_Payload.saveProject,
    14: ClientMessage_Payload.closeProject,
    15: ClientMessage_Payload.getProject,
    16: ClientMessage_Payload.applyEdit,
    17: ClientMessage_Payload.undo,
    18: ClientMessage_Payload.redo,
    19: ClientMessage_Payload.setLayout,
    20: ClientMessage_Payload.subscribeProject,
    21: ClientMessage_Payload.shutdown,
    22: ClientMessage_Payload.runAnalysis,
    23: ClientMessage_Payload.startSimulation,
    24: ClientMessage_Payload.stepSimulation,
    25: ClientMessage_Payload.resetSimulation,
    26: ClientMessage_Payload.listTargets,
    27: ClientMessage_Payload.analyzeDeployment,
    28: ClientMessage_Payload.analyzeDefinitionDraft,
    29: ClientMessage_Payload.discardDefinitionDraft,
    30: ClientMessage_Payload.completeDefinitionDraft,
    31: ClientMessage_Payload.hoverDefinitionDraft,
    32: ClientMessage_Payload.hoverEntity,
    33: ClientMessage_Payload.listSemanticActions,
    0: ClientMessage_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ClientMessage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ClientMessage.$_createMessage)
    ..oo(0, [
      10,
      11,
      12,
      13,
      14,
      15,
      16,
      17,
      18,
      19,
      20,
      21,
      22,
      23,
      24,
      25,
      26,
      27,
      28,
      29,
      30,
      31,
      32,
      33
    ])
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'requestId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<HandshakeRequest>(10, _omitFieldNames ? '' : 'handshake',
        subBuilder: HandshakeRequest.$_createMessage)
    ..aOM<OpenProjectRequest>(11, _omitFieldNames ? '' : 'openProject',
        subBuilder: OpenProjectRequest.$_createMessage)
    ..aOM<InitProjectRequest>(12, _omitFieldNames ? '' : 'initProject',
        subBuilder: InitProjectRequest.$_createMessage)
    ..aOM<SaveProjectRequest>(13, _omitFieldNames ? '' : 'saveProject',
        subBuilder: SaveProjectRequest.$_createMessage)
    ..aOM<CloseProjectRequest>(14, _omitFieldNames ? '' : 'closeProject',
        subBuilder: CloseProjectRequest.$_createMessage)
    ..aOM<GetProjectRequest>(15, _omitFieldNames ? '' : 'getProject',
        subBuilder: GetProjectRequest.$_createMessage)
    ..aOM<ApplyEditRequest>(16, _omitFieldNames ? '' : 'applyEdit',
        subBuilder: ApplyEditRequest.$_createMessage)
    ..aOM<UndoRequest>(17, _omitFieldNames ? '' : 'undo', subBuilder: UndoRequest.$_createMessage)
    ..aOM<RedoRequest>(18, _omitFieldNames ? '' : 'redo', subBuilder: RedoRequest.$_createMessage)
    ..aOM<SetLayoutRequest>(19, _omitFieldNames ? '' : 'setLayout',
        subBuilder: SetLayoutRequest.$_createMessage)
    ..aOM<SubscribeProjectRequest>(20, _omitFieldNames ? '' : 'subscribeProject',
        subBuilder: SubscribeProjectRequest.$_createMessage)
    ..aOM<ShutdownRequest>(21, _omitFieldNames ? '' : 'shutdown',
        subBuilder: ShutdownRequest.$_createMessage)
    ..aOM<RunAnalysisRequest>(22, _omitFieldNames ? '' : 'runAnalysis',
        subBuilder: RunAnalysisRequest.$_createMessage)
    ..aOM<StartSimulationRequest>(23, _omitFieldNames ? '' : 'startSimulation',
        subBuilder: StartSimulationRequest.$_createMessage)
    ..aOM<StepSimulationRequest>(24, _omitFieldNames ? '' : 'stepSimulation',
        subBuilder: StepSimulationRequest.$_createMessage)
    ..aOM<ResetSimulationRequest>(25, _omitFieldNames ? '' : 'resetSimulation',
        subBuilder: ResetSimulationRequest.$_createMessage)
    ..aOM<ListTargetsRequest>(26, _omitFieldNames ? '' : 'listTargets',
        subBuilder: ListTargetsRequest.$_createMessage)
    ..aOM<AnalyzeDeploymentRequest>(27, _omitFieldNames ? '' : 'analyzeDeployment',
        subBuilder: AnalyzeDeploymentRequest.$_createMessage)
    ..aOM<AnalyzeDefinitionDraftRequest>(28, _omitFieldNames ? '' : 'analyzeDefinitionDraft',
        subBuilder: AnalyzeDefinitionDraftRequest.$_createMessage)
    ..aOM<DiscardDefinitionDraftRequest>(29, _omitFieldNames ? '' : 'discardDefinitionDraft',
        subBuilder: DiscardDefinitionDraftRequest.$_createMessage)
    ..aOM<CompleteDefinitionDraftRequest>(30, _omitFieldNames ? '' : 'completeDefinitionDraft',
        subBuilder: CompleteDefinitionDraftRequest.$_createMessage)
    ..aOM<HoverDefinitionDraftRequest>(31, _omitFieldNames ? '' : 'hoverDefinitionDraft',
        subBuilder: HoverDefinitionDraftRequest.$_createMessage)
    ..aOM<HoverEntityRequest>(32, _omitFieldNames ? '' : 'hoverEntity',
        subBuilder: HoverEntityRequest.$_createMessage)
    ..aOM<ListSemanticActionsRequest>(33, _omitFieldNames ? '' : 'listSemanticActions',
        subBuilder: ListSemanticActionsRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientMessage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientMessage copyWith(void Function(ClientMessage) updates) =>
      super.copyWith((message) => updates(message as ClientMessage)) as ClientMessage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ClientMessage() / ClientMessage.new instead')
  static ClientMessage create() => ClientMessage._();
  static $pb.GeneratedMessage $_createMessage() => ClientMessage._();
  @$core.override
  ClientMessage createEmptyInstance() => ClientMessage._();
  @$core.pragma('dart2js:noInline')
  static ClientMessage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientMessage>(ClientMessage.$_createMessage);
  static ClientMessage? _defaultInstance;

  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  @$pb.TagNumber(22)
  @$pb.TagNumber(23)
  @$pb.TagNumber(24)
  @$pb.TagNumber(25)
  @$pb.TagNumber(26)
  @$pb.TagNumber(27)
  @$pb.TagNumber(28)
  @$pb.TagNumber(29)
  @$pb.TagNumber(30)
  @$pb.TagNumber(31)
  @$pb.TagNumber(32)
  @$pb.TagNumber(33)
  ClientMessage_Payload whichPayload() => _ClientMessage_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  @$pb.TagNumber(22)
  @$pb.TagNumber(23)
  @$pb.TagNumber(24)
  @$pb.TagNumber(25)
  @$pb.TagNumber(26)
  @$pb.TagNumber(27)
  @$pb.TagNumber(28)
  @$pb.TagNumber(29)
  @$pb.TagNumber(30)
  @$pb.TagNumber(31)
  @$pb.TagNumber(32)
  @$pb.TagNumber(33)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get requestId => $_getI64(0);
  @$pb.TagNumber(1)
  set requestId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRequestId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRequestId() => $_clearField(1);

  @$pb.TagNumber(10)
  HandshakeRequest get handshake => $_getN(1);
  @$pb.TagNumber(10)
  set handshake(HandshakeRequest value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasHandshake() => $_has(1);
  @$pb.TagNumber(10)
  void clearHandshake() => $_clearField(10);
  @$pb.TagNumber(10)
  HandshakeRequest ensureHandshake() => $_ensure(1);

  @$pb.TagNumber(11)
  OpenProjectRequest get openProject => $_getN(2);
  @$pb.TagNumber(11)
  set openProject(OpenProjectRequest value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasOpenProject() => $_has(2);
  @$pb.TagNumber(11)
  void clearOpenProject() => $_clearField(11);
  @$pb.TagNumber(11)
  OpenProjectRequest ensureOpenProject() => $_ensure(2);

  @$pb.TagNumber(12)
  InitProjectRequest get initProject => $_getN(3);
  @$pb.TagNumber(12)
  set initProject(InitProjectRequest value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasInitProject() => $_has(3);
  @$pb.TagNumber(12)
  void clearInitProject() => $_clearField(12);
  @$pb.TagNumber(12)
  InitProjectRequest ensureInitProject() => $_ensure(3);

  @$pb.TagNumber(13)
  SaveProjectRequest get saveProject => $_getN(4);
  @$pb.TagNumber(13)
  set saveProject(SaveProjectRequest value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasSaveProject() => $_has(4);
  @$pb.TagNumber(13)
  void clearSaveProject() => $_clearField(13);
  @$pb.TagNumber(13)
  SaveProjectRequest ensureSaveProject() => $_ensure(4);

  @$pb.TagNumber(14)
  CloseProjectRequest get closeProject => $_getN(5);
  @$pb.TagNumber(14)
  set closeProject(CloseProjectRequest value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasCloseProject() => $_has(5);
  @$pb.TagNumber(14)
  void clearCloseProject() => $_clearField(14);
  @$pb.TagNumber(14)
  CloseProjectRequest ensureCloseProject() => $_ensure(5);

  @$pb.TagNumber(15)
  GetProjectRequest get getProject => $_getN(6);
  @$pb.TagNumber(15)
  set getProject(GetProjectRequest value) => $_setField(15, value);
  @$pb.TagNumber(15)
  $core.bool hasGetProject() => $_has(6);
  @$pb.TagNumber(15)
  void clearGetProject() => $_clearField(15);
  @$pb.TagNumber(15)
  GetProjectRequest ensureGetProject() => $_ensure(6);

  @$pb.TagNumber(16)
  ApplyEditRequest get applyEdit => $_getN(7);
  @$pb.TagNumber(16)
  set applyEdit(ApplyEditRequest value) => $_setField(16, value);
  @$pb.TagNumber(16)
  $core.bool hasApplyEdit() => $_has(7);
  @$pb.TagNumber(16)
  void clearApplyEdit() => $_clearField(16);
  @$pb.TagNumber(16)
  ApplyEditRequest ensureApplyEdit() => $_ensure(7);

  @$pb.TagNumber(17)
  UndoRequest get undo => $_getN(8);
  @$pb.TagNumber(17)
  set undo(UndoRequest value) => $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasUndo() => $_has(8);
  @$pb.TagNumber(17)
  void clearUndo() => $_clearField(17);
  @$pb.TagNumber(17)
  UndoRequest ensureUndo() => $_ensure(8);

  @$pb.TagNumber(18)
  RedoRequest get redo => $_getN(9);
  @$pb.TagNumber(18)
  set redo(RedoRequest value) => $_setField(18, value);
  @$pb.TagNumber(18)
  $core.bool hasRedo() => $_has(9);
  @$pb.TagNumber(18)
  void clearRedo() => $_clearField(18);
  @$pb.TagNumber(18)
  RedoRequest ensureRedo() => $_ensure(9);

  @$pb.TagNumber(19)
  SetLayoutRequest get setLayout => $_getN(10);
  @$pb.TagNumber(19)
  set setLayout(SetLayoutRequest value) => $_setField(19, value);
  @$pb.TagNumber(19)
  $core.bool hasSetLayout() => $_has(10);
  @$pb.TagNumber(19)
  void clearSetLayout() => $_clearField(19);
  @$pb.TagNumber(19)
  SetLayoutRequest ensureSetLayout() => $_ensure(10);

  @$pb.TagNumber(20)
  SubscribeProjectRequest get subscribeProject => $_getN(11);
  @$pb.TagNumber(20)
  set subscribeProject(SubscribeProjectRequest value) => $_setField(20, value);
  @$pb.TagNumber(20)
  $core.bool hasSubscribeProject() => $_has(11);
  @$pb.TagNumber(20)
  void clearSubscribeProject() => $_clearField(20);
  @$pb.TagNumber(20)
  SubscribeProjectRequest ensureSubscribeProject() => $_ensure(11);

  @$pb.TagNumber(21)
  ShutdownRequest get shutdown => $_getN(12);
  @$pb.TagNumber(21)
  set shutdown(ShutdownRequest value) => $_setField(21, value);
  @$pb.TagNumber(21)
  $core.bool hasShutdown() => $_has(12);
  @$pb.TagNumber(21)
  void clearShutdown() => $_clearField(21);
  @$pb.TagNumber(21)
  ShutdownRequest ensureShutdown() => $_ensure(12);

  @$pb.TagNumber(22)
  RunAnalysisRequest get runAnalysis => $_getN(13);
  @$pb.TagNumber(22)
  set runAnalysis(RunAnalysisRequest value) => $_setField(22, value);
  @$pb.TagNumber(22)
  $core.bool hasRunAnalysis() => $_has(13);
  @$pb.TagNumber(22)
  void clearRunAnalysis() => $_clearField(22);
  @$pb.TagNumber(22)
  RunAnalysisRequest ensureRunAnalysis() => $_ensure(13);

  @$pb.TagNumber(23)
  StartSimulationRequest get startSimulation => $_getN(14);
  @$pb.TagNumber(23)
  set startSimulation(StartSimulationRequest value) => $_setField(23, value);
  @$pb.TagNumber(23)
  $core.bool hasStartSimulation() => $_has(14);
  @$pb.TagNumber(23)
  void clearStartSimulation() => $_clearField(23);
  @$pb.TagNumber(23)
  StartSimulationRequest ensureStartSimulation() => $_ensure(14);

  @$pb.TagNumber(24)
  StepSimulationRequest get stepSimulation => $_getN(15);
  @$pb.TagNumber(24)
  set stepSimulation(StepSimulationRequest value) => $_setField(24, value);
  @$pb.TagNumber(24)
  $core.bool hasStepSimulation() => $_has(15);
  @$pb.TagNumber(24)
  void clearStepSimulation() => $_clearField(24);
  @$pb.TagNumber(24)
  StepSimulationRequest ensureStepSimulation() => $_ensure(15);

  @$pb.TagNumber(25)
  ResetSimulationRequest get resetSimulation => $_getN(16);
  @$pb.TagNumber(25)
  set resetSimulation(ResetSimulationRequest value) => $_setField(25, value);
  @$pb.TagNumber(25)
  $core.bool hasResetSimulation() => $_has(16);
  @$pb.TagNumber(25)
  void clearResetSimulation() => $_clearField(25);
  @$pb.TagNumber(25)
  ResetSimulationRequest ensureResetSimulation() => $_ensure(16);

  @$pb.TagNumber(26)
  ListTargetsRequest get listTargets => $_getN(17);
  @$pb.TagNumber(26)
  set listTargets(ListTargetsRequest value) => $_setField(26, value);
  @$pb.TagNumber(26)
  $core.bool hasListTargets() => $_has(17);
  @$pb.TagNumber(26)
  void clearListTargets() => $_clearField(26);
  @$pb.TagNumber(26)
  ListTargetsRequest ensureListTargets() => $_ensure(17);

  @$pb.TagNumber(27)
  AnalyzeDeploymentRequest get analyzeDeployment => $_getN(18);
  @$pb.TagNumber(27)
  set analyzeDeployment(AnalyzeDeploymentRequest value) => $_setField(27, value);
  @$pb.TagNumber(27)
  $core.bool hasAnalyzeDeployment() => $_has(18);
  @$pb.TagNumber(27)
  void clearAnalyzeDeployment() => $_clearField(27);
  @$pb.TagNumber(27)
  AnalyzeDeploymentRequest ensureAnalyzeDeployment() => $_ensure(18);

  @$pb.TagNumber(28)
  AnalyzeDefinitionDraftRequest get analyzeDefinitionDraft => $_getN(19);
  @$pb.TagNumber(28)
  set analyzeDefinitionDraft(AnalyzeDefinitionDraftRequest value) => $_setField(28, value);
  @$pb.TagNumber(28)
  $core.bool hasAnalyzeDefinitionDraft() => $_has(19);
  @$pb.TagNumber(28)
  void clearAnalyzeDefinitionDraft() => $_clearField(28);
  @$pb.TagNumber(28)
  AnalyzeDefinitionDraftRequest ensureAnalyzeDefinitionDraft() => $_ensure(19);

  @$pb.TagNumber(29)
  DiscardDefinitionDraftRequest get discardDefinitionDraft => $_getN(20);
  @$pb.TagNumber(29)
  set discardDefinitionDraft(DiscardDefinitionDraftRequest value) => $_setField(29, value);
  @$pb.TagNumber(29)
  $core.bool hasDiscardDefinitionDraft() => $_has(20);
  @$pb.TagNumber(29)
  void clearDiscardDefinitionDraft() => $_clearField(29);
  @$pb.TagNumber(29)
  DiscardDefinitionDraftRequest ensureDiscardDefinitionDraft() => $_ensure(20);

  @$pb.TagNumber(30)
  CompleteDefinitionDraftRequest get completeDefinitionDraft => $_getN(21);
  @$pb.TagNumber(30)
  set completeDefinitionDraft(CompleteDefinitionDraftRequest value) => $_setField(30, value);
  @$pb.TagNumber(30)
  $core.bool hasCompleteDefinitionDraft() => $_has(21);
  @$pb.TagNumber(30)
  void clearCompleteDefinitionDraft() => $_clearField(30);
  @$pb.TagNumber(30)
  CompleteDefinitionDraftRequest ensureCompleteDefinitionDraft() => $_ensure(21);

  @$pb.TagNumber(31)
  HoverDefinitionDraftRequest get hoverDefinitionDraft => $_getN(22);
  @$pb.TagNumber(31)
  set hoverDefinitionDraft(HoverDefinitionDraftRequest value) => $_setField(31, value);
  @$pb.TagNumber(31)
  $core.bool hasHoverDefinitionDraft() => $_has(22);
  @$pb.TagNumber(31)
  void clearHoverDefinitionDraft() => $_clearField(31);
  @$pb.TagNumber(31)
  HoverDefinitionDraftRequest ensureHoverDefinitionDraft() => $_ensure(22);

  @$pb.TagNumber(32)
  HoverEntityRequest get hoverEntity => $_getN(23);
  @$pb.TagNumber(32)
  set hoverEntity(HoverEntityRequest value) => $_setField(32, value);
  @$pb.TagNumber(32)
  $core.bool hasHoverEntity() => $_has(23);
  @$pb.TagNumber(32)
  void clearHoverEntity() => $_clearField(32);
  @$pb.TagNumber(32)
  HoverEntityRequest ensureHoverEntity() => $_ensure(23);

  @$pb.TagNumber(33)
  ListSemanticActionsRequest get listSemanticActions => $_getN(24);
  @$pb.TagNumber(33)
  set listSemanticActions(ListSemanticActionsRequest value) => $_setField(33, value);
  @$pb.TagNumber(33)
  $core.bool hasListSemanticActions() => $_has(24);
  @$pb.TagNumber(33)
  void clearListSemanticActions() => $_clearField(33);
  @$pb.TagNumber(33)
  ListSemanticActionsRequest ensureListSemanticActions() => $_ensure(24);
}

enum ServerMessage_Payload { response, event, notSet }

class ServerMessage extends $pb.GeneratedMessage {
  factory ServerMessage({
    Response? response,
    Event? event,
  }) {
    final result = ServerMessage._();
    if (response != null) result.response = response;
    if (event != null) result.event = event;
    return result;
  }

  ServerMessage._();

  factory ServerMessage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ServerMessage()..mergeFromBuffer(data, registry);
  factory ServerMessage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ServerMessage()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ServerMessage_Payload> _ServerMessage_PayloadByTag = {
    1: ServerMessage_Payload.response,
    2: ServerMessage_Payload.event,
    0: ServerMessage_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ServerMessage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ServerMessage.$_createMessage)
    ..oo(0, [1, 2])
    ..aOM<Response>(1, _omitFieldNames ? '' : 'response', subBuilder: Response.$_createMessage)
    ..aOM<Event>(2, _omitFieldNames ? '' : 'event', subBuilder: Event.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServerMessage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServerMessage copyWith(void Function(ServerMessage) updates) =>
      super.copyWith((message) => updates(message as ServerMessage)) as ServerMessage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ServerMessage() / ServerMessage.new instead')
  static ServerMessage create() => ServerMessage._();
  static $pb.GeneratedMessage $_createMessage() => ServerMessage._();
  @$core.override
  ServerMessage createEmptyInstance() => ServerMessage._();
  @$core.pragma('dart2js:noInline')
  static ServerMessage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ServerMessage>(ServerMessage.$_createMessage);
  static ServerMessage? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  ServerMessage_Payload whichPayload() => _ServerMessage_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Response get response => $_getN(0);
  @$pb.TagNumber(1)
  set response(Response value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasResponse() => $_has(0);
  @$pb.TagNumber(1)
  void clearResponse() => $_clearField(1);
  @$pb.TagNumber(1)
  Response ensureResponse() => $_ensure(0);

  @$pb.TagNumber(2)
  Event get event => $_getN(1);
  @$pb.TagNumber(2)
  set event(Event value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEvent() => $_has(1);
  @$pb.TagNumber(2)
  void clearEvent() => $_clearField(2);
  @$pb.TagNumber(2)
  Event ensureEvent() => $_ensure(1);
}

enum Response_Payload {
  error,
  handshake,
  project,
  editApplied,
  ack,
  analysis,
  simulation,
  targets,
  deployment,
  definitionDraft,
  draftCompletion,
  draftHover,
  semanticActions,
  notSet
}

class Response extends $pb.GeneratedMessage {
  factory Response({
    $fixnum.Int64? requestId,
    Error? error,
    HandshakeResponse? handshake,
    ProjectResponse? project,
    EditApplied? editApplied,
    Ack? ack,
    AnalysisResponse? analysis,
    SimulationResponse? simulation,
    TargetsResponse? targets,
    DeploymentResponse? deployment,
    DefinitionDraftAnalysis? definitionDraft,
    DraftCompletionResponse? draftCompletion,
    DraftHoverResponse? draftHover,
    SemanticActionsResponse? semanticActions,
  }) {
    final result = Response._();
    if (requestId != null) result.requestId = requestId;
    if (error != null) result.error = error;
    if (handshake != null) result.handshake = handshake;
    if (project != null) result.project = project;
    if (editApplied != null) result.editApplied = editApplied;
    if (ack != null) result.ack = ack;
    if (analysis != null) result.analysis = analysis;
    if (simulation != null) result.simulation = simulation;
    if (targets != null) result.targets = targets;
    if (deployment != null) result.deployment = deployment;
    if (definitionDraft != null) result.definitionDraft = definitionDraft;
    if (draftCompletion != null) result.draftCompletion = draftCompletion;
    if (draftHover != null) result.draftHover = draftHover;
    if (semanticActions != null) result.semanticActions = semanticActions;
    return result;
  }

  Response._();

  factory Response.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Response()..mergeFromBuffer(data, registry);
  factory Response.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Response()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Response_Payload> _Response_PayloadByTag = {
    2: Response_Payload.error,
    10: Response_Payload.handshake,
    11: Response_Payload.project,
    12: Response_Payload.editApplied,
    13: Response_Payload.ack,
    14: Response_Payload.analysis,
    15: Response_Payload.simulation,
    16: Response_Payload.targets,
    17: Response_Payload.deployment,
    18: Response_Payload.definitionDraft,
    19: Response_Payload.draftCompletion,
    20: Response_Payload.draftHover,
    21: Response_Payload.semanticActions,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Response.$_createMessage)
    ..oo(0, [2, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21])
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'requestId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Error>(2, _omitFieldNames ? '' : 'error', subBuilder: Error.$_createMessage)
    ..aOM<HandshakeResponse>(10, _omitFieldNames ? '' : 'handshake',
        subBuilder: HandshakeResponse.$_createMessage)
    ..aOM<ProjectResponse>(11, _omitFieldNames ? '' : 'project',
        subBuilder: ProjectResponse.$_createMessage)
    ..aOM<EditApplied>(12, _omitFieldNames ? '' : 'editApplied',
        subBuilder: EditApplied.$_createMessage)
    ..aOM<Ack>(13, _omitFieldNames ? '' : 'ack', subBuilder: Ack.$_createMessage)
    ..aOM<AnalysisResponse>(14, _omitFieldNames ? '' : 'analysis',
        subBuilder: AnalysisResponse.$_createMessage)
    ..aOM<SimulationResponse>(15, _omitFieldNames ? '' : 'simulation',
        subBuilder: SimulationResponse.$_createMessage)
    ..aOM<TargetsResponse>(16, _omitFieldNames ? '' : 'targets',
        subBuilder: TargetsResponse.$_createMessage)
    ..aOM<DeploymentResponse>(17, _omitFieldNames ? '' : 'deployment',
        subBuilder: DeploymentResponse.$_createMessage)
    ..aOM<DefinitionDraftAnalysis>(18, _omitFieldNames ? '' : 'definitionDraft',
        subBuilder: DefinitionDraftAnalysis.$_createMessage)
    ..aOM<DraftCompletionResponse>(19, _omitFieldNames ? '' : 'draftCompletion',
        subBuilder: DraftCompletionResponse.$_createMessage)
    ..aOM<DraftHoverResponse>(20, _omitFieldNames ? '' : 'draftHover',
        subBuilder: DraftHoverResponse.$_createMessage)
    ..aOM<SemanticActionsResponse>(21, _omitFieldNames ? '' : 'semanticActions',
        subBuilder: SemanticActionsResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Response clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Response copyWith(void Function(Response) updates) =>
      super.copyWith((message) => updates(message as Response)) as Response;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Response() / Response.new instead')
  static Response create() => Response._();
  static $pb.GeneratedMessage $_createMessage() => Response._();
  @$core.override
  Response createEmptyInstance() => Response._();
  @$core.pragma('dart2js:noInline')
  static Response getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Response>(Response.$_createMessage);
  static Response? _defaultInstance;

  @$pb.TagNumber(2)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(2)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get requestId => $_getI64(0);
  @$pb.TagNumber(1)
  set requestId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRequestId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRequestId() => $_clearField(1);

  @$pb.TagNumber(2)
  Error get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(Error value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
  @$pb.TagNumber(2)
  Error ensureError() => $_ensure(1);

  @$pb.TagNumber(10)
  HandshakeResponse get handshake => $_getN(2);
  @$pb.TagNumber(10)
  set handshake(HandshakeResponse value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasHandshake() => $_has(2);
  @$pb.TagNumber(10)
  void clearHandshake() => $_clearField(10);
  @$pb.TagNumber(10)
  HandshakeResponse ensureHandshake() => $_ensure(2);

  @$pb.TagNumber(11)
  ProjectResponse get project => $_getN(3);
  @$pb.TagNumber(11)
  set project(ProjectResponse value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasProject() => $_has(3);
  @$pb.TagNumber(11)
  void clearProject() => $_clearField(11);
  @$pb.TagNumber(11)
  ProjectResponse ensureProject() => $_ensure(3);

  @$pb.TagNumber(12)
  EditApplied get editApplied => $_getN(4);
  @$pb.TagNumber(12)
  set editApplied(EditApplied value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasEditApplied() => $_has(4);
  @$pb.TagNumber(12)
  void clearEditApplied() => $_clearField(12);
  @$pb.TagNumber(12)
  EditApplied ensureEditApplied() => $_ensure(4);

  @$pb.TagNumber(13)
  Ack get ack => $_getN(5);
  @$pb.TagNumber(13)
  set ack(Ack value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasAck() => $_has(5);
  @$pb.TagNumber(13)
  void clearAck() => $_clearField(13);
  @$pb.TagNumber(13)
  Ack ensureAck() => $_ensure(5);

  @$pb.TagNumber(14)
  AnalysisResponse get analysis => $_getN(6);
  @$pb.TagNumber(14)
  set analysis(AnalysisResponse value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasAnalysis() => $_has(6);
  @$pb.TagNumber(14)
  void clearAnalysis() => $_clearField(14);
  @$pb.TagNumber(14)
  AnalysisResponse ensureAnalysis() => $_ensure(6);

  @$pb.TagNumber(15)
  SimulationResponse get simulation => $_getN(7);
  @$pb.TagNumber(15)
  set simulation(SimulationResponse value) => $_setField(15, value);
  @$pb.TagNumber(15)
  $core.bool hasSimulation() => $_has(7);
  @$pb.TagNumber(15)
  void clearSimulation() => $_clearField(15);
  @$pb.TagNumber(15)
  SimulationResponse ensureSimulation() => $_ensure(7);

  @$pb.TagNumber(16)
  TargetsResponse get targets => $_getN(8);
  @$pb.TagNumber(16)
  set targets(TargetsResponse value) => $_setField(16, value);
  @$pb.TagNumber(16)
  $core.bool hasTargets() => $_has(8);
  @$pb.TagNumber(16)
  void clearTargets() => $_clearField(16);
  @$pb.TagNumber(16)
  TargetsResponse ensureTargets() => $_ensure(8);

  @$pb.TagNumber(17)
  DeploymentResponse get deployment => $_getN(9);
  @$pb.TagNumber(17)
  set deployment(DeploymentResponse value) => $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasDeployment() => $_has(9);
  @$pb.TagNumber(17)
  void clearDeployment() => $_clearField(17);
  @$pb.TagNumber(17)
  DeploymentResponse ensureDeployment() => $_ensure(9);

  @$pb.TagNumber(18)
  DefinitionDraftAnalysis get definitionDraft => $_getN(10);
  @$pb.TagNumber(18)
  set definitionDraft(DefinitionDraftAnalysis value) => $_setField(18, value);
  @$pb.TagNumber(18)
  $core.bool hasDefinitionDraft() => $_has(10);
  @$pb.TagNumber(18)
  void clearDefinitionDraft() => $_clearField(18);
  @$pb.TagNumber(18)
  DefinitionDraftAnalysis ensureDefinitionDraft() => $_ensure(10);

  @$pb.TagNumber(19)
  DraftCompletionResponse get draftCompletion => $_getN(11);
  @$pb.TagNumber(19)
  set draftCompletion(DraftCompletionResponse value) => $_setField(19, value);
  @$pb.TagNumber(19)
  $core.bool hasDraftCompletion() => $_has(11);
  @$pb.TagNumber(19)
  void clearDraftCompletion() => $_clearField(19);
  @$pb.TagNumber(19)
  DraftCompletionResponse ensureDraftCompletion() => $_ensure(11);

  @$pb.TagNumber(20)
  DraftHoverResponse get draftHover => $_getN(12);
  @$pb.TagNumber(20)
  set draftHover(DraftHoverResponse value) => $_setField(20, value);
  @$pb.TagNumber(20)
  $core.bool hasDraftHover() => $_has(12);
  @$pb.TagNumber(20)
  void clearDraftHover() => $_clearField(20);
  @$pb.TagNumber(20)
  DraftHoverResponse ensureDraftHover() => $_ensure(12);

  @$pb.TagNumber(21)
  SemanticActionsResponse get semanticActions => $_getN(13);
  @$pb.TagNumber(21)
  set semanticActions(SemanticActionsResponse value) => $_setField(21, value);
  @$pb.TagNumber(21)
  $core.bool hasSemanticActions() => $_has(13);
  @$pb.TagNumber(21)
  void clearSemanticActions() => $_clearField(21);
  @$pb.TagNumber(21)
  SemanticActionsResponse ensureSemanticActions() => $_ensure(13);
}

enum Event_Payload { projectChanged, log, analysisReady, notSet }

class Event extends $pb.GeneratedMessage {
  factory Event({
    ProjectChanged? projectChanged,
    DaemonLog? log,
    AnalysisReady? analysisReady,
  }) {
    final result = Event._();
    if (projectChanged != null) result.projectChanged = projectChanged;
    if (log != null) result.log = log;
    if (analysisReady != null) result.analysisReady = analysisReady;
    return result;
  }

  Event._();

  factory Event.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Event()..mergeFromBuffer(data, registry);
  factory Event.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Event()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Event_Payload> _Event_PayloadByTag = {
    1: Event_Payload.projectChanged,
    2: Event_Payload.log,
    3: Event_Payload.analysisReady,
    0: Event_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Event',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Event.$_createMessage)
    ..oo(0, [1, 2, 3])
    ..aOM<ProjectChanged>(1, _omitFieldNames ? '' : 'projectChanged',
        subBuilder: ProjectChanged.$_createMessage)
    ..aOM<DaemonLog>(2, _omitFieldNames ? '' : 'log', subBuilder: DaemonLog.$_createMessage)
    ..aOM<AnalysisReady>(3, _omitFieldNames ? '' : 'analysisReady',
        subBuilder: AnalysisReady.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Event clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Event copyWith(void Function(Event) updates) =>
      super.copyWith((message) => updates(message as Event)) as Event;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Event() / Event.new instead')
  static Event create() => Event._();
  static $pb.GeneratedMessage $_createMessage() => Event._();
  @$core.override
  Event createEmptyInstance() => Event._();
  @$core.pragma('dart2js:noInline')
  static Event getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Event>(Event.$_createMessage);
  static Event? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  Event_Payload whichPayload() => _Event_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  ProjectChanged get projectChanged => $_getN(0);
  @$pb.TagNumber(1)
  set projectChanged(ProjectChanged value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProjectChanged() => $_has(0);
  @$pb.TagNumber(1)
  void clearProjectChanged() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectChanged ensureProjectChanged() => $_ensure(0);

  @$pb.TagNumber(2)
  DaemonLog get log => $_getN(1);
  @$pb.TagNumber(2)
  set log(DaemonLog value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasLog() => $_has(1);
  @$pb.TagNumber(2)
  void clearLog() => $_clearField(2);
  @$pb.TagNumber(2)
  DaemonLog ensureLog() => $_ensure(1);

  /// Pushed to subscribers after every committed revision, following the
  /// ProjectChanged event for the same revision.
  @$pb.TagNumber(3)
  AnalysisReady get analysisReady => $_getN(2);
  @$pb.TagNumber(3)
  set analysisReady(AnalysisReady value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasAnalysisReady() => $_has(2);
  @$pb.TagNumber(3)
  void clearAnalysisReady() => $_clearField(3);
  @$pb.TagNumber(3)
  AnalysisReady ensureAnalysisReady() => $_ensure(2);
}

class Ack extends $pb.GeneratedMessage {
  factory Ack() => Ack._();

  Ack._();

  factory Ack.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Ack()..mergeFromBuffer(data, registry);
  factory Ack.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Ack()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Ack',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Ack.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Ack clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Ack copyWith(void Function(Ack) updates) =>
      super.copyWith((message) => updates(message as Ack)) as Ack;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Ack() / Ack.new instead')
  static Ack create() => Ack._();
  static $pb.GeneratedMessage $_createMessage() => Ack._();
  @$core.override
  Ack createEmptyInstance() => Ack._();
  @$core.pragma('dart2js:noInline')
  static Ack getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Ack>(Ack.$_createMessage);
  static Ack? _defaultInstance;
}

/// A structured failure of one request.  `code` is a stable machine-readable
/// identifier (e.g. "edit.duplicate_concept_name"); `message` is designer
/// readable; `details_json` carries the typed error for the expert view.
class Error extends $pb.GeneratedMessage {
  factory Error({
    $core.String? code,
    $core.String? message,
    $core.String? detailsJson,
  }) {
    final result = Error._();
    if (code != null) result.code = code;
    if (message != null) result.message = message;
    if (detailsJson != null) result.detailsJson = detailsJson;
    return result;
  }

  Error._();

  factory Error.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Error()..mergeFromBuffer(data, registry);
  factory Error.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Error()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Error',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Error.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'code')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..aOS(3, _omitFieldNames ? '' : 'detailsJson')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Error clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Error copyWith(void Function(Error) updates) =>
      super.copyWith((message) => updates(message as Error)) as Error;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Error() / Error.new instead')
  static Error create() => Error._();
  static $pb.GeneratedMessage $_createMessage() => Error._();
  @$core.override
  Error createEmptyInstance() => Error._();
  @$core.pragma('dart2js:noInline')
  static Error getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Error>(Error.$_createMessage);
  static Error? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get code => $_getSZ(0);
  @$pb.TagNumber(1)
  set code($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCode() => $_has(0);
  @$pb.TagNumber(1)
  void clearCode() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get detailsJson => $_getSZ(2);
  @$pb.TagNumber(3)
  set detailsJson($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDetailsJson() => $_has(2);
  @$pb.TagNumber(3)
  void clearDetailsJson() => $_clearField(3);
}

class Version extends $pb.GeneratedMessage {
  factory Version({
    $core.int? major,
    $core.int? minor,
    $core.int? patch,
  }) {
    final result = Version._();
    if (major != null) result.major = major;
    if (minor != null) result.minor = minor;
    if (patch != null) result.patch = patch;
    return result;
  }

  Version._();

  factory Version.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Version()..mergeFromBuffer(data, registry);
  factory Version.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Version()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Version',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Version.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'major', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'minor', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'patch', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Version clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Version copyWith(void Function(Version) updates) =>
      super.copyWith((message) => updates(message as Version)) as Version;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Version() / Version.new instead')
  static Version create() => Version._();
  static $pb.GeneratedMessage $_createMessage() => Version._();
  @$core.override
  Version createEmptyInstance() => Version._();
  @$core.pragma('dart2js:noInline')
  static Version getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Version>(Version.$_createMessage);
  static Version? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get major => $_getIZ(0);
  @$pb.TagNumber(1)
  set major($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMajor() => $_has(0);
  @$pb.TagNumber(1)
  void clearMajor() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get minor => $_getIZ(1);
  @$pb.TagNumber(2)
  set minor($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMinor() => $_has(1);
  @$pb.TagNumber(2)
  void clearMinor() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get patch => $_getIZ(2);
  @$pb.TagNumber(3)
  set patch($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPatch() => $_has(2);
  @$pb.TagNumber(3)
  void clearPatch() => $_clearField(3);
}

class HandshakeRequest extends $pb.GeneratedMessage {
  factory HandshakeRequest({
    Version? clientProtocolVersion,
    $core.String? clientName,
    $core.String? clientVersion,
  }) {
    final result = HandshakeRequest._();
    if (clientProtocolVersion != null) result.clientProtocolVersion = clientProtocolVersion;
    if (clientName != null) result.clientName = clientName;
    if (clientVersion != null) result.clientVersion = clientVersion;
    return result;
  }

  HandshakeRequest._();

  factory HandshakeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HandshakeRequest()..mergeFromBuffer(data, registry);
  factory HandshakeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HandshakeRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'HandshakeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: HandshakeRequest.$_createMessage)
    ..aOM<Version>(1, _omitFieldNames ? '' : 'clientProtocolVersion',
        subBuilder: Version.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'clientName')
    ..aOS(3, _omitFieldNames ? '' : 'clientVersion')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HandshakeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HandshakeRequest copyWith(void Function(HandshakeRequest) updates) =>
      super.copyWith((message) => updates(message as HandshakeRequest)) as HandshakeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HandshakeRequest() / HandshakeRequest.new instead')
  static HandshakeRequest create() => HandshakeRequest._();
  static $pb.GeneratedMessage $_createMessage() => HandshakeRequest._();
  @$core.override
  HandshakeRequest createEmptyInstance() => HandshakeRequest._();
  @$core.pragma('dart2js:noInline')
  static HandshakeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HandshakeRequest>(HandshakeRequest.$_createMessage);
  static HandshakeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  Version get clientProtocolVersion => $_getN(0);
  @$pb.TagNumber(1)
  set clientProtocolVersion(Version value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasClientProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientProtocolVersion() => $_clearField(1);
  @$pb.TagNumber(1)
  Version ensureClientProtocolVersion() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get clientName => $_getSZ(1);
  @$pb.TagNumber(2)
  set clientName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasClientName() => $_has(1);
  @$pb.TagNumber(2)
  void clearClientName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get clientVersion => $_getSZ(2);
  @$pb.TagNumber(3)
  set clientVersion($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasClientVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientVersion() => $_clearField(3);
}

class HandshakeResponse extends $pb.GeneratedMessage {
  factory HandshakeResponse({
    Version? protocolVersion,
    $core.String? compilerVersion,
    $core.String? daemonName,
    $core.bool? compatible,
  }) {
    final result = HandshakeResponse._();
    if (protocolVersion != null) result.protocolVersion = protocolVersion;
    if (compilerVersion != null) result.compilerVersion = compilerVersion;
    if (daemonName != null) result.daemonName = daemonName;
    if (compatible != null) result.compatible = compatible;
    return result;
  }

  HandshakeResponse._();

  factory HandshakeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HandshakeResponse()..mergeFromBuffer(data, registry);
  factory HandshakeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HandshakeResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'HandshakeResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: HandshakeResponse.$_createMessage)
    ..aOM<Version>(1, _omitFieldNames ? '' : 'protocolVersion', subBuilder: Version.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'compilerVersion')
    ..aOS(3, _omitFieldNames ? '' : 'daemonName')
    ..aOB(4, _omitFieldNames ? '' : 'compatible')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HandshakeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HandshakeResponse copyWith(void Function(HandshakeResponse) updates) =>
      super.copyWith((message) => updates(message as HandshakeResponse)) as HandshakeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HandshakeResponse() / HandshakeResponse.new instead')
  static HandshakeResponse create() => HandshakeResponse._();
  static $pb.GeneratedMessage $_createMessage() => HandshakeResponse._();
  @$core.override
  HandshakeResponse createEmptyInstance() => HandshakeResponse._();
  @$core.pragma('dart2js:noInline')
  static HandshakeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HandshakeResponse>(HandshakeResponse.$_createMessage);
  static HandshakeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Version get protocolVersion => $_getN(0);
  @$pb.TagNumber(1)
  set protocolVersion(Version value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProtocolVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearProtocolVersion() => $_clearField(1);
  @$pb.TagNumber(1)
  Version ensureProtocolVersion() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get compilerVersion => $_getSZ(1);
  @$pb.TagNumber(2)
  set compilerVersion($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCompilerVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearCompilerVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get daemonName => $_getSZ(2);
  @$pb.TagNumber(3)
  set daemonName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDaemonName() => $_has(2);
  @$pb.TagNumber(3)
  void clearDaemonName() => $_clearField(3);

  /// True when the daemon accepts the client's protocol version.
  @$pb.TagNumber(4)
  $core.bool get compatible => $_getBF(3);
  @$pb.TagNumber(4)
  set compatible($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCompatible() => $_has(3);
  @$pb.TagNumber(4)
  void clearCompatible() => $_clearField(4);
}

class OpenProjectRequest extends $pb.GeneratedMessage {
  factory OpenProjectRequest({
    $core.String? rootPath,
  }) {
    final result = OpenProjectRequest._();
    if (rootPath != null) result.rootPath = rootPath;
    return result;
  }

  OpenProjectRequest._();

  factory OpenProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OpenProjectRequest()..mergeFromBuffer(data, registry);
  factory OpenProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OpenProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'OpenProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: OpenProjectRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'rootPath')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OpenProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OpenProjectRequest copyWith(void Function(OpenProjectRequest) updates) =>
      super.copyWith((message) => updates(message as OpenProjectRequest)) as OpenProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use OpenProjectRequest() / OpenProjectRequest.new instead')
  static OpenProjectRequest create() => OpenProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => OpenProjectRequest._();
  @$core.override
  OpenProjectRequest createEmptyInstance() => OpenProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static OpenProjectRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OpenProjectRequest>(OpenProjectRequest.$_createMessage);
  static OpenProjectRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get rootPath => $_getSZ(0);
  @$pb.TagNumber(1)
  set rootPath($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRootPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearRootPath() => $_clearField(1);
}

class InitProjectRequest extends $pb.GeneratedMessage {
  factory InitProjectRequest({
    $core.String? rootPath,
    $core.String? name,
  }) {
    final result = InitProjectRequest._();
    if (rootPath != null) result.rootPath = rootPath;
    if (name != null) result.name = name;
    return result;
  }

  InitProjectRequest._();

  factory InitProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InitProjectRequest()..mergeFromBuffer(data, registry);
  factory InitProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InitProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'InitProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: InitProjectRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'rootPath')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InitProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InitProjectRequest copyWith(void Function(InitProjectRequest) updates) =>
      super.copyWith((message) => updates(message as InitProjectRequest)) as InitProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use InitProjectRequest() / InitProjectRequest.new instead')
  static InitProjectRequest create() => InitProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => InitProjectRequest._();
  @$core.override
  InitProjectRequest createEmptyInstance() => InitProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static InitProjectRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InitProjectRequest>(InitProjectRequest.$_createMessage);
  static InitProjectRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get rootPath => $_getSZ(0);
  @$pb.TagNumber(1)
  set rootPath($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRootPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearRootPath() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SaveProjectRequest extends $pb.GeneratedMessage {
  factory SaveProjectRequest() => SaveProjectRequest._();

  SaveProjectRequest._();

  factory SaveProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SaveProjectRequest()..mergeFromBuffer(data, registry);
  factory SaveProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SaveProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SaveProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SaveProjectRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SaveProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SaveProjectRequest copyWith(void Function(SaveProjectRequest) updates) =>
      super.copyWith((message) => updates(message as SaveProjectRequest)) as SaveProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SaveProjectRequest() / SaveProjectRequest.new instead')
  static SaveProjectRequest create() => SaveProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => SaveProjectRequest._();
  @$core.override
  SaveProjectRequest createEmptyInstance() => SaveProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static SaveProjectRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SaveProjectRequest>(SaveProjectRequest.$_createMessage);
  static SaveProjectRequest? _defaultInstance;
}

class CloseProjectRequest extends $pb.GeneratedMessage {
  factory CloseProjectRequest() => CloseProjectRequest._();

  CloseProjectRequest._();

  factory CloseProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CloseProjectRequest()..mergeFromBuffer(data, registry);
  factory CloseProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CloseProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CloseProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CloseProjectRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CloseProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CloseProjectRequest copyWith(void Function(CloseProjectRequest) updates) =>
      super.copyWith((message) => updates(message as CloseProjectRequest)) as CloseProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CloseProjectRequest() / CloseProjectRequest.new instead')
  static CloseProjectRequest create() => CloseProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => CloseProjectRequest._();
  @$core.override
  CloseProjectRequest createEmptyInstance() => CloseProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static CloseProjectRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CloseProjectRequest>(CloseProjectRequest.$_createMessage);
  static CloseProjectRequest? _defaultInstance;
}

class GetProjectRequest extends $pb.GeneratedMessage {
  factory GetProjectRequest() => GetProjectRequest._();

  GetProjectRequest._();

  factory GetProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetProjectRequest()..mergeFromBuffer(data, registry);
  factory GetProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'GetProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: GetProjectRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetProjectRequest copyWith(void Function(GetProjectRequest) updates) =>
      super.copyWith((message) => updates(message as GetProjectRequest)) as GetProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetProjectRequest() / GetProjectRequest.new instead')
  static GetProjectRequest create() => GetProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetProjectRequest._();
  @$core.override
  GetProjectRequest createEmptyInstance() => GetProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static GetProjectRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetProjectRequest>(GetProjectRequest.$_createMessage);
  static GetProjectRequest? _defaultInstance;
}

class SubscribeProjectRequest extends $pb.GeneratedMessage {
  factory SubscribeProjectRequest() => SubscribeProjectRequest._();

  SubscribeProjectRequest._();

  factory SubscribeProjectRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeProjectRequest()..mergeFromBuffer(data, registry);
  factory SubscribeProjectRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeProjectRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SubscribeProjectRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SubscribeProjectRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeProjectRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeProjectRequest copyWith(void Function(SubscribeProjectRequest) updates) =>
      super.copyWith((message) => updates(message as SubscribeProjectRequest))
          as SubscribeProjectRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SubscribeProjectRequest() / SubscribeProjectRequest.new instead')
  static SubscribeProjectRequest create() => SubscribeProjectRequest._();
  static $pb.GeneratedMessage $_createMessage() => SubscribeProjectRequest._();
  @$core.override
  SubscribeProjectRequest createEmptyInstance() => SubscribeProjectRequest._();
  @$core.pragma('dart2js:noInline')
  static SubscribeProjectRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SubscribeProjectRequest>(
          SubscribeProjectRequest.$_createMessage);
  static SubscribeProjectRequest? _defaultInstance;
}

class ShutdownRequest extends $pb.GeneratedMessage {
  factory ShutdownRequest() => ShutdownRequest._();

  ShutdownRequest._();

  factory ShutdownRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ShutdownRequest()..mergeFromBuffer(data, registry);
  factory ShutdownRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ShutdownRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ShutdownRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ShutdownRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ShutdownRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ShutdownRequest copyWith(void Function(ShutdownRequest) updates) =>
      super.copyWith((message) => updates(message as ShutdownRequest)) as ShutdownRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ShutdownRequest() / ShutdownRequest.new instead')
  static ShutdownRequest create() => ShutdownRequest._();
  static $pb.GeneratedMessage $_createMessage() => ShutdownRequest._();
  @$core.override
  ShutdownRequest createEmptyInstance() => ShutdownRequest._();
  @$core.pragma('dart2js:noInline')
  static ShutdownRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ShutdownRequest>(ShutdownRequest.$_createMessage);
  static ShutdownRequest? _defaultInstance;
}

class ProjectResponse extends $pb.GeneratedMessage {
  factory ProjectResponse({
    ProjectProjection? project,
  }) {
    final result = ProjectResponse._();
    if (project != null) result.project = project;
    return result;
  }

  ProjectResponse._();

  factory ProjectResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectResponse()..mergeFromBuffer(data, registry);
  factory ProjectResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ProjectResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ProjectResponse.$_createMessage)
    ..aOM<ProjectProjection>(1, _omitFieldNames ? '' : 'project',
        subBuilder: ProjectProjection.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectResponse copyWith(void Function(ProjectResponse) updates) =>
      super.copyWith((message) => updates(message as ProjectResponse)) as ProjectResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ProjectResponse() / ProjectResponse.new instead')
  static ProjectResponse create() => ProjectResponse._();
  static $pb.GeneratedMessage $_createMessage() => ProjectResponse._();
  @$core.override
  ProjectResponse createEmptyInstance() => ProjectResponse._();
  @$core.pragma('dart2js:noInline')
  static ProjectResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProjectResponse>(ProjectResponse.$_createMessage);
  static ProjectResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ProjectProjection get project => $_getN(0);
  @$pb.TagNumber(1)
  set project(ProjectProjection value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProject() => $_has(0);
  @$pb.TagNumber(1)
  void clearProject() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectProjection ensureProject() => $_ensure(0);
}

class ApplyEditRequest extends $pb.GeneratedMessage {
  factory ApplyEditRequest({
    $fixnum.Int64? baseRevision,
    EditOp? op,
  }) {
    final result = ApplyEditRequest._();
    if (baseRevision != null) result.baseRevision = baseRevision;
    if (op != null) result.op = op;
    return result;
  }

  ApplyEditRequest._();

  factory ApplyEditRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApplyEditRequest()..mergeFromBuffer(data, registry);
  factory ApplyEditRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApplyEditRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ApplyEditRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ApplyEditRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'baseRevision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<EditOp>(2, _omitFieldNames ? '' : 'op', subBuilder: EditOp.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApplyEditRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApplyEditRequest copyWith(void Function(ApplyEditRequest) updates) =>
      super.copyWith((message) => updates(message as ApplyEditRequest)) as ApplyEditRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ApplyEditRequest() / ApplyEditRequest.new instead')
  static ApplyEditRequest create() => ApplyEditRequest._();
  static $pb.GeneratedMessage $_createMessage() => ApplyEditRequest._();
  @$core.override
  ApplyEditRequest createEmptyInstance() => ApplyEditRequest._();
  @$core.pragma('dart2js:noInline')
  static ApplyEditRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApplyEditRequest>(ApplyEditRequest.$_createMessage);
  static ApplyEditRequest? _defaultInstance;

  /// Revision the client believes it is editing.  The daemon refuses the edit
  /// if the project has moved on (code "edit.stale_revision").
  @$pb.TagNumber(1)
  $fixnum.Int64 get baseRevision => $_getI64(0);
  @$pb.TagNumber(1)
  set baseRevision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasBaseRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearBaseRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  EditOp get op => $_getN(1);
  @$pb.TagNumber(2)
  set op(EditOp value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasOp() => $_has(1);
  @$pb.TagNumber(2)
  void clearOp() => $_clearField(2);
  @$pb.TagNumber(2)
  EditOp ensureOp() => $_ensure(1);
}

class UndoRequest extends $pb.GeneratedMessage {
  factory UndoRequest() => UndoRequest._();

  UndoRequest._();

  factory UndoRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UndoRequest()..mergeFromBuffer(data, registry);
  factory UndoRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UndoRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'UndoRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: UndoRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UndoRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UndoRequest copyWith(void Function(UndoRequest) updates) =>
      super.copyWith((message) => updates(message as UndoRequest)) as UndoRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use UndoRequest() / UndoRequest.new instead')
  static UndoRequest create() => UndoRequest._();
  static $pb.GeneratedMessage $_createMessage() => UndoRequest._();
  @$core.override
  UndoRequest createEmptyInstance() => UndoRequest._();
  @$core.pragma('dart2js:noInline')
  static UndoRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UndoRequest>(UndoRequest.$_createMessage);
  static UndoRequest? _defaultInstance;
}

class RedoRequest extends $pb.GeneratedMessage {
  factory RedoRequest() => RedoRequest._();

  RedoRequest._();

  factory RedoRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RedoRequest()..mergeFromBuffer(data, registry);
  factory RedoRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RedoRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RedoRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RedoRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RedoRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RedoRequest copyWith(void Function(RedoRequest) updates) =>
      super.copyWith((message) => updates(message as RedoRequest)) as RedoRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RedoRequest() / RedoRequest.new instead')
  static RedoRequest create() => RedoRequest._();
  static $pb.GeneratedMessage $_createMessage() => RedoRequest._();
  @$core.override
  RedoRequest createEmptyInstance() => RedoRequest._();
  @$core.pragma('dart2js:noInline')
  static RedoRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RedoRequest>(RedoRequest.$_createMessage);
  static RedoRequest? _defaultInstance;
}

class EditApplied extends $pb.GeneratedMessage {
  factory EditApplied({
    ProjectProjection? project,
    EditOutcome? outcome,
  }) {
    final result = EditApplied._();
    if (project != null) result.project = project;
    if (outcome != null) result.outcome = outcome;
    return result;
  }

  EditApplied._();

  factory EditApplied.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditApplied()..mergeFromBuffer(data, registry);
  factory EditApplied.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditApplied()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'EditApplied',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: EditApplied.$_createMessage)
    ..aOM<ProjectProjection>(1, _omitFieldNames ? '' : 'project',
        subBuilder: ProjectProjection.$_createMessage)
    ..aOM<EditOutcome>(2, _omitFieldNames ? '' : 'outcome', subBuilder: EditOutcome.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditApplied clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditApplied copyWith(void Function(EditApplied) updates) =>
      super.copyWith((message) => updates(message as EditApplied)) as EditApplied;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EditApplied() / EditApplied.new instead')
  static EditApplied create() => EditApplied._();
  static $pb.GeneratedMessage $_createMessage() => EditApplied._();
  @$core.override
  EditApplied createEmptyInstance() => EditApplied._();
  @$core.pragma('dart2js:noInline')
  static EditApplied getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EditApplied>(EditApplied.$_createMessage);
  static EditApplied? _defaultInstance;

  @$pb.TagNumber(1)
  ProjectProjection get project => $_getN(0);
  @$pb.TagNumber(1)
  set project(ProjectProjection value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProject() => $_has(0);
  @$pb.TagNumber(1)
  void clearProject() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectProjection ensureProject() => $_ensure(0);

  @$pb.TagNumber(2)
  EditOutcome get outcome => $_getN(1);
  @$pb.TagNumber(2)
  set outcome(EditOutcome value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasOutcome() => $_has(1);
  @$pb.TagNumber(2)
  void clearOutcome() => $_clearField(2);
  @$pb.TagNumber(2)
  EditOutcome ensureOutcome() => $_ensure(1);
}

enum EditOp_Op {
  createConcept,
  renameConcept,
  setConceptDescription,
  setConceptRepresentation,
  deleteConcept,
  createMapping,
  renameMapping,
  setMappingDescription,
  setMappingSignature,
  attachDefinition,
  replaceDefinition,
  deleteMapping,
  createClockDomain,
  renameClockDomain,
  deleteClockDomain,
  setMappingClock,
  createOutput,
  renameOutput,
  setOutputAccepts,
  setOutputClock,
  setOutputRequired,
  deleteOutput,
  setMappingDrive,
  createDevice,
  renameDevice,
  setDeviceKind,
  setDeviceOutput,
  setDevicePin,
  deleteDevice,
  notSet
}

class EditOp extends $pb.GeneratedMessage {
  factory EditOp({
    CreateConcept? createConcept,
    RenameConcept? renameConcept,
    SetConceptDescription? setConceptDescription,
    SetConceptRepresentation? setConceptRepresentation,
    DeleteConcept? deleteConcept,
    CreateMapping? createMapping,
    RenameMapping? renameMapping,
    SetMappingDescription? setMappingDescription,
    SetMappingSignature? setMappingSignature,
    AttachDefinition? attachDefinition,
    ReplaceDefinition? replaceDefinition,
    DeleteMapping? deleteMapping,
    CreateClockDomain? createClockDomain,
    RenameClockDomain? renameClockDomain,
    DeleteClockDomain? deleteClockDomain,
    SetMappingClock? setMappingClock,
    CreateOutput? createOutput,
    RenameOutput? renameOutput,
    SetOutputAccepts? setOutputAccepts,
    SetOutputClock? setOutputClock,
    SetOutputRequired? setOutputRequired,
    DeleteOutput? deleteOutput,
    SetMappingDrive? setMappingDrive,
    CreateDevice? createDevice,
    RenameDevice? renameDevice,
    SetDeviceKind? setDeviceKind,
    SetDeviceOutput? setDeviceOutput,
    SetDevicePin? setDevicePin,
    DeleteDevice? deleteDevice,
  }) {
    final result = EditOp._();
    if (createConcept != null) result.createConcept = createConcept;
    if (renameConcept != null) result.renameConcept = renameConcept;
    if (setConceptDescription != null) result.setConceptDescription = setConceptDescription;
    if (setConceptRepresentation != null)
      result.setConceptRepresentation = setConceptRepresentation;
    if (deleteConcept != null) result.deleteConcept = deleteConcept;
    if (createMapping != null) result.createMapping = createMapping;
    if (renameMapping != null) result.renameMapping = renameMapping;
    if (setMappingDescription != null) result.setMappingDescription = setMappingDescription;
    if (setMappingSignature != null) result.setMappingSignature = setMappingSignature;
    if (attachDefinition != null) result.attachDefinition = attachDefinition;
    if (replaceDefinition != null) result.replaceDefinition = replaceDefinition;
    if (deleteMapping != null) result.deleteMapping = deleteMapping;
    if (createClockDomain != null) result.createClockDomain = createClockDomain;
    if (renameClockDomain != null) result.renameClockDomain = renameClockDomain;
    if (deleteClockDomain != null) result.deleteClockDomain = deleteClockDomain;
    if (setMappingClock != null) result.setMappingClock = setMappingClock;
    if (createOutput != null) result.createOutput = createOutput;
    if (renameOutput != null) result.renameOutput = renameOutput;
    if (setOutputAccepts != null) result.setOutputAccepts = setOutputAccepts;
    if (setOutputClock != null) result.setOutputClock = setOutputClock;
    if (setOutputRequired != null) result.setOutputRequired = setOutputRequired;
    if (deleteOutput != null) result.deleteOutput = deleteOutput;
    if (setMappingDrive != null) result.setMappingDrive = setMappingDrive;
    if (createDevice != null) result.createDevice = createDevice;
    if (renameDevice != null) result.renameDevice = renameDevice;
    if (setDeviceKind != null) result.setDeviceKind = setDeviceKind;
    if (setDeviceOutput != null) result.setDeviceOutput = setDeviceOutput;
    if (setDevicePin != null) result.setDevicePin = setDevicePin;
    if (deleteDevice != null) result.deleteDevice = deleteDevice;
    return result;
  }

  EditOp._();

  factory EditOp.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditOp()..mergeFromBuffer(data, registry);
  factory EditOp.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditOp()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EditOp_Op> _EditOp_OpByTag = {
    1: EditOp_Op.createConcept,
    2: EditOp_Op.renameConcept,
    3: EditOp_Op.setConceptDescription,
    4: EditOp_Op.setConceptRepresentation,
    5: EditOp_Op.deleteConcept,
    6: EditOp_Op.createMapping,
    7: EditOp_Op.renameMapping,
    8: EditOp_Op.setMappingDescription,
    9: EditOp_Op.setMappingSignature,
    10: EditOp_Op.attachDefinition,
    11: EditOp_Op.replaceDefinition,
    12: EditOp_Op.deleteMapping,
    13: EditOp_Op.createClockDomain,
    14: EditOp_Op.renameClockDomain,
    15: EditOp_Op.deleteClockDomain,
    16: EditOp_Op.setMappingClock,
    17: EditOp_Op.createOutput,
    18: EditOp_Op.renameOutput,
    19: EditOp_Op.setOutputAccepts,
    20: EditOp_Op.setOutputClock,
    21: EditOp_Op.setOutputRequired,
    22: EditOp_Op.deleteOutput,
    23: EditOp_Op.setMappingDrive,
    24: EditOp_Op.createDevice,
    25: EditOp_Op.renameDevice,
    26: EditOp_Op.setDeviceKind,
    27: EditOp_Op.setDeviceOutput,
    28: EditOp_Op.setDevicePin,
    29: EditOp_Op.deleteDevice,
    0: EditOp_Op.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'EditOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: EditOp.$_createMessage)
    ..oo(0, [
      1,
      2,
      3,
      4,
      5,
      6,
      7,
      8,
      9,
      10,
      11,
      12,
      13,
      14,
      15,
      16,
      17,
      18,
      19,
      20,
      21,
      22,
      23,
      24,
      25,
      26,
      27,
      28,
      29
    ])
    ..aOM<CreateConcept>(1, _omitFieldNames ? '' : 'createConcept',
        subBuilder: CreateConcept.$_createMessage)
    ..aOM<RenameConcept>(2, _omitFieldNames ? '' : 'renameConcept',
        subBuilder: RenameConcept.$_createMessage)
    ..aOM<SetConceptDescription>(3, _omitFieldNames ? '' : 'setConceptDescription',
        subBuilder: SetConceptDescription.$_createMessage)
    ..aOM<SetConceptRepresentation>(4, _omitFieldNames ? '' : 'setConceptRepresentation',
        subBuilder: SetConceptRepresentation.$_createMessage)
    ..aOM<DeleteConcept>(5, _omitFieldNames ? '' : 'deleteConcept',
        subBuilder: DeleteConcept.$_createMessage)
    ..aOM<CreateMapping>(6, _omitFieldNames ? '' : 'createMapping',
        subBuilder: CreateMapping.$_createMessage)
    ..aOM<RenameMapping>(7, _omitFieldNames ? '' : 'renameMapping',
        subBuilder: RenameMapping.$_createMessage)
    ..aOM<SetMappingDescription>(8, _omitFieldNames ? '' : 'setMappingDescription',
        subBuilder: SetMappingDescription.$_createMessage)
    ..aOM<SetMappingSignature>(9, _omitFieldNames ? '' : 'setMappingSignature',
        subBuilder: SetMappingSignature.$_createMessage)
    ..aOM<AttachDefinition>(10, _omitFieldNames ? '' : 'attachDefinition',
        subBuilder: AttachDefinition.$_createMessage)
    ..aOM<ReplaceDefinition>(11, _omitFieldNames ? '' : 'replaceDefinition',
        subBuilder: ReplaceDefinition.$_createMessage)
    ..aOM<DeleteMapping>(12, _omitFieldNames ? '' : 'deleteMapping',
        subBuilder: DeleteMapping.$_createMessage)
    ..aOM<CreateClockDomain>(13, _omitFieldNames ? '' : 'createClockDomain',
        subBuilder: CreateClockDomain.$_createMessage)
    ..aOM<RenameClockDomain>(14, _omitFieldNames ? '' : 'renameClockDomain',
        subBuilder: RenameClockDomain.$_createMessage)
    ..aOM<DeleteClockDomain>(15, _omitFieldNames ? '' : 'deleteClockDomain',
        subBuilder: DeleteClockDomain.$_createMessage)
    ..aOM<SetMappingClock>(16, _omitFieldNames ? '' : 'setMappingClock',
        subBuilder: SetMappingClock.$_createMessage)
    ..aOM<CreateOutput>(17, _omitFieldNames ? '' : 'createOutput',
        subBuilder: CreateOutput.$_createMessage)
    ..aOM<RenameOutput>(18, _omitFieldNames ? '' : 'renameOutput',
        subBuilder: RenameOutput.$_createMessage)
    ..aOM<SetOutputAccepts>(19, _omitFieldNames ? '' : 'setOutputAccepts',
        subBuilder: SetOutputAccepts.$_createMessage)
    ..aOM<SetOutputClock>(20, _omitFieldNames ? '' : 'setOutputClock',
        subBuilder: SetOutputClock.$_createMessage)
    ..aOM<SetOutputRequired>(21, _omitFieldNames ? '' : 'setOutputRequired',
        subBuilder: SetOutputRequired.$_createMessage)
    ..aOM<DeleteOutput>(22, _omitFieldNames ? '' : 'deleteOutput',
        subBuilder: DeleteOutput.$_createMessage)
    ..aOM<SetMappingDrive>(23, _omitFieldNames ? '' : 'setMappingDrive',
        subBuilder: SetMappingDrive.$_createMessage)
    ..aOM<CreateDevice>(24, _omitFieldNames ? '' : 'createDevice',
        subBuilder: CreateDevice.$_createMessage)
    ..aOM<RenameDevice>(25, _omitFieldNames ? '' : 'renameDevice',
        subBuilder: RenameDevice.$_createMessage)
    ..aOM<SetDeviceKind>(26, _omitFieldNames ? '' : 'setDeviceKind',
        subBuilder: SetDeviceKind.$_createMessage)
    ..aOM<SetDeviceOutput>(27, _omitFieldNames ? '' : 'setDeviceOutput',
        subBuilder: SetDeviceOutput.$_createMessage)
    ..aOM<SetDevicePin>(28, _omitFieldNames ? '' : 'setDevicePin',
        subBuilder: SetDevicePin.$_createMessage)
    ..aOM<DeleteDevice>(29, _omitFieldNames ? '' : 'deleteDevice',
        subBuilder: DeleteDevice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditOp clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditOp copyWith(void Function(EditOp) updates) =>
      super.copyWith((message) => updates(message as EditOp)) as EditOp;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EditOp() / EditOp.new instead')
  static EditOp create() => EditOp._();
  static $pb.GeneratedMessage $_createMessage() => EditOp._();
  @$core.override
  EditOp createEmptyInstance() => EditOp._();
  @$core.pragma('dart2js:noInline')
  static EditOp getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EditOp>(EditOp.$_createMessage);
  static EditOp? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  @$pb.TagNumber(22)
  @$pb.TagNumber(23)
  @$pb.TagNumber(24)
  @$pb.TagNumber(25)
  @$pb.TagNumber(26)
  @$pb.TagNumber(27)
  @$pb.TagNumber(28)
  @$pb.TagNumber(29)
  EditOp_Op whichOp() => _EditOp_OpByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(16)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  @$pb.TagNumber(19)
  @$pb.TagNumber(20)
  @$pb.TagNumber(21)
  @$pb.TagNumber(22)
  @$pb.TagNumber(23)
  @$pb.TagNumber(24)
  @$pb.TagNumber(25)
  @$pb.TagNumber(26)
  @$pb.TagNumber(27)
  @$pb.TagNumber(28)
  @$pb.TagNumber(29)
  void clearOp() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  CreateConcept get createConcept => $_getN(0);
  @$pb.TagNumber(1)
  set createConcept(CreateConcept value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCreateConcept() => $_has(0);
  @$pb.TagNumber(1)
  void clearCreateConcept() => $_clearField(1);
  @$pb.TagNumber(1)
  CreateConcept ensureCreateConcept() => $_ensure(0);

  @$pb.TagNumber(2)
  RenameConcept get renameConcept => $_getN(1);
  @$pb.TagNumber(2)
  set renameConcept(RenameConcept value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRenameConcept() => $_has(1);
  @$pb.TagNumber(2)
  void clearRenameConcept() => $_clearField(2);
  @$pb.TagNumber(2)
  RenameConcept ensureRenameConcept() => $_ensure(1);

  @$pb.TagNumber(3)
  SetConceptDescription get setConceptDescription => $_getN(2);
  @$pb.TagNumber(3)
  set setConceptDescription(SetConceptDescription value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSetConceptDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearSetConceptDescription() => $_clearField(3);
  @$pb.TagNumber(3)
  SetConceptDescription ensureSetConceptDescription() => $_ensure(2);

  @$pb.TagNumber(4)
  SetConceptRepresentation get setConceptRepresentation => $_getN(3);
  @$pb.TagNumber(4)
  set setConceptRepresentation(SetConceptRepresentation value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSetConceptRepresentation() => $_has(3);
  @$pb.TagNumber(4)
  void clearSetConceptRepresentation() => $_clearField(4);
  @$pb.TagNumber(4)
  SetConceptRepresentation ensureSetConceptRepresentation() => $_ensure(3);

  @$pb.TagNumber(5)
  DeleteConcept get deleteConcept => $_getN(4);
  @$pb.TagNumber(5)
  set deleteConcept(DeleteConcept value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasDeleteConcept() => $_has(4);
  @$pb.TagNumber(5)
  void clearDeleteConcept() => $_clearField(5);
  @$pb.TagNumber(5)
  DeleteConcept ensureDeleteConcept() => $_ensure(4);

  @$pb.TagNumber(6)
  CreateMapping get createMapping => $_getN(5);
  @$pb.TagNumber(6)
  set createMapping(CreateMapping value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasCreateMapping() => $_has(5);
  @$pb.TagNumber(6)
  void clearCreateMapping() => $_clearField(6);
  @$pb.TagNumber(6)
  CreateMapping ensureCreateMapping() => $_ensure(5);

  @$pb.TagNumber(7)
  RenameMapping get renameMapping => $_getN(6);
  @$pb.TagNumber(7)
  set renameMapping(RenameMapping value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasRenameMapping() => $_has(6);
  @$pb.TagNumber(7)
  void clearRenameMapping() => $_clearField(7);
  @$pb.TagNumber(7)
  RenameMapping ensureRenameMapping() => $_ensure(6);

  @$pb.TagNumber(8)
  SetMappingDescription get setMappingDescription => $_getN(7);
  @$pb.TagNumber(8)
  set setMappingDescription(SetMappingDescription value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasSetMappingDescription() => $_has(7);
  @$pb.TagNumber(8)
  void clearSetMappingDescription() => $_clearField(8);
  @$pb.TagNumber(8)
  SetMappingDescription ensureSetMappingDescription() => $_ensure(7);

  @$pb.TagNumber(9)
  SetMappingSignature get setMappingSignature => $_getN(8);
  @$pb.TagNumber(9)
  set setMappingSignature(SetMappingSignature value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasSetMappingSignature() => $_has(8);
  @$pb.TagNumber(9)
  void clearSetMappingSignature() => $_clearField(9);
  @$pb.TagNumber(9)
  SetMappingSignature ensureSetMappingSignature() => $_ensure(8);

  @$pb.TagNumber(10)
  AttachDefinition get attachDefinition => $_getN(9);
  @$pb.TagNumber(10)
  set attachDefinition(AttachDefinition value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasAttachDefinition() => $_has(9);
  @$pb.TagNumber(10)
  void clearAttachDefinition() => $_clearField(10);
  @$pb.TagNumber(10)
  AttachDefinition ensureAttachDefinition() => $_ensure(9);

  @$pb.TagNumber(11)
  ReplaceDefinition get replaceDefinition => $_getN(10);
  @$pb.TagNumber(11)
  set replaceDefinition(ReplaceDefinition value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasReplaceDefinition() => $_has(10);
  @$pb.TagNumber(11)
  void clearReplaceDefinition() => $_clearField(11);
  @$pb.TagNumber(11)
  ReplaceDefinition ensureReplaceDefinition() => $_ensure(10);

  @$pb.TagNumber(12)
  DeleteMapping get deleteMapping => $_getN(11);
  @$pb.TagNumber(12)
  set deleteMapping(DeleteMapping value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasDeleteMapping() => $_has(11);
  @$pb.TagNumber(12)
  void clearDeleteMapping() => $_clearField(12);
  @$pb.TagNumber(12)
  DeleteMapping ensureDeleteMapping() => $_ensure(11);

  @$pb.TagNumber(13)
  CreateClockDomain get createClockDomain => $_getN(12);
  @$pb.TagNumber(13)
  set createClockDomain(CreateClockDomain value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasCreateClockDomain() => $_has(12);
  @$pb.TagNumber(13)
  void clearCreateClockDomain() => $_clearField(13);
  @$pb.TagNumber(13)
  CreateClockDomain ensureCreateClockDomain() => $_ensure(12);

  @$pb.TagNumber(14)
  RenameClockDomain get renameClockDomain => $_getN(13);
  @$pb.TagNumber(14)
  set renameClockDomain(RenameClockDomain value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasRenameClockDomain() => $_has(13);
  @$pb.TagNumber(14)
  void clearRenameClockDomain() => $_clearField(14);
  @$pb.TagNumber(14)
  RenameClockDomain ensureRenameClockDomain() => $_ensure(13);

  @$pb.TagNumber(15)
  DeleteClockDomain get deleteClockDomain => $_getN(14);
  @$pb.TagNumber(15)
  set deleteClockDomain(DeleteClockDomain value) => $_setField(15, value);
  @$pb.TagNumber(15)
  $core.bool hasDeleteClockDomain() => $_has(14);
  @$pb.TagNumber(15)
  void clearDeleteClockDomain() => $_clearField(15);
  @$pb.TagNumber(15)
  DeleteClockDomain ensureDeleteClockDomain() => $_ensure(14);

  @$pb.TagNumber(16)
  SetMappingClock get setMappingClock => $_getN(15);
  @$pb.TagNumber(16)
  set setMappingClock(SetMappingClock value) => $_setField(16, value);
  @$pb.TagNumber(16)
  $core.bool hasSetMappingClock() => $_has(15);
  @$pb.TagNumber(16)
  void clearSetMappingClock() => $_clearField(16);
  @$pb.TagNumber(16)
  SetMappingClock ensureSetMappingClock() => $_ensure(15);

  @$pb.TagNumber(17)
  CreateOutput get createOutput => $_getN(16);
  @$pb.TagNumber(17)
  set createOutput(CreateOutput value) => $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasCreateOutput() => $_has(16);
  @$pb.TagNumber(17)
  void clearCreateOutput() => $_clearField(17);
  @$pb.TagNumber(17)
  CreateOutput ensureCreateOutput() => $_ensure(16);

  @$pb.TagNumber(18)
  RenameOutput get renameOutput => $_getN(17);
  @$pb.TagNumber(18)
  set renameOutput(RenameOutput value) => $_setField(18, value);
  @$pb.TagNumber(18)
  $core.bool hasRenameOutput() => $_has(17);
  @$pb.TagNumber(18)
  void clearRenameOutput() => $_clearField(18);
  @$pb.TagNumber(18)
  RenameOutput ensureRenameOutput() => $_ensure(17);

  @$pb.TagNumber(19)
  SetOutputAccepts get setOutputAccepts => $_getN(18);
  @$pb.TagNumber(19)
  set setOutputAccepts(SetOutputAccepts value) => $_setField(19, value);
  @$pb.TagNumber(19)
  $core.bool hasSetOutputAccepts() => $_has(18);
  @$pb.TagNumber(19)
  void clearSetOutputAccepts() => $_clearField(19);
  @$pb.TagNumber(19)
  SetOutputAccepts ensureSetOutputAccepts() => $_ensure(18);

  @$pb.TagNumber(20)
  SetOutputClock get setOutputClock => $_getN(19);
  @$pb.TagNumber(20)
  set setOutputClock(SetOutputClock value) => $_setField(20, value);
  @$pb.TagNumber(20)
  $core.bool hasSetOutputClock() => $_has(19);
  @$pb.TagNumber(20)
  void clearSetOutputClock() => $_clearField(20);
  @$pb.TagNumber(20)
  SetOutputClock ensureSetOutputClock() => $_ensure(19);

  @$pb.TagNumber(21)
  SetOutputRequired get setOutputRequired => $_getN(20);
  @$pb.TagNumber(21)
  set setOutputRequired(SetOutputRequired value) => $_setField(21, value);
  @$pb.TagNumber(21)
  $core.bool hasSetOutputRequired() => $_has(20);
  @$pb.TagNumber(21)
  void clearSetOutputRequired() => $_clearField(21);
  @$pb.TagNumber(21)
  SetOutputRequired ensureSetOutputRequired() => $_ensure(20);

  @$pb.TagNumber(22)
  DeleteOutput get deleteOutput => $_getN(21);
  @$pb.TagNumber(22)
  set deleteOutput(DeleteOutput value) => $_setField(22, value);
  @$pb.TagNumber(22)
  $core.bool hasDeleteOutput() => $_has(21);
  @$pb.TagNumber(22)
  void clearDeleteOutput() => $_clearField(22);
  @$pb.TagNumber(22)
  DeleteOutput ensureDeleteOutput() => $_ensure(21);

  @$pb.TagNumber(23)
  SetMappingDrive get setMappingDrive => $_getN(22);
  @$pb.TagNumber(23)
  set setMappingDrive(SetMappingDrive value) => $_setField(23, value);
  @$pb.TagNumber(23)
  $core.bool hasSetMappingDrive() => $_has(22);
  @$pb.TagNumber(23)
  void clearSetMappingDrive() => $_clearField(23);
  @$pb.TagNumber(23)
  SetMappingDrive ensureSetMappingDrive() => $_ensure(22);

  @$pb.TagNumber(24)
  CreateDevice get createDevice => $_getN(23);
  @$pb.TagNumber(24)
  set createDevice(CreateDevice value) => $_setField(24, value);
  @$pb.TagNumber(24)
  $core.bool hasCreateDevice() => $_has(23);
  @$pb.TagNumber(24)
  void clearCreateDevice() => $_clearField(24);
  @$pb.TagNumber(24)
  CreateDevice ensureCreateDevice() => $_ensure(23);

  @$pb.TagNumber(25)
  RenameDevice get renameDevice => $_getN(24);
  @$pb.TagNumber(25)
  set renameDevice(RenameDevice value) => $_setField(25, value);
  @$pb.TagNumber(25)
  $core.bool hasRenameDevice() => $_has(24);
  @$pb.TagNumber(25)
  void clearRenameDevice() => $_clearField(25);
  @$pb.TagNumber(25)
  RenameDevice ensureRenameDevice() => $_ensure(24);

  @$pb.TagNumber(26)
  SetDeviceKind get setDeviceKind => $_getN(25);
  @$pb.TagNumber(26)
  set setDeviceKind(SetDeviceKind value) => $_setField(26, value);
  @$pb.TagNumber(26)
  $core.bool hasSetDeviceKind() => $_has(25);
  @$pb.TagNumber(26)
  void clearSetDeviceKind() => $_clearField(26);
  @$pb.TagNumber(26)
  SetDeviceKind ensureSetDeviceKind() => $_ensure(25);

  @$pb.TagNumber(27)
  SetDeviceOutput get setDeviceOutput => $_getN(26);
  @$pb.TagNumber(27)
  set setDeviceOutput(SetDeviceOutput value) => $_setField(27, value);
  @$pb.TagNumber(27)
  $core.bool hasSetDeviceOutput() => $_has(26);
  @$pb.TagNumber(27)
  void clearSetDeviceOutput() => $_clearField(27);
  @$pb.TagNumber(27)
  SetDeviceOutput ensureSetDeviceOutput() => $_ensure(26);

  @$pb.TagNumber(28)
  SetDevicePin get setDevicePin => $_getN(27);
  @$pb.TagNumber(28)
  set setDevicePin(SetDevicePin value) => $_setField(28, value);
  @$pb.TagNumber(28)
  $core.bool hasSetDevicePin() => $_has(27);
  @$pb.TagNumber(28)
  void clearSetDevicePin() => $_clearField(28);
  @$pb.TagNumber(28)
  SetDevicePin ensureSetDevicePin() => $_ensure(27);

  @$pb.TagNumber(29)
  DeleteDevice get deleteDevice => $_getN(28);
  @$pb.TagNumber(29)
  set deleteDevice(DeleteDevice value) => $_setField(29, value);
  @$pb.TagNumber(29)
  $core.bool hasDeleteDevice() => $_has(28);
  @$pb.TagNumber(29)
  void clearDeleteDevice() => $_clearField(29);
  @$pb.TagNumber(29)
  DeleteDevice ensureDeleteDevice() => $_ensure(28);
}

class CreateConcept extends $pb.GeneratedMessage {
  factory CreateConcept({
    $core.String? name,
    $core.String? description,
    Representation? representation,
  }) {
    final result = CreateConcept._();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (representation != null) result.representation = representation;
    return result;
  }

  CreateConcept._();

  factory CreateConcept.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateConcept()..mergeFromBuffer(data, registry);
  factory CreateConcept.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateConcept()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateConcept',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CreateConcept.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOM<Representation>(3, _omitFieldNames ? '' : 'representation',
        subBuilder: Representation.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateConcept clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateConcept copyWith(void Function(CreateConcept) updates) =>
      super.copyWith((message) => updates(message as CreateConcept)) as CreateConcept;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CreateConcept() / CreateConcept.new instead')
  static CreateConcept create() => CreateConcept._();
  static $pb.GeneratedMessage $_createMessage() => CreateConcept._();
  @$core.override
  CreateConcept createEmptyInstance() => CreateConcept._();
  @$core.pragma('dart2js:noInline')
  static CreateConcept getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateConcept>(CreateConcept.$_createMessage);
  static CreateConcept? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  Representation get representation => $_getN(2);
  @$pb.TagNumber(3)
  set representation(Representation value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRepresentation() => $_has(2);
  @$pb.TagNumber(3)
  void clearRepresentation() => $_clearField(3);
  @$pb.TagNumber(3)
  Representation ensureRepresentation() => $_ensure(2);
}

class RenameConcept extends $pb.GeneratedMessage {
  factory RenameConcept({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = RenameConcept._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  RenameConcept._();

  factory RenameConcept.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameConcept()..mergeFromBuffer(data, registry);
  factory RenameConcept.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameConcept()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RenameConcept',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RenameConcept.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameConcept clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameConcept copyWith(void Function(RenameConcept) updates) =>
      super.copyWith((message) => updates(message as RenameConcept)) as RenameConcept;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RenameConcept() / RenameConcept.new instead')
  static RenameConcept create() => RenameConcept._();
  static $pb.GeneratedMessage $_createMessage() => RenameConcept._();
  @$core.override
  RenameConcept createEmptyInstance() => RenameConcept._();
  @$core.pragma('dart2js:noInline')
  static RenameConcept getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameConcept>(RenameConcept.$_createMessage);
  static RenameConcept? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SetConceptDescription extends $pb.GeneratedMessage {
  factory SetConceptDescription({
    $fixnum.Int64? id,
    $core.String? description,
  }) {
    final result = SetConceptDescription._();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    return result;
  }

  SetConceptDescription._();

  factory SetConceptDescription.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetConceptDescription()..mergeFromBuffer(data, registry);
  factory SetConceptDescription.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetConceptDescription()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetConceptDescription',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetConceptDescription.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetConceptDescription clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetConceptDescription copyWith(void Function(SetConceptDescription) updates) =>
      super.copyWith((message) => updates(message as SetConceptDescription))
          as SetConceptDescription;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetConceptDescription() / SetConceptDescription.new instead')
  static SetConceptDescription create() => SetConceptDescription._();
  static $pb.GeneratedMessage $_createMessage() => SetConceptDescription._();
  @$core.override
  SetConceptDescription createEmptyInstance() => SetConceptDescription._();
  @$core.pragma('dart2js:noInline')
  static SetConceptDescription getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SetConceptDescription>(
          SetConceptDescription.$_createMessage);
  static SetConceptDescription? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);
}

class SetConceptRepresentation extends $pb.GeneratedMessage {
  factory SetConceptRepresentation({
    $fixnum.Int64? id,
    Representation? representation,
  }) {
    final result = SetConceptRepresentation._();
    if (id != null) result.id = id;
    if (representation != null) result.representation = representation;
    return result;
  }

  SetConceptRepresentation._();

  factory SetConceptRepresentation.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetConceptRepresentation()..mergeFromBuffer(data, registry);
  factory SetConceptRepresentation.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetConceptRepresentation()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetConceptRepresentation',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetConceptRepresentation.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Representation>(2, _omitFieldNames ? '' : 'representation',
        subBuilder: Representation.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetConceptRepresentation clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetConceptRepresentation copyWith(void Function(SetConceptRepresentation) updates) =>
      super.copyWith((message) => updates(message as SetConceptRepresentation))
          as SetConceptRepresentation;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetConceptRepresentation() / SetConceptRepresentation.new instead')
  static SetConceptRepresentation create() => SetConceptRepresentation._();
  static $pb.GeneratedMessage $_createMessage() => SetConceptRepresentation._();
  @$core.override
  SetConceptRepresentation createEmptyInstance() => SetConceptRepresentation._();
  @$core.pragma('dart2js:noInline')
  static SetConceptRepresentation getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SetConceptRepresentation>(
          SetConceptRepresentation.$_createMessage);
  static SetConceptRepresentation? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  Representation get representation => $_getN(1);
  @$pb.TagNumber(2)
  set representation(Representation value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRepresentation() => $_has(1);
  @$pb.TagNumber(2)
  void clearRepresentation() => $_clearField(2);
  @$pb.TagNumber(2)
  Representation ensureRepresentation() => $_ensure(1);
}

class DeleteConcept extends $pb.GeneratedMessage {
  factory DeleteConcept({
    $fixnum.Int64? id,
  }) {
    final result = DeleteConcept._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteConcept._();

  factory DeleteConcept.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConcept()..mergeFromBuffer(data, registry);
  factory DeleteConcept.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConcept()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeleteConcept',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeleteConcept.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConcept clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConcept copyWith(void Function(DeleteConcept) updates) =>
      super.copyWith((message) => updates(message as DeleteConcept)) as DeleteConcept;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeleteConcept() / DeleteConcept.new instead')
  static DeleteConcept create() => DeleteConcept._();
  static $pb.GeneratedMessage $_createMessage() => DeleteConcept._();
  @$core.override
  DeleteConcept createEmptyInstance() => DeleteConcept._();
  @$core.pragma('dart2js:noInline')
  static DeleteConcept getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteConcept>(DeleteConcept.$_createMessage);
  static DeleteConcept? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class CreateMapping extends $pb.GeneratedMessage {
  factory CreateMapping({
    $core.String? name,
    $core.String? description,
    Signature? signature,
  }) {
    final result = CreateMapping._();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (signature != null) result.signature = signature;
    return result;
  }

  CreateMapping._();

  factory CreateMapping.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateMapping()..mergeFromBuffer(data, registry);
  factory CreateMapping.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateMapping()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateMapping',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CreateMapping.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOM<Signature>(3, _omitFieldNames ? '' : 'signature', subBuilder: Signature.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateMapping clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateMapping copyWith(void Function(CreateMapping) updates) =>
      super.copyWith((message) => updates(message as CreateMapping)) as CreateMapping;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CreateMapping() / CreateMapping.new instead')
  static CreateMapping create() => CreateMapping._();
  static $pb.GeneratedMessage $_createMessage() => CreateMapping._();
  @$core.override
  CreateMapping createEmptyInstance() => CreateMapping._();
  @$core.pragma('dart2js:noInline')
  static CreateMapping getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateMapping>(CreateMapping.$_createMessage);
  static CreateMapping? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  Signature get signature => $_getN(2);
  @$pb.TagNumber(3)
  set signature(Signature value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSignature() => $_has(2);
  @$pb.TagNumber(3)
  void clearSignature() => $_clearField(3);
  @$pb.TagNumber(3)
  Signature ensureSignature() => $_ensure(2);
}

class RenameMapping extends $pb.GeneratedMessage {
  factory RenameMapping({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = RenameMapping._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  RenameMapping._();

  factory RenameMapping.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameMapping()..mergeFromBuffer(data, registry);
  factory RenameMapping.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameMapping()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RenameMapping',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RenameMapping.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameMapping clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameMapping copyWith(void Function(RenameMapping) updates) =>
      super.copyWith((message) => updates(message as RenameMapping)) as RenameMapping;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RenameMapping() / RenameMapping.new instead')
  static RenameMapping create() => RenameMapping._();
  static $pb.GeneratedMessage $_createMessage() => RenameMapping._();
  @$core.override
  RenameMapping createEmptyInstance() => RenameMapping._();
  @$core.pragma('dart2js:noInline')
  static RenameMapping getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameMapping>(RenameMapping.$_createMessage);
  static RenameMapping? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SetMappingDescription extends $pb.GeneratedMessage {
  factory SetMappingDescription({
    $fixnum.Int64? id,
    $core.String? description,
  }) {
    final result = SetMappingDescription._();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    return result;
  }

  SetMappingDescription._();

  factory SetMappingDescription.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingDescription()..mergeFromBuffer(data, registry);
  factory SetMappingDescription.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingDescription()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetMappingDescription',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetMappingDescription.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingDescription clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingDescription copyWith(void Function(SetMappingDescription) updates) =>
      super.copyWith((message) => updates(message as SetMappingDescription))
          as SetMappingDescription;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetMappingDescription() / SetMappingDescription.new instead')
  static SetMappingDescription create() => SetMappingDescription._();
  static $pb.GeneratedMessage $_createMessage() => SetMappingDescription._();
  @$core.override
  SetMappingDescription createEmptyInstance() => SetMappingDescription._();
  @$core.pragma('dart2js:noInline')
  static SetMappingDescription getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SetMappingDescription>(
          SetMappingDescription.$_createMessage);
  static SetMappingDescription? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);
}

class SetMappingSignature extends $pb.GeneratedMessage {
  factory SetMappingSignature({
    $fixnum.Int64? id,
    Signature? signature,
  }) {
    final result = SetMappingSignature._();
    if (id != null) result.id = id;
    if (signature != null) result.signature = signature;
    return result;
  }

  SetMappingSignature._();

  factory SetMappingSignature.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingSignature()..mergeFromBuffer(data, registry);
  factory SetMappingSignature.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingSignature()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetMappingSignature',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetMappingSignature.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Signature>(2, _omitFieldNames ? '' : 'signature', subBuilder: Signature.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingSignature clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingSignature copyWith(void Function(SetMappingSignature) updates) =>
      super.copyWith((message) => updates(message as SetMappingSignature)) as SetMappingSignature;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetMappingSignature() / SetMappingSignature.new instead')
  static SetMappingSignature create() => SetMappingSignature._();
  static $pb.GeneratedMessage $_createMessage() => SetMappingSignature._();
  @$core.override
  SetMappingSignature createEmptyInstance() => SetMappingSignature._();
  @$core.pragma('dart2js:noInline')
  static SetMappingSignature getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetMappingSignature>(SetMappingSignature.$_createMessage);
  static SetMappingSignature? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  Signature get signature => $_getN(1);
  @$pb.TagNumber(2)
  set signature(Signature value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasSignature() => $_has(1);
  @$pb.TagNumber(2)
  void clearSignature() => $_clearField(2);
  @$pb.TagNumber(2)
  Signature ensureSignature() => $_ensure(1);
}

class AttachDefinition extends $pb.GeneratedMessage {
  factory AttachDefinition({
    $fixnum.Int64? id,
    Definition? definition,
  }) {
    final result = AttachDefinition._();
    if (id != null) result.id = id;
    if (definition != null) result.definition = definition;
    return result;
  }

  AttachDefinition._();

  factory AttachDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AttachDefinition()..mergeFromBuffer(data, registry);
  factory AttachDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AttachDefinition()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'AttachDefinition',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: AttachDefinition.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Definition>(2, _omitFieldNames ? '' : 'definition',
        subBuilder: Definition.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AttachDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AttachDefinition copyWith(void Function(AttachDefinition) updates) =>
      super.copyWith((message) => updates(message as AttachDefinition)) as AttachDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AttachDefinition() / AttachDefinition.new instead')
  static AttachDefinition create() => AttachDefinition._();
  static $pb.GeneratedMessage $_createMessage() => AttachDefinition._();
  @$core.override
  AttachDefinition createEmptyInstance() => AttachDefinition._();
  @$core.pragma('dart2js:noInline')
  static AttachDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AttachDefinition>(AttachDefinition.$_createMessage);
  static AttachDefinition? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  Definition get definition => $_getN(1);
  @$pb.TagNumber(2)
  set definition(Definition value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasDefinition() => $_has(1);
  @$pb.TagNumber(2)
  void clearDefinition() => $_clearField(2);
  @$pb.TagNumber(2)
  Definition ensureDefinition() => $_ensure(1);
}

class ReplaceDefinition extends $pb.GeneratedMessage {
  factory ReplaceDefinition({
    $fixnum.Int64? id,
    Definition? definition,
  }) {
    final result = ReplaceDefinition._();
    if (id != null) result.id = id;
    if (definition != null) result.definition = definition;
    return result;
  }

  ReplaceDefinition._();

  factory ReplaceDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReplaceDefinition()..mergeFromBuffer(data, registry);
  factory ReplaceDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReplaceDefinition()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ReplaceDefinition',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ReplaceDefinition.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Definition>(2, _omitFieldNames ? '' : 'definition',
        subBuilder: Definition.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplaceDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplaceDefinition copyWith(void Function(ReplaceDefinition) updates) =>
      super.copyWith((message) => updates(message as ReplaceDefinition)) as ReplaceDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ReplaceDefinition() / ReplaceDefinition.new instead')
  static ReplaceDefinition create() => ReplaceDefinition._();
  static $pb.GeneratedMessage $_createMessage() => ReplaceDefinition._();
  @$core.override
  ReplaceDefinition createEmptyInstance() => ReplaceDefinition._();
  @$core.pragma('dart2js:noInline')
  static ReplaceDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReplaceDefinition>(ReplaceDefinition.$_createMessage);
  static ReplaceDefinition? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// Absent = detach.
  @$pb.TagNumber(2)
  Definition get definition => $_getN(1);
  @$pb.TagNumber(2)
  set definition(Definition value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasDefinition() => $_has(1);
  @$pb.TagNumber(2)
  void clearDefinition() => $_clearField(2);
  @$pb.TagNumber(2)
  Definition ensureDefinition() => $_ensure(1);
}

class DeleteMapping extends $pb.GeneratedMessage {
  factory DeleteMapping({
    $fixnum.Int64? id,
  }) {
    final result = DeleteMapping._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteMapping._();

  factory DeleteMapping.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMapping()..mergeFromBuffer(data, registry);
  factory DeleteMapping.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMapping()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeleteMapping',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeleteMapping.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMapping clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMapping copyWith(void Function(DeleteMapping) updates) =>
      super.copyWith((message) => updates(message as DeleteMapping)) as DeleteMapping;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeleteMapping() / DeleteMapping.new instead')
  static DeleteMapping create() => DeleteMapping._();
  static $pb.GeneratedMessage $_createMessage() => DeleteMapping._();
  @$core.override
  DeleteMapping createEmptyInstance() => DeleteMapping._();
  @$core.pragma('dart2js:noInline')
  static DeleteMapping getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteMapping>(DeleteMapping.$_createMessage);
  static DeleteMapping? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class CreateClockDomain extends $pb.GeneratedMessage {
  factory CreateClockDomain({
    $core.String? name,
  }) {
    final result = CreateClockDomain._();
    if (name != null) result.name = name;
    return result;
  }

  CreateClockDomain._();

  factory CreateClockDomain.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateClockDomain()..mergeFromBuffer(data, registry);
  factory CreateClockDomain.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateClockDomain()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateClockDomain',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CreateClockDomain.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateClockDomain clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateClockDomain copyWith(void Function(CreateClockDomain) updates) =>
      super.copyWith((message) => updates(message as CreateClockDomain)) as CreateClockDomain;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CreateClockDomain() / CreateClockDomain.new instead')
  static CreateClockDomain create() => CreateClockDomain._();
  static $pb.GeneratedMessage $_createMessage() => CreateClockDomain._();
  @$core.override
  CreateClockDomain createEmptyInstance() => CreateClockDomain._();
  @$core.pragma('dart2js:noInline')
  static CreateClockDomain getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateClockDomain>(CreateClockDomain.$_createMessage);
  static CreateClockDomain? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class RenameClockDomain extends $pb.GeneratedMessage {
  factory RenameClockDomain({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = RenameClockDomain._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  RenameClockDomain._();

  factory RenameClockDomain.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameClockDomain()..mergeFromBuffer(data, registry);
  factory RenameClockDomain.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameClockDomain()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RenameClockDomain',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RenameClockDomain.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameClockDomain clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameClockDomain copyWith(void Function(RenameClockDomain) updates) =>
      super.copyWith((message) => updates(message as RenameClockDomain)) as RenameClockDomain;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RenameClockDomain() / RenameClockDomain.new instead')
  static RenameClockDomain create() => RenameClockDomain._();
  static $pb.GeneratedMessage $_createMessage() => RenameClockDomain._();
  @$core.override
  RenameClockDomain createEmptyInstance() => RenameClockDomain._();
  @$core.pragma('dart2js:noInline')
  static RenameClockDomain getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameClockDomain>(RenameClockDomain.$_createMessage);
  static RenameClockDomain? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

/// Refused while a mapping or an output is in the domain.
class DeleteClockDomain extends $pb.GeneratedMessage {
  factory DeleteClockDomain({
    $fixnum.Int64? id,
  }) {
    final result = DeleteClockDomain._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteClockDomain._();

  factory DeleteClockDomain.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteClockDomain()..mergeFromBuffer(data, registry);
  factory DeleteClockDomain.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteClockDomain()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeleteClockDomain',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeleteClockDomain.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteClockDomain clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteClockDomain copyWith(void Function(DeleteClockDomain) updates) =>
      super.copyWith((message) => updates(message as DeleteClockDomain)) as DeleteClockDomain;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeleteClockDomain() / DeleteClockDomain.new instead')
  static DeleteClockDomain create() => DeleteClockDomain._();
  static $pb.GeneratedMessage $_createMessage() => DeleteClockDomain._();
  @$core.override
  DeleteClockDomain createEmptyInstance() => DeleteClockDomain._();
  @$core.pragma('dart2js:noInline')
  static DeleteClockDomain getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteClockDomain>(DeleteClockDomain.$_createMessage);
  static DeleteClockDomain? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class SetMappingClock extends $pb.GeneratedMessage {
  factory SetMappingClock({
    $fixnum.Int64? id,
    $fixnum.Int64? clockId,
  }) {
    final result = SetMappingClock._();
    if (id != null) result.id = id;
    if (clockId != null) result.clockId = clockId;
    return result;
  }

  SetMappingClock._();

  factory SetMappingClock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingClock()..mergeFromBuffer(data, registry);
  factory SetMappingClock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingClock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetMappingClock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetMappingClock.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingClock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingClock copyWith(void Function(SetMappingClock) updates) =>
      super.copyWith((message) => updates(message as SetMappingClock)) as SetMappingClock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetMappingClock() / SetMappingClock.new instead')
  static SetMappingClock create() => SetMappingClock._();
  static $pb.GeneratedMessage $_createMessage() => SetMappingClock._();
  @$core.override
  SetMappingClock createEmptyInstance() => SetMappingClock._();
  @$core.pragma('dart2js:noInline')
  static SetMappingClock getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetMappingClock>(SetMappingClock.$_createMessage);
  static SetMappingClock? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get clockId => $_getI64(1);
  @$pb.TagNumber(2)
  set clockId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasClockId() => $_has(1);
  @$pb.TagNumber(2)
  void clearClockId() => $_clearField(2);
}

class CreateOutput extends $pb.GeneratedMessage {
  factory CreateOutput({
    $core.String? name,
    $core.String? description,
    $fixnum.Int64? accepts,
    $fixnum.Int64? clockId,
  }) {
    final result = CreateOutput._();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (accepts != null) result.accepts = accepts;
    if (clockId != null) result.clockId = clockId;
    return result;
  }

  CreateOutput._();

  factory CreateOutput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateOutput()..mergeFromBuffer(data, registry);
  factory CreateOutput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateOutput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateOutput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CreateOutput.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'accepts', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateOutput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateOutput copyWith(void Function(CreateOutput) updates) =>
      super.copyWith((message) => updates(message as CreateOutput)) as CreateOutput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CreateOutput() / CreateOutput.new instead')
  static CreateOutput create() => CreateOutput._();
  static $pb.GeneratedMessage $_createMessage() => CreateOutput._();
  @$core.override
  CreateOutput createEmptyInstance() => CreateOutput._();
  @$core.pragma('dart2js:noInline')
  static CreateOutput getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateOutput>(CreateOutput.$_createMessage);
  static CreateOutput? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  /// The concept the sink accepts.
  @$pb.TagNumber(3)
  $fixnum.Int64 get accepts => $_getI64(2);
  @$pb.TagNumber(3)
  set accepts($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAccepts() => $_has(2);
  @$pb.TagNumber(3)
  void clearAccepts() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get clockId => $_getI64(3);
  @$pb.TagNumber(4)
  set clockId($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasClockId() => $_has(3);
  @$pb.TagNumber(4)
  void clearClockId() => $_clearField(4);
}

class RenameOutput extends $pb.GeneratedMessage {
  factory RenameOutput({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = RenameOutput._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  RenameOutput._();

  factory RenameOutput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameOutput()..mergeFromBuffer(data, registry);
  factory RenameOutput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameOutput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RenameOutput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RenameOutput.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameOutput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameOutput copyWith(void Function(RenameOutput) updates) =>
      super.copyWith((message) => updates(message as RenameOutput)) as RenameOutput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RenameOutput() / RenameOutput.new instead')
  static RenameOutput create() => RenameOutput._();
  static $pb.GeneratedMessage $_createMessage() => RenameOutput._();
  @$core.override
  RenameOutput createEmptyInstance() => RenameOutput._();
  @$core.pragma('dart2js:noInline')
  static RenameOutput getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameOutput>(RenameOutput.$_createMessage);
  static RenameOutput? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SetOutputAccepts extends $pb.GeneratedMessage {
  factory SetOutputAccepts({
    $fixnum.Int64? id,
    $fixnum.Int64? accepts,
  }) {
    final result = SetOutputAccepts._();
    if (id != null) result.id = id;
    if (accepts != null) result.accepts = accepts;
    return result;
  }

  SetOutputAccepts._();

  factory SetOutputAccepts.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputAccepts()..mergeFromBuffer(data, registry);
  factory SetOutputAccepts.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputAccepts()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetOutputAccepts',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetOutputAccepts.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'accepts', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputAccepts clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputAccepts copyWith(void Function(SetOutputAccepts) updates) =>
      super.copyWith((message) => updates(message as SetOutputAccepts)) as SetOutputAccepts;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetOutputAccepts() / SetOutputAccepts.new instead')
  static SetOutputAccepts create() => SetOutputAccepts._();
  static $pb.GeneratedMessage $_createMessage() => SetOutputAccepts._();
  @$core.override
  SetOutputAccepts createEmptyInstance() => SetOutputAccepts._();
  @$core.pragma('dart2js:noInline')
  static SetOutputAccepts getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetOutputAccepts>(SetOutputAccepts.$_createMessage);
  static SetOutputAccepts? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get accepts => $_getI64(1);
  @$pb.TagNumber(2)
  set accepts($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAccepts() => $_has(1);
  @$pb.TagNumber(2)
  void clearAccepts() => $_clearField(2);
}

class SetOutputClock extends $pb.GeneratedMessage {
  factory SetOutputClock({
    $fixnum.Int64? id,
    $fixnum.Int64? clockId,
  }) {
    final result = SetOutputClock._();
    if (id != null) result.id = id;
    if (clockId != null) result.clockId = clockId;
    return result;
  }

  SetOutputClock._();

  factory SetOutputClock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputClock()..mergeFromBuffer(data, registry);
  factory SetOutputClock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputClock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetOutputClock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetOutputClock.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputClock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputClock copyWith(void Function(SetOutputClock) updates) =>
      super.copyWith((message) => updates(message as SetOutputClock)) as SetOutputClock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetOutputClock() / SetOutputClock.new instead')
  static SetOutputClock create() => SetOutputClock._();
  static $pb.GeneratedMessage $_createMessage() => SetOutputClock._();
  @$core.override
  SetOutputClock createEmptyInstance() => SetOutputClock._();
  @$core.pragma('dart2js:noInline')
  static SetOutputClock getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetOutputClock>(SetOutputClock.$_createMessage);
  static SetOutputClock? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get clockId => $_getI64(1);
  @$pb.TagNumber(2)
  set clockId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasClockId() => $_has(1);
  @$pb.TagNumber(2)
  void clearClockId() => $_clearField(2);
}

class SetOutputRequired extends $pb.GeneratedMessage {
  factory SetOutputRequired({
    $fixnum.Int64? id,
    $core.bool? required,
  }) {
    final result = SetOutputRequired._();
    if (id != null) result.id = id;
    if (required != null) result.required = required;
    return result;
  }

  SetOutputRequired._();

  factory SetOutputRequired.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputRequired()..mergeFromBuffer(data, registry);
  factory SetOutputRequired.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetOutputRequired()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetOutputRequired',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetOutputRequired.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(2, _omitFieldNames ? '' : 'required')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputRequired clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetOutputRequired copyWith(void Function(SetOutputRequired) updates) =>
      super.copyWith((message) => updates(message as SetOutputRequired)) as SetOutputRequired;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetOutputRequired() / SetOutputRequired.new instead')
  static SetOutputRequired create() => SetOutputRequired._();
  static $pb.GeneratedMessage $_createMessage() => SetOutputRequired._();
  @$core.override
  SetOutputRequired createEmptyInstance() => SetOutputRequired._();
  @$core.pragma('dart2js:noInline')
  static SetOutputRequired getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetOutputRequired>(SetOutputRequired.$_createMessage);
  static SetOutputRequired? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get required => $_getBF(1);
  @$pb.TagNumber(2)
  set required($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRequired() => $_has(1);
  @$pb.TagNumber(2)
  void clearRequired() => $_clearField(2);
}

/// Refused while a mapping drives the sink or a device realises it.
class DeleteOutput extends $pb.GeneratedMessage {
  factory DeleteOutput({
    $fixnum.Int64? id,
  }) {
    final result = DeleteOutput._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteOutput._();

  factory DeleteOutput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteOutput()..mergeFromBuffer(data, registry);
  factory DeleteOutput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteOutput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeleteOutput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeleteOutput.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteOutput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteOutput copyWith(void Function(DeleteOutput) updates) =>
      super.copyWith((message) => updates(message as DeleteOutput)) as DeleteOutput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeleteOutput() / DeleteOutput.new instead')
  static DeleteOutput create() => DeleteOutput._();
  static $pb.GeneratedMessage $_createMessage() => DeleteOutput._();
  @$core.override
  DeleteOutput createEmptyInstance() => DeleteOutput._();
  @$core.pragma('dart2js:noInline')
  static DeleteOutput getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteOutput>(DeleteOutput.$_createMessage);
  static DeleteOutput? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

/// Connect (or disconnect) a mapping to the output it commits to.
class SetMappingDrive extends $pb.GeneratedMessage {
  factory SetMappingDrive({
    $fixnum.Int64? id,
    $fixnum.Int64? outputId,
  }) {
    final result = SetMappingDrive._();
    if (id != null) result.id = id;
    if (outputId != null) result.outputId = outputId;
    return result;
  }

  SetMappingDrive._();

  factory SetMappingDrive.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingDrive()..mergeFromBuffer(data, registry);
  factory SetMappingDrive.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetMappingDrive()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetMappingDrive',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetMappingDrive.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'outputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingDrive clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetMappingDrive copyWith(void Function(SetMappingDrive) updates) =>
      super.copyWith((message) => updates(message as SetMappingDrive)) as SetMappingDrive;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetMappingDrive() / SetMappingDrive.new instead')
  static SetMappingDrive create() => SetMappingDrive._();
  static $pb.GeneratedMessage $_createMessage() => SetMappingDrive._();
  @$core.override
  SetMappingDrive createEmptyInstance() => SetMappingDrive._();
  @$core.pragma('dart2js:noInline')
  static SetMappingDrive getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetMappingDrive>(SetMappingDrive.$_createMessage);
  static SetMappingDrive? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get outputId => $_getI64(1);
  @$pb.TagNumber(2)
  set outputId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOutputId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOutputId() => $_clearField(2);
}

class CreateDevice extends $pb.GeneratedMessage {
  factory CreateDevice({
    $core.String? name,
    DeviceKind? kind,
    $fixnum.Int64? outputId,
  }) {
    final result = CreateDevice._();
    if (name != null) result.name = name;
    if (kind != null) result.kind = kind;
    if (outputId != null) result.outputId = outputId;
    return result;
  }

  CreateDevice._();

  factory CreateDevice.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateDevice()..mergeFromBuffer(data, registry);
  factory CreateDevice.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateDevice()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'CreateDevice',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CreateDevice.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aE<DeviceKind>(2, _omitFieldNames ? '' : 'kind', enumValues: DeviceKind.values)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'outputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDevice clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDevice copyWith(void Function(CreateDevice) updates) =>
      super.copyWith((message) => updates(message as CreateDevice)) as CreateDevice;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CreateDevice() / CreateDevice.new instead')
  static CreateDevice create() => CreateDevice._();
  static $pb.GeneratedMessage $_createMessage() => CreateDevice._();
  @$core.override
  CreateDevice createEmptyInstance() => CreateDevice._();
  @$core.pragma('dart2js:noInline')
  static CreateDevice getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateDevice>(CreateDevice.$_createMessage);
  static CreateDevice? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  DeviceKind get kind => $_getN(1);
  @$pb.TagNumber(2)
  set kind(DeviceKind value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get outputId => $_getI64(2);
  @$pb.TagNumber(3)
  set outputId($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOutputId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOutputId() => $_clearField(3);
}

class RenameDevice extends $pb.GeneratedMessage {
  factory RenameDevice({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = RenameDevice._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  RenameDevice._();

  factory RenameDevice.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameDevice()..mergeFromBuffer(data, registry);
  factory RenameDevice.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RenameDevice()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RenameDevice',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RenameDevice.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameDevice clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RenameDevice copyWith(void Function(RenameDevice) updates) =>
      super.copyWith((message) => updates(message as RenameDevice)) as RenameDevice;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RenameDevice() / RenameDevice.new instead')
  static RenameDevice create() => RenameDevice._();
  static $pb.GeneratedMessage $_createMessage() => RenameDevice._();
  @$core.override
  RenameDevice createEmptyInstance() => RenameDevice._();
  @$core.pragma('dart2js:noInline')
  static RenameDevice getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RenameDevice>(RenameDevice.$_createMessage);
  static RenameDevice? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SetDeviceKind extends $pb.GeneratedMessage {
  factory SetDeviceKind({
    $fixnum.Int64? id,
    DeviceKind? kind,
  }) {
    final result = SetDeviceKind._();
    if (id != null) result.id = id;
    if (kind != null) result.kind = kind;
    return result;
  }

  SetDeviceKind._();

  factory SetDeviceKind.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDeviceKind()..mergeFromBuffer(data, registry);
  factory SetDeviceKind.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDeviceKind()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetDeviceKind',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetDeviceKind.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<DeviceKind>(2, _omitFieldNames ? '' : 'kind', enumValues: DeviceKind.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDeviceKind clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDeviceKind copyWith(void Function(SetDeviceKind) updates) =>
      super.copyWith((message) => updates(message as SetDeviceKind)) as SetDeviceKind;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetDeviceKind() / SetDeviceKind.new instead')
  static SetDeviceKind create() => SetDeviceKind._();
  static $pb.GeneratedMessage $_createMessage() => SetDeviceKind._();
  @$core.override
  SetDeviceKind createEmptyInstance() => SetDeviceKind._();
  @$core.pragma('dart2js:noInline')
  static SetDeviceKind getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetDeviceKind>(SetDeviceKind.$_createMessage);
  static SetDeviceKind? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  DeviceKind get kind => $_getN(1);
  @$pb.TagNumber(2)
  set kind(DeviceKind value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);
}

class SetDeviceOutput extends $pb.GeneratedMessage {
  factory SetDeviceOutput({
    $fixnum.Int64? id,
    $fixnum.Int64? outputId,
  }) {
    final result = SetDeviceOutput._();
    if (id != null) result.id = id;
    if (outputId != null) result.outputId = outputId;
    return result;
  }

  SetDeviceOutput._();

  factory SetDeviceOutput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDeviceOutput()..mergeFromBuffer(data, registry);
  factory SetDeviceOutput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDeviceOutput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetDeviceOutput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetDeviceOutput.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'outputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDeviceOutput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDeviceOutput copyWith(void Function(SetDeviceOutput) updates) =>
      super.copyWith((message) => updates(message as SetDeviceOutput)) as SetDeviceOutput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetDeviceOutput() / SetDeviceOutput.new instead')
  static SetDeviceOutput create() => SetDeviceOutput._();
  static $pb.GeneratedMessage $_createMessage() => SetDeviceOutput._();
  @$core.override
  SetDeviceOutput createEmptyInstance() => SetDeviceOutput._();
  @$core.pragma('dart2js:noInline')
  static SetDeviceOutput getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetDeviceOutput>(SetDeviceOutput.$_createMessage);
  static SetDeviceOutput? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get outputId => $_getI64(1);
  @$pb.TagNumber(2)
  set outputId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOutputId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOutputId() => $_clearField(2);
}

/// Pin one of the device's requirements (by its index in the kind's list)
/// to a named board resource, or release it.
class SetDevicePin extends $pb.GeneratedMessage {
  factory SetDevicePin({
    $fixnum.Int64? id,
    $core.int? index,
    $core.String? resource,
  }) {
    final result = SetDevicePin._();
    if (id != null) result.id = id;
    if (index != null) result.index = index;
    if (resource != null) result.resource = resource;
    return result;
  }

  SetDevicePin._();

  factory SetDevicePin.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDevicePin()..mergeFromBuffer(data, registry);
  factory SetDevicePin.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetDevicePin()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetDevicePin',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetDevicePin.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(2, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'resource')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDevicePin clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetDevicePin copyWith(void Function(SetDevicePin) updates) =>
      super.copyWith((message) => updates(message as SetDevicePin)) as SetDevicePin;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetDevicePin() / SetDevicePin.new instead')
  static SetDevicePin create() => SetDevicePin._();
  static $pb.GeneratedMessage $_createMessage() => SetDevicePin._();
  @$core.override
  SetDevicePin createEmptyInstance() => SetDevicePin._();
  @$core.pragma('dart2js:noInline')
  static SetDevicePin getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetDevicePin>(SetDevicePin.$_createMessage);
  static SetDevicePin? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get index => $_getIZ(1);
  @$pb.TagNumber(2)
  set index($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get resource => $_getSZ(2);
  @$pb.TagNumber(3)
  set resource($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResource() => $_has(2);
  @$pb.TagNumber(3)
  void clearResource() => $_clearField(3);
}

class DeleteDevice extends $pb.GeneratedMessage {
  factory DeleteDevice({
    $fixnum.Int64? id,
  }) {
    final result = DeleteDevice._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteDevice._();

  factory DeleteDevice.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteDevice()..mergeFromBuffer(data, registry);
  factory DeleteDevice.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteDevice()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeleteDevice',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeleteDevice.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDevice clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDevice copyWith(void Function(DeleteDevice) updates) =>
      super.copyWith((message) => updates(message as DeleteDevice)) as DeleteDevice;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeleteDevice() / DeleteDevice.new instead')
  static DeleteDevice create() => DeleteDevice._();
  static $pb.GeneratedMessage $_createMessage() => DeleteDevice._();
  @$core.override
  DeleteDevice createEmptyInstance() => DeleteDevice._();
  @$core.pragma('dart2js:noInline')
  static DeleteDevice getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteDevice>(DeleteDevice.$_createMessage);
  static DeleteDevice? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class EditOutcome extends $pb.GeneratedMessage {
  factory EditOutcome({
    EditKind? kind,
    $core.Iterable<Invalidation>? invalidates,
    $core.Iterable<$fixnum.Int64>? originDecls,
    $fixnum.Int64? createdConcept,
    $fixnum.Int64? createdMapping,
    $fixnum.Int64? createdClock,
    $fixnum.Int64? createdOutput,
    $fixnum.Int64? createdDevice,
  }) {
    final result = EditOutcome._();
    if (kind != null) result.kind = kind;
    if (invalidates != null) result.invalidates.addAll(invalidates);
    if (originDecls != null) result.originDecls.addAll(originDecls);
    if (createdConcept != null) result.createdConcept = createdConcept;
    if (createdMapping != null) result.createdMapping = createdMapping;
    if (createdClock != null) result.createdClock = createdClock;
    if (createdOutput != null) result.createdOutput = createdOutput;
    if (createdDevice != null) result.createdDevice = createdDevice;
    return result;
  }

  EditOutcome._();

  factory EditOutcome.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditOutcome()..mergeFromBuffer(data, registry);
  factory EditOutcome.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EditOutcome()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'EditOutcome',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: EditOutcome.$_createMessage)
    ..aE<EditKind>(1, _omitFieldNames ? '' : 'kind', enumValues: EditKind.values)
    ..pc<Invalidation>(2, _omitFieldNames ? '' : 'invalidates', $pb.PbFieldType.KE,
        valueOf: Invalidation.valueOf,
        enumValues: Invalidation.values,
        defaultEnumValue: Invalidation.INVALIDATION_UNSPECIFIED)
    ..p<$fixnum.Int64>(3, _omitFieldNames ? '' : 'originDecls', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'createdConcept', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'createdMapping', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(6, _omitFieldNames ? '' : 'createdClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(7, _omitFieldNames ? '' : 'createdOutput', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(8, _omitFieldNames ? '' : 'createdDevice', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditOutcome clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EditOutcome copyWith(void Function(EditOutcome) updates) =>
      super.copyWith((message) => updates(message as EditOutcome)) as EditOutcome;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EditOutcome() / EditOutcome.new instead')
  static EditOutcome create() => EditOutcome._();
  static $pb.GeneratedMessage $_createMessage() => EditOutcome._();
  @$core.override
  EditOutcome createEmptyInstance() => EditOutcome._();
  @$core.pragma('dart2js:noInline')
  static EditOutcome getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EditOutcome>(EditOutcome.$_createMessage);
  static EditOutcome? _defaultInstance;

  @$pb.TagNumber(1)
  EditKind get kind => $_getN(0);
  @$pb.TagNumber(1)
  set kind(EditKind value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasKind() => $_has(0);
  @$pb.TagNumber(1)
  void clearKind() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<Invalidation> get invalidates => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<$fixnum.Int64> get originDecls => $_getList(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get createdConcept => $_getI64(3);
  @$pb.TagNumber(4)
  set createdConcept($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedConcept() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedConcept() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get createdMapping => $_getI64(4);
  @$pb.TagNumber(5)
  set createdMapping($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCreatedMapping() => $_has(4);
  @$pb.TagNumber(5)
  void clearCreatedMapping() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get createdClock => $_getI64(5);
  @$pb.TagNumber(6)
  set createdClock($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCreatedClock() => $_has(5);
  @$pb.TagNumber(6)
  void clearCreatedClock() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get createdOutput => $_getI64(6);
  @$pb.TagNumber(7)
  set createdOutput($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedOutput() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedOutput() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get createdDevice => $_getI64(7);
  @$pb.TagNumber(8)
  set createdDevice($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedDevice() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedDevice() => $_clearField(8);
}

class ProjectProjection extends $pb.GeneratedMessage {
  factory ProjectProjection({
    $fixnum.Int64? revision,
    $core.String? name,
    $core.String? rootPath,
    $core.Iterable<ConceptView>? concepts,
    $core.Iterable<MappingView>? mappings,
    Layout? layout,
    $core.bool? canUndo,
    $core.bool? canRedo,
    $core.bool? dirty,
    $core.Iterable<ClockView>? clocks,
    $core.Iterable<OutputView>? outputs,
    $core.Iterable<DeviceView>? devices,
  }) {
    final result = ProjectProjection._();
    if (revision != null) result.revision = revision;
    if (name != null) result.name = name;
    if (rootPath != null) result.rootPath = rootPath;
    if (concepts != null) result.concepts.addAll(concepts);
    if (mappings != null) result.mappings.addAll(mappings);
    if (layout != null) result.layout = layout;
    if (canUndo != null) result.canUndo = canUndo;
    if (canRedo != null) result.canRedo = canRedo;
    if (dirty != null) result.dirty = dirty;
    if (clocks != null) result.clocks.addAll(clocks);
    if (outputs != null) result.outputs.addAll(outputs);
    if (devices != null) result.devices.addAll(devices);
    return result;
  }

  ProjectProjection._();

  factory ProjectProjection.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectProjection()..mergeFromBuffer(data, registry);
  factory ProjectProjection.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectProjection()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ProjectProjection',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ProjectProjection.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'rootPath')
    ..pPM<ConceptView>(4, _omitFieldNames ? '' : 'concepts',
        subBuilder: ConceptView.$_createMessage)
    ..pPM<MappingView>(5, _omitFieldNames ? '' : 'mappings',
        subBuilder: MappingView.$_createMessage)
    ..aOM<Layout>(6, _omitFieldNames ? '' : 'layout', subBuilder: Layout.$_createMessage)
    ..aOB(7, _omitFieldNames ? '' : 'canUndo')
    ..aOB(8, _omitFieldNames ? '' : 'canRedo')
    ..aOB(9, _omitFieldNames ? '' : 'dirty')
    ..pPM<ClockView>(10, _omitFieldNames ? '' : 'clocks', subBuilder: ClockView.$_createMessage)
    ..pPM<OutputView>(11, _omitFieldNames ? '' : 'outputs', subBuilder: OutputView.$_createMessage)
    ..pPM<DeviceView>(12, _omitFieldNames ? '' : 'devices', subBuilder: DeviceView.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectProjection clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectProjection copyWith(void Function(ProjectProjection) updates) =>
      super.copyWith((message) => updates(message as ProjectProjection)) as ProjectProjection;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ProjectProjection() / ProjectProjection.new instead')
  static ProjectProjection create() => ProjectProjection._();
  static $pb.GeneratedMessage $_createMessage() => ProjectProjection._();
  @$core.override
  ProjectProjection createEmptyInstance() => ProjectProjection._();
  @$core.pragma('dart2js:noInline')
  static ProjectProjection getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProjectProjection>(ProjectProjection.$_createMessage);
  static ProjectProjection? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get rootPath => $_getSZ(2);
  @$pb.TagNumber(3)
  set rootPath($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRootPath() => $_has(2);
  @$pb.TagNumber(3)
  void clearRootPath() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<ConceptView> get concepts => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<MappingView> get mappings => $_getList(4);

  @$pb.TagNumber(6)
  Layout get layout => $_getN(5);
  @$pb.TagNumber(6)
  set layout(Layout value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasLayout() => $_has(5);
  @$pb.TagNumber(6)
  void clearLayout() => $_clearField(6);
  @$pb.TagNumber(6)
  Layout ensureLayout() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.bool get canUndo => $_getBF(6);
  @$pb.TagNumber(7)
  set canUndo($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCanUndo() => $_has(6);
  @$pb.TagNumber(7)
  void clearCanUndo() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get canRedo => $_getBF(7);
  @$pb.TagNumber(8)
  set canRedo($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCanRedo() => $_has(7);
  @$pb.TagNumber(8)
  void clearCanRedo() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.bool get dirty => $_getBF(8);
  @$pb.TagNumber(9)
  set dirty($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasDirty() => $_has(8);
  @$pb.TagNumber(9)
  void clearDirty() => $_clearField(9);

  @$pb.TagNumber(10)
  $pb.PbList<ClockView> get clocks => $_getList(9);

  @$pb.TagNumber(11)
  $pb.PbList<OutputView> get outputs => $_getList(10);

  @$pb.TagNumber(12)
  $pb.PbList<DeviceView> get devices => $_getList(11);
}

class ClockView extends $pb.GeneratedMessage {
  factory ClockView({
    $fixnum.Int64? id,
    $core.String? name,
  }) {
    final result = ClockView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    return result;
  }

  ClockView._();

  factory ClockView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClockView()..mergeFromBuffer(data, registry);
  factory ClockView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClockView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ClockView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ClockView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClockView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClockView copyWith(void Function(ClockView) updates) =>
      super.copyWith((message) => updates(message as ClockView)) as ClockView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ClockView() / ClockView.new instead')
  static ClockView create() => ClockView._();
  static $pb.GeneratedMessage $_createMessage() => ClockView._();
  @$core.override
  ClockView createEmptyInstance() => ClockView._();
  @$core.pragma('dart2js:noInline')
  static ClockView getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ClockView>(ClockView.$_createMessage);
  static ClockView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class OutputView extends $pb.GeneratedMessage {
  factory OutputView({
    $fixnum.Int64? id,
    $core.String? name,
    $core.String? description,
    $fixnum.Int64? accepts,
    $fixnum.Int64? clockId,
    $core.bool? required,
  }) {
    final result = OutputView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (accepts != null) result.accepts = accepts;
    if (clockId != null) result.clockId = clockId;
    if (required != null) result.required = required;
    return result;
  }

  OutputView._();

  factory OutputView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OutputView()..mergeFromBuffer(data, registry);
  factory OutputView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OutputView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'OutputView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: OutputView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'accepts', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(6, _omitFieldNames ? '' : 'required')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OutputView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OutputView copyWith(void Function(OutputView) updates) =>
      super.copyWith((message) => updates(message as OutputView)) as OutputView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use OutputView() / OutputView.new instead')
  static OutputView create() => OutputView._();
  static $pb.GeneratedMessage $_createMessage() => OutputView._();
  @$core.override
  OutputView createEmptyInstance() => OutputView._();
  @$core.pragma('dart2js:noInline')
  static OutputView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OutputView>(OutputView.$_createMessage);
  static OutputView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get accepts => $_getI64(3);
  @$pb.TagNumber(4)
  set accepts($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAccepts() => $_has(3);
  @$pb.TagNumber(4)
  void clearAccepts() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get clockId => $_getI64(4);
  @$pb.TagNumber(5)
  set clockId($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasClockId() => $_has(4);
  @$pb.TagNumber(5)
  void clearClockId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get required => $_getBF(5);
  @$pb.TagNumber(6)
  set required($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasRequired() => $_has(5);
  @$pb.TagNumber(6)
  void clearRequired() => $_clearField(6);
}

class DeviceView extends $pb.GeneratedMessage {
  factory DeviceView({
    $fixnum.Int64? id,
    $core.String? name,
    DeviceKind? kind,
    $fixnum.Int64? outputId,
    $core.Iterable<DevicePin>? fixedPins,
    $core.Iterable<RequirementLabel>? requirements,
  }) {
    final result = DeviceView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (kind != null) result.kind = kind;
    if (outputId != null) result.outputId = outputId;
    if (fixedPins != null) result.fixedPins.addAll(fixedPins);
    if (requirements != null) result.requirements.addAll(requirements);
    return result;
  }

  DeviceView._();

  factory DeviceView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeviceView()..mergeFromBuffer(data, registry);
  factory DeviceView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeviceView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeviceView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeviceView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aE<DeviceKind>(3, _omitFieldNames ? '' : 'kind', enumValues: DeviceKind.values)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'outputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..pPM<DevicePin>(5, _omitFieldNames ? '' : 'fixedPins', subBuilder: DevicePin.$_createMessage)
    ..pPM<RequirementLabel>(6, _omitFieldNames ? '' : 'requirements',
        subBuilder: RequirementLabel.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeviceView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeviceView copyWith(void Function(DeviceView) updates) =>
      super.copyWith((message) => updates(message as DeviceView)) as DeviceView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeviceView() / DeviceView.new instead')
  static DeviceView create() => DeviceView._();
  static $pb.GeneratedMessage $_createMessage() => DeviceView._();
  @$core.override
  DeviceView createEmptyInstance() => DeviceView._();
  @$core.pragma('dart2js:noInline')
  static DeviceView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeviceView>(DeviceView.$_createMessage);
  static DeviceView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  DeviceKind get kind => $_getN(2);
  @$pb.TagNumber(3)
  set kind(DeviceKind value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get outputId => $_getI64(3);
  @$pb.TagNumber(4)
  set outputId($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOutputId() => $_has(3);
  @$pb.TagNumber(4)
  void clearOutputId() => $_clearField(4);

  /// Manual pin choices by requirement index.
  @$pb.TagNumber(5)
  $pb.PbList<DevicePin> get fixedPins => $_getList(4);

  /// The requirements this kind implies, for the editor's pin table.
  @$pb.TagNumber(6)
  $pb.PbList<RequirementLabel> get requirements => $_getList(5);
}

class DevicePin extends $pb.GeneratedMessage {
  factory DevicePin({
    $core.int? index,
    $core.String? resource,
  }) {
    final result = DevicePin._();
    if (index != null) result.index = index;
    if (resource != null) result.resource = resource;
    return result;
  }

  DevicePin._();

  factory DevicePin.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DevicePin()..mergeFromBuffer(data, registry);
  factory DevicePin.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DevicePin()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DevicePin',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DevicePin.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'resource')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DevicePin clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DevicePin copyWith(void Function(DevicePin) updates) =>
      super.copyWith((message) => updates(message as DevicePin)) as DevicePin;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DevicePin() / DevicePin.new instead')
  static DevicePin create() => DevicePin._();
  static $pb.GeneratedMessage $_createMessage() => DevicePin._();
  @$core.override
  DevicePin createEmptyInstance() => DevicePin._();
  @$core.pragma('dart2js:noInline')
  static DevicePin getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DevicePin>(DevicePin.$_createMessage);
  static DevicePin? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get index => $_getIZ(0);
  @$pb.TagNumber(1)
  set index($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearIndex() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get resource => $_getSZ(1);
  @$pb.TagNumber(2)
  set resource($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasResource() => $_has(1);
  @$pb.TagNumber(2)
  void clearResource() => $_clearField(2);
}

class RequirementLabel extends $pb.GeneratedMessage {
  factory RequirementLabel({
    $core.int? index,
    $core.String? capability,
    $core.String? label,
  }) {
    final result = RequirementLabel._();
    if (index != null) result.index = index;
    if (capability != null) result.capability = capability;
    if (label != null) result.label = label;
    return result;
  }

  RequirementLabel._();

  factory RequirementLabel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RequirementLabel()..mergeFromBuffer(data, registry);
  factory RequirementLabel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RequirementLabel()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RequirementLabel',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RequirementLabel.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'capability')
    ..aOS(3, _omitFieldNames ? '' : 'label')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RequirementLabel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RequirementLabel copyWith(void Function(RequirementLabel) updates) =>
      super.copyWith((message) => updates(message as RequirementLabel)) as RequirementLabel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RequirementLabel() / RequirementLabel.new instead')
  static RequirementLabel create() => RequirementLabel._();
  static $pb.GeneratedMessage $_createMessage() => RequirementLabel._();
  @$core.override
  RequirementLabel createEmptyInstance() => RequirementLabel._();
  @$core.pragma('dart2js:noInline')
  static RequirementLabel getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RequirementLabel>(RequirementLabel.$_createMessage);
  static RequirementLabel? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get index => $_getIZ(0);
  @$pb.TagNumber(1)
  set index($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearIndex() => $_clearField(1);

  /// Capability name, e.g. "pwm", "digital_out", "i2c_sda".
  @$pb.TagNumber(2)
  $core.String get capability => $_getSZ(1);
  @$pb.TagNumber(2)
  set capability($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCapability() => $_has(1);
  @$pb.TagNumber(2)
  void clearCapability() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get label => $_getSZ(2);
  @$pb.TagNumber(3)
  set label($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLabel() => $_has(2);
  @$pb.TagNumber(3)
  void clearLabel() => $_clearField(3);
}

class ConceptView extends $pb.GeneratedMessage {
  factory ConceptView({
    $fixnum.Int64? id,
    $core.String? name,
    $core.String? description,
    Representation? representation,
  }) {
    final result = ConceptView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (representation != null) result.representation = representation;
    return result;
  }

  ConceptView._();

  factory ConceptView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConceptView()..mergeFromBuffer(data, registry);
  factory ConceptView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConceptView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ConceptView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ConceptView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOM<Representation>(4, _omitFieldNames ? '' : 'representation',
        subBuilder: Representation.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConceptView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConceptView copyWith(void Function(ConceptView) updates) =>
      super.copyWith((message) => updates(message as ConceptView)) as ConceptView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ConceptView() / ConceptView.new instead')
  static ConceptView create() => ConceptView._();
  static $pb.GeneratedMessage $_createMessage() => ConceptView._();
  @$core.override
  ConceptView createEmptyInstance() => ConceptView._();
  @$core.pragma('dart2js:noInline')
  static ConceptView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConceptView>(ConceptView.$_createMessage);
  static ConceptView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  Representation get representation => $_getN(3);
  @$pb.TagNumber(4)
  set representation(Representation value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasRepresentation() => $_has(3);
  @$pb.TagNumber(4)
  void clearRepresentation() => $_clearField(4);
  @$pb.TagNumber(4)
  Representation ensureRepresentation() => $_ensure(3);
}

class MappingView extends $pb.GeneratedMessage {
  factory MappingView({
    $fixnum.Int64? id,
    $core.String? name,
    $core.String? description,
    Signature? signature,
    Definition? definition,
    AcceptanceState? state,
    $fixnum.Int64? clockId,
    $fixnum.Int64? drivesOutputId,
  }) {
    final result = MappingView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (signature != null) result.signature = signature;
    if (definition != null) result.definition = definition;
    if (state != null) result.state = state;
    if (clockId != null) result.clockId = clockId;
    if (drivesOutputId != null) result.drivesOutputId = drivesOutputId;
    return result;
  }

  MappingView._();

  factory MappingView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MappingView()..mergeFromBuffer(data, registry);
  factory MappingView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MappingView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'MappingView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: MappingView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOM<Signature>(4, _omitFieldNames ? '' : 'signature', subBuilder: Signature.$_createMessage)
    ..aOM<Definition>(5, _omitFieldNames ? '' : 'definition',
        subBuilder: Definition.$_createMessage)
    ..aE<AcceptanceState>(6, _omitFieldNames ? '' : 'state', enumValues: AcceptanceState.values)
    ..a<$fixnum.Int64>(7, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(8, _omitFieldNames ? '' : 'drivesOutputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MappingView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MappingView copyWith(void Function(MappingView) updates) =>
      super.copyWith((message) => updates(message as MappingView)) as MappingView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MappingView() / MappingView.new instead')
  static MappingView create() => MappingView._();
  static $pb.GeneratedMessage $_createMessage() => MappingView._();
  @$core.override
  MappingView createEmptyInstance() => MappingView._();
  @$core.pragma('dart2js:noInline')
  static MappingView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MappingView>(MappingView.$_createMessage);
  static MappingView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  Signature get signature => $_getN(3);
  @$pb.TagNumber(4)
  set signature(Signature value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSignature() => $_has(3);
  @$pb.TagNumber(4)
  void clearSignature() => $_clearField(4);
  @$pb.TagNumber(4)
  Signature ensureSignature() => $_ensure(3);

  @$pb.TagNumber(5)
  Definition get definition => $_getN(4);
  @$pb.TagNumber(5)
  set definition(Definition value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasDefinition() => $_has(4);
  @$pb.TagNumber(5)
  void clearDefinition() => $_clearField(5);
  @$pb.TagNumber(5)
  Definition ensureDefinition() => $_ensure(4);

  @$pb.TagNumber(6)
  AcceptanceState get state => $_getN(5);
  @$pb.TagNumber(6)
  set state(AcceptanceState value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasState() => $_has(5);
  @$pb.TagNumber(6)
  void clearState() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get clockId => $_getI64(6);
  @$pb.TagNumber(7)
  set clockId($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasClockId() => $_has(6);
  @$pb.TagNumber(7)
  void clearClockId() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get drivesOutputId => $_getI64(7);
  @$pb.TagNumber(8)
  set drivesOutputId($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasDrivesOutputId() => $_has(7);
  @$pb.TagNumber(8)
  void clearDrivesOutputId() => $_clearField(8);
}

class Signature extends $pb.GeneratedMessage {
  factory Signature({
    $core.Iterable<$fixnum.Int64>? inputs,
    $fixnum.Int64? output,
  }) {
    final result = Signature._();
    if (inputs != null) result.inputs.addAll(inputs);
    if (output != null) result.output = output;
    return result;
  }

  Signature._();

  factory Signature.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Signature()..mergeFromBuffer(data, registry);
  factory Signature.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Signature()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Signature',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Signature.$_createMessage)
    ..p<$fixnum.Int64>(1, _omitFieldNames ? '' : 'inputs', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'output', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Signature clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Signature copyWith(void Function(Signature) updates) =>
      super.copyWith((message) => updates(message as Signature)) as Signature;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Signature() / Signature.new instead')
  static Signature create() => Signature._();
  static $pb.GeneratedMessage $_createMessage() => Signature._();
  @$core.override
  Signature createEmptyInstance() => Signature._();
  @$core.pragma('dart2js:noInline')
  static Signature getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Signature>(Signature.$_createMessage);
  static Signature? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$fixnum.Int64> get inputs => $_getList(0);

  @$pb.TagNumber(2)
  $fixnum.Int64 get output => $_getI64(1);
  @$pb.TagNumber(2)
  set output($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOutput() => $_has(1);
  @$pb.TagNumber(2)
  void clearOutput() => $_clearField(2);
}

enum Definition_Kind { formula, notSet }

class Definition extends $pb.GeneratedMessage {
  factory Definition({
    $core.String? formula,
  }) {
    final result = Definition._();
    if (formula != null) result.formula = formula;
    return result;
  }

  Definition._();

  factory Definition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Definition()..mergeFromBuffer(data, registry);
  factory Definition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Definition()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Definition_Kind> _Definition_KindByTag = {
    1: Definition_Kind.formula,
    0: Definition_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Definition',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Definition.$_createMessage)
    ..oo(0, [1])
    ..aOS(1, _omitFieldNames ? '' : 'formula')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Definition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Definition copyWith(void Function(Definition) updates) =>
      super.copyWith((message) => updates(message as Definition)) as Definition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Definition() / Definition.new instead')
  static Definition create() => Definition._();
  static $pb.GeneratedMessage $_createMessage() => Definition._();
  @$core.override
  Definition createEmptyInstance() => Definition._();
  @$core.pragma('dart2js:noInline')
  static Definition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Definition>(Definition.$_createMessage);
  static Definition? _defaultInstance;

  @$pb.TagNumber(1)
  Definition_Kind whichKind() => _Definition_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get formula => $_getSZ(0);
  @$pb.TagNumber(1)
  set formula($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFormula() => $_has(0);
  @$pb.TagNumber(1)
  void clearFormula() => $_clearField(1);
}

enum Representation_Kind { quantity, boolean, count, notSet }

class Representation extends $pb.GeneratedMessage {
  factory Representation({
    Dim? quantity,
    Unit? boolean,
    Unit? count,
  }) {
    final result = Representation._();
    if (quantity != null) result.quantity = quantity;
    if (boolean != null) result.boolean = boolean;
    if (count != null) result.count = count;
    return result;
  }

  Representation._();

  factory Representation.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Representation()..mergeFromBuffer(data, registry);
  factory Representation.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Representation()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Representation_Kind> _Representation_KindByTag = {
    1: Representation_Kind.quantity,
    2: Representation_Kind.boolean,
    3: Representation_Kind.count,
    0: Representation_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Representation',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Representation.$_createMessage)
    ..oo(0, [1, 2, 3])
    ..aOM<Dim>(1, _omitFieldNames ? '' : 'quantity', subBuilder: Dim.$_createMessage)
    ..aOM<Unit>(2, _omitFieldNames ? '' : 'boolean', subBuilder: Unit.$_createMessage)
    ..aOM<Unit>(3, _omitFieldNames ? '' : 'count', subBuilder: Unit.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Representation clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Representation copyWith(void Function(Representation) updates) =>
      super.copyWith((message) => updates(message as Representation)) as Representation;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Representation() / Representation.new instead')
  static Representation create() => Representation._();
  static $pb.GeneratedMessage $_createMessage() => Representation._();
  @$core.override
  Representation createEmptyInstance() => Representation._();
  @$core.pragma('dart2js:noInline')
  static Representation getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Representation>(Representation.$_createMessage);
  static Representation? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  Representation_Kind whichKind() => _Representation_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Dim get quantity => $_getN(0);
  @$pb.TagNumber(1)
  set quantity(Dim value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasQuantity() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuantity() => $_clearField(1);
  @$pb.TagNumber(1)
  Dim ensureQuantity() => $_ensure(0);

  @$pb.TagNumber(2)
  Unit get boolean => $_getN(1);
  @$pb.TagNumber(2)
  set boolean(Unit value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasBoolean() => $_has(1);
  @$pb.TagNumber(2)
  void clearBoolean() => $_clearField(2);
  @$pb.TagNumber(2)
  Unit ensureBoolean() => $_ensure(1);

  @$pb.TagNumber(3)
  Unit get count => $_getN(2);
  @$pb.TagNumber(3)
  set count(Unit value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearCount() => $_clearField(3);
  @$pb.TagNumber(3)
  Unit ensureCount() => $_ensure(2);
}

class Unit extends $pb.GeneratedMessage {
  factory Unit() => Unit._();

  Unit._();

  factory Unit.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Unit()..mergeFromBuffer(data, registry);
  factory Unit.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Unit()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Unit',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Unit.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Unit clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Unit copyWith(void Function(Unit) updates) =>
      super.copyWith((message) => updates(message as Unit)) as Unit;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Unit() / Unit.new instead')
  static Unit create() => Unit._();
  static $pb.GeneratedMessage $_createMessage() => Unit._();
  @$core.override
  Unit createEmptyInstance() => Unit._();
  @$core.pragma('dart2js:noInline')
  static Unit getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Unit>(Unit.$_createMessage);
  static Unit? _defaultInstance;
}

class Dim extends $pb.GeneratedMessage {
  factory Dim({
    $core.int? length,
    $core.int? mass,
    $core.int? time,
    $core.int? current,
    $core.int? temperature,
    $core.int? amount,
    $core.int? luminous,
    $core.int? angle,
  }) {
    final result = Dim._();
    if (length != null) result.length = length;
    if (mass != null) result.mass = mass;
    if (time != null) result.time = time;
    if (current != null) result.current = current;
    if (temperature != null) result.temperature = temperature;
    if (amount != null) result.amount = amount;
    if (luminous != null) result.luminous = luminous;
    if (angle != null) result.angle = angle;
    return result;
  }

  Dim._();

  factory Dim.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Dim()..mergeFromBuffer(data, registry);
  factory Dim.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Dim()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Dim',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Dim.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'length', fieldType: $pb.PbFieldType.OS3)
    ..aI(2, _omitFieldNames ? '' : 'mass', fieldType: $pb.PbFieldType.OS3)
    ..aI(3, _omitFieldNames ? '' : 'time', fieldType: $pb.PbFieldType.OS3)
    ..aI(4, _omitFieldNames ? '' : 'current', fieldType: $pb.PbFieldType.OS3)
    ..aI(5, _omitFieldNames ? '' : 'temperature', fieldType: $pb.PbFieldType.OS3)
    ..aI(6, _omitFieldNames ? '' : 'amount', fieldType: $pb.PbFieldType.OS3)
    ..aI(7, _omitFieldNames ? '' : 'luminous', fieldType: $pb.PbFieldType.OS3)
    ..aI(8, _omitFieldNames ? '' : 'angle', fieldType: $pb.PbFieldType.OS3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Dim clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Dim copyWith(void Function(Dim) updates) =>
      super.copyWith((message) => updates(message as Dim)) as Dim;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Dim() / Dim.new instead')
  static Dim create() => Dim._();
  static $pb.GeneratedMessage $_createMessage() => Dim._();
  @$core.override
  Dim createEmptyInstance() => Dim._();
  @$core.pragma('dart2js:noInline')
  static Dim getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Dim>(Dim.$_createMessage);
  static Dim? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get length => $_getIZ(0);
  @$pb.TagNumber(1)
  set length($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLength() => $_has(0);
  @$pb.TagNumber(1)
  void clearLength() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get mass => $_getIZ(1);
  @$pb.TagNumber(2)
  set mass($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMass() => $_has(1);
  @$pb.TagNumber(2)
  void clearMass() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get time => $_getIZ(2);
  @$pb.TagNumber(3)
  set time($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTime() => $_has(2);
  @$pb.TagNumber(3)
  void clearTime() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get current => $_getIZ(3);
  @$pb.TagNumber(4)
  set current($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCurrent() => $_has(3);
  @$pb.TagNumber(4)
  void clearCurrent() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get temperature => $_getIZ(4);
  @$pb.TagNumber(5)
  set temperature($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTemperature() => $_has(4);
  @$pb.TagNumber(5)
  void clearTemperature() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get amount => $_getIZ(5);
  @$pb.TagNumber(6)
  set amount($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAmount() => $_has(5);
  @$pb.TagNumber(6)
  void clearAmount() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get luminous => $_getIZ(6);
  @$pb.TagNumber(7)
  set luminous($core.int value) => $_setSignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasLuminous() => $_has(6);
  @$pb.TagNumber(7)
  void clearLuminous() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get angle => $_getIZ(7);
  @$pb.TagNumber(8)
  set angle($core.int value) => $_setSignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasAngle() => $_has(7);
  @$pb.TagNumber(8)
  void clearAngle() => $_clearField(8);
}

class Layout extends $pb.GeneratedMessage {
  factory Layout({
    $core.Iterable<NodePosition>? concepts,
    $core.Iterable<NodePosition>? mappings,
    $core.Iterable<NodePosition>? outputs,
  }) {
    final result = Layout._();
    if (concepts != null) result.concepts.addAll(concepts);
    if (mappings != null) result.mappings.addAll(mappings);
    if (outputs != null) result.outputs.addAll(outputs);
    return result;
  }

  Layout._();

  factory Layout.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Layout()..mergeFromBuffer(data, registry);
  factory Layout.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Layout()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Layout',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Layout.$_createMessage)
    ..pPM<NodePosition>(1, _omitFieldNames ? '' : 'concepts',
        subBuilder: NodePosition.$_createMessage)
    ..pPM<NodePosition>(2, _omitFieldNames ? '' : 'mappings',
        subBuilder: NodePosition.$_createMessage)
    ..pPM<NodePosition>(3, _omitFieldNames ? '' : 'outputs',
        subBuilder: NodePosition.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Layout clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Layout copyWith(void Function(Layout) updates) =>
      super.copyWith((message) => updates(message as Layout)) as Layout;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Layout() / Layout.new instead')
  static Layout create() => Layout._();
  static $pb.GeneratedMessage $_createMessage() => Layout._();
  @$core.override
  Layout createEmptyInstance() => Layout._();
  @$core.pragma('dart2js:noInline')
  static Layout getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Layout>(Layout.$_createMessage);
  static Layout? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<NodePosition> get concepts => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<NodePosition> get mappings => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<NodePosition> get outputs => $_getList(2);
}

class NodePosition extends $pb.GeneratedMessage {
  factory NodePosition({
    $fixnum.Int64? id,
    $core.double? x,
    $core.double? y,
  }) {
    final result = NodePosition._();
    if (id != null) result.id = id;
    if (x != null) result.x = x;
    if (y != null) result.y = y;
    return result;
  }

  NodePosition._();

  factory NodePosition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      NodePosition()..mergeFromBuffer(data, registry);
  factory NodePosition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      NodePosition()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'NodePosition',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: NodePosition.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aD(2, _omitFieldNames ? '' : 'x')
    ..aD(3, _omitFieldNames ? '' : 'y')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NodePosition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NodePosition copyWith(void Function(NodePosition) updates) =>
      super.copyWith((message) => updates(message as NodePosition)) as NodePosition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use NodePosition() / NodePosition.new instead')
  static NodePosition create() => NodePosition._();
  static $pb.GeneratedMessage $_createMessage() => NodePosition._();
  @$core.override
  NodePosition createEmptyInstance() => NodePosition._();
  @$core.pragma('dart2js:noInline')
  static NodePosition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<NodePosition>(NodePosition.$_createMessage);
  static NodePosition? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get x => $_getN(1);
  @$pb.TagNumber(2)
  set x($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasX() => $_has(1);
  @$pb.TagNumber(2)
  void clearX() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get y => $_getN(2);
  @$pb.TagNumber(3)
  set y($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasY() => $_has(2);
  @$pb.TagNumber(3)
  void clearY() => $_clearField(3);
}

class SetLayoutRequest extends $pb.GeneratedMessage {
  factory SetLayoutRequest({
    Layout? layout,
  }) {
    final result = SetLayoutRequest._();
    if (layout != null) result.layout = layout;
    return result;
  }

  SetLayoutRequest._();

  factory SetLayoutRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetLayoutRequest()..mergeFromBuffer(data, registry);
  factory SetLayoutRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetLayoutRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SetLayoutRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SetLayoutRequest.$_createMessage)
    ..aOM<Layout>(1, _omitFieldNames ? '' : 'layout', subBuilder: Layout.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetLayoutRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetLayoutRequest copyWith(void Function(SetLayoutRequest) updates) =>
      super.copyWith((message) => updates(message as SetLayoutRequest)) as SetLayoutRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SetLayoutRequest() / SetLayoutRequest.new instead')
  static SetLayoutRequest create() => SetLayoutRequest._();
  static $pb.GeneratedMessage $_createMessage() => SetLayoutRequest._();
  @$core.override
  SetLayoutRequest createEmptyInstance() => SetLayoutRequest._();
  @$core.pragma('dart2js:noInline')
  static SetLayoutRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetLayoutRequest>(SetLayoutRequest.$_createMessage);
  static SetLayoutRequest? _defaultInstance;

  @$pb.TagNumber(1)
  Layout get layout => $_getN(0);
  @$pb.TagNumber(1)
  set layout(Layout value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasLayout() => $_has(0);
  @$pb.TagNumber(1)
  void clearLayout() => $_clearField(1);
  @$pb.TagNumber(1)
  Layout ensureLayout() => $_ensure(0);
}

/// Sent to subscribers after every committed revision.  v0.1 sends the full
/// projection; deltas are a planned protocol change (docs/PROTOCOL.md).
class ProjectChanged extends $pb.GeneratedMessage {
  factory ProjectChanged({
    ProjectProjection? project,
    EditOutcome? outcome,
  }) {
    final result = ProjectChanged._();
    if (project != null) result.project = project;
    if (outcome != null) result.outcome = outcome;
    return result;
  }

  ProjectChanged._();

  factory ProjectChanged.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectChanged()..mergeFromBuffer(data, registry);
  factory ProjectChanged.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectChanged()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ProjectChanged',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ProjectChanged.$_createMessage)
    ..aOM<ProjectProjection>(1, _omitFieldNames ? '' : 'project',
        subBuilder: ProjectProjection.$_createMessage)
    ..aOM<EditOutcome>(2, _omitFieldNames ? '' : 'outcome', subBuilder: EditOutcome.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectChanged clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectChanged copyWith(void Function(ProjectChanged) updates) =>
      super.copyWith((message) => updates(message as ProjectChanged)) as ProjectChanged;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ProjectChanged() / ProjectChanged.new instead')
  static ProjectChanged create() => ProjectChanged._();
  static $pb.GeneratedMessage $_createMessage() => ProjectChanged._();
  @$core.override
  ProjectChanged createEmptyInstance() => ProjectChanged._();
  @$core.pragma('dart2js:noInline')
  static ProjectChanged getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProjectChanged>(ProjectChanged.$_createMessage);
  static ProjectChanged? _defaultInstance;

  @$pb.TagNumber(1)
  ProjectProjection get project => $_getN(0);
  @$pb.TagNumber(1)
  set project(ProjectProjection value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProject() => $_has(0);
  @$pb.TagNumber(1)
  void clearProject() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectProjection ensureProject() => $_ensure(0);

  @$pb.TagNumber(2)
  EditOutcome get outcome => $_getN(1);
  @$pb.TagNumber(2)
  set outcome(EditOutcome value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasOutcome() => $_has(1);
  @$pb.TagNumber(2)
  void clearOutcome() => $_clearField(2);
  @$pb.TagNumber(2)
  EditOutcome ensureOutcome() => $_ensure(1);
}

class DaemonLog extends $pb.GeneratedMessage {
  factory DaemonLog({
    $core.String? level,
    $core.String? message,
  }) {
    final result = DaemonLog._();
    if (level != null) result.level = level;
    if (message != null) result.message = message;
    return result;
  }

  DaemonLog._();

  factory DaemonLog.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DaemonLog()..mergeFromBuffer(data, registry);
  factory DaemonLog.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DaemonLog()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DaemonLog',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DaemonLog.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'level')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DaemonLog clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DaemonLog copyWith(void Function(DaemonLog) updates) =>
      super.copyWith((message) => updates(message as DaemonLog)) as DaemonLog;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DaemonLog() / DaemonLog.new instead')
  static DaemonLog create() => DaemonLog._();
  static $pb.GeneratedMessage $_createMessage() => DaemonLog._();
  @$core.override
  DaemonLog createEmptyInstance() => DaemonLog._();
  @$core.pragma('dart2js:noInline')
  static DaemonLog getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DaemonLog>(DaemonLog.$_createMessage);
  static DaemonLog? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get level => $_getSZ(0);
  @$pb.TagNumber(1)
  set level($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLevel() => $_has(0);
  @$pb.TagNumber(1)
  void clearLevel() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);
}

enum Value_Kind { boolean, count, quantity, semantic, none, some, opaque, notSet }

/// A runtime value in the design's own terms.
class Value extends $pb.GeneratedMessage {
  factory Value({
    $core.bool? boolean,
    $fixnum.Int64? count,
    Quantity? quantity,
    SemanticValue? semantic,
    Unit? none,
    Value? some,
    $core.String? opaque,
  }) {
    final result = Value._();
    if (boolean != null) result.boolean = boolean;
    if (count != null) result.count = count;
    if (quantity != null) result.quantity = quantity;
    if (semantic != null) result.semantic = semantic;
    if (none != null) result.none = none;
    if (some != null) result.some = some;
    if (opaque != null) result.opaque = opaque;
    return result;
  }

  Value._();

  factory Value.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Value()..mergeFromBuffer(data, registry);
  factory Value.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Value()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Value_Kind> _Value_KindByTag = {
    1: Value_Kind.boolean,
    2: Value_Kind.count,
    3: Value_Kind.quantity,
    4: Value_Kind.semantic,
    5: Value_Kind.none,
    6: Value_Kind.some,
    7: Value_Kind.opaque,
    0: Value_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Value',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Value.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7])
    ..aOB(1, _omitFieldNames ? '' : 'boolean')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'count', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Quantity>(3, _omitFieldNames ? '' : 'quantity', subBuilder: Quantity.$_createMessage)
    ..aOM<SemanticValue>(4, _omitFieldNames ? '' : 'semantic',
        subBuilder: SemanticValue.$_createMessage)
    ..aOM<Unit>(5, _omitFieldNames ? '' : 'none', subBuilder: Unit.$_createMessage)
    ..aOM<Value>(6, _omitFieldNames ? '' : 'some', subBuilder: Value.$_createMessage)
    ..aOS(7, _omitFieldNames ? '' : 'opaque')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Value clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Value copyWith(void Function(Value) updates) =>
      super.copyWith((message) => updates(message as Value)) as Value;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Value() / Value.new instead')
  static Value create() => Value._();
  static $pb.GeneratedMessage $_createMessage() => Value._();
  @$core.override
  Value createEmptyInstance() => Value._();
  @$core.pragma('dart2js:noInline')
  static Value getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Value>(Value.$_createMessage);
  static Value? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  Value_Kind whichKind() => _Value_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.bool get boolean => $_getBF(0);
  @$pb.TagNumber(1)
  set boolean($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasBoolean() => $_has(0);
  @$pb.TagNumber(1)
  void clearBoolean() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get count => $_getI64(1);
  @$pb.TagNumber(2)
  set count($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearCount() => $_clearField(2);

  @$pb.TagNumber(3)
  Quantity get quantity => $_getN(2);
  @$pb.TagNumber(3)
  set quantity(Quantity value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasQuantity() => $_has(2);
  @$pb.TagNumber(3)
  void clearQuantity() => $_clearField(3);
  @$pb.TagNumber(3)
  Quantity ensureQuantity() => $_ensure(2);

  @$pb.TagNumber(4)
  SemanticValue get semantic => $_getN(3);
  @$pb.TagNumber(4)
  set semantic(SemanticValue value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSemantic() => $_has(3);
  @$pb.TagNumber(4)
  void clearSemantic() => $_clearField(4);
  @$pb.TagNumber(4)
  SemanticValue ensureSemantic() => $_ensure(3);

  @$pb.TagNumber(5)
  Unit get none => $_getN(4);
  @$pb.TagNumber(5)
  set none(Unit value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasNone() => $_has(4);
  @$pb.TagNumber(5)
  void clearNone() => $_clearField(5);
  @$pb.TagNumber(5)
  Unit ensureNone() => $_ensure(4);

  @$pb.TagNumber(6)
  Value get some => $_getN(5);
  @$pb.TagNumber(6)
  set some(Value value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasSome() => $_has(5);
  @$pb.TagNumber(6)
  void clearSome() => $_clearField(6);
  @$pb.TagNumber(6)
  Value ensureSome() => $_ensure(5);

  /// A function or partially applied primitive: shown, never transported.
  @$pb.TagNumber(7)
  $core.String get opaque => $_getSZ(6);
  @$pb.TagNumber(7)
  set opaque($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasOpaque() => $_has(6);
  @$pb.TagNumber(7)
  void clearOpaque() => $_clearField(7);
}

class Quantity extends $pb.GeneratedMessage {
  factory Quantity({
    Dim? dim,
    $core.double? value,
  }) {
    final result = Quantity._();
    if (dim != null) result.dim = dim;
    if (value != null) result.value = value;
    return result;
  }

  Quantity._();

  factory Quantity.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Quantity()..mergeFromBuffer(data, registry);
  factory Quantity.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Quantity()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Quantity',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Quantity.$_createMessage)
    ..aOM<Dim>(1, _omitFieldNames ? '' : 'dim', subBuilder: Dim.$_createMessage)
    ..aD(2, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Quantity clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Quantity copyWith(void Function(Quantity) updates) =>
      super.copyWith((message) => updates(message as Quantity)) as Quantity;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Quantity() / Quantity.new instead')
  static Quantity create() => Quantity._();
  static $pb.GeneratedMessage $_createMessage() => Quantity._();
  @$core.override
  Quantity createEmptyInstance() => Quantity._();
  @$core.pragma('dart2js:noInline')
  static Quantity getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Quantity>(Quantity.$_createMessage);
  static Quantity? _defaultInstance;

  @$pb.TagNumber(1)
  Dim get dim => $_getN(0);
  @$pb.TagNumber(1)
  set dim(Dim value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDim() => $_has(0);
  @$pb.TagNumber(1)
  void clearDim() => $_clearField(1);
  @$pb.TagNumber(1)
  Dim ensureDim() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.double get value => $_getN(1);
  @$pb.TagNumber(2)
  set value($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
}

class SemanticValue extends $pb.GeneratedMessage {
  factory SemanticValue({
    $fixnum.Int64? conceptId,
    Value? repr,
  }) {
    final result = SemanticValue._();
    if (conceptId != null) result.conceptId = conceptId;
    if (repr != null) result.repr = repr;
    return result;
  }

  SemanticValue._();

  factory SemanticValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticValue()..mergeFromBuffer(data, registry);
  factory SemanticValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticValue()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SemanticValue',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SemanticValue.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'conceptId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Value>(2, _omitFieldNames ? '' : 'repr', subBuilder: Value.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticValue copyWith(void Function(SemanticValue) updates) =>
      super.copyWith((message) => updates(message as SemanticValue)) as SemanticValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SemanticValue() / SemanticValue.new instead')
  static SemanticValue create() => SemanticValue._();
  static $pb.GeneratedMessage $_createMessage() => SemanticValue._();
  @$core.override
  SemanticValue createEmptyInstance() => SemanticValue._();
  @$core.pragma('dart2js:noInline')
  static SemanticValue getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SemanticValue>(SemanticValue.$_createMessage);
  static SemanticValue? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get conceptId => $_getI64(0);
  @$pb.TagNumber(1)
  set conceptId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConceptId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConceptId() => $_clearField(1);

  @$pb.TagNumber(2)
  Value get repr => $_getN(1);
  @$pb.TagNumber(2)
  set repr(Value value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRepr() => $_has(1);
  @$pb.TagNumber(2)
  void clearRepr() => $_clearField(2);
  @$pb.TagNumber(2)
  Value ensureRepr() => $_ensure(1);
}

/// One value for an unresolved declaration at one tick.
class SimulationInput extends $pb.GeneratedMessage {
  factory SimulationInput({
    $fixnum.Int64? mappingId,
    $fixnum.Int64? tick,
    Value? value,
  }) {
    final result = SimulationInput._();
    if (mappingId != null) result.mappingId = mappingId;
    if (tick != null) result.tick = tick;
    if (value != null) result.value = value;
    return result;
  }

  SimulationInput._();

  factory SimulationInput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SimulationInput()..mergeFromBuffer(data, registry);
  factory SimulationInput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SimulationInput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SimulationInput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SimulationInput.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'tick', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Value>(3, _omitFieldNames ? '' : 'value', subBuilder: Value.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SimulationInput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SimulationInput copyWith(void Function(SimulationInput) updates) =>
      super.copyWith((message) => updates(message as SimulationInput)) as SimulationInput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SimulationInput() / SimulationInput.new instead')
  static SimulationInput create() => SimulationInput._();
  static $pb.GeneratedMessage $_createMessage() => SimulationInput._();
  @$core.override
  SimulationInput createEmptyInstance() => SimulationInput._();
  @$core.pragma('dart2js:noInline')
  static SimulationInput getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SimulationInput>(SimulationInput.$_createMessage);
  static SimulationInput? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get mappingId => $_getI64(0);
  @$pb.TagNumber(1)
  set mappingId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMappingId() => $_has(0);
  @$pb.TagNumber(1)
  void clearMappingId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get tick => $_getI64(1);
  @$pb.TagNumber(2)
  set tick($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTick() => $_has(1);
  @$pb.TagNumber(2)
  void clearTick() => $_clearField(2);

  @$pb.TagNumber(3)
  Value get value => $_getN(2);
  @$pb.TagNumber(3)
  set value(Value value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);
  @$pb.TagNumber(3)
  Value ensureValue() => $_ensure(2);
}

/// Which domains activate: period 1 = every tick.  Domains not listed never
/// activate; an empty schedule means every domain every tick.
class SchedulePeriod extends $pb.GeneratedMessage {
  factory SchedulePeriod({
    $fixnum.Int64? clockId,
    $fixnum.Int64? period,
  }) {
    final result = SchedulePeriod._();
    if (clockId != null) result.clockId = clockId;
    if (period != null) result.period = period;
    return result;
  }

  SchedulePeriod._();

  factory SchedulePeriod.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SchedulePeriod()..mergeFromBuffer(data, registry);
  factory SchedulePeriod.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SchedulePeriod()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SchedulePeriod',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SchedulePeriod.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'period', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchedulePeriod clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchedulePeriod copyWith(void Function(SchedulePeriod) updates) =>
      super.copyWith((message) => updates(message as SchedulePeriod)) as SchedulePeriod;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SchedulePeriod() / SchedulePeriod.new instead')
  static SchedulePeriod create() => SchedulePeriod._();
  static $pb.GeneratedMessage $_createMessage() => SchedulePeriod._();
  @$core.override
  SchedulePeriod createEmptyInstance() => SchedulePeriod._();
  @$core.pragma('dart2js:noInline')
  static SchedulePeriod getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SchedulePeriod>(SchedulePeriod.$_createMessage);
  static SchedulePeriod? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get clockId => $_getI64(0);
  @$pb.TagNumber(1)
  set clockId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClockId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClockId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get period => $_getI64(1);
  @$pb.TagNumber(2)
  set period($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPeriod() => $_has(1);
  @$pb.TagNumber(2)
  void clearPeriod() => $_clearField(2);
}

/// Starts a run against the *current* revision; refused if the design is
/// not causal.  Inputs may be extended by later StartSimulation calls only by
/// restarting.
class StartSimulationRequest extends $pb.GeneratedMessage {
  factory StartSimulationRequest({
    $core.Iterable<SimulationInput>? inputs,
    $core.Iterable<SchedulePeriod>? schedule,
  }) {
    final result = StartSimulationRequest._();
    if (inputs != null) result.inputs.addAll(inputs);
    if (schedule != null) result.schedule.addAll(schedule);
    return result;
  }

  StartSimulationRequest._();

  factory StartSimulationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartSimulationRequest()..mergeFromBuffer(data, registry);
  factory StartSimulationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartSimulationRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartSimulationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: StartSimulationRequest.$_createMessage)
    ..pPM<SimulationInput>(1, _omitFieldNames ? '' : 'inputs',
        subBuilder: SimulationInput.$_createMessage)
    ..pPM<SchedulePeriod>(2, _omitFieldNames ? '' : 'schedule',
        subBuilder: SchedulePeriod.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSimulationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSimulationRequest copyWith(void Function(StartSimulationRequest) updates) =>
      super.copyWith((message) => updates(message as StartSimulationRequest))
          as StartSimulationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StartSimulationRequest() / StartSimulationRequest.new instead')
  static StartSimulationRequest create() => StartSimulationRequest._();
  static $pb.GeneratedMessage $_createMessage() => StartSimulationRequest._();
  @$core.override
  StartSimulationRequest createEmptyInstance() => StartSimulationRequest._();
  @$core.pragma('dart2js:noInline')
  static StartSimulationRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StartSimulationRequest>(
          StartSimulationRequest.$_createMessage);
  static StartSimulationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SimulationInput> get inputs => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<SchedulePeriod> get schedule => $_getList(1);
}

class StepSimulationRequest extends $pb.GeneratedMessage {
  factory StepSimulationRequest({
    $fixnum.Int64? ticks,
  }) {
    final result = StepSimulationRequest._();
    if (ticks != null) result.ticks = ticks;
    return result;
  }

  StepSimulationRequest._();

  factory StepSimulationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StepSimulationRequest()..mergeFromBuffer(data, registry);
  factory StepSimulationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StepSimulationRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StepSimulationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: StepSimulationRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'ticks', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StepSimulationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StepSimulationRequest copyWith(void Function(StepSimulationRequest) updates) =>
      super.copyWith((message) => updates(message as StepSimulationRequest))
          as StepSimulationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StepSimulationRequest() / StepSimulationRequest.new instead')
  static StepSimulationRequest create() => StepSimulationRequest._();
  static $pb.GeneratedMessage $_createMessage() => StepSimulationRequest._();
  @$core.override
  StepSimulationRequest createEmptyInstance() => StepSimulationRequest._();
  @$core.pragma('dart2js:noInline')
  static StepSimulationRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StepSimulationRequest>(
          StepSimulationRequest.$_createMessage);
  static StepSimulationRequest? _defaultInstance;

  /// Ticks to evaluate; the response carries their samples.
  @$pb.TagNumber(1)
  $fixnum.Int64 get ticks => $_getI64(0);
  @$pb.TagNumber(1)
  set ticks($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTicks() => $_has(0);
  @$pb.TagNumber(1)
  void clearTicks() => $_clearField(1);
}

class ResetSimulationRequest extends $pb.GeneratedMessage {
  factory ResetSimulationRequest() => ResetSimulationRequest._();

  ResetSimulationRequest._();

  factory ResetSimulationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ResetSimulationRequest()..mergeFromBuffer(data, registry);
  factory ResetSimulationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ResetSimulationRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResetSimulationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ResetSimulationRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetSimulationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetSimulationRequest copyWith(void Function(ResetSimulationRequest) updates) =>
      super.copyWith((message) => updates(message as ResetSimulationRequest))
          as ResetSimulationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ResetSimulationRequest() / ResetSimulationRequest.new instead')
  static ResetSimulationRequest create() => ResetSimulationRequest._();
  static $pb.GeneratedMessage $_createMessage() => ResetSimulationRequest._();
  @$core.override
  ResetSimulationRequest createEmptyInstance() => ResetSimulationRequest._();
  @$core.pragma('dart2js:noInline')
  static ResetSimulationRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ResetSimulationRequest>(
          ResetSimulationRequest.$_createMessage);
  static ResetSimulationRequest? _defaultInstance;
}

class SimulationResponse extends $pb.GeneratedMessage {
  factory SimulationResponse({
    $fixnum.Int64? revision,
    $fixnum.Int64? nextTick,
    $core.Iterable<TickSample>? samples,
    Diagnostic? error,
  }) {
    final result = SimulationResponse._();
    if (revision != null) result.revision = revision;
    if (nextTick != null) result.nextTick = nextTick;
    if (samples != null) result.samples.addAll(samples);
    if (error != null) result.error = error;
    return result;
  }

  SimulationResponse._();

  factory SimulationResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SimulationResponse()..mergeFromBuffer(data, registry);
  factory SimulationResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SimulationResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SimulationResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SimulationResponse.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'nextTick', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..pPM<TickSample>(3, _omitFieldNames ? '' : 'samples', subBuilder: TickSample.$_createMessage)
    ..aOM<Diagnostic>(4, _omitFieldNames ? '' : 'error', subBuilder: Diagnostic.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SimulationResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SimulationResponse copyWith(void Function(SimulationResponse) updates) =>
      super.copyWith((message) => updates(message as SimulationResponse)) as SimulationResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SimulationResponse() / SimulationResponse.new instead')
  static SimulationResponse create() => SimulationResponse._();
  static $pb.GeneratedMessage $_createMessage() => SimulationResponse._();
  @$core.override
  SimulationResponse createEmptyInstance() => SimulationResponse._();
  @$core.pragma('dart2js:noInline')
  static SimulationResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SimulationResponse>(SimulationResponse.$_createMessage);
  static SimulationResponse? _defaultInstance;

  /// The project revision the simulation runs against.
  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  /// Next tick to be evaluated.
  @$pb.TagNumber(2)
  $fixnum.Int64 get nextTick => $_getI64(1);
  @$pb.TagNumber(2)
  set nextTick($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNextTick() => $_has(1);
  @$pb.TagNumber(2)
  void clearNextTick() => $_clearField(2);

  /// Samples produced by this request (all ticks so far for Start/Reset).
  @$pb.TagNumber(3)
  $pb.PbList<TickSample> get samples => $_getList(2);

  /// Set when a tick could not be evaluated (missing input, division by zero…).
  @$pb.TagNumber(4)
  Diagnostic get error => $_getN(3);
  @$pb.TagNumber(4)
  set error(Diagnostic value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasError() => $_has(3);
  @$pb.TagNumber(4)
  void clearError() => $_clearField(4);
  @$pb.TagNumber(4)
  Diagnostic ensureError() => $_ensure(3);
}

class TickSample extends $pb.GeneratedMessage {
  factory TickSample({
    $fixnum.Int64? tick,
    $core.Iterable<$fixnum.Int64>? activeClockIds,
    $core.Iterable<DeclarationSample>? values,
  }) {
    final result = TickSample._();
    if (tick != null) result.tick = tick;
    if (activeClockIds != null) result.activeClockIds.addAll(activeClockIds);
    if (values != null) result.values.addAll(values);
    return result;
  }

  TickSample._();

  factory TickSample.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TickSample()..mergeFromBuffer(data, registry);
  factory TickSample.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TickSample()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'TickSample',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: TickSample.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'tick', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..p<$fixnum.Int64>(2, _omitFieldNames ? '' : 'activeClockIds', $pb.PbFieldType.KU6)
    ..pPM<DeclarationSample>(3, _omitFieldNames ? '' : 'values',
        subBuilder: DeclarationSample.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TickSample clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TickSample copyWith(void Function(TickSample) updates) =>
      super.copyWith((message) => updates(message as TickSample)) as TickSample;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use TickSample() / TickSample.new instead')
  static TickSample create() => TickSample._();
  static $pb.GeneratedMessage $_createMessage() => TickSample._();
  @$core.override
  TickSample createEmptyInstance() => TickSample._();
  @$core.pragma('dart2js:noInline')
  static TickSample getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TickSample>(TickSample.$_createMessage);
  static TickSample? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get tick => $_getI64(0);
  @$pb.TagNumber(1)
  set tick($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTick() => $_has(0);
  @$pb.TagNumber(1)
  void clearTick() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$fixnum.Int64> get activeClockIds => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<DeclarationSample> get values => $_getList(2);
}

class DeclarationSample extends $pb.GeneratedMessage {
  factory DeclarationSample({
    $fixnum.Int64? mappingId,
    Value? value,
    $core.String? rendered,
  }) {
    final result = DeclarationSample._();
    if (mappingId != null) result.mappingId = mappingId;
    if (value != null) result.value = value;
    if (rendered != null) result.rendered = rendered;
    return result;
  }

  DeclarationSample._();

  factory DeclarationSample.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclarationSample()..mergeFromBuffer(data, registry);
  factory DeclarationSample.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclarationSample()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeclarationSample',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeclarationSample.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<Value>(2, _omitFieldNames ? '' : 'value', subBuilder: Value.$_createMessage)
    ..aOS(3, _omitFieldNames ? '' : 'rendered')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclarationSample clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclarationSample copyWith(void Function(DeclarationSample) updates) =>
      super.copyWith((message) => updates(message as DeclarationSample)) as DeclarationSample;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeclarationSample() / DeclarationSample.new instead')
  static DeclarationSample create() => DeclarationSample._();
  static $pb.GeneratedMessage $_createMessage() => DeclarationSample._();
  @$core.override
  DeclarationSample createEmptyInstance() => DeclarationSample._();
  @$core.pragma('dart2js:noInline')
  static DeclarationSample getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeclarationSample>(DeclarationSample.$_createMessage);
  static DeclarationSample? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get mappingId => $_getI64(0);
  @$pb.TagNumber(1)
  set mappingId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMappingId() => $_has(0);
  @$pb.TagNumber(1)
  void clearMappingId() => $_clearField(1);

  @$pb.TagNumber(2)
  Value get value => $_getN(1);
  @$pb.TagNumber(2)
  set value(Value value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
  @$pb.TagNumber(2)
  Value ensureValue() => $_ensure(1);

  /// Rendered in the design's terms, e.g. "Brightness(0.5)".
  @$pb.TagNumber(3)
  $core.String get rendered => $_getSZ(2);
  @$pb.TagNumber(3)
  set rendered($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRendered() => $_has(2);
  @$pb.TagNumber(3)
  void clearRendered() => $_clearField(3);
}

class RunAnalysisRequest extends $pb.GeneratedMessage {
  factory RunAnalysisRequest() => RunAnalysisRequest._();

  RunAnalysisRequest._();

  factory RunAnalysisRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RunAnalysisRequest()..mergeFromBuffer(data, registry);
  factory RunAnalysisRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RunAnalysisRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RunAnalysisRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RunAnalysisRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RunAnalysisRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RunAnalysisRequest copyWith(void Function(RunAnalysisRequest) updates) =>
      super.copyWith((message) => updates(message as RunAnalysisRequest)) as RunAnalysisRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RunAnalysisRequest() / RunAnalysisRequest.new instead')
  static RunAnalysisRequest create() => RunAnalysisRequest._();
  static $pb.GeneratedMessage $_createMessage() => RunAnalysisRequest._();
  @$core.override
  RunAnalysisRequest createEmptyInstance() => RunAnalysisRequest._();
  @$core.pragma('dart2js:noInline')
  static RunAnalysisRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RunAnalysisRequest>(RunAnalysisRequest.$_createMessage);
  static RunAnalysisRequest? _defaultInstance;
}

class AnalysisResponse extends $pb.GeneratedMessage {
  factory AnalysisResponse({
    ProjectAnalysis? analysis,
  }) {
    final result = AnalysisResponse._();
    if (analysis != null) result.analysis = analysis;
    return result;
  }

  AnalysisResponse._();

  factory AnalysisResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalysisResponse()..mergeFromBuffer(data, registry);
  factory AnalysisResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalysisResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'AnalysisResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: AnalysisResponse.$_createMessage)
    ..aOM<ProjectAnalysis>(1, _omitFieldNames ? '' : 'analysis',
        subBuilder: ProjectAnalysis.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalysisResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalysisResponse copyWith(void Function(AnalysisResponse) updates) =>
      super.copyWith((message) => updates(message as AnalysisResponse)) as AnalysisResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AnalysisResponse() / AnalysisResponse.new instead')
  static AnalysisResponse create() => AnalysisResponse._();
  static $pb.GeneratedMessage $_createMessage() => AnalysisResponse._();
  @$core.override
  AnalysisResponse createEmptyInstance() => AnalysisResponse._();
  @$core.pragma('dart2js:noInline')
  static AnalysisResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AnalysisResponse>(AnalysisResponse.$_createMessage);
  static AnalysisResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ProjectAnalysis get analysis => $_getN(0);
  @$pb.TagNumber(1)
  set analysis(ProjectAnalysis value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAnalysis() => $_has(0);
  @$pb.TagNumber(1)
  void clearAnalysis() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectAnalysis ensureAnalysis() => $_ensure(0);
}

class AnalysisReady extends $pb.GeneratedMessage {
  factory AnalysisReady({
    ProjectAnalysis? analysis,
  }) {
    final result = AnalysisReady._();
    if (analysis != null) result.analysis = analysis;
    return result;
  }

  AnalysisReady._();

  factory AnalysisReady.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalysisReady()..mergeFromBuffer(data, registry);
  factory AnalysisReady.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalysisReady()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'AnalysisReady',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: AnalysisReady.$_createMessage)
    ..aOM<ProjectAnalysis>(1, _omitFieldNames ? '' : 'analysis',
        subBuilder: ProjectAnalysis.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalysisReady clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalysisReady copyWith(void Function(AnalysisReady) updates) =>
      super.copyWith((message) => updates(message as AnalysisReady)) as AnalysisReady;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AnalysisReady() / AnalysisReady.new instead')
  static AnalysisReady create() => AnalysisReady._();
  static $pb.GeneratedMessage $_createMessage() => AnalysisReady._();
  @$core.override
  AnalysisReady createEmptyInstance() => AnalysisReady._();
  @$core.pragma('dart2js:noInline')
  static AnalysisReady getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AnalysisReady>(AnalysisReady.$_createMessage);
  static AnalysisReady? _defaultInstance;

  @$pb.TagNumber(1)
  ProjectAnalysis get analysis => $_getN(0);
  @$pb.TagNumber(1)
  set analysis(ProjectAnalysis value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAnalysis() => $_has(0);
  @$pb.TagNumber(1)
  void clearAnalysis() => $_clearField(1);
  @$pb.TagNumber(1)
  ProjectAnalysis ensureAnalysis() => $_ensure(0);
}

class ProjectAnalysis extends $pb.GeneratedMessage {
  factory ProjectAnalysis({
    $fixnum.Int64? revision,
    $core.Iterable<MappingAnalysis>? mappings,
    $core.Iterable<Diagnostic>? diagnostics,
    $core.bool? causal,
    $core.bool? clockConsistent,
    $core.Iterable<DeclarationCycle>? cycles,
    $core.Iterable<$fixnum.Int64>? evaluationOrder,
    $core.Iterable<OutputAnalysis>? outputs,
    $core.Iterable<$fixnum.Int64>? openOutputs,
    $core.bool? outputComplete,
  }) {
    final result = ProjectAnalysis._();
    if (revision != null) result.revision = revision;
    if (mappings != null) result.mappings.addAll(mappings);
    if (diagnostics != null) result.diagnostics.addAll(diagnostics);
    if (causal != null) result.causal = causal;
    if (clockConsistent != null) result.clockConsistent = clockConsistent;
    if (cycles != null) result.cycles.addAll(cycles);
    if (evaluationOrder != null) result.evaluationOrder.addAll(evaluationOrder);
    if (outputs != null) result.outputs.addAll(outputs);
    if (openOutputs != null) result.openOutputs.addAll(openOutputs);
    if (outputComplete != null) result.outputComplete = outputComplete;
    return result;
  }

  ProjectAnalysis._();

  factory ProjectAnalysis.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectAnalysis()..mergeFromBuffer(data, registry);
  factory ProjectAnalysis.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectAnalysis()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ProjectAnalysis',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ProjectAnalysis.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..pPM<MappingAnalysis>(2, _omitFieldNames ? '' : 'mappings',
        subBuilder: MappingAnalysis.$_createMessage)
    ..pPM<Diagnostic>(3, _omitFieldNames ? '' : 'diagnostics',
        subBuilder: Diagnostic.$_createMessage)
    ..aOB(4, _omitFieldNames ? '' : 'causal')
    ..aOB(5, _omitFieldNames ? '' : 'clockConsistent')
    ..pPM<DeclarationCycle>(6, _omitFieldNames ? '' : 'cycles',
        subBuilder: DeclarationCycle.$_createMessage)
    ..p<$fixnum.Int64>(7, _omitFieldNames ? '' : 'evaluationOrder', $pb.PbFieldType.KU6)
    ..pPM<OutputAnalysis>(8, _omitFieldNames ? '' : 'outputs',
        subBuilder: OutputAnalysis.$_createMessage)
    ..p<$fixnum.Int64>(9, _omitFieldNames ? '' : 'openOutputs', $pb.PbFieldType.KU6)
    ..aOB(10, _omitFieldNames ? '' : 'outputComplete')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectAnalysis clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectAnalysis copyWith(void Function(ProjectAnalysis) updates) =>
      super.copyWith((message) => updates(message as ProjectAnalysis)) as ProjectAnalysis;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ProjectAnalysis() / ProjectAnalysis.new instead')
  static ProjectAnalysis create() => ProjectAnalysis._();
  static $pb.GeneratedMessage $_createMessage() => ProjectAnalysis._();
  @$core.override
  ProjectAnalysis createEmptyInstance() => ProjectAnalysis._();
  @$core.pragma('dart2js:noInline')
  static ProjectAnalysis getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ProjectAnalysis>(ProjectAnalysis.$_createMessage);
  static ProjectAnalysis? _defaultInstance;

  /// The revision the analysis was computed for.
  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<MappingAnalysis> get mappings => $_getList(1);

  /// Every diagnostic, in the documented stable order (entity, span, code).
  @$pb.TagNumber(3)
  $pb.PbList<Diagnostic> get diagnostics => $_getList(2);

  /// Whole-design verdicts of the reactive passes.
  @$pb.TagNumber(4)
  $core.bool get causal => $_getBF(3);
  @$pb.TagNumber(4)
  set causal($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCausal() => $_has(3);
  @$pb.TagNumber(4)
  void clearCausal() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get clockConsistent => $_getBF(4);
  @$pb.TagNumber(5)
  set clockConsistent($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasClockConsistent() => $_has(4);
  @$pb.TagNumber(5)
  void clearClockConsistent() => $_clearField(5);

  /// Instantaneous cycles, each as the mapping ids involved.
  @$pb.TagNumber(6)
  $pb.PbList<DeclarationCycle> get cycles => $_getList(5);

  /// Evaluation order the causality pass produced (empty when not causal).
  @$pb.TagNumber(7)
  $pb.PbList<$fixnum.Int64> get evaluationOrder => $_getList(6);

  /// Output pass: every sink with a domain and where it stands.
  @$pb.TagNumber(8)
  $pb.PbList<OutputAnalysis> get outputs => $_getList(7);

  /// Surface outputs still without a domain (neither driven nor missing).
  @$pb.TagNumber(9)
  $pb.PbList<$fixnum.Int64> get openOutputs => $_getList(8);

  /// Every drive edge well formed, one driver per sink, every required
  /// sink driven, no output open.
  @$pb.TagNumber(10)
  $core.bool get outputComplete => $_getBF(9);
  @$pb.TagNumber(10)
  set outputComplete($core.bool value) => $_setBool(9, value);
  @$pb.TagNumber(10)
  $core.bool hasOutputComplete() => $_has(9);
  @$pb.TagNumber(10)
  void clearOutputComplete() => $_clearField(10);
}

class OutputAnalysis extends $pb.GeneratedMessage {
  factory OutputAnalysis({
    $fixnum.Int64? id,
    OutputState? state,
    $fixnum.Int64? driver,
    $core.Iterable<$fixnum.Int64>? claimants,
  }) {
    final result = OutputAnalysis._();
    if (id != null) result.id = id;
    if (state != null) result.state = state;
    if (driver != null) result.driver = driver;
    if (claimants != null) result.claimants.addAll(claimants);
    return result;
  }

  OutputAnalysis._();

  factory OutputAnalysis.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OutputAnalysis()..mergeFromBuffer(data, registry);
  factory OutputAnalysis.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      OutputAnalysis()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'OutputAnalysis',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: OutputAnalysis.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<OutputState>(2, _omitFieldNames ? '' : 'state', enumValues: OutputState.values)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'driver', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..p<$fixnum.Int64>(4, _omitFieldNames ? '' : 'claimants', $pb.PbFieldType.KU6)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OutputAnalysis clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OutputAnalysis copyWith(void Function(OutputAnalysis) updates) =>
      super.copyWith((message) => updates(message as OutputAnalysis)) as OutputAnalysis;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use OutputAnalysis() / OutputAnalysis.new instead')
  static OutputAnalysis create() => OutputAnalysis._();
  static $pb.GeneratedMessage $_createMessage() => OutputAnalysis._();
  @$core.override
  OutputAnalysis createEmptyInstance() => OutputAnalysis._();
  @$core.pragma('dart2js:noInline')
  static OutputAnalysis getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OutputAnalysis>(OutputAnalysis.$_createMessage);
  static OutputAnalysis? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  OutputState get state => $_getN(1);
  @$pb.TagNumber(2)
  set state(OutputState value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasState() => $_has(1);
  @$pb.TagNumber(2)
  void clearState() => $_clearField(2);

  /// The single well-formed driver, when driven.
  @$pb.TagNumber(3)
  $fixnum.Int64 get driver => $_getI64(2);
  @$pb.TagNumber(3)
  set driver($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDriver() => $_has(2);
  @$pb.TagNumber(3)
  void clearDriver() => $_clearField(3);

  /// Every mapping that claims the sink (well formed or not).
  @$pb.TagNumber(4)
  $pb.PbList<$fixnum.Int64> get claimants => $_getList(3);
}

class DeclarationCycle extends $pb.GeneratedMessage {
  factory DeclarationCycle({
    $core.Iterable<$fixnum.Int64>? mappingIds,
  }) {
    final result = DeclarationCycle._();
    if (mappingIds != null) result.mappingIds.addAll(mappingIds);
    return result;
  }

  DeclarationCycle._();

  factory DeclarationCycle.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclarationCycle()..mergeFromBuffer(data, registry);
  factory DeclarationCycle.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclarationCycle()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeclarationCycle',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeclarationCycle.$_createMessage)
    ..p<$fixnum.Int64>(1, _omitFieldNames ? '' : 'mappingIds', $pb.PbFieldType.KU6)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclarationCycle clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclarationCycle copyWith(void Function(DeclarationCycle) updates) =>
      super.copyWith((message) => updates(message as DeclarationCycle)) as DeclarationCycle;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeclarationCycle() / DeclarationCycle.new instead')
  static DeclarationCycle create() => DeclarationCycle._();
  static $pb.GeneratedMessage $_createMessage() => DeclarationCycle._();
  @$core.override
  DeclarationCycle createEmptyInstance() => DeclarationCycle._();
  @$core.pragma('dart2js:noInline')
  static DeclarationCycle getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeclarationCycle>(DeclarationCycle.$_createMessage);
  static DeclarationCycle? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$fixnum.Int64> get mappingIds => $_getList(0);
}

class MappingAnalysis extends $pb.GeneratedMessage {
  factory MappingAnalysis({
    $fixnum.Int64? id,
    MappingStatus? status,
    $core.String? interface,
    $core.String? inferredType,
    $core.String? coreExpr,
    $core.Iterable<Diagnostic>? diagnostics,
  }) {
    final result = MappingAnalysis._();
    if (id != null) result.id = id;
    if (status != null) result.status = status;
    if (interface != null) result.interface = interface;
    if (inferredType != null) result.inferredType = inferredType;
    if (coreExpr != null) result.coreExpr = coreExpr;
    if (diagnostics != null) result.diagnostics.addAll(diagnostics);
    return result;
  }

  MappingAnalysis._();

  factory MappingAnalysis.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MappingAnalysis()..mergeFromBuffer(data, registry);
  factory MappingAnalysis.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MappingAnalysis()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'MappingAnalysis',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: MappingAnalysis.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'id', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<MappingStatus>(2, _omitFieldNames ? '' : 'status', enumValues: MappingStatus.values)
    ..aOS(3, _omitFieldNames ? '' : 'interface')
    ..aOS(4, _omitFieldNames ? '' : 'inferredType')
    ..aOS(5, _omitFieldNames ? '' : 'coreExpr')
    ..pPM<Diagnostic>(6, _omitFieldNames ? '' : 'diagnostics',
        subBuilder: Diagnostic.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MappingAnalysis clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MappingAnalysis copyWith(void Function(MappingAnalysis) updates) =>
      super.copyWith((message) => updates(message as MappingAnalysis)) as MappingAnalysis;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MappingAnalysis() / MappingAnalysis.new instead')
  static MappingAnalysis create() => MappingAnalysis._();
  static $pb.GeneratedMessage $_createMessage() => MappingAnalysis._();
  @$core.override
  MappingAnalysis createEmptyInstance() => MappingAnalysis._();
  @$core.pragma('dart2js:noInline')
  static MappingAnalysis getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MappingAnalysis>(MappingAnalysis.$_createMessage);
  static MappingAnalysis? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get id => $_getI64(0);
  @$pb.TagNumber(1)
  set id($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  MappingStatus get status => $_getN(1);
  @$pb.TagNumber(2)
  set status(MappingStatus value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  /// Kernel notation, for the explanation view only.
  @$pb.TagNumber(3)
  $core.String get interface => $_getSZ(2);
  @$pb.TagNumber(3)
  set interface($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInterface() => $_has(2);
  @$pb.TagNumber(3)
  void clearInterface() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get inferredType => $_getSZ(3);
  @$pb.TagNumber(4)
  set inferredType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasInferredType() => $_has(3);
  @$pb.TagNumber(4)
  void clearInferredType() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get coreExpr => $_getSZ(4);
  @$pb.TagNumber(5)
  set coreExpr($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCoreExpr() => $_has(4);
  @$pb.TagNumber(5)
  void clearCoreExpr() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<Diagnostic> get diagnostics => $_getList(5);
}

class SourceSpan extends $pb.GeneratedMessage {
  factory SourceSpan({
    $core.int? start,
    $core.int? end,
  }) {
    final result = SourceSpan._();
    if (start != null) result.start = start;
    if (end != null) result.end = end;
    return result;
  }

  SourceSpan._();

  factory SourceSpan.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SourceSpan()..mergeFromBuffer(data, registry);
  factory SourceSpan.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SourceSpan()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SourceSpan',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SourceSpan.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'start', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'end', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SourceSpan clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SourceSpan copyWith(void Function(SourceSpan) updates) =>
      super.copyWith((message) => updates(message as SourceSpan)) as SourceSpan;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SourceSpan() / SourceSpan.new instead')
  static SourceSpan create() => SourceSpan._();
  static $pb.GeneratedMessage $_createMessage() => SourceSpan._();
  @$core.override
  SourceSpan createEmptyInstance() => SourceSpan._();
  @$core.pragma('dart2js:noInline')
  static SourceSpan getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SourceSpan>(SourceSpan.$_createMessage);
  static SourceSpan? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get start => $_getIZ(0);
  @$pb.TagNumber(1)
  set start($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStart() => $_has(0);
  @$pb.TagNumber(1)
  void clearStart() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get end => $_getIZ(1);
  @$pb.TagNumber(2)
  set end($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnd() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnd() => $_clearField(2);
}

enum Diagnostic_Entity { project, conceptId, mappingId, notSet }

class Diagnostic extends $pb.GeneratedMessage {
  factory Diagnostic({
    $core.String? code,
    DiagnosticSeverity? severity,
    Unit? project,
    $fixnum.Int64? conceptId,
    $fixnum.Int64? mappingId,
    SourceSpan? span,
    $core.String? message,
    $core.String? explanation,
    $core.String? technical,
    $core.Iterable<$core.String>? fixes,
  }) {
    final result = Diagnostic._();
    if (code != null) result.code = code;
    if (severity != null) result.severity = severity;
    if (project != null) result.project = project;
    if (conceptId != null) result.conceptId = conceptId;
    if (mappingId != null) result.mappingId = mappingId;
    if (span != null) result.span = span;
    if (message != null) result.message = message;
    if (explanation != null) result.explanation = explanation;
    if (technical != null) result.technical = technical;
    if (fixes != null) result.fixes.addAll(fixes);
    return result;
  }

  Diagnostic._();

  factory Diagnostic.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Diagnostic()..mergeFromBuffer(data, registry);
  factory Diagnostic.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Diagnostic()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Diagnostic_Entity> _Diagnostic_EntityByTag = {
    3: Diagnostic_Entity.project,
    4: Diagnostic_Entity.conceptId,
    5: Diagnostic_Entity.mappingId,
    0: Diagnostic_Entity.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Diagnostic',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Diagnostic.$_createMessage)
    ..oo(0, [3, 4, 5])
    ..aOS(1, _omitFieldNames ? '' : 'code')
    ..aE<DiagnosticSeverity>(2, _omitFieldNames ? '' : 'severity',
        enumValues: DiagnosticSeverity.values)
    ..aOM<Unit>(3, _omitFieldNames ? '' : 'project', subBuilder: Unit.$_createMessage)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'conceptId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<SourceSpan>(6, _omitFieldNames ? '' : 'span', subBuilder: SourceSpan.$_createMessage)
    ..aOS(7, _omitFieldNames ? '' : 'message')
    ..aOS(8, _omitFieldNames ? '' : 'explanation')
    ..aOS(9, _omitFieldNames ? '' : 'technical')
    ..pPS(10, _omitFieldNames ? '' : 'fixes')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Diagnostic clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Diagnostic copyWith(void Function(Diagnostic) updates) =>
      super.copyWith((message) => updates(message as Diagnostic)) as Diagnostic;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Diagnostic() / Diagnostic.new instead')
  static Diagnostic create() => Diagnostic._();
  static $pb.GeneratedMessage $_createMessage() => Diagnostic._();
  @$core.override
  Diagnostic createEmptyInstance() => Diagnostic._();
  @$core.pragma('dart2js:noInline')
  static Diagnostic getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Diagnostic>(Diagnostic.$_createMessage);
  static Diagnostic? _defaultInstance;

  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  Diagnostic_Entity whichEntity() => _Diagnostic_EntityByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  void clearEntity() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get code => $_getSZ(0);
  @$pb.TagNumber(1)
  set code($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCode() => $_has(0);
  @$pb.TagNumber(1)
  void clearCode() => $_clearField(1);

  @$pb.TagNumber(2)
  DiagnosticSeverity get severity => $_getN(1);
  @$pb.TagNumber(2)
  set severity(DiagnosticSeverity value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasSeverity() => $_has(1);
  @$pb.TagNumber(2)
  void clearSeverity() => $_clearField(2);

  @$pb.TagNumber(3)
  Unit get project => $_getN(2);
  @$pb.TagNumber(3)
  set project(Unit value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasProject() => $_has(2);
  @$pb.TagNumber(3)
  void clearProject() => $_clearField(3);
  @$pb.TagNumber(3)
  Unit ensureProject() => $_ensure(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get conceptId => $_getI64(3);
  @$pb.TagNumber(4)
  set conceptId($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasConceptId() => $_has(3);
  @$pb.TagNumber(4)
  void clearConceptId() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get mappingId => $_getI64(4);
  @$pb.TagNumber(5)
  set mappingId($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMappingId() => $_has(4);
  @$pb.TagNumber(5)
  void clearMappingId() => $_clearField(5);

  @$pb.TagNumber(6)
  SourceSpan get span => $_getN(5);
  @$pb.TagNumber(6)
  set span(SourceSpan value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasSpan() => $_has(5);
  @$pb.TagNumber(6)
  void clearSpan() => $_clearField(6);
  @$pb.TagNumber(6)
  SourceSpan ensureSpan() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.String get message => $_getSZ(6);
  @$pb.TagNumber(7)
  set message($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasMessage() => $_has(6);
  @$pb.TagNumber(7)
  void clearMessage() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get explanation => $_getSZ(7);
  @$pb.TagNumber(8)
  set explanation($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasExplanation() => $_has(7);
  @$pb.TagNumber(8)
  void clearExplanation() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get technical => $_getSZ(8);
  @$pb.TagNumber(9)
  set technical($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasTechnical() => $_has(8);
  @$pb.TagNumber(9)
  void clearTechnical() => $_clearField(9);

  @$pb.TagNumber(10)
  $pb.PbList<$core.String> get fixes => $_getList(9);
}

/// Analyse `source` as if it were the definition of `mapping_id` at
/// `revision`, without committing anything.  Refused with
/// `draft.stale_revision` if the project has moved on and
/// `draft.unknown_mapping` if the mapping does not exist at that revision.
/// `generation` is a client-chosen tag echoed back unchanged so the client
/// can drop responses that a newer draft has superseded.
class AnalyzeDefinitionDraftRequest extends $pb.GeneratedMessage {
  factory AnalyzeDefinitionDraftRequest({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $fixnum.Int64? generation,
    $core.String? source,
  }) {
    final result = AnalyzeDefinitionDraftRequest._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (generation != null) result.generation = generation;
    if (source != null) result.source = source;
    return result;
  }

  AnalyzeDefinitionDraftRequest._();

  factory AnalyzeDefinitionDraftRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalyzeDefinitionDraftRequest()..mergeFromBuffer(data, registry);
  factory AnalyzeDefinitionDraftRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalyzeDefinitionDraftRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AnalyzeDefinitionDraftRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: AnalyzeDefinitionDraftRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'generation', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, _omitFieldNames ? '' : 'source')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalyzeDefinitionDraftRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalyzeDefinitionDraftRequest copyWith(void Function(AnalyzeDefinitionDraftRequest) updates) =>
      super.copyWith((message) => updates(message as AnalyzeDefinitionDraftRequest))
          as AnalyzeDefinitionDraftRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use AnalyzeDefinitionDraftRequest() / AnalyzeDefinitionDraftRequest.new instead')
  static AnalyzeDefinitionDraftRequest create() => AnalyzeDefinitionDraftRequest._();
  static $pb.GeneratedMessage $_createMessage() => AnalyzeDefinitionDraftRequest._();
  @$core.override
  AnalyzeDefinitionDraftRequest createEmptyInstance() => AnalyzeDefinitionDraftRequest._();
  @$core.pragma('dart2js:noInline')
  static AnalyzeDefinitionDraftRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AnalyzeDefinitionDraftRequest>(
          AnalyzeDefinitionDraftRequest.$_createMessage);
  static AnalyzeDefinitionDraftRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get generation => $_getI64(2);
  @$pb.TagNumber(3)
  set generation($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasGeneration() => $_has(2);
  @$pb.TagNumber(3)
  void clearGeneration() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get source => $_getSZ(3);
  @$pb.TagNumber(4)
  set source($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSource() => $_has(3);
  @$pb.TagNumber(4)
  void clearSource() => $_clearField(4);
}

class DefinitionDraftAnalysis extends $pb.GeneratedMessage {
  factory DefinitionDraftAnalysis({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $fixnum.Int64? generation,
    $core.bool? parseOk,
    MappingAnalysis? analysis,
  }) {
    final result = DefinitionDraftAnalysis._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (generation != null) result.generation = generation;
    if (parseOk != null) result.parseOk = parseOk;
    if (analysis != null) result.analysis = analysis;
    return result;
  }

  DefinitionDraftAnalysis._();

  factory DefinitionDraftAnalysis.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DefinitionDraftAnalysis()..mergeFromBuffer(data, registry);
  factory DefinitionDraftAnalysis.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DefinitionDraftAnalysis()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DefinitionDraftAnalysis',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DefinitionDraftAnalysis.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'generation', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(4, _omitFieldNames ? '' : 'parseOk')
    ..aOM<MappingAnalysis>(5, _omitFieldNames ? '' : 'analysis',
        subBuilder: MappingAnalysis.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DefinitionDraftAnalysis clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DefinitionDraftAnalysis copyWith(void Function(DefinitionDraftAnalysis) updates) =>
      super.copyWith((message) => updates(message as DefinitionDraftAnalysis))
          as DefinitionDraftAnalysis;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DefinitionDraftAnalysis() / DefinitionDraftAnalysis.new instead')
  static DefinitionDraftAnalysis create() => DefinitionDraftAnalysis._();
  static $pb.GeneratedMessage $_createMessage() => DefinitionDraftAnalysis._();
  @$core.override
  DefinitionDraftAnalysis createEmptyInstance() => DefinitionDraftAnalysis._();
  @$core.pragma('dart2js:noInline')
  static DefinitionDraftAnalysis getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DefinitionDraftAnalysis>(
          DefinitionDraftAnalysis.$_createMessage);
  static DefinitionDraftAnalysis? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get generation => $_getI64(2);
  @$pb.TagNumber(3)
  set generation($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasGeneration() => $_has(2);
  @$pb.TagNumber(3)
  void clearGeneration() => $_clearField(3);

  /// False when the source did not parse; the diagnostics say where.
  @$pb.TagNumber(4)
  $core.bool get parseOk => $_getBF(3);
  @$pb.TagNumber(4)
  set parseOk($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasParseOk() => $_has(3);
  @$pb.TagNumber(4)
  void clearParseOk() => $_clearField(4);

  /// The same verdict a committed definition gets; diagnostic spans index
  /// the draft source of the request, not any committed source.
  @$pb.TagNumber(5)
  MappingAnalysis get analysis => $_getN(4);
  @$pb.TagNumber(5)
  set analysis(MappingAnalysis value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasAnalysis() => $_has(4);
  @$pb.TagNumber(5)
  void clearAnalysis() => $_clearField(5);
  @$pb.TagNumber(5)
  MappingAnalysis ensureAnalysis() => $_ensure(4);
}

/// The draft is gone (revert, reload, detach): the mapping's effective
/// definition is the committed one again for every later query, on every
/// surface.  Ack, also when there was no draft.
class DiscardDefinitionDraftRequest extends $pb.GeneratedMessage {
  factory DiscardDefinitionDraftRequest({
    $fixnum.Int64? mappingId,
  }) {
    final result = DiscardDefinitionDraftRequest._();
    if (mappingId != null) result.mappingId = mappingId;
    return result;
  }

  DiscardDefinitionDraftRequest._();

  factory DiscardDefinitionDraftRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DiscardDefinitionDraftRequest()..mergeFromBuffer(data, registry);
  factory DiscardDefinitionDraftRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DiscardDefinitionDraftRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DiscardDefinitionDraftRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DiscardDefinitionDraftRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiscardDefinitionDraftRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiscardDefinitionDraftRequest copyWith(void Function(DiscardDefinitionDraftRequest) updates) =>
      super.copyWith((message) => updates(message as DiscardDefinitionDraftRequest))
          as DiscardDefinitionDraftRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use DiscardDefinitionDraftRequest() / DiscardDefinitionDraftRequest.new instead')
  static DiscardDefinitionDraftRequest create() => DiscardDefinitionDraftRequest._();
  static $pb.GeneratedMessage $_createMessage() => DiscardDefinitionDraftRequest._();
  @$core.override
  DiscardDefinitionDraftRequest createEmptyInstance() => DiscardDefinitionDraftRequest._();
  @$core.pragma('dart2js:noInline')
  static DiscardDefinitionDraftRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DiscardDefinitionDraftRequest>(
          DiscardDefinitionDraftRequest.$_createMessage);
  static DiscardDefinitionDraftRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get mappingId => $_getI64(0);
  @$pb.TagNumber(1)
  set mappingId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMappingId() => $_has(0);
  @$pb.TagNumber(1)
  void clearMappingId() => $_clearField(1);
}

/// Completion inside the draft at a byte `offset` into `source`.  Sets the
/// draft overlay to `source` first (so the candidates are for exactly this
/// text), then asks the IDE service.  Same refusals as
/// AnalyzeDefinitionDraft; an empty list is a normal answer.
class CompleteDefinitionDraftRequest extends $pb.GeneratedMessage {
  factory CompleteDefinitionDraftRequest({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $core.String? source,
    $core.int? offset,
  }) {
    final result = CompleteDefinitionDraftRequest._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (source != null) result.source = source;
    if (offset != null) result.offset = offset;
    return result;
  }

  CompleteDefinitionDraftRequest._();

  factory CompleteDefinitionDraftRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteDefinitionDraftRequest()..mergeFromBuffer(data, registry);
  factory CompleteDefinitionDraftRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteDefinitionDraftRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteDefinitionDraftRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: CompleteDefinitionDraftRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'source')
    ..aI(4, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteDefinitionDraftRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteDefinitionDraftRequest copyWith(void Function(CompleteDefinitionDraftRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteDefinitionDraftRequest))
          as CompleteDefinitionDraftRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CompleteDefinitionDraftRequest() / CompleteDefinitionDraftRequest.new instead')
  static CompleteDefinitionDraftRequest create() => CompleteDefinitionDraftRequest._();
  static $pb.GeneratedMessage $_createMessage() => CompleteDefinitionDraftRequest._();
  @$core.override
  CompleteDefinitionDraftRequest createEmptyInstance() => CompleteDefinitionDraftRequest._();
  @$core.pragma('dart2js:noInline')
  static CompleteDefinitionDraftRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CompleteDefinitionDraftRequest>(
          CompleteDefinitionDraftRequest.$_createMessage);
  static CompleteDefinitionDraftRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get source => $_getSZ(2);
  @$pb.TagNumber(3)
  set source($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get offset => $_getIZ(3);
  @$pb.TagNumber(4)
  set offset($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOffset() => $_has(3);
  @$pb.TagNumber(4)
  void clearOffset() => $_clearField(4);
}

class DraftCompletionResponse extends $pb.GeneratedMessage {
  factory DraftCompletionResponse({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $core.Iterable<DraftCompletionItem>? items,
  }) {
    final result = DraftCompletionResponse._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (items != null) result.items.addAll(items);
    return result;
  }

  DraftCompletionResponse._();

  factory DraftCompletionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftCompletionResponse()..mergeFromBuffer(data, registry);
  factory DraftCompletionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftCompletionResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DraftCompletionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DraftCompletionResponse.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..pPM<DraftCompletionItem>(3, _omitFieldNames ? '' : 'items',
        subBuilder: DraftCompletionItem.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftCompletionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftCompletionResponse copyWith(void Function(DraftCompletionResponse) updates) =>
      super.copyWith((message) => updates(message as DraftCompletionResponse))
          as DraftCompletionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DraftCompletionResponse() / DraftCompletionResponse.new instead')
  static DraftCompletionResponse create() => DraftCompletionResponse._();
  static $pb.GeneratedMessage $_createMessage() => DraftCompletionResponse._();
  @$core.override
  DraftCompletionResponse createEmptyInstance() => DraftCompletionResponse._();
  @$core.pragma('dart2js:noInline')
  static DraftCompletionResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DraftCompletionResponse>(
          DraftCompletionResponse.$_createMessage);
  static DraftCompletionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<DraftCompletionItem> get items => $_getList(2);
}

class DraftCompletionItem extends $pb.GeneratedMessage {
  factory DraftCompletionItem({
    $core.String? label,
    $core.String? kind,
    $core.int? replaceStart,
    $core.int? replaceEnd,
    $core.String? insert,
    $core.String? resultingType,
    $core.String? documentation,
    $core.int? relevance,
  }) {
    final result = DraftCompletionItem._();
    if (label != null) result.label = label;
    if (kind != null) result.kind = kind;
    if (replaceStart != null) result.replaceStart = replaceStart;
    if (replaceEnd != null) result.replaceEnd = replaceEnd;
    if (insert != null) result.insert = insert;
    if (resultingType != null) result.resultingType = resultingType;
    if (documentation != null) result.documentation = documentation;
    if (relevance != null) result.relevance = relevance;
    return result;
  }

  DraftCompletionItem._();

  factory DraftCompletionItem.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftCompletionItem()..mergeFromBuffer(data, registry);
  factory DraftCompletionItem.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftCompletionItem()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DraftCompletionItem',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DraftCompletionItem.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'label')
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..aI(3, _omitFieldNames ? '' : 'replaceStart', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'replaceEnd', fieldType: $pb.PbFieldType.OU3)
    ..aOS(5, _omitFieldNames ? '' : 'insert')
    ..aOS(6, _omitFieldNames ? '' : 'resultingType')
    ..aOS(7, _omitFieldNames ? '' : 'documentation')
    ..aI(8, _omitFieldNames ? '' : 'relevance', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftCompletionItem clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftCompletionItem copyWith(void Function(DraftCompletionItem) updates) =>
      super.copyWith((message) => updates(message as DraftCompletionItem)) as DraftCompletionItem;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DraftCompletionItem() / DraftCompletionItem.new instead')
  static DraftCompletionItem create() => DraftCompletionItem._();
  static $pb.GeneratedMessage $_createMessage() => DraftCompletionItem._();
  @$core.override
  DraftCompletionItem createEmptyInstance() => DraftCompletionItem._();
  @$core.pragma('dart2js:noInline')
  static DraftCompletionItem getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DraftCompletionItem>(DraftCompletionItem.$_createMessage);
  static DraftCompletionItem? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get label => $_getSZ(0);
  @$pb.TagNumber(1)
  set label($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLabel() => $_has(0);
  @$pb.TagNumber(1)
  void clearLabel() => $_clearField(1);

  /// "input", "unit", "keyword", "concept", "representation", "mapping".
  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  /// Replace the byte range [replace_start, replace_end) of the source
  /// with `insert`.
  @$pb.TagNumber(3)
  $core.int get replaceStart => $_getIZ(2);
  @$pb.TagNumber(3)
  set replaceStart($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasReplaceStart() => $_has(2);
  @$pb.TagNumber(3)
  void clearReplaceStart() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get replaceEnd => $_getIZ(3);
  @$pb.TagNumber(4)
  set replaceEnd($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReplaceEnd() => $_has(3);
  @$pb.TagNumber(4)
  void clearReplaceEnd() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get insert => $_getSZ(4);
  @$pb.TagNumber(5)
  set insert($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasInsert() => $_has(4);
  @$pb.TagNumber(5)
  void clearInsert() => $_clearField(5);

  /// Product-language type of the completed expression, when known.
  @$pb.TagNumber(6)
  $core.String get resultingType => $_getSZ(5);
  @$pb.TagNumber(6)
  set resultingType($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasResultingType() => $_has(5);
  @$pb.TagNumber(6)
  void clearResultingType() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get documentation => $_getSZ(6);
  @$pb.TagNumber(7)
  set documentation($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasDocumentation() => $_has(6);
  @$pb.TagNumber(7)
  void clearDocumentation() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get relevance => $_getIZ(7);
  @$pb.TagNumber(8)
  set relevance($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasRelevance() => $_has(7);
  @$pb.TagNumber(8)
  void clearRelevance() => $_clearField(8);
}

/// The concept under a byte `offset` into the draft `source`, as the IDE
/// service explains it.  `found` is false when nothing semantic is there.
class HoverDefinitionDraftRequest extends $pb.GeneratedMessage {
  factory HoverDefinitionDraftRequest({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $core.String? source,
    $core.int? offset,
  }) {
    final result = HoverDefinitionDraftRequest._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (source != null) result.source = source;
    if (offset != null) result.offset = offset;
    return result;
  }

  HoverDefinitionDraftRequest._();

  factory HoverDefinitionDraftRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverDefinitionDraftRequest()..mergeFromBuffer(data, registry);
  factory HoverDefinitionDraftRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverDefinitionDraftRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HoverDefinitionDraftRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: HoverDefinitionDraftRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'source')
    ..aI(4, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverDefinitionDraftRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverDefinitionDraftRequest copyWith(void Function(HoverDefinitionDraftRequest) updates) =>
      super.copyWith((message) => updates(message as HoverDefinitionDraftRequest))
          as HoverDefinitionDraftRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HoverDefinitionDraftRequest() / HoverDefinitionDraftRequest.new instead')
  static HoverDefinitionDraftRequest create() => HoverDefinitionDraftRequest._();
  static $pb.GeneratedMessage $_createMessage() => HoverDefinitionDraftRequest._();
  @$core.override
  HoverDefinitionDraftRequest createEmptyInstance() => HoverDefinitionDraftRequest._();
  @$core.pragma('dart2js:noInline')
  static HoverDefinitionDraftRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<HoverDefinitionDraftRequest>(
          HoverDefinitionDraftRequest.$_createMessage);
  static HoverDefinitionDraftRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get source => $_getSZ(2);
  @$pb.TagNumber(3)
  set source($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get offset => $_getIZ(3);
  @$pb.TagNumber(4)
  set offset($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOffset() => $_has(3);
  @$pb.TagNumber(4)
  void clearOffset() => $_clearField(4);
}

class DraftHoverResponse extends $pb.GeneratedMessage {
  factory DraftHoverResponse({
    $fixnum.Int64? revision,
    $fixnum.Int64? mappingId,
    $core.bool? found,
    SourceSpan? span,
    $fixnum.Int64? conceptId,
    $core.String? title,
    $core.String? representation,
    $core.String? status,
    $core.Iterable<HoverDetail>? details,
    $core.String? explanation,
    EntityRef? entity,
    $core.String? signature,
    $core.bool? open,
  }) {
    final result = DraftHoverResponse._();
    if (revision != null) result.revision = revision;
    if (mappingId != null) result.mappingId = mappingId;
    if (found != null) result.found = found;
    if (span != null) result.span = span;
    if (conceptId != null) result.conceptId = conceptId;
    if (title != null) result.title = title;
    if (representation != null) result.representation = representation;
    if (status != null) result.status = status;
    if (details != null) result.details.addAll(details);
    if (explanation != null) result.explanation = explanation;
    if (entity != null) result.entity = entity;
    if (signature != null) result.signature = signature;
    if (open != null) result.open = open;
    return result;
  }

  DraftHoverResponse._();

  factory DraftHoverResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftHoverResponse()..mergeFromBuffer(data, registry);
  factory DraftHoverResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DraftHoverResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DraftHoverResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DraftHoverResponse.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(3, _omitFieldNames ? '' : 'found')
    ..aOM<SourceSpan>(4, _omitFieldNames ? '' : 'span', subBuilder: SourceSpan.$_createMessage)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'conceptId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(6, _omitFieldNames ? '' : 'title')
    ..aOS(7, _omitFieldNames ? '' : 'representation')
    ..aOS(8, _omitFieldNames ? '' : 'status')
    ..pPM<HoverDetail>(9, _omitFieldNames ? '' : 'details', subBuilder: HoverDetail.$_createMessage)
    ..aOS(10, _omitFieldNames ? '' : 'explanation')
    ..aOM<EntityRef>(11, _omitFieldNames ? '' : 'entity', subBuilder: EntityRef.$_createMessage)
    ..aOS(12, _omitFieldNames ? '' : 'signature')
    ..aOB(13, _omitFieldNames ? '' : 'open')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftHoverResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DraftHoverResponse copyWith(void Function(DraftHoverResponse) updates) =>
      super.copyWith((message) => updates(message as DraftHoverResponse)) as DraftHoverResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DraftHoverResponse() / DraftHoverResponse.new instead')
  static DraftHoverResponse create() => DraftHoverResponse._();
  static $pb.GeneratedMessage $_createMessage() => DraftHoverResponse._();
  @$core.override
  DraftHoverResponse createEmptyInstance() => DraftHoverResponse._();
  @$core.pragma('dart2js:noInline')
  static DraftHoverResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DraftHoverResponse>(DraftHoverResponse.$_createMessage);
  static DraftHoverResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get mappingId => $_getI64(1);
  @$pb.TagNumber(2)
  set mappingId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMappingId() => $_has(1);
  @$pb.TagNumber(2)
  void clearMappingId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get found => $_getBF(2);
  @$pb.TagNumber(3)
  set found($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFound() => $_has(2);
  @$pb.TagNumber(3)
  void clearFound() => $_clearField(3);

  /// The byte range of the name under the cursor.
  @$pb.TagNumber(4)
  SourceSpan get span => $_getN(3);
  @$pb.TagNumber(4)
  set span(SourceSpan value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSpan() => $_has(3);
  @$pb.TagNumber(4)
  void clearSpan() => $_clearField(4);
  @$pb.TagNumber(4)
  SourceSpan ensureSpan() => $_ensure(3);

  @$pb.TagNumber(5)
  $fixnum.Int64 get conceptId => $_getI64(4);
  @$pb.TagNumber(5)
  set conceptId($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasConceptId() => $_has(4);
  @$pb.TagNumber(5)
  void clearConceptId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get title => $_getSZ(5);
  @$pb.TagNumber(6)
  set title($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTitle() => $_has(5);
  @$pb.TagNumber(6)
  void clearTitle() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get representation => $_getSZ(6);
  @$pb.TagNumber(7)
  set representation($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasRepresentation() => $_has(6);
  @$pb.TagNumber(7)
  void clearRepresentation() => $_clearField(7);

  /// "declared", "open", "defined", … (bdl-ide EntityStatus, product words).
  @$pb.TagNumber(8)
  $core.String get status => $_getSZ(7);
  @$pb.TagNumber(8)
  set status($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasStatus() => $_has(7);
  @$pb.TagNumber(8)
  void clearStatus() => $_clearField(8);

  @$pb.TagNumber(9)
  $pb.PbList<HoverDetail> get details => $_getList(8);

  @$pb.TagNumber(10)
  $core.String get explanation => $_getSZ(9);
  @$pb.TagNumber(10)
  set explanation($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasExplanation() => $_has(9);
  @$pb.TagNumber(10)
  void clearExplanation() => $_clearField(10);

  /// Set by HoverEntity for entities other than concepts.
  @$pb.TagNumber(11)
  EntityRef get entity => $_getN(10);
  @$pb.TagNumber(11)
  set entity(EntityRef value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasEntity() => $_has(10);
  @$pb.TagNumber(11)
  void clearEntity() => $_clearField(11);
  @$pb.TagNumber(11)
  EntityRef ensureEntity() => $_ensure(10);

  /// The surface signature (`dimByTilt : Tilt -> Brightness`), when the
  /// entity has one.
  @$pb.TagNumber(12)
  $core.String get signature => $_getSZ(11);
  @$pb.TagNumber(12)
  set signature($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasSignature() => $_has(11);
  @$pb.TagNumber(12)
  void clearSignature() => $_clearField(12);

  /// Whether the status is one of the open (undecided) states rather than
  /// a settled or a wrong one.
  @$pb.TagNumber(13)
  $core.bool get open => $_getBF(12);
  @$pb.TagNumber(13)
  set open($core.bool value) => $_setBool(12, value);
  @$pb.TagNumber(13)
  $core.bool hasOpen() => $_has(12);
  @$pb.TagNumber(13)
  void clearOpen() => $_clearField(13);
}

enum EntityRef_Kind { project, conceptId, mappingId, clockId, outputId, deviceId, notSet }

/// A stable entity of the design, by identity (never by name).
class EntityRef extends $pb.GeneratedMessage {
  factory EntityRef({
    Unit? project,
    $fixnum.Int64? conceptId,
    $fixnum.Int64? mappingId,
    $fixnum.Int64? clockId,
    $fixnum.Int64? outputId,
    $fixnum.Int64? deviceId,
  }) {
    final result = EntityRef._();
    if (project != null) result.project = project;
    if (conceptId != null) result.conceptId = conceptId;
    if (mappingId != null) result.mappingId = mappingId;
    if (clockId != null) result.clockId = clockId;
    if (outputId != null) result.outputId = outputId;
    if (deviceId != null) result.deviceId = deviceId;
    return result;
  }

  EntityRef._();

  factory EntityRef.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EntityRef()..mergeFromBuffer(data, registry);
  factory EntityRef.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EntityRef()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EntityRef_Kind> _EntityRef_KindByTag = {
    1: EntityRef_Kind.project,
    2: EntityRef_Kind.conceptId,
    3: EntityRef_Kind.mappingId,
    4: EntityRef_Kind.clockId,
    5: EntityRef_Kind.outputId,
    6: EntityRef_Kind.deviceId,
    0: EntityRef_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'EntityRef',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: EntityRef.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6])
    ..aOM<Unit>(1, _omitFieldNames ? '' : 'project', subBuilder: Unit.$_createMessage)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'conceptId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'mappingId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'clockId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'outputId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(6, _omitFieldNames ? '' : 'deviceId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntityRef clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntityRef copyWith(void Function(EntityRef) updates) =>
      super.copyWith((message) => updates(message as EntityRef)) as EntityRef;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EntityRef() / EntityRef.new instead')
  static EntityRef create() => EntityRef._();
  static $pb.GeneratedMessage $_createMessage() => EntityRef._();
  @$core.override
  EntityRef createEmptyInstance() => EntityRef._();
  @$core.pragma('dart2js:noInline')
  static EntityRef getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EntityRef>(EntityRef.$_createMessage);
  static EntityRef? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  EntityRef_Kind whichKind() => _EntityRef_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Unit get project => $_getN(0);
  @$pb.TagNumber(1)
  set project(Unit value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasProject() => $_has(0);
  @$pb.TagNumber(1)
  void clearProject() => $_clearField(1);
  @$pb.TagNumber(1)
  Unit ensureProject() => $_ensure(0);

  @$pb.TagNumber(2)
  $fixnum.Int64 get conceptId => $_getI64(1);
  @$pb.TagNumber(2)
  set conceptId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasConceptId() => $_has(1);
  @$pb.TagNumber(2)
  void clearConceptId() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get mappingId => $_getI64(2);
  @$pb.TagNumber(3)
  set mappingId($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMappingId() => $_has(2);
  @$pb.TagNumber(3)
  void clearMappingId() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get clockId => $_getI64(3);
  @$pb.TagNumber(4)
  set clockId($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasClockId() => $_has(3);
  @$pb.TagNumber(4)
  void clearClockId() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get outputId => $_getI64(4);
  @$pb.TagNumber(5)
  set outputId($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOutputId() => $_has(4);
  @$pb.TagNumber(5)
  void clearOutputId() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get deviceId => $_getI64(5);
  @$pb.TagNumber(6)
  set deviceId($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDeviceId() => $_has(5);
  @$pb.TagNumber(6)
  void clearDeviceId() => $_clearField(6);
}

/// The everyday hover card for any entity (a canvas node, a library row):
/// the same DraftHoverResponse as a formula hover, with `entity` set.
/// `found` is false when the entity does not exist at `revision`.
class HoverEntityRequest extends $pb.GeneratedMessage {
  factory HoverEntityRequest({
    $fixnum.Int64? revision,
    EntityRef? entity,
  }) {
    final result = HoverEntityRequest._();
    if (revision != null) result.revision = revision;
    if (entity != null) result.entity = entity;
    return result;
  }

  HoverEntityRequest._();

  factory HoverEntityRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverEntityRequest()..mergeFromBuffer(data, registry);
  factory HoverEntityRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverEntityRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'HoverEntityRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: HoverEntityRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<EntityRef>(2, _omitFieldNames ? '' : 'entity', subBuilder: EntityRef.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverEntityRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverEntityRequest copyWith(void Function(HoverEntityRequest) updates) =>
      super.copyWith((message) => updates(message as HoverEntityRequest)) as HoverEntityRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HoverEntityRequest() / HoverEntityRequest.new instead')
  static HoverEntityRequest create() => HoverEntityRequest._();
  static $pb.GeneratedMessage $_createMessage() => HoverEntityRequest._();
  @$core.override
  HoverEntityRequest createEmptyInstance() => HoverEntityRequest._();
  @$core.pragma('dart2js:noInline')
  static HoverEntityRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HoverEntityRequest>(HoverEntityRequest.$_createMessage);
  static HoverEntityRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  EntityRef get entity => $_getN(1);
  @$pb.TagNumber(2)
  set entity(EntityRef value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEntity() => $_has(1);
  @$pb.TagNumber(2)
  void clearEntity() => $_clearField(2);
  @$pb.TagNumber(2)
  EntityRef ensureEntity() => $_ensure(1);
}

/// The actions bdl-ide offers for an entity: the fixes for its diagnostics
/// plus its context actions.  Read-only; applying one is an ordinary
/// ApplyEdit of the plan's edits, sent by the client against the revision
/// it holds.
class ListSemanticActionsRequest extends $pb.GeneratedMessage {
  factory ListSemanticActionsRequest({
    $fixnum.Int64? revision,
    EntityRef? entity,
  }) {
    final result = ListSemanticActionsRequest._();
    if (revision != null) result.revision = revision;
    if (entity != null) result.entity = entity;
    return result;
  }

  ListSemanticActionsRequest._();

  factory ListSemanticActionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListSemanticActionsRequest()..mergeFromBuffer(data, registry);
  factory ListSemanticActionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListSemanticActionsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListSemanticActionsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ListSemanticActionsRequest.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<EntityRef>(2, _omitFieldNames ? '' : 'entity', subBuilder: EntityRef.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSemanticActionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSemanticActionsRequest copyWith(void Function(ListSemanticActionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListSemanticActionsRequest))
          as ListSemanticActionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListSemanticActionsRequest() / ListSemanticActionsRequest.new instead')
  static ListSemanticActionsRequest create() => ListSemanticActionsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListSemanticActionsRequest._();
  @$core.override
  ListSemanticActionsRequest createEmptyInstance() => ListSemanticActionsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListSemanticActionsRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListSemanticActionsRequest>(
          ListSemanticActionsRequest.$_createMessage);
  static ListSemanticActionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  EntityRef get entity => $_getN(1);
  @$pb.TagNumber(2)
  set entity(EntityRef value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEntity() => $_has(1);
  @$pb.TagNumber(2)
  void clearEntity() => $_clearField(2);
  @$pb.TagNumber(2)
  EntityRef ensureEntity() => $_ensure(1);
}

class SemanticActionsResponse extends $pb.GeneratedMessage {
  factory SemanticActionsResponse({
    $fixnum.Int64? revision,
    EntityRef? entity,
    $core.Iterable<SemanticActionView>? actions,
  }) {
    final result = SemanticActionsResponse._();
    if (revision != null) result.revision = revision;
    if (entity != null) result.entity = entity;
    if (actions != null) result.actions.addAll(actions);
    return result;
  }

  SemanticActionsResponse._();

  factory SemanticActionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticActionsResponse()..mergeFromBuffer(data, registry);
  factory SemanticActionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticActionsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SemanticActionsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SemanticActionsResponse.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<EntityRef>(2, _omitFieldNames ? '' : 'entity', subBuilder: EntityRef.$_createMessage)
    ..pPM<SemanticActionView>(3, _omitFieldNames ? '' : 'actions',
        subBuilder: SemanticActionView.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticActionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticActionsResponse copyWith(void Function(SemanticActionsResponse) updates) =>
      super.copyWith((message) => updates(message as SemanticActionsResponse))
          as SemanticActionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SemanticActionsResponse() / SemanticActionsResponse.new instead')
  static SemanticActionsResponse create() => SemanticActionsResponse._();
  static $pb.GeneratedMessage $_createMessage() => SemanticActionsResponse._();
  @$core.override
  SemanticActionsResponse createEmptyInstance() => SemanticActionsResponse._();
  @$core.pragma('dart2js:noInline')
  static SemanticActionsResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SemanticActionsResponse>(
          SemanticActionsResponse.$_createMessage);
  static SemanticActionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  EntityRef get entity => $_getN(1);
  @$pb.TagNumber(2)
  set entity(EntityRef value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEntity() => $_has(1);
  @$pb.TagNumber(2)
  void clearEntity() => $_clearField(2);
  @$pb.TagNumber(2)
  EntityRef ensureEntity() => $_ensure(1);

  @$pb.TagNumber(3)
  $pb.PbList<SemanticActionView> get actions => $_getList(2);
}

class SemanticActionView extends $pb.GeneratedMessage {
  factory SemanticActionView({
    $core.String? id,
    $core.String? title,
    $core.String? kind,
    ActionApplicability? applicability,
    $core.String? reason,
    $core.Iterable<ActionChoiceView>? options,
    $core.String? explanation,
    $core.Iterable<EditOp>? edits,
    $core.Iterable<$core.String>? addresses,
    $core.String? invalidation,
  }) {
    final result = SemanticActionView._();
    if (id != null) result.id = id;
    if (title != null) result.title = title;
    if (kind != null) result.kind = kind;
    if (applicability != null) result.applicability = applicability;
    if (reason != null) result.reason = reason;
    if (options != null) result.options.addAll(options);
    if (explanation != null) result.explanation = explanation;
    if (edits != null) result.edits.addAll(edits);
    if (addresses != null) result.addresses.addAll(addresses);
    if (invalidation != null) result.invalidation = invalidation;
    return result;
  }

  SemanticActionView._();

  factory SemanticActionView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticActionView()..mergeFromBuffer(data, registry);
  factory SemanticActionView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SemanticActionView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SemanticActionView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: SemanticActionView.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'title')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..aE<ActionApplicability>(4, _omitFieldNames ? '' : 'applicability',
        enumValues: ActionApplicability.values)
    ..aOS(5, _omitFieldNames ? '' : 'reason')
    ..pPM<ActionChoiceView>(6, _omitFieldNames ? '' : 'options',
        subBuilder: ActionChoiceView.$_createMessage)
    ..aOS(7, _omitFieldNames ? '' : 'explanation')
    ..pPM<EditOp>(8, _omitFieldNames ? '' : 'edits', subBuilder: EditOp.$_createMessage)
    ..pPS(9, _omitFieldNames ? '' : 'addresses')
    ..aOS(10, _omitFieldNames ? '' : 'invalidation')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticActionView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SemanticActionView copyWith(void Function(SemanticActionView) updates) =>
      super.copyWith((message) => updates(message as SemanticActionView)) as SemanticActionView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SemanticActionView() / SemanticActionView.new instead')
  static SemanticActionView create() => SemanticActionView._();
  static $pb.GeneratedMessage $_createMessage() => SemanticActionView._();
  @$core.override
  SemanticActionView createEmptyInstance() => SemanticActionView._();
  @$core.pragma('dart2js:noInline')
  static SemanticActionView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SemanticActionView>(SemanticActionView.$_createMessage);
  static SemanticActionView? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get title => $_getSZ(1);
  @$pb.TagNumber(2)
  set title($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTitle() => $_has(1);
  @$pb.TagNumber(2)
  void clearTitle() => $_clearField(2);

  /// "quick_fix" or "refactor".
  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  @$pb.TagNumber(4)
  ActionApplicability get applicability => $_getN(3);
  @$pb.TagNumber(4)
  set applicability(ActionApplicability value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasApplicability() => $_has(3);
  @$pb.TagNumber(4)
  void clearApplicability() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get reason => $_getSZ(4);
  @$pb.TagNumber(5)
  set reason($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasReason() => $_has(4);
  @$pb.TagNumber(5)
  void clearReason() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<ActionChoiceView> get options => $_getList(5);

  @$pb.TagNumber(7)
  $core.String get explanation => $_getSZ(6);
  @$pb.TagNumber(7)
  set explanation($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasExplanation() => $_has(6);
  @$pb.TagNumber(7)
  void clearExplanation() => $_clearField(7);

  /// The model edits of a ready plan, in order.  Text/draft edits of a
  /// plan are not carried here (the LSP applies those).
  @$pb.TagNumber(8)
  $pb.PbList<EditOp> get edits => $_getList(7);

  /// Diagnostic codes the action addresses.
  @$pb.TagNumber(9)
  $pb.PbList<$core.String> get addresses => $_getList(8);

  /// Product-language summary of what the plan would reopen.
  @$pb.TagNumber(10)
  $core.String get invalidation => $_getSZ(9);
  @$pb.TagNumber(10)
  set invalidation($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasInvalidation() => $_has(9);
  @$pb.TagNumber(10)
  void clearInvalidation() => $_clearField(10);
}

class ActionChoiceView extends $pb.GeneratedMessage {
  factory ActionChoiceView({
    $core.String? label,
    EditOp? edit,
  }) {
    final result = ActionChoiceView._();
    if (label != null) result.label = label;
    if (edit != null) result.edit = edit;
    return result;
  }

  ActionChoiceView._();

  factory ActionChoiceView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ActionChoiceView()..mergeFromBuffer(data, registry);
  factory ActionChoiceView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ActionChoiceView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ActionChoiceView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ActionChoiceView.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'label')
    ..aOM<EditOp>(2, _omitFieldNames ? '' : 'edit', subBuilder: EditOp.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActionChoiceView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActionChoiceView copyWith(void Function(ActionChoiceView) updates) =>
      super.copyWith((message) => updates(message as ActionChoiceView)) as ActionChoiceView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ActionChoiceView() / ActionChoiceView.new instead')
  static ActionChoiceView create() => ActionChoiceView._();
  static $pb.GeneratedMessage $_createMessage() => ActionChoiceView._();
  @$core.override
  ActionChoiceView createEmptyInstance() => ActionChoiceView._();
  @$core.pragma('dart2js:noInline')
  static ActionChoiceView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ActionChoiceView>(ActionChoiceView.$_createMessage);
  static ActionChoiceView? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get label => $_getSZ(0);
  @$pb.TagNumber(1)
  set label($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLabel() => $_has(0);
  @$pb.TagNumber(1)
  void clearLabel() => $_clearField(1);

  @$pb.TagNumber(2)
  EditOp get edit => $_getN(1);
  @$pb.TagNumber(2)
  set edit(EditOp value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEdit() => $_has(1);
  @$pb.TagNumber(2)
  void clearEdit() => $_clearField(2);
  @$pb.TagNumber(2)
  EditOp ensureEdit() => $_ensure(1);
}

class HoverDetail extends $pb.GeneratedMessage {
  factory HoverDetail({
    $core.String? label,
    $core.String? value,
  }) {
    final result = HoverDetail._();
    if (label != null) result.label = label;
    if (value != null) result.value = value;
    return result;
  }

  HoverDetail._();

  factory HoverDetail.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverDetail()..mergeFromBuffer(data, registry);
  factory HoverDetail.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HoverDetail()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'HoverDetail',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: HoverDetail.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'label')
    ..aOS(2, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverDetail clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HoverDetail copyWith(void Function(HoverDetail) updates) =>
      super.copyWith((message) => updates(message as HoverDetail)) as HoverDetail;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HoverDetail() / HoverDetail.new instead')
  static HoverDetail create() => HoverDetail._();
  static $pb.GeneratedMessage $_createMessage() => HoverDetail._();
  @$core.override
  HoverDetail createEmptyInstance() => HoverDetail._();
  @$core.pragma('dart2js:noInline')
  static HoverDetail getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HoverDetail>(HoverDetail.$_createMessage);
  static HoverDetail? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get label => $_getSZ(0);
  @$pb.TagNumber(1)
  set label($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLabel() => $_has(0);
  @$pb.TagNumber(1)
  void clearLabel() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
}

class ListTargetsRequest extends $pb.GeneratedMessage {
  factory ListTargetsRequest() => ListTargetsRequest._();

  ListTargetsRequest._();

  factory ListTargetsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTargetsRequest()..mergeFromBuffer(data, registry);
  factory ListTargetsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTargetsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ListTargetsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ListTargetsRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTargetsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTargetsRequest copyWith(void Function(ListTargetsRequest) updates) =>
      super.copyWith((message) => updates(message as ListTargetsRequest)) as ListTargetsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListTargetsRequest() / ListTargetsRequest.new instead')
  static ListTargetsRequest create() => ListTargetsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListTargetsRequest._();
  @$core.override
  ListTargetsRequest createEmptyInstance() => ListTargetsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListTargetsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTargetsRequest>(ListTargetsRequest.$_createMessage);
  static ListTargetsRequest? _defaultInstance;
}

class TargetsResponse extends $pb.GeneratedMessage {
  factory TargetsResponse({
    $core.Iterable<TargetView>? targets,
  }) {
    final result = TargetsResponse._();
    if (targets != null) result.targets.addAll(targets);
    return result;
  }

  TargetsResponse._();

  factory TargetsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TargetsResponse()..mergeFromBuffer(data, registry);
  factory TargetsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TargetsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'TargetsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: TargetsResponse.$_createMessage)
    ..pPM<TargetView>(1, _omitFieldNames ? '' : 'targets', subBuilder: TargetView.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TargetsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TargetsResponse copyWith(void Function(TargetsResponse) updates) =>
      super.copyWith((message) => updates(message as TargetsResponse)) as TargetsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use TargetsResponse() / TargetsResponse.new instead')
  static TargetsResponse create() => TargetsResponse._();
  static $pb.GeneratedMessage $_createMessage() => TargetsResponse._();
  @$core.override
  TargetsResponse createEmptyInstance() => TargetsResponse._();
  @$core.pragma('dart2js:noInline')
  static TargetsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TargetsResponse>(TargetsResponse.$_createMessage);
  static TargetsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<TargetView> get targets => $_getList(0);
}

class TargetView extends $pb.GeneratedMessage {
  factory TargetView({
    $core.String? id,
    $core.String? name,
    $core.int? resourceCount,
  }) {
    final result = TargetView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (resourceCount != null) result.resourceCount = resourceCount;
    return result;
  }

  TargetView._();

  factory TargetView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TargetView()..mergeFromBuffer(data, registry);
  factory TargetView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TargetView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'TargetView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: TargetView.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aI(3, _omitFieldNames ? '' : 'resourceCount', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TargetView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TargetView copyWith(void Function(TargetView) updates) =>
      super.copyWith((message) => updates(message as TargetView)) as TargetView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use TargetView() / TargetView.new instead')
  static TargetView create() => TargetView._();
  static $pb.GeneratedMessage $_createMessage() => TargetView._();
  @$core.override
  TargetView createEmptyInstance() => TargetView._();
  @$core.pragma('dart2js:noInline')
  static TargetView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TargetView>(TargetView.$_createMessage);
  static TargetView? _defaultInstance;

  /// Registry id, e.g. "arduino_nano".
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get resourceCount => $_getIZ(2);
  @$pb.TagNumber(3)
  set resourceCount($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResourceCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearResourceCount() => $_clearField(3);
}

class AnalyzeDeploymentRequest extends $pb.GeneratedMessage {
  factory AnalyzeDeploymentRequest({
    $core.String? targetId,
  }) {
    final result = AnalyzeDeploymentRequest._();
    if (targetId != null) result.targetId = targetId;
    return result;
  }

  AnalyzeDeploymentRequest._();

  factory AnalyzeDeploymentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalyzeDeploymentRequest()..mergeFromBuffer(data, registry);
  factory AnalyzeDeploymentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AnalyzeDeploymentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AnalyzeDeploymentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: AnalyzeDeploymentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'targetId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalyzeDeploymentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AnalyzeDeploymentRequest copyWith(void Function(AnalyzeDeploymentRequest) updates) =>
      super.copyWith((message) => updates(message as AnalyzeDeploymentRequest))
          as AnalyzeDeploymentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AnalyzeDeploymentRequest() / AnalyzeDeploymentRequest.new instead')
  static AnalyzeDeploymentRequest create() => AnalyzeDeploymentRequest._();
  static $pb.GeneratedMessage $_createMessage() => AnalyzeDeploymentRequest._();
  @$core.override
  AnalyzeDeploymentRequest createEmptyInstance() => AnalyzeDeploymentRequest._();
  @$core.pragma('dart2js:noInline')
  static AnalyzeDeploymentRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AnalyzeDeploymentRequest>(
          AnalyzeDeploymentRequest.$_createMessage);
  static AnalyzeDeploymentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get targetId => $_getSZ(0);
  @$pb.TagNumber(1)
  set targetId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTargetId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTargetId() => $_clearField(1);
}

class DeploymentResponse extends $pb.GeneratedMessage {
  factory DeploymentResponse({
    DeploymentAnalysis? deployment,
  }) {
    final result = DeploymentResponse._();
    if (deployment != null) result.deployment = deployment;
    return result;
  }

  DeploymentResponse._();

  factory DeploymentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeploymentResponse()..mergeFromBuffer(data, registry);
  factory DeploymentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeploymentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeploymentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeploymentResponse.$_createMessage)
    ..aOM<DeploymentAnalysis>(1, _omitFieldNames ? '' : 'deployment',
        subBuilder: DeploymentAnalysis.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeploymentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeploymentResponse copyWith(void Function(DeploymentResponse) updates) =>
      super.copyWith((message) => updates(message as DeploymentResponse)) as DeploymentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeploymentResponse() / DeploymentResponse.new instead')
  static DeploymentResponse create() => DeploymentResponse._();
  static $pb.GeneratedMessage $_createMessage() => DeploymentResponse._();
  @$core.override
  DeploymentResponse createEmptyInstance() => DeploymentResponse._();
  @$core.pragma('dart2js:noInline')
  static DeploymentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeploymentResponse>(DeploymentResponse.$_createMessage);
  static DeploymentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  DeploymentAnalysis get deployment => $_getN(0);
  @$pb.TagNumber(1)
  set deployment(DeploymentAnalysis value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDeployment() => $_has(0);
  @$pb.TagNumber(1)
  void clearDeployment() => $_clearField(1);
  @$pb.TagNumber(1)
  DeploymentAnalysis ensureDeployment() => $_ensure(0);
}

class DeploymentAnalysis extends $pb.GeneratedMessage {
  factory DeploymentAnalysis({
    $fixnum.Int64? revision,
    $core.String? target,
    DeploymentStatus? status,
    $core.Iterable<RequirementView>? requirements,
    $core.Iterable<Placement>? assignment,
    DeadEnd? deadEnd,
    $core.Iterable<$fixnum.Int64>? unboundDevices,
    $core.Iterable<$fixnum.Int64>? unrealisedOutputs,
    $core.Iterable<Diagnostic>? diagnostics,
  }) {
    final result = DeploymentAnalysis._();
    if (revision != null) result.revision = revision;
    if (target != null) result.target = target;
    if (status != null) result.status = status;
    if (requirements != null) result.requirements.addAll(requirements);
    if (assignment != null) result.assignment.addAll(assignment);
    if (deadEnd != null) result.deadEnd = deadEnd;
    if (unboundDevices != null) result.unboundDevices.addAll(unboundDevices);
    if (unrealisedOutputs != null) result.unrealisedOutputs.addAll(unrealisedOutputs);
    if (diagnostics != null) result.diagnostics.addAll(diagnostics);
    return result;
  }

  DeploymentAnalysis._();

  factory DeploymentAnalysis.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeploymentAnalysis()..mergeFromBuffer(data, registry);
  factory DeploymentAnalysis.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeploymentAnalysis()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeploymentAnalysis',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeploymentAnalysis.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'revision', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'target')
    ..aE<DeploymentStatus>(3, _omitFieldNames ? '' : 'status', enumValues: DeploymentStatus.values)
    ..pPM<RequirementView>(4, _omitFieldNames ? '' : 'requirements',
        subBuilder: RequirementView.$_createMessage)
    ..pPM<Placement>(5, _omitFieldNames ? '' : 'assignment', subBuilder: Placement.$_createMessage)
    ..aOM<DeadEnd>(6, _omitFieldNames ? '' : 'deadEnd', subBuilder: DeadEnd.$_createMessage)
    ..p<$fixnum.Int64>(7, _omitFieldNames ? '' : 'unboundDevices', $pb.PbFieldType.KU6)
    ..p<$fixnum.Int64>(8, _omitFieldNames ? '' : 'unrealisedOutputs', $pb.PbFieldType.KU6)
    ..pPM<Diagnostic>(9, _omitFieldNames ? '' : 'diagnostics',
        subBuilder: Diagnostic.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeploymentAnalysis clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeploymentAnalysis copyWith(void Function(DeploymentAnalysis) updates) =>
      super.copyWith((message) => updates(message as DeploymentAnalysis)) as DeploymentAnalysis;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeploymentAnalysis() / DeploymentAnalysis.new instead')
  static DeploymentAnalysis create() => DeploymentAnalysis._();
  static $pb.GeneratedMessage $_createMessage() => DeploymentAnalysis._();
  @$core.override
  DeploymentAnalysis createEmptyInstance() => DeploymentAnalysis._();
  @$core.pragma('dart2js:noInline')
  static DeploymentAnalysis getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeploymentAnalysis>(DeploymentAnalysis.$_createMessage);
  static DeploymentAnalysis? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get revision => $_getI64(0);
  @$pb.TagNumber(1)
  set revision($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRevision() => $_has(0);
  @$pb.TagNumber(1)
  void clearRevision() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get target => $_getSZ(1);
  @$pb.TagNumber(2)
  set target($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTarget() => $_has(1);
  @$pb.TagNumber(2)
  void clearTarget() => $_clearField(2);

  @$pb.TagNumber(3)
  DeploymentStatus get status => $_getN(2);
  @$pb.TagNumber(3)
  set status(DeploymentStatus value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<RequirementView> get requirements => $_getList(3);

  /// The witness, when feasible: one placement per requirement.
  @$pb.TagNumber(5)
  $pb.PbList<Placement> get assignment => $_getList(4);

  @$pb.TagNumber(6)
  DeadEnd get deadEnd => $_getN(5);
  @$pb.TagNumber(6)
  set deadEnd(DeadEnd value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasDeadEnd() => $_has(5);
  @$pb.TagNumber(6)
  void clearDeadEnd() => $_clearField(6);
  @$pb.TagNumber(6)
  DeadEnd ensureDeadEnd() => $_ensure(5);

  @$pb.TagNumber(7)
  $pb.PbList<$fixnum.Int64> get unboundDevices => $_getList(6);

  @$pb.TagNumber(8)
  $pb.PbList<$fixnum.Int64> get unrealisedOutputs => $_getList(7);

  @$pb.TagNumber(9)
  $pb.PbList<Diagnostic> get diagnostics => $_getList(8);
}

class RequirementView extends $pb.GeneratedMessage {
  factory RequirementView({
    $fixnum.Int64? deviceId,
    $core.int? index,
    $core.String? capability,
    $core.String? fixed,
    $core.String? label,
  }) {
    final result = RequirementView._();
    if (deviceId != null) result.deviceId = deviceId;
    if (index != null) result.index = index;
    if (capability != null) result.capability = capability;
    if (fixed != null) result.fixed = fixed;
    if (label != null) result.label = label;
    return result;
  }

  RequirementView._();

  factory RequirementView.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RequirementView()..mergeFromBuffer(data, registry);
  factory RequirementView.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RequirementView()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RequirementView',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: RequirementView.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'deviceId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(2, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'capability')
    ..aOS(4, _omitFieldNames ? '' : 'fixed')
    ..aOS(5, _omitFieldNames ? '' : 'label')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RequirementView clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RequirementView copyWith(void Function(RequirementView) updates) =>
      super.copyWith((message) => updates(message as RequirementView)) as RequirementView;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RequirementView() / RequirementView.new instead')
  static RequirementView create() => RequirementView._();
  static $pb.GeneratedMessage $_createMessage() => RequirementView._();
  @$core.override
  RequirementView createEmptyInstance() => RequirementView._();
  @$core.pragma('dart2js:noInline')
  static RequirementView getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RequirementView>(RequirementView.$_createMessage);
  static RequirementView? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get deviceId => $_getI64(0);
  @$pb.TagNumber(1)
  set deviceId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDeviceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDeviceId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get index => $_getIZ(1);
  @$pb.TagNumber(2)
  set index($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get capability => $_getSZ(2);
  @$pb.TagNumber(3)
  set capability($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCapability() => $_has(2);
  @$pb.TagNumber(3)
  void clearCapability() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get fixed => $_getSZ(3);
  @$pb.TagNumber(4)
  set fixed($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFixed() => $_has(3);
  @$pb.TagNumber(4)
  void clearFixed() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get label => $_getSZ(4);
  @$pb.TagNumber(5)
  set label($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLabel() => $_has(4);
  @$pb.TagNumber(5)
  void clearLabel() => $_clearField(5);
}

class Placement extends $pb.GeneratedMessage {
  factory Placement({
    $fixnum.Int64? deviceId,
    $core.int? index,
    $core.String? resource,
  }) {
    final result = Placement._();
    if (deviceId != null) result.deviceId = deviceId;
    if (index != null) result.index = index;
    if (resource != null) result.resource = resource;
    return result;
  }

  Placement._();

  factory Placement.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Placement()..mergeFromBuffer(data, registry);
  factory Placement.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Placement()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Placement',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Placement.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'deviceId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(2, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'resource')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Placement clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Placement copyWith(void Function(Placement) updates) =>
      super.copyWith((message) => updates(message as Placement)) as Placement;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Placement() / Placement.new instead')
  static Placement create() => Placement._();
  static $pb.GeneratedMessage $_createMessage() => Placement._();
  @$core.override
  Placement createEmptyInstance() => Placement._();
  @$core.pragma('dart2js:noInline')
  static Placement getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Placement>(Placement.$_createMessage);
  static Placement? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get deviceId => $_getI64(0);
  @$pb.TagNumber(1)
  set deviceId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDeviceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDeviceId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get index => $_getIZ(1);
  @$pb.TagNumber(2)
  set index($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get resource => $_getSZ(2);
  @$pb.TagNumber(3)
  set resource($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResource() => $_has(2);
  @$pb.TagNumber(3)
  void clearResource() => $_clearField(3);
}

enum DeadEnd_Reason { noCapableResource, fixedUnavailable, blocked, notSet }

/// One explanation of infeasibility: the first requirement greedy placement
/// could not place, and what blocked each candidate.  Not a minimal core.
class DeadEnd extends $pb.GeneratedMessage {
  factory DeadEnd({
    $fixnum.Int64? deviceId,
    $core.int? index,
    Unit? noCapableResource,
    $core.String? fixedUnavailable,
    BlockedCandidates? blocked,
    $core.Iterable<Placement>? placed,
  }) {
    final result = DeadEnd._();
    if (deviceId != null) result.deviceId = deviceId;
    if (index != null) result.index = index;
    if (noCapableResource != null) result.noCapableResource = noCapableResource;
    if (fixedUnavailable != null) result.fixedUnavailable = fixedUnavailable;
    if (blocked != null) result.blocked = blocked;
    if (placed != null) result.placed.addAll(placed);
    return result;
  }

  DeadEnd._();

  factory DeadEnd.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeadEnd()..mergeFromBuffer(data, registry);
  factory DeadEnd.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeadEnd()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, DeadEnd_Reason> _DeadEnd_ReasonByTag = {
    3: DeadEnd_Reason.noCapableResource,
    4: DeadEnd_Reason.fixedUnavailable,
    5: DeadEnd_Reason.blocked,
    0: DeadEnd_Reason.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'DeadEnd',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: DeadEnd.$_createMessage)
    ..oo(0, [3, 4, 5])
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'deviceId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(2, _omitFieldNames ? '' : 'index', fieldType: $pb.PbFieldType.OU3)
    ..aOM<Unit>(3, _omitFieldNames ? '' : 'noCapableResource', subBuilder: Unit.$_createMessage)
    ..aOS(4, _omitFieldNames ? '' : 'fixedUnavailable')
    ..aOM<BlockedCandidates>(5, _omitFieldNames ? '' : 'blocked',
        subBuilder: BlockedCandidates.$_createMessage)
    ..pPM<Placement>(6, _omitFieldNames ? '' : 'placed', subBuilder: Placement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeadEnd clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeadEnd copyWith(void Function(DeadEnd) updates) =>
      super.copyWith((message) => updates(message as DeadEnd)) as DeadEnd;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DeadEnd() / DeadEnd.new instead')
  static DeadEnd create() => DeadEnd._();
  static $pb.GeneratedMessage $_createMessage() => DeadEnd._();
  @$core.override
  DeadEnd createEmptyInstance() => DeadEnd._();
  @$core.pragma('dart2js:noInline')
  static DeadEnd getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeadEnd>(DeadEnd.$_createMessage);
  static DeadEnd? _defaultInstance;

  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  DeadEnd_Reason whichReason() => _DeadEnd_ReasonByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  void clearReason() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get deviceId => $_getI64(0);
  @$pb.TagNumber(1)
  set deviceId($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDeviceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDeviceId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get index => $_getIZ(1);
  @$pb.TagNumber(2)
  set index($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  Unit get noCapableResource => $_getN(2);
  @$pb.TagNumber(3)
  set noCapableResource(Unit value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasNoCapableResource() => $_has(2);
  @$pb.TagNumber(3)
  void clearNoCapableResource() => $_clearField(3);
  @$pb.TagNumber(3)
  Unit ensureNoCapableResource() => $_ensure(2);

  @$pb.TagNumber(4)
  $core.String get fixedUnavailable => $_getSZ(3);
  @$pb.TagNumber(4)
  set fixedUnavailable($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFixedUnavailable() => $_has(3);
  @$pb.TagNumber(4)
  void clearFixedUnavailable() => $_clearField(4);

  @$pb.TagNumber(5)
  BlockedCandidates get blocked => $_getN(4);
  @$pb.TagNumber(5)
  set blocked(BlockedCandidates value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasBlocked() => $_has(4);
  @$pb.TagNumber(5)
  void clearBlocked() => $_clearField(5);
  @$pb.TagNumber(5)
  BlockedCandidates ensureBlocked() => $_ensure(4);

  @$pb.TagNumber(6)
  $pb.PbList<Placement> get placed => $_getList(5);
}

class BlockedCandidates extends $pb.GeneratedMessage {
  factory BlockedCandidates({
    $core.Iterable<BlockedCandidate>? candidates,
  }) {
    final result = BlockedCandidates._();
    if (candidates != null) result.candidates.addAll(candidates);
    return result;
  }

  BlockedCandidates._();

  factory BlockedCandidates.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BlockedCandidates()..mergeFromBuffer(data, registry);
  factory BlockedCandidates.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BlockedCandidates()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'BlockedCandidates',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: BlockedCandidates.$_createMessage)
    ..pPM<BlockedCandidate>(1, _omitFieldNames ? '' : 'candidates',
        subBuilder: BlockedCandidate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BlockedCandidates clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BlockedCandidates copyWith(void Function(BlockedCandidates) updates) =>
      super.copyWith((message) => updates(message as BlockedCandidates)) as BlockedCandidates;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use BlockedCandidates() / BlockedCandidates.new instead')
  static BlockedCandidates create() => BlockedCandidates._();
  static $pb.GeneratedMessage $_createMessage() => BlockedCandidates._();
  @$core.override
  BlockedCandidates createEmptyInstance() => BlockedCandidates._();
  @$core.pragma('dart2js:noInline')
  static BlockedCandidates getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BlockedCandidates>(BlockedCandidates.$_createMessage);
  static BlockedCandidates? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<BlockedCandidate> get candidates => $_getList(0);
}

class BlockedCandidate extends $pb.GeneratedMessage {
  factory BlockedCandidate({
    $core.String? resource,
    $fixnum.Int64? heldByDeviceId,
    $core.int? heldByIndex,
  }) {
    final result = BlockedCandidate._();
    if (resource != null) result.resource = resource;
    if (heldByDeviceId != null) result.heldByDeviceId = heldByDeviceId;
    if (heldByIndex != null) result.heldByIndex = heldByIndex;
    return result;
  }

  BlockedCandidate._();

  factory BlockedCandidate.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BlockedCandidate()..mergeFromBuffer(data, registry);
  factory BlockedCandidate.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BlockedCandidate()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'BlockedCandidate',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: BlockedCandidate.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'resource')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'heldByDeviceId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aI(3, _omitFieldNames ? '' : 'heldByIndex', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BlockedCandidate clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BlockedCandidate copyWith(void Function(BlockedCandidate) updates) =>
      super.copyWith((message) => updates(message as BlockedCandidate)) as BlockedCandidate;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use BlockedCandidate() / BlockedCandidate.new instead')
  static BlockedCandidate create() => BlockedCandidate._();
  static $pb.GeneratedMessage $_createMessage() => BlockedCandidate._();
  @$core.override
  BlockedCandidate createEmptyInstance() => BlockedCandidate._();
  @$core.pragma('dart2js:noInline')
  static BlockedCandidate getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BlockedCandidate>(BlockedCandidate.$_createMessage);
  static BlockedCandidate? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get resource => $_getSZ(0);
  @$pb.TagNumber(1)
  set resource($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasResource() => $_has(0);
  @$pb.TagNumber(1)
  void clearResource() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get heldByDeviceId => $_getI64(1);
  @$pb.TagNumber(2)
  set heldByDeviceId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasHeldByDeviceId() => $_has(1);
  @$pb.TagNumber(2)
  void clearHeldByDeviceId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get heldByIndex => $_getIZ(2);
  @$pb.TagNumber(3)
  set heldByIndex($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasHeldByIndex() => $_has(2);
  @$pb.TagNumber(3)
  void clearHeldByIndex() => $_clearField(3);
}

const $core.bool _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
