// This is a generated file - do not edit.
//
// Generated from bdl/v1/bdl.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use editKindDescriptor instead')
const EditKind$json = {
  '1': 'EditKind',
  '2': [
    {'1': 'EDIT_KIND_UNSPECIFIED', '2': 0},
    {'1': 'EDIT_KIND_REFINEMENT', '2': 1},
    {'1': 'EDIT_KIND_EDIT', '2': 2},
  ],
};

/// Descriptor for `EditKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List editKindDescriptor = $convert
    .base64Decode('CghFZGl0S2luZBIZChVFRElUX0tJTkRfVU5TUEVDSUZJRUQQABIYChRFRElUX0tJTkRfUkVGSU'
        '5FTUVOVBABEhIKDkVESVRfS0lORF9FRElUEAI=');

@$core.Deprecated('Use invalidationDescriptor instead')
const Invalidation$json = {
  '1': 'Invalidation',
  '2': [
    {'1': 'INVALIDATION_UNSPECIFIED', '2': 0},
    {'1': 'INVALIDATION_INTERFACE', '2': 1},
    {'1': 'INVALIDATION_REALIZATION', '2': 2},
    {'1': 'INVALIDATION_SEMANTIC', '2': 3},
    {'1': 'INVALIDATION_REACTIVE', '2': 4},
    {'1': 'INVALIDATION_CLOCK', '2': 5},
    {'1': 'INVALIDATION_OUTPUT', '2': 6},
    {'1': 'INVALIDATION_DEPLOYMENT', '2': 7},
  ],
};

/// Descriptor for `Invalidation`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List invalidationDescriptor = $convert
    .base64Decode('CgxJbnZhbGlkYXRpb24SHAoYSU5WQUxJREFUSU9OX1VOU1BFQ0lGSUVEEAASGgoWSU5WQUxJRE'
        'FUSU9OX0lOVEVSRkFDRRABEhwKGElOVkFMSURBVElPTl9SRUFMSVpBVElPThACEhkKFUlOVkFM'
        'SURBVElPTl9TRU1BTlRJQxADEhkKFUlOVkFMSURBVElPTl9SRUFDVElWRRAEEhYKEklOVkFMSU'
        'RBVElPTl9DTE9DSxAFEhcKE0lOVkFMSURBVElPTl9PVVRQVVQQBhIbChdJTlZBTElEQVRJT05f'
        'REVQTE9ZTUVOVBAH');

@$core.Deprecated('Use acceptanceStateDescriptor instead')
const AcceptanceState$json = {
  '1': 'AcceptanceState',
  '2': [
    {'1': 'ACCEPTANCE_STATE_UNSPECIFIED', '2': 0},
    {'1': 'ACCEPTANCE_STATE_DECLARED', '2': 1},
    {'1': 'ACCEPTANCE_STATE_DEFINED', '2': 2},
    {'1': 'ACCEPTANCE_STATE_TYPE_VALID', '2': 3},
    {'1': 'ACCEPTANCE_STATE_TEMPORALLY_VALID', '2': 4},
    {'1': 'ACCEPTANCE_STATE_CLOCK_CONSISTENT', '2': 5},
    {'1': 'ACCEPTANCE_STATE_OUTPUT_COMPLETE', '2': 6},
    {'1': 'ACCEPTANCE_STATE_HARDWARE_FEASIBLE', '2': 7},
  ],
};

/// Descriptor for `AcceptanceState`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List acceptanceStateDescriptor = $convert
    .base64Decode('Cg9BY2NlcHRhbmNlU3RhdGUSIAocQUNDRVBUQU5DRV9TVEFURV9VTlNQRUNJRklFRBAAEh0KGU'
        'FDQ0VQVEFOQ0VfU1RBVEVfREVDTEFSRUQQARIcChhBQ0NFUFRBTkNFX1NUQVRFX0RFRklORUQQ'
        'AhIfChtBQ0NFUFRBTkNFX1NUQVRFX1RZUEVfVkFMSUQQAxIlCiFBQ0NFUFRBTkNFX1NUQVRFX1'
        'RFTVBPUkFMTFlfVkFMSUQQBBIlCiFBQ0NFUFRBTkNFX1NUQVRFX0NMT0NLX0NPTlNJU1RFTlQQ'
        'BRIkCiBBQ0NFUFRBTkNFX1NUQVRFX09VVFBVVF9DT01QTEVURRAGEiYKIkFDQ0VQVEFOQ0VfU1'
        'RBVEVfSEFSRFdBUkVfRkVBU0lCTEUQBw==');

