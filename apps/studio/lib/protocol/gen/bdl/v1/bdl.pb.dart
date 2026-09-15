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
    0: ClientMessage_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'ClientMessage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: ClientMessage.$_createMessage)
    ..oo(0, [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21])
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

enum Response_Payload { error, handshake, project, editApplied, ack, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    $fixnum.Int64? requestId,
    Error? error,
    HandshakeResponse? handshake,
    ProjectResponse? project,
    EditApplied? editApplied,
    Ack? ack,
  }) {
    final result = Response._();
    if (requestId != null) result.requestId = requestId;
    if (error != null) result.error = error;
    if (handshake != null) result.handshake = handshake;
    if (project != null) result.project = project;
    if (editApplied != null) result.editApplied = editApplied;
    if (ack != null) result.ack = ack;
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
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Response.$_createMessage)
    ..oo(0, [2, 10, 11, 12, 13])
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
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(2)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
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
}

enum Event_Payload { projectChanged, log, notSet }

class Event extends $pb.GeneratedMessage {
  factory Event({
    ProjectChanged? projectChanged,
    DaemonLog? log,
  }) {
    final result = Event._();
    if (projectChanged != null) result.projectChanged = projectChanged;
    if (log != null) result.log = log;
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
    0: Event_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'Event',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: Event.$_createMessage)
    ..oo(0, [1, 2])
    ..aOM<ProjectChanged>(1, _omitFieldNames ? '' : 'projectChanged',
        subBuilder: ProjectChanged.$_createMessage)
    ..aOM<DaemonLog>(2, _omitFieldNames ? '' : 'log', subBuilder: DaemonLog.$_createMessage)
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
  Event_Payload whichPayload() => _Event_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
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
    0: EditOp_Op.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'EditOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'bdl.v1'),
      createEmptyInstance: EditOp.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
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

class EditOutcome extends $pb.GeneratedMessage {
  factory EditOutcome({
    EditKind? kind,
    $core.Iterable<Invalidation>? invalidates,
    $core.Iterable<$fixnum.Int64>? originDecls,
    $fixnum.Int64? createdConcept,
    $fixnum.Int64? createdMapping,
  }) {
    final result = EditOutcome._();
    if (kind != null) result.kind = kind;
    if (invalidates != null) result.invalidates.addAll(invalidates);
    if (originDecls != null) result.originDecls.addAll(originDecls);
    if (createdConcept != null) result.createdConcept = createdConcept;
    if (createdMapping != null) result.createdMapping = createdMapping;
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
  }) {
    final result = MappingView._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (signature != null) result.signature = signature;
    if (definition != null) result.definition = definition;
    if (state != null) result.state = state;
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
  }) {
    final result = Layout._();
    if (concepts != null) result.concepts.addAll(concepts);
    if (mappings != null) result.mappings.addAll(mappings);
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

const $core.bool _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