@$core.Deprecated('Use mappingStatusDescriptor instead')
const MappingStatus$json = {
  '1': 'MappingStatus',
  '2': [
    {'1': 'MAPPING_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'MAPPING_STATUS_DECLARED', '2': 1},
    {'1': 'MAPPING_STATUS_OPEN', '2': 2},
    {'1': 'MAPPING_STATUS_INVALID', '2': 3},
    {'1': 'MAPPING_STATUS_TYPE_VALID', '2': 4},
  ],
};

/// Descriptor for `MappingStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List mappingStatusDescriptor = $convert
    .base64Decode('Cg1NYXBwaW5nU3RhdHVzEh4KGk1BUFBJTkdfU1RBVFVTX1VOU1BFQ0lGSUVEEAASGwoXTUFQUE'
        'lOR19TVEFUVVNfREVDTEFSRUQQARIXChNNQVBQSU5HX1NUQVRVU19PUEVOEAISGgoWTUFQUElO'
        'R19TVEFUVVNfSU5WQUxJRBADEh0KGU1BUFBJTkdfU1RBVFVTX1RZUEVfVkFMSUQQBA==');

@$core.Deprecated('Use diagnosticSeverityDescriptor instead')
const DiagnosticSeverity$json = {
  '1': 'DiagnosticSeverity',
  '2': [
    {'1': 'DIAGNOSTIC_SEVERITY_UNSPECIFIED', '2': 0},
    {'1': 'DIAGNOSTIC_SEVERITY_ERROR', '2': 1},
    {'1': 'DIAGNOSTIC_SEVERITY_WARNING', '2': 2},
    {'1': 'DIAGNOSTIC_SEVERITY_INFO', '2': 3},
  ],
};

/// Descriptor for `DiagnosticSeverity`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List diagnosticSeverityDescriptor = $convert
    .base64Decode('ChJEaWFnbm9zdGljU2V2ZXJpdHkSIwofRElBR05PU1RJQ19TRVZFUklUWV9VTlNQRUNJRklFRB'
        'AAEh0KGURJQUdOT1NUSUNfU0VWRVJJVFlfRVJST1IQARIfChtESUFHTk9TVElDX1NFVkVSSVRZ'
        'X1dBUk5JTkcQAhIcChhESUFHTk9TVElDX1NFVkVSSVRZX0lORk8QAw==');

@$core.Deprecated('Use clientMessageDescriptor instead')
const ClientMessage$json = {
  '1': 'ClientMessage',
  '2': [
    {'1': 'request_id', '3': 1, '4': 1, '5': 4, '10': 'requestId'},
    {
      '1': 'handshake',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.HandshakeRequest',
      '9': 0,
      '10': 'handshake'
    },
    {
      '1': 'open_project',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.OpenProjectRequest',
      '9': 0,
      '10': 'openProject'
    },
    {
      '1': 'init_project',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.InitProjectRequest',
      '9': 0,
      '10': 'initProject'
    },
    {
      '1': 'save_project',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SaveProjectRequest',
      '9': 0,
      '10': 'saveProject'
    },
    {
      '1': 'close_project',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CloseProjectRequest',
      '9': 0,
      '10': 'closeProject'
    },
    {
      '1': 'get_project',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.GetProjectRequest',
      '9': 0,
      '10': 'getProject'
    },
    {
      '1': 'apply_edit',
      '3': 16,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ApplyEditRequest',
      '9': 0,
      '10': 'applyEdit'
    },
    {'1': 'undo', '3': 17, '4': 1, '5': 11, '6': '.bdl.v1.UndoRequest', '9': 0, '10': 'undo'},
    {'1': 'redo', '3': 18, '4': 1, '5': 11, '6': '.bdl.v1.RedoRequest', '9': 0, '10': 'redo'},
    {
      '1': 'set_layout',
      '3': 19,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetLayoutRequest',
      '9': 0,
      '10': 'setLayout'
    },
    {
      '1': 'subscribe_project',
      '3': 20,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SubscribeProjectRequest',
      '9': 0,
      '10': 'subscribeProject'
    },
    {
      '1': 'shutdown',
      '3': 21,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ShutdownRequest',
      '9': 0,
      '10': 'shutdown'
    },
    {
      '1': 'run_analysis',
      '3': 22,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RunAnalysisRequest',
      '9': 0,
      '10': 'runAnalysis'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ClientMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientMessageDescriptor = $convert
    .base64Decode('Cg1DbGllbnRNZXNzYWdlEh0KCnJlcXVlc3RfaWQYASABKARSCXJlcXVlc3RJZBI4CgloYW5kc2'
        'hha2UYCiABKAsyGC5iZGwudjEuSGFuZHNoYWtlUmVxdWVzdEgAUgloYW5kc2hha2USPwoMb3Bl'
        'bl9wcm9qZWN0GAsgASgLMhouYmRsLnYxLk9wZW5Qcm9qZWN0UmVxdWVzdEgAUgtvcGVuUHJvam'
        'VjdBI/Cgxpbml0X3Byb2plY3QYDCABKAsyGi5iZGwudjEuSW5pdFByb2plY3RSZXF1ZXN0SABS'
        'C2luaXRQcm9qZWN0Ej8KDHNhdmVfcHJvamVjdBgNIAEoCzIaLmJkbC52MS5TYXZlUHJvamVjdF'
        'JlcXVlc3RIAFILc2F2ZVByb2plY3QSQgoNY2xvc2VfcHJvamVjdBgOIAEoCzIbLmJkbC52MS5D'
        'bG9zZVByb2plY3RSZXF1ZXN0SABSDGNsb3NlUHJvamVjdBI8CgtnZXRfcHJvamVjdBgPIAEoCz'
        'IZLmJkbC52MS5HZXRQcm9qZWN0UmVxdWVzdEgAUgpnZXRQcm9qZWN0EjkKCmFwcGx5X2VkaXQY'
        'ECABKAsyGC5iZGwudjEuQXBwbHlFZGl0UmVxdWVzdEgAUglhcHBseUVkaXQSKQoEdW5kbxgRIA'
        'EoCzITLmJkbC52MS5VbmRvUmVxdWVzdEgAUgR1bmRvEikKBHJlZG8YEiABKAsyEy5iZGwudjEu'
        'UmVkb1JlcXVlc3RIAFIEcmVkbxI5CgpzZXRfbGF5b3V0GBMgASgLMhguYmRsLnYxLlNldExheW'
        '91dFJlcXVlc3RIAFIJc2V0TGF5b3V0Ek4KEXN1YnNjcmliZV9wcm9qZWN0GBQgASgLMh8uYmRs'
        'LnYxLlN1YnNjcmliZVByb2plY3RSZXF1ZXN0SABSEHN1YnNjcmliZVByb2plY3QSNQoIc2h1dG'
        'Rvd24YFSABKAsyFy5iZGwudjEuU2h1dGRvd25SZXF1ZXN0SABSCHNodXRkb3duEj8KDHJ1bl9h'
        'bmFseXNpcxgWIAEoCzIaLmJkbC52MS5SdW5BbmFseXNpc1JlcXVlc3RIAFILcnVuQW5hbHlzaX'
        'NCCQoHcGF5bG9hZA==');

@$core.Deprecated('Use serverMessageDescriptor instead')
const ServerMessage$json = {
  '1': 'ServerMessage',
  '2': [
    {'1': 'response', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Response', '9': 0, '10': 'response'},
    {'1': 'event', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Event', '9': 0, '10': 'event'},
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ServerMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serverMessageDescriptor = $convert
    .base64Decode('Cg1TZXJ2ZXJNZXNzYWdlEi4KCHJlc3BvbnNlGAEgASgLMhAuYmRsLnYxLlJlc3BvbnNlSABSCH'
        'Jlc3BvbnNlEiUKBWV2ZW50GAIgASgLMg0uYmRsLnYxLkV2ZW50SABSBWV2ZW50QgkKB3BheWxv'
        'YWQ=');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {'1': 'request_id', '3': 1, '4': 1, '5': 4, '10': 'requestId'},
    {'1': 'error', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Error', '9': 0, '10': 'error'},
    {
      '1': 'handshake',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.HandshakeResponse',
      '9': 0,
      '10': 'handshake'
    },
    {
      '1': 'project',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ProjectResponse',
      '9': 0,
      '10': 'project'
    },
    {
      '1': 'edit_applied',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EditApplied',
      '9': 0,
      '10': 'editApplied'
    },
    {'1': 'ack', '3': 13, '4': 1, '5': 11, '6': '.bdl.v1.Ack', '9': 0, '10': 'ack'},
    {
      '1': 'analysis',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AnalysisResponse',
      '9': 0,
      '10': 'analysis'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert
    .base64Decode('CghSZXNwb25zZRIdCgpyZXF1ZXN0X2lkGAEgASgEUglyZXF1ZXN0SWQSJQoFZXJyb3IYAiABKA'
        'syDS5iZGwudjEuRXJyb3JIAFIFZXJyb3ISOQoJaGFuZHNoYWtlGAogASgLMhkuYmRsLnYxLkhh'
        'bmRzaGFrZVJlc3BvbnNlSABSCWhhbmRzaGFrZRIzCgdwcm9qZWN0GAsgASgLMhcuYmRsLnYxLl'
        'Byb2plY3RSZXNwb25zZUgAUgdwcm9qZWN0EjgKDGVkaXRfYXBwbGllZBgMIAEoCzITLmJkbC52'
        'MS5FZGl0QXBwbGllZEgAUgtlZGl0QXBwbGllZBIfCgNhY2sYDSABKAsyCy5iZGwudjEuQWNrSA'
        'BSA2FjaxI2CghhbmFseXNpcxgOIAEoCzIYLmJkbC52MS5BbmFseXNpc1Jlc3BvbnNlSABSCGFu'
        'YWx5c2lzQgkKB3BheWxvYWQ=');

@$core.Deprecated('Use eventDescriptor instead')
const Event$json = {
  '1': 'Event',
  '2': [
    {
      '1': 'project_changed',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ProjectChanged',
      '9': 0,
      '10': 'projectChanged'
    },
    {'1': 'log', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.DaemonLog', '9': 0, '10': 'log'},
    {
      '1': 'analysis_ready',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AnalysisReady',
      '9': 0,
      '10': 'analysisReady'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Event`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventDescriptor = $convert
    .base64Decode('CgVFdmVudBJBCg9wcm9qZWN0X2NoYW5nZWQYASABKAsyFi5iZGwudjEuUHJvamVjdENoYW5nZW'
        'RIAFIOcHJvamVjdENoYW5nZWQSJQoDbG9nGAIgASgLMhEuYmRsLnYxLkRhZW1vbkxvZ0gAUgNs'
        'b2cSPgoOYW5hbHlzaXNfcmVhZHkYAyABKAsyFS5iZGwudjEuQW5hbHlzaXNSZWFkeUgAUg1hbm'
        'FseXNpc1JlYWR5QgkKB3BheWxvYWQ=');

@$core.Deprecated('Use ackDescriptor instead')
const Ack$json = {
  '1': 'Ack',
};

/// Descriptor for `Ack`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List ackDescriptor = $convert.base64Decode('CgNBY2s=');

@$core.Deprecated('Use errorDescriptor instead')
const Error$json = {
  '1': 'Error',
  '2': [
    {'1': 'code', '3': 1, '4': 1, '5': 9, '10': 'code'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
    {'1': 'details_json', '3': 3, '4': 1, '5': 9, '10': 'detailsJson'},
  ],
};

/// Descriptor for `Error`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List errorDescriptor = $convert
    .base64Decode('CgVFcnJvchISCgRjb2RlGAEgASgJUgRjb2RlEhgKB21lc3NhZ2UYAiABKAlSB21lc3NhZ2USIQ'
        'oMZGV0YWlsc19qc29uGAMgASgJUgtkZXRhaWxzSnNvbg==');

@$core.Deprecated('Use versionDescriptor instead')
const Version$json = {
  '1': 'Version',
  '2': [
    {'1': 'major', '3': 1, '4': 1, '5': 13, '10': 'major'},
    {'1': 'minor', '3': 2, '4': 1, '5': 13, '10': 'minor'},
    {'1': 'patch', '3': 3, '4': 1, '5': 13, '10': 'patch'},
  ],
};

/// Descriptor for `Version`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List versionDescriptor = $convert
    .base64Decode('CgdWZXJzaW9uEhQKBW1ham9yGAEgASgNUgVtYWpvchIUCgVtaW5vchgCIAEoDVIFbWlub3ISFA'
        'oFcGF0Y2gYAyABKA1SBXBhdGNo');

@$core.Deprecated('Use handshakeRequestDescriptor instead')
const HandshakeRequest$json = {
  '1': 'HandshakeRequest',
  '2': [
    {
      '1': 'client_protocol_version',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Version',
      '10': 'clientProtocolVersion'
    },
    {'1': 'client_name', '3': 2, '4': 1, '5': 9, '10': 'clientName'},
    {'1': 'client_version', '3': 3, '4': 1, '5': 9, '10': 'clientVersion'},
  ],
};

/// Descriptor for `HandshakeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List handshakeRequestDescriptor = $convert
    .base64Decode('ChBIYW5kc2hha2VSZXF1ZXN0EkcKF2NsaWVudF9wcm90b2NvbF92ZXJzaW9uGAEgASgLMg8uYm'
        'RsLnYxLlZlcnNpb25SFWNsaWVudFByb3RvY29sVmVyc2lvbhIfCgtjbGllbnRfbmFtZRgCIAEo'
        'CVIKY2xpZW50TmFtZRIlCg5jbGllbnRfdmVyc2lvbhgDIAEoCVINY2xpZW50VmVyc2lvbg==');

@$core.Deprecated('Use handshakeResponseDescriptor instead')
const HandshakeResponse$json = {
  '1': 'HandshakeResponse',
  '2': [
    {
      '1': 'protocol_version',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Version',
      '10': 'protocolVersion'
    },
    {'1': 'compiler_version', '3': 2, '4': 1, '5': 9, '10': 'compilerVersion'},
    {'1': 'daemon_name', '3': 3, '4': 1, '5': 9, '10': 'daemonName'},
    {'1': 'compatible', '3': 4, '4': 1, '5': 8, '10': 'compatible'},
  ],
};

/// Descriptor for `HandshakeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List handshakeResponseDescriptor = $convert
    .base64Decode('ChFIYW5kc2hha2VSZXNwb25zZRI6ChBwcm90b2NvbF92ZXJzaW9uGAEgASgLMg8uYmRsLnYxLl'
        'ZlcnNpb25SD3Byb3RvY29sVmVyc2lvbhIpChBjb21waWxlcl92ZXJzaW9uGAIgASgJUg9jb21w'
        'aWxlclZlcnNpb24SHwoLZGFlbW9uX25hbWUYAyABKAlSCmRhZW1vbk5hbWUSHgoKY29tcGF0aW'
        'JsZRgEIAEoCFIKY29tcGF0aWJsZQ==');

@$core.Deprecated('Use openProjectRequestDescriptor instead')
const OpenProjectRequest$json = {
  '1': 'OpenProjectRequest',
  '2': [
    {'1': 'root_path', '3': 1, '4': 1, '5': 9, '10': 'rootPath'},
  ],
};

/// Descriptor for `OpenProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List openProjectRequestDescriptor =
    $convert.base64Decode('ChJPcGVuUHJvamVjdFJlcXVlc3QSGwoJcm9vdF9wYXRoGAEgASgJUghyb290UGF0aA==');

@$core.Deprecated('Use initProjectRequestDescriptor instead')
const InitProjectRequest$json = {
  '1': 'InitProjectRequest',
  '2': [
    {'1': 'root_path', '3': 1, '4': 1, '5': 9, '10': 'rootPath'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `InitProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initProjectRequestDescriptor = $convert
    .base64Decode('ChJJbml0UHJvamVjdFJlcXVlc3QSGwoJcm9vdF9wYXRoGAEgASgJUghyb290UGF0aBISCgRuYW'
        '1lGAIgASgJUgRuYW1l');

@$core.Deprecated('Use saveProjectRequestDescriptor instead')
const SaveProjectRequest$json = {
  '1': 'SaveProjectRequest',
};

/// Descriptor for `SaveProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List saveProjectRequestDescriptor =
    $convert.base64Decode('ChJTYXZlUHJvamVjdFJlcXVlc3Q=');

@$core.Deprecated('Use closeProjectRequestDescriptor instead')
const CloseProjectRequest$json = {
  '1': 'CloseProjectRequest',
};

/// Descriptor for `CloseProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List closeProjectRequestDescriptor =
    $convert.base64Decode('ChNDbG9zZVByb2plY3RSZXF1ZXN0');

@$core.Deprecated('Use getProjectRequestDescriptor instead')
const GetProjectRequest$json = {
  '1': 'GetProjectRequest',
};

/// Descriptor for `GetProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getProjectRequestDescriptor =
    $convert.base64Decode('ChFHZXRQcm9qZWN0UmVxdWVzdA==');

@$core.Deprecated('Use subscribeProjectRequestDescriptor instead')
const SubscribeProjectRequest$json = {
  '1': 'SubscribeProjectRequest',
};

/// Descriptor for `SubscribeProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List subscribeProjectRequestDescriptor =
    $convert.base64Decode('ChdTdWJzY3JpYmVQcm9qZWN0UmVxdWVzdA==');

@$core.Deprecated('Use shutdownRequestDescriptor instead')
const ShutdownRequest$json = {
  '1': 'ShutdownRequest',
};

/// Descriptor for `ShutdownRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List shutdownRequestDescriptor =
    $convert.base64Decode('Cg9TaHV0ZG93blJlcXVlc3Q=');

@$core.Deprecated('Use projectResponseDescriptor instead')
const ProjectResponse$json = {
  '1': 'ProjectResponse',
  '2': [
    {'1': 'project', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'project'},
  ],
};

/// Descriptor for `ProjectResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectResponseDescriptor = $convert
    .base64Decode('Cg9Qcm9qZWN0UmVzcG9uc2USMwoHcHJvamVjdBgBIAEoCzIZLmJkbC52MS5Qcm9qZWN0UHJvam'
        'VjdGlvblIHcHJvamVjdA==');

@$core.Deprecated('Use applyEditRequestDescriptor instead')
const ApplyEditRequest$json = {
  '1': 'ApplyEditRequest',
  '2': [
    {'1': 'base_revision', '3': 1, '4': 1, '5': 4, '10': 'baseRevision'},
    {'1': 'op', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EditOp', '10': 'op'},
  ],
};

/// Descriptor for `ApplyEditRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List applyEditRequestDescriptor = $convert
    .base64Decode('ChBBcHBseUVkaXRSZXF1ZXN0EiMKDWJhc2VfcmV2aXNpb24YASABKARSDGJhc2VSZXZpc2lvbh'
        'IeCgJvcBgCIAEoCzIOLmJkbC52MS5FZGl0T3BSAm9w');

@$core.Deprecated('Use undoRequestDescriptor instead')
const UndoRequest$json = {
  '1': 'UndoRequest',
};

/// Descriptor for `UndoRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List undoRequestDescriptor = $convert.base64Decode('CgtVbmRvUmVxdWVzdA==');

@$core.Deprecated('Use redoRequestDescriptor instead')
const RedoRequest$json = {
  '1': 'RedoRequest',
};

/// Descriptor for `RedoRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List redoRequestDescriptor = $convert.base64Decode('CgtSZWRvUmVxdWVzdA==');

@$core.Deprecated('Use editAppliedDescriptor instead')
const EditApplied$json = {
  '1': 'EditApplied',
  '2': [
    {'1': 'project', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'project'},
    {'1': 'outcome', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EditOutcome', '10': 'outcome'},
  ],
};

/// Descriptor for `EditApplied`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List editAppliedDescriptor = $convert
    .base64Decode('CgtFZGl0QXBwbGllZBIzCgdwcm9qZWN0GAEgASgLMhkuYmRsLnYxLlByb2plY3RQcm9qZWN0aW'
        '9uUgdwcm9qZWN0Ei0KB291dGNvbWUYAiABKAsyEy5iZGwudjEuRWRpdE91dGNvbWVSB291dGNv'
        'bWU=');

@$core.Deprecated('Use editOpDescriptor instead')
const EditOp$json = {
  '1': 'EditOp',
  '2': [
    {
      '1': 'create_concept',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateConcept',
      '9': 0,
      '10': 'createConcept'
    },
    {
      '1': 'rename_concept',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameConcept',
      '9': 0,
      '10': 'renameConcept'
    },
    {
      '1': 'set_concept_description',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetConceptDescription',
      '9': 0,
      '10': 'setConceptDescription'
    },
    {
      '1': 'set_concept_representation',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetConceptRepresentation',
      '9': 0,
      '10': 'setConceptRepresentation'
    },
    {
      '1': 'delete_concept',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteConcept',
      '9': 0,
      '10': 'deleteConcept'
    },
    {
      '1': 'create_mapping',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateMapping',
      '9': 0,
      '10': 'createMapping'
    },
    {
      '1': 'rename_mapping',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameMapping',
      '9': 0,
      '10': 'renameMapping'
    },
    {
      '1': 'set_mapping_description',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetMappingDescription',
      '9': 0,
      '10': 'setMappingDescription'
    },
    {
      '1': 'set_mapping_signature',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetMappingSignature',
      '9': 0,
      '10': 'setMappingSignature'
    },
    {
      '1': 'attach_definition',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AttachDefinition',
      '9': 0,
      '10': 'attachDefinition'
    },
    {
      '1': 'replace_definition',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ReplaceDefinition',
      '9': 0,
      '10': 'replaceDefinition'
    },
    {
      '1': 'delete_mapping',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteMapping',
      '9': 0,
      '10': 'deleteMapping'
    },
  ],
  '8': [
    {'1': 'op'},
  ],
};

/// Descriptor for `EditOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List editOpDescriptor = $convert
    .base64Decode('CgZFZGl0T3ASPgoOY3JlYXRlX2NvbmNlcHQYASABKAsyFS5iZGwudjEuQ3JlYXRlQ29uY2VwdE'
        'gAUg1jcmVhdGVDb25jZXB0Ej4KDnJlbmFtZV9jb25jZXB0GAIgASgLMhUuYmRsLnYxLlJlbmFt'
        'ZUNvbmNlcHRIAFINcmVuYW1lQ29uY2VwdBJXChdzZXRfY29uY2VwdF9kZXNjcmlwdGlvbhgDIA'
        'EoCzIdLmJkbC52MS5TZXRDb25jZXB0RGVzY3JpcHRpb25IAFIVc2V0Q29uY2VwdERlc2NyaXB0'
        'aW9uEmAKGnNldF9jb25jZXB0X3JlcHJlc2VudGF0aW9uGAQgASgLMiAuYmRsLnYxLlNldENvbm'
        'NlcHRSZXByZXNlbnRhdGlvbkgAUhhzZXRDb25jZXB0UmVwcmVzZW50YXRpb24SPgoOZGVsZXRl'
        'X2NvbmNlcHQYBSABKAsyFS5iZGwudjEuRGVsZXRlQ29uY2VwdEgAUg1kZWxldGVDb25jZXB0Ej'
        '4KDmNyZWF0ZV9tYXBwaW5nGAYgASgLMhUuYmRsLnYxLkNyZWF0ZU1hcHBpbmdIAFINY3JlYXRl'
        'TWFwcGluZxI+Cg5yZW5hbWVfbWFwcGluZxgHIAEoCzIVLmJkbC52MS5SZW5hbWVNYXBwaW5nSA'
        'BSDXJlbmFtZU1hcHBpbmcSVwoXc2V0X21hcHBpbmdfZGVzY3JpcHRpb24YCCABKAsyHS5iZGwu'
        'djEuU2V0TWFwcGluZ0Rlc2NyaXB0aW9uSABSFXNldE1hcHBpbmdEZXNjcmlwdGlvbhJRChVzZX'
        'RfbWFwcGluZ19zaWduYXR1cmUYCSABKAsyGy5iZGwudjEuU2V0TWFwcGluZ1NpZ25hdHVyZUgA'
        'UhNzZXRNYXBwaW5nU2lnbmF0dXJlEkcKEWF0dGFjaF9kZWZpbml0aW9uGAogASgLMhguYmRsLn'
        'YxLkF0dGFjaERlZmluaXRpb25IAFIQYXR0YWNoRGVmaW5pdGlvbhJKChJyZXBsYWNlX2RlZmlu'
        'aXRpb24YCyABKAsyGS5iZGwudjEuUmVwbGFjZURlZmluaXRpb25IAFIRcmVwbGFjZURlZmluaX'
        'Rpb24SPgoOZGVsZXRlX21hcHBpbmcYDCABKAsyFS5iZGwudjEuRGVsZXRlTWFwcGluZ0gAUg1k'
        'ZWxldGVNYXBwaW5nQgQKAm9w');

@$core.Deprecated('Use createConceptDescriptor instead')
const CreateConcept$json = {
  '1': 'CreateConcept',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'representation',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Representation',
      '9': 0,
      '10': 'representation',
      '17': true
    },
  ],
  '8': [
    {'1': '_representation'},
  ],
};

/// Descriptor for `CreateConcept`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createConceptDescriptor = $convert
    .base64Decode('Cg1DcmVhdGVDb25jZXB0EhIKBG5hbWUYASABKAlSBG5hbWUSIAoLZGVzY3JpcHRpb24YAiABKA'
        'lSC2Rlc2NyaXB0aW9uEkMKDnJlcHJlc2VudGF0aW9uGAMgASgLMhYuYmRsLnYxLlJlcHJlc2Vu'
        'dGF0aW9uSABSDnJlcHJlc2VudGF0aW9uiAEBQhEKD19yZXByZXNlbnRhdGlvbg==');

@$core.Deprecated('Use renameConceptDescriptor instead')
const RenameConcept$json = {
  '1': 'RenameConcept',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameConcept`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameConceptDescriptor =
    $convert.base64Decode('Cg1SZW5hbWVDb25jZXB0Eg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1l');

@$core.Deprecated('Use setConceptDescriptionDescriptor instead')
const SetConceptDescription$json = {
  '1': 'SetConceptDescription',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `SetConceptDescription`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setConceptDescriptionDescriptor = $convert
    .base64Decode('ChVTZXRDb25jZXB0RGVzY3JpcHRpb24SDgoCaWQYASABKARSAmlkEiAKC2Rlc2NyaXB0aW9uGA'
        'IgASgJUgtkZXNjcmlwdGlvbg==');

@$core.Deprecated('Use setConceptRepresentationDescriptor instead')
const SetConceptRepresentation$json = {
  '1': 'SetConceptRepresentation',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {
      '1': 'representation',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Representation',
      '9': 0,
      '10': 'representation',
      '17': true
    },
  ],
  '8': [
    {'1': '_representation'},
  ],
};

/// Descriptor for `SetConceptRepresentation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setConceptRepresentationDescriptor = $convert
    .base64Decode('ChhTZXRDb25jZXB0UmVwcmVzZW50YXRpb24SDgoCaWQYASABKARSAmlkEkMKDnJlcHJlc2VudG'
        'F0aW9uGAIgASgLMhYuYmRsLnYxLlJlcHJlc2VudGF0aW9uSABSDnJlcHJlc2VudGF0aW9uiAEB'
        'QhEKD19yZXByZXNlbnRhdGlvbg==');

@$core.Deprecated('Use deleteConceptDescriptor instead')
const DeleteConcept$json = {
  '1': 'DeleteConcept',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteConcept`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteConceptDescriptor =
    $convert.base64Decode('Cg1EZWxldGVDb25jZXB0Eg4KAmlkGAEgASgEUgJpZA==');

@$core.Deprecated('Use createMappingDescriptor instead')
const CreateMapping$json = {
  '1': 'CreateMapping',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'signature', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Signature', '10': 'signature'},
  ],
};

/// Descriptor for `CreateMapping`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createMappingDescriptor = $convert
    .base64Decode('Cg1DcmVhdGVNYXBwaW5nEhIKBG5hbWUYASABKAlSBG5hbWUSIAoLZGVzY3JpcHRpb24YAiABKA'
        'lSC2Rlc2NyaXB0aW9uEi8KCXNpZ25hdHVyZRgDIAEoCzIRLmJkbC52MS5TaWduYXR1cmVSCXNp'
        'Z25hdHVyZQ==');

@$core.Deprecated('Use renameMappingDescriptor instead')
const RenameMapping$json = {
  '1': 'RenameMapping',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameMapping`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameMappingDescriptor =
    $convert.base64Decode('Cg1SZW5hbWVNYXBwaW5nEg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1l');

@$core.Deprecated('Use setMappingDescriptionDescriptor instead')
const SetMappingDescription$json = {
  '1': 'SetMappingDescription',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `SetMappingDescription`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setMappingDescriptionDescriptor = $convert
    .base64Decode('ChVTZXRNYXBwaW5nRGVzY3JpcHRpb24SDgoCaWQYASABKARSAmlkEiAKC2Rlc2NyaXB0aW9uGA'
        'IgASgJUgtkZXNjcmlwdGlvbg==');

@$core.Deprecated('Use setMappingSignatureDescriptor instead')
const SetMappingSignature$json = {
  '1': 'SetMappingSignature',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'signature', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Signature', '10': 'signature'},
  ],
};

/// Descriptor for `SetMappingSignature`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setMappingSignatureDescriptor = $convert
    .base64Decode('ChNTZXRNYXBwaW5nU2lnbmF0dXJlEg4KAmlkGAEgASgEUgJpZBIvCglzaWduYXR1cmUYAiABKA'
        'syES5iZGwudjEuU2lnbmF0dXJlUglzaWduYXR1cmU=');

@$core.Deprecated('Use attachDefinitionDescriptor instead')
const AttachDefinition$json = {
  '1': 'AttachDefinition',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'definition', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Definition', '10': 'definition'},
  ],
};

/// Descriptor for `AttachDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List attachDefinitionDescriptor = $convert
    .base64Decode('ChBBdHRhY2hEZWZpbml0aW9uEg4KAmlkGAEgASgEUgJpZBIyCgpkZWZpbml0aW9uGAIgASgLMh'
        'IuYmRsLnYxLkRlZmluaXRpb25SCmRlZmluaXRpb24=');

@$core.Deprecated('Use replaceDefinitionDescriptor instead')
const ReplaceDefinition$json = {
  '1': 'ReplaceDefinition',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {
      '1': 'definition',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Definition',
      '9': 0,
      '10': 'definition',
      '17': true
    },
  ],
  '8': [
    {'1': '_definition'},
  ],
};

/// Descriptor for `ReplaceDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replaceDefinitionDescriptor = $convert
    .base64Decode('ChFSZXBsYWNlRGVmaW5pdGlvbhIOCgJpZBgBIAEoBFICaWQSNwoKZGVmaW5pdGlvbhgCIAEoCz'
        'ISLmJkbC52MS5EZWZpbml0aW9uSABSCmRlZmluaXRpb26IAQFCDQoLX2RlZmluaXRpb24=');

@$core.Deprecated('Use deleteMappingDescriptor instead')
const DeleteMapping$json = {
  '1': 'DeleteMapping',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteMapping`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteMappingDescriptor =
    $convert.base64Decode('Cg1EZWxldGVNYXBwaW5nEg4KAmlkGAEgASgEUgJpZA==');

@$core.Deprecated('Use editOutcomeDescriptor instead')
const EditOutcome$json = {
  '1': 'EditOutcome',
  '2': [
    {'1': 'kind', '3': 1, '4': 1, '5': 14, '6': '.bdl.v1.EditKind', '10': 'kind'},
    {'1': 'invalidates', '3': 2, '4': 3, '5': 14, '6': '.bdl.v1.Invalidation', '10': 'invalidates'},
    {'1': 'origin_decls', '3': 3, '4': 3, '5': 4, '10': 'originDecls'},
    {'1': 'created_concept', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'createdConcept', '17': true},
    {'1': 'created_mapping', '3': 5, '4': 1, '5': 4, '9': 1, '10': 'createdMapping', '17': true},
  ],
  '8': [
    {'1': '_created_concept'},
    {'1': '_created_mapping'},
  ],
};

/// Descriptor for `EditOutcome`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List editOutcomeDescriptor = $convert
    .base64Decode('CgtFZGl0T3V0Y29tZRIkCgRraW5kGAEgASgOMhAuYmRsLnYxLkVkaXRLaW5kUgRraW5kEjYKC2'
        'ludmFsaWRhdGVzGAIgAygOMhQuYmRsLnYxLkludmFsaWRhdGlvblILaW52YWxpZGF0ZXMSIQoM'
        'b3JpZ2luX2RlY2xzGAMgAygEUgtvcmlnaW5EZWNscxIsCg9jcmVhdGVkX2NvbmNlcHQYBCABKA'
        'RIAFIOY3JlYXRlZENvbmNlcHSIAQESLAoPY3JlYXRlZF9tYXBwaW5nGAUgASgESAFSDmNyZWF0'
        'ZWRNYXBwaW5niAEBQhIKEF9jcmVhdGVkX2NvbmNlcHRCEgoQX2NyZWF0ZWRfbWFwcGluZw==');

@$core.Deprecated('Use projectProjectionDescriptor instead')
const ProjectProjection$json = {
  '1': 'ProjectProjection',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'root_path', '3': 3, '4': 1, '5': 9, '10': 'rootPath'},
    {'1': 'concepts', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.ConceptView', '10': 'concepts'},
    {'1': 'mappings', '3': 5, '4': 3, '5': 11, '6': '.bdl.v1.MappingView', '10': 'mappings'},
    {'1': 'layout', '3': 6, '4': 1, '5': 11, '6': '.bdl.v1.Layout', '10': 'layout'},
    {'1': 'can_undo', '3': 7, '4': 1, '5': 8, '10': 'canUndo'},
    {'1': 'can_redo', '3': 8, '4': 1, '5': 8, '10': 'canRedo'},
    {'1': 'dirty', '3': 9, '4': 1, '5': 8, '10': 'dirty'},
  ],
};

/// Descriptor for `ProjectProjection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectProjectionDescriptor = $convert
    .base64Decode('ChFQcm9qZWN0UHJvamVjdGlvbhIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SEgoEbmFtZR'
        'gCIAEoCVIEbmFtZRIbCglyb290X3BhdGgYAyABKAlSCHJvb3RQYXRoEi8KCGNvbmNlcHRzGAQg'
        'AygLMhMuYmRsLnYxLkNvbmNlcHRWaWV3Ughjb25jZXB0cxIvCghtYXBwaW5ncxgFIAMoCzITLm'
        'JkbC52MS5NYXBwaW5nVmlld1IIbWFwcGluZ3MSJgoGbGF5b3V0GAYgASgLMg4uYmRsLnYxLkxh'
        'eW91dFIGbGF5b3V0EhkKCGNhbl91bmRvGAcgASgIUgdjYW5VbmRvEhkKCGNhbl9yZWRvGAggAS'
        'gIUgdjYW5SZWRvEhQKBWRpcnR5GAkgASgIUgVkaXJ0eQ==');

@$core.Deprecated('Use conceptViewDescriptor instead')
const ConceptView$json = {
  '1': 'ConceptView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'representation',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Representation',
      '9': 0,
      '10': 'representation',
      '17': true
    },
  ],
  '8': [
    {'1': '_representation'},
  ],
};

/// Descriptor for `ConceptView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List conceptViewDescriptor = $convert
    .base64Decode('CgtDb25jZXB0VmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCgtkZX'
        'NjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SQwoOcmVwcmVzZW50YXRpb24YBCABKAsyFi5i'
        'ZGwudjEuUmVwcmVzZW50YXRpb25IAFIOcmVwcmVzZW50YXRpb26IAQFCEQoPX3JlcHJlc2VudG'
        'F0aW9u');

@$core.Deprecated('Use mappingViewDescriptor instead')
const MappingView$json = {
  '1': 'MappingView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'signature', '3': 4, '4': 1, '5': 11, '6': '.bdl.v1.Signature', '10': 'signature'},
    {
      '1': 'definition',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Definition',
      '9': 0,
      '10': 'definition',
      '17': true
    },
    {'1': 'state', '3': 6, '4': 1, '5': 14, '6': '.bdl.v1.AcceptanceState', '10': 'state'},
  ],
  '8': [
    {'1': '_definition'},
  ],
};

/// Descriptor for `MappingView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mappingViewDescriptor = $convert
    .base64Decode('CgtNYXBwaW5nVmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCgtkZX'
        'NjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SLwoJc2lnbmF0dXJlGAQgASgLMhEuYmRsLnYx'
        'LlNpZ25hdHVyZVIJc2lnbmF0dXJlEjcKCmRlZmluaXRpb24YBSABKAsyEi5iZGwudjEuRGVmaW'
        '5pdGlvbkgAUgpkZWZpbml0aW9uiAEBEi0KBXN0YXRlGAYgASgOMhcuYmRsLnYxLkFjY2VwdGFu'
        'Y2VTdGF0ZVIFc3RhdGVCDQoLX2RlZmluaXRpb24=');

@$core.Deprecated('Use signatureDescriptor instead')
const Signature$json = {
  '1': 'Signature',
  '2': [
    {'1': 'inputs', '3': 1, '4': 3, '5': 4, '10': 'inputs'},
    {'1': 'output', '3': 2, '4': 1, '5': 4, '10': 'output'},
  ],
};

/// Descriptor for `Signature`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signatureDescriptor = $convert
    .base64Decode('CglTaWduYXR1cmUSFgoGaW5wdXRzGAEgAygEUgZpbnB1dHMSFgoGb3V0cHV0GAIgASgEUgZvdX'
        'RwdXQ=');

@$core.Deprecated('Use definitionDescriptor instead')
const Definition$json = {
  '1': 'Definition',
  '2': [
    {'1': 'formula', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'formula'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Definition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List definitionDescriptor =
    $convert.base64Decode('CgpEZWZpbml0aW9uEhoKB2Zvcm11bGEYASABKAlIAFIHZm9ybXVsYUIGCgRraW5k');

@$core.Deprecated('Use representationDescriptor instead')
const Representation$json = {
  '1': 'Representation',
  '2': [
    {'1': 'quantity', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Dim', '9': 0, '10': 'quantity'},
    {'1': 'boolean', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'boolean'},
    {'1': 'count', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'count'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Representation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List representationDescriptor = $convert
    .base64Decode('Cg5SZXByZXNlbnRhdGlvbhIpCghxdWFudGl0eRgBIAEoCzILLmJkbC52MS5EaW1IAFIIcXVhbn'
        'RpdHkSKAoHYm9vbGVhbhgCIAEoCzIMLmJkbC52MS5Vbml0SABSB2Jvb2xlYW4SJAoFY291bnQY'
        'AyABKAsyDC5iZGwudjEuVW5pdEgAUgVjb3VudEIGCgRraW5k');

@$core.Deprecated('Use unitDescriptor instead')
const Unit$json = {
  '1': 'Unit',
};

/// Descriptor for `Unit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unitDescriptor = $convert.base64Decode('CgRVbml0');

@$core.Deprecated('Use dimDescriptor instead')
const Dim$json = {
  '1': 'Dim',
  '2': [
    {'1': 'length', '3': 1, '4': 1, '5': 17, '10': 'length'},
    {'1': 'mass', '3': 2, '4': 1, '5': 17, '10': 'mass'},
    {'1': 'time', '3': 3, '4': 1, '5': 17, '10': 'time'},
    {'1': 'current', '3': 4, '4': 1, '5': 17, '10': 'current'},
    {'1': 'temperature', '3': 5, '4': 1, '5': 17, '10': 'temperature'},
    {'1': 'amount', '3': 6, '4': 1, '5': 17, '10': 'amount'},
    {'1': 'luminous', '3': 7, '4': 1, '5': 17, '10': 'luminous'},
    {'1': 'angle', '3': 8, '4': 1, '5': 17, '10': 'angle'},
  ],
};

/// Descriptor for `Dim`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List dimDescriptor = $convert
    .base64Decode('CgNEaW0SFgoGbGVuZ3RoGAEgASgRUgZsZW5ndGgSEgoEbWFzcxgCIAEoEVIEbWFzcxISCgR0aW'
        '1lGAMgASgRUgR0aW1lEhgKB2N1cnJlbnQYBCABKBFSB2N1cnJlbnQSIAoLdGVtcGVyYXR1cmUY'
        'BSABKBFSC3RlbXBlcmF0dXJlEhYKBmFtb3VudBgGIAEoEVIGYW1vdW50EhoKCGx1bWlub3VzGA'
        'cgASgRUghsdW1pbm91cxIUCgVhbmdsZRgIIAEoEVIFYW5nbGU=');

@$core.Deprecated('Use layoutDescriptor instead')
const Layout$json = {
  '1': 'Layout',
  '2': [
    {'1': 'concepts', '3': 1, '4': 3, '5': 11, '6': '.bdl.v1.NodePosition', '10': 'concepts'},
    {'1': 'mappings', '3': 2, '4': 3, '5': 11, '6': '.bdl.v1.NodePosition', '10': 'mappings'},
  ],
};

/// Descriptor for `Layout`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List layoutDescriptor = $convert
    .base64Decode('CgZMYXlvdXQSMAoIY29uY2VwdHMYASADKAsyFC5iZGwudjEuTm9kZVBvc2l0aW9uUghjb25jZX'
        'B0cxIwCghtYXBwaW5ncxgCIAMoCzIULmJkbC52MS5Ob2RlUG9zaXRpb25SCG1hcHBpbmdz');

@$core.Deprecated('Use nodePositionDescriptor instead')
const NodePosition$json = {
  '1': 'NodePosition',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'x', '3': 2, '4': 1, '5': 1, '10': 'x'},
    {'1': 'y', '3': 3, '4': 1, '5': 1, '10': 'y'},
  ],
};

/// Descriptor for `NodePosition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List nodePositionDescriptor = $convert
    .base64Decode('CgxOb2RlUG9zaXRpb24SDgoCaWQYASABKARSAmlkEgwKAXgYAiABKAFSAXgSDAoBeRgDIAEoAV'
        'IBeQ==');

@$core.Deprecated('Use setLayoutRequestDescriptor instead')
const SetLayoutRequest$json = {
  '1': 'SetLayoutRequest',
  '2': [
    {'1': 'layout', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Layout', '10': 'layout'},
  ],
};

/// Descriptor for `SetLayoutRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setLayoutRequestDescriptor = $convert
    .base64Decode('ChBTZXRMYXlvdXRSZXF1ZXN0EiYKBmxheW91dBgBIAEoCzIOLmJkbC52MS5MYXlvdXRSBmxheW'
        '91dA==');

@$core.Deprecated('Use projectChangedDescriptor instead')
const ProjectChanged$json = {
  '1': 'ProjectChanged',
  '2': [
    {'1': 'project', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'project'},
    {
      '1': 'outcome',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EditOutcome',
      '9': 0,
      '10': 'outcome',
      '17': true
    },
  ],
  '8': [
    {'1': '_outcome'},
  ],
};

/// Descriptor for `ProjectChanged`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectChangedDescriptor = $convert
    .base64Decode('Cg5Qcm9qZWN0Q2hhbmdlZBIzCgdwcm9qZWN0GAEgASgLMhkuYmRsLnYxLlByb2plY3RQcm9qZW'
        'N0aW9uUgdwcm9qZWN0EjIKB291dGNvbWUYAiABKAsyEy5iZGwudjEuRWRpdE91dGNvbWVIAFIH'
        'b3V0Y29tZYgBAUIKCghfb3V0Y29tZQ==');

@$core.Deprecated('Use daemonLogDescriptor instead')
const DaemonLog$json = {
  '1': 'DaemonLog',
  '2': [
    {'1': 'level', '3': 1, '4': 1, '5': 9, '10': 'level'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DaemonLog`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List daemonLogDescriptor = $convert
    .base64Decode('CglEYWVtb25Mb2cSFAoFbGV2ZWwYASABKAlSBWxldmVsEhgKB21lc3NhZ2UYAiABKAlSB21lc3'
        'NhZ2U=');

@$core.Deprecated('Use runAnalysisRequestDescriptor instead')
const RunAnalysisRequest$json = {
  '1': 'RunAnalysisRequest',
};

/// Descriptor for `RunAnalysisRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List runAnalysisRequestDescriptor =
    $convert.base64Decode('ChJSdW5BbmFseXNpc1JlcXVlc3Q=');

@$core.Deprecated('Use analysisResponseDescriptor instead')
const AnalysisResponse$json = {
  '1': 'AnalysisResponse',
  '2': [
    {'1': 'analysis', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.ProjectAnalysis', '10': 'analysis'},
  ],
};

/// Descriptor for `AnalysisResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List analysisResponseDescriptor = $convert
    .base64Decode('ChBBbmFseXNpc1Jlc3BvbnNlEjMKCGFuYWx5c2lzGAEgASgLMhcuYmRsLnYxLlByb2plY3RBbm'
        'FseXNpc1IIYW5hbHlzaXM=');

@$core.Deprecated('Use analysisReadyDescriptor instead')
const AnalysisReady$json = {
  '1': 'AnalysisReady',
  '2': [
    {'1': 'analysis', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.ProjectAnalysis', '10': 'analysis'},
  ],
};

/// Descriptor for `AnalysisReady`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List analysisReadyDescriptor = $convert
    .base64Decode('Cg1BbmFseXNpc1JlYWR5EjMKCGFuYWx5c2lzGAEgASgLMhcuYmRsLnYxLlByb2plY3RBbmFseX'
        'Npc1IIYW5hbHlzaXM=');

@$core.Deprecated('Use projectAnalysisDescriptor instead')
const ProjectAnalysis$json = {
  '1': 'ProjectAnalysis',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mappings', '3': 2, '4': 3, '5': 11, '6': '.bdl.v1.MappingAnalysis', '10': 'mappings'},
    {'1': 'diagnostics', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'diagnostics'},
  ],
};

/// Descriptor for `ProjectAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectAnalysisDescriptor = $convert
    .base64Decode('Cg9Qcm9qZWN0QW5hbHlzaXMSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEjMKCG1hcHBpbm'
        'dzGAIgAygLMhcuYmRsLnYxLk1hcHBpbmdBbmFseXNpc1IIbWFwcGluZ3MSNAoLZGlhZ25vc3Rp'
        'Y3MYAyADKAsyEi5iZGwudjEuRGlhZ25vc3RpY1ILZGlhZ25vc3RpY3M=');

@$core.Deprecated('Use mappingAnalysisDescriptor instead')
const MappingAnalysis$json = {
  '1': 'MappingAnalysis',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'status', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.MappingStatus', '10': 'status'},
    {'1': 'interface', '3': 3, '4': 1, '5': 9, '10': 'interface'},
    {'1': 'inferred_type', '3': 4, '4': 1, '5': 9, '10': 'inferredType'},
    {'1': 'core_expr', '3': 5, '4': 1, '5': 9, '10': 'coreExpr'},
    {'1': 'diagnostics', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'diagnostics'},
  ],
};

/// Descriptor for `MappingAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mappingAnalysisDescriptor = $convert
    .base64Decode('Cg9NYXBwaW5nQW5hbHlzaXMSDgoCaWQYASABKARSAmlkEi0KBnN0YXR1cxgCIAEoDjIVLmJkbC'
        '52MS5NYXBwaW5nU3RhdHVzUgZzdGF0dXMSHAoJaW50ZXJmYWNlGAMgASgJUglpbnRlcmZhY2US'
        'IwoNaW5mZXJyZWRfdHlwZRgEIAEoCVIMaW5mZXJyZWRUeXBlEhsKCWNvcmVfZXhwchgFIAEoCV'
        'IIY29yZUV4cHISNAoLZGlhZ25vc3RpY3MYBiADKAsyEi5iZGwudjEuRGlhZ25vc3RpY1ILZGlh'
        'Z25vc3RpY3M=');

@$core.Deprecated('Use sourceSpanDescriptor instead')
const SourceSpan$json = {
  '1': 'SourceSpan',
  '2': [
    {'1': 'start', '3': 1, '4': 1, '5': 13, '10': 'start'},
    {'1': 'end', '3': 2, '4': 1, '5': 13, '10': 'end'},
  ],
};

/// Descriptor for `SourceSpan`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourceSpanDescriptor = $convert
    .base64Decode('CgpTb3VyY2VTcGFuEhQKBXN0YXJ0GAEgASgNUgVzdGFydBIQCgNlbmQYAiABKA1SA2VuZA==');

@$core.Deprecated('Use diagnosticDescriptor instead')
const Diagnostic$json = {
  '1': 'Diagnostic',
  '2': [
    {'1': 'code', '3': 1, '4': 1, '5': 9, '10': 'code'},
    {'1': 'severity', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.DiagnosticSeverity', '10': 'severity'},
    {'1': 'project', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'project'},
    {'1': 'concept_id', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'conceptId'},
    {'1': 'mapping_id', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'mappingId'},
    {
      '1': 'span',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SourceSpan',
      '9': 1,
      '10': 'span',
      '17': true
    },
    {'1': 'message', '3': 7, '4': 1, '5': 9, '10': 'message'},
    {'1': 'explanation', '3': 8, '4': 1, '5': 9, '10': 'explanation'},
    {'1': 'technical', '3': 9, '4': 1, '5': 9, '10': 'technical'},
    {'1': 'fixes', '3': 10, '4': 3, '5': 9, '10': 'fixes'},
  ],
  '8': [
    {'1': 'entity'},
    {'1': '_span'},
  ],
};

/// Descriptor for `Diagnostic`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List diagnosticDescriptor = $convert
    .base64Decode('CgpEaWFnbm9zdGljEhIKBGNvZGUYASABKAlSBGNvZGUSNgoIc2V2ZXJpdHkYAiABKA4yGi5iZG'
        'wudjEuRGlhZ25vc3RpY1NldmVyaXR5UghzZXZlcml0eRIoCgdwcm9qZWN0GAMgASgLMgwuYmRs'
        'LnYxLlVuaXRIAFIHcHJvamVjdBIfCgpjb25jZXB0X2lkGAQgASgESABSCWNvbmNlcHRJZBIfCg'
        'ptYXBwaW5nX2lkGAUgASgESABSCW1hcHBpbmdJZBIrCgRzcGFuGAYgASgLMhIuYmRsLnYxLlNv'
        'dXJjZVNwYW5IAVIEc3BhbogBARIYCgdtZXNzYWdlGAcgASgJUgdtZXNzYWdlEiAKC2V4cGxhbm'
        'F0aW9uGAggASgJUgtleHBsYW5hdGlvbhIcCgl0ZWNobmljYWwYCSABKAlSCXRlY2huaWNhbBIU'
        'CgVmaXhlcxgKIAMoCVIFZml4ZXNCCAoGZW50aXR5QgcKBV9zcGFu');
