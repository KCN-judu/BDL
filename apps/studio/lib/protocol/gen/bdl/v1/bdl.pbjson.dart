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

@$core.Deprecated('Use deviceKindDescriptor instead')
const DeviceKind$json = {
  '1': 'DeviceKind',
  '2': [
    {'1': 'DEVICE_KIND_UNSPECIFIED', '2': 0},
    {'1': 'DEVICE_KIND_PWM_CHANNEL', '2': 1},
    {'1': 'DEVICE_KIND_DIGITAL_OUTPUT', '2': 2},
    {'1': 'DEVICE_KIND_H_BRIDGE_CHANNEL', '2': 3},
    {'1': 'DEVICE_KIND_I2C_SENSOR', '2': 4},
    {'1': 'DEVICE_KIND_QUADRATURE_ENCODER', '2': 5},
    {'1': 'DEVICE_KIND_UART', '2': 6},
  ],
};

/// Descriptor for `DeviceKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List deviceKindDescriptor = $convert
    .base64Decode('CgpEZXZpY2VLaW5kEhsKF0RFVklDRV9LSU5EX1VOU1BFQ0lGSUVEEAASGwoXREVWSUNFX0tJTk'
        'RfUFdNX0NIQU5ORUwQARIeChpERVZJQ0VfS0lORF9ESUdJVEFMX09VVFBVVBACEiAKHERFVklD'
        'RV9LSU5EX0hfQlJJREdFX0NIQU5ORUwQAxIaChZERVZJQ0VfS0lORF9JMkNfU0VOU09SEAQSIg'
        'oeREVWSUNFX0tJTkRfUVVBRFJBVFVSRV9FTkNPREVSEAUSFAoQREVWSUNFX0tJTkRfVUFSVBAG');

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

@$core.Deprecated('Use projectKindDescriptor instead')
const ProjectKind$json = {
  '1': 'ProjectKind',
  '2': [
    {'1': 'PROJECT_KIND_UNSPECIFIED', '2': 0},
    {'1': 'PROJECT_KIND_FLAT', '2': 1},
    {'1': 'PROJECT_KIND_SYSTEM', '2': 2},
    {'1': 'PROJECT_KIND_TEXT', '2': 3},
  ],
};

/// Descriptor for `ProjectKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List projectKindDescriptor = $convert
    .base64Decode('CgtQcm9qZWN0S2luZBIcChhQUk9KRUNUX0tJTkRfVU5TUEVDSUZJRUQQABIVChFQUk9KRUNUX0'
        'tJTkRfRkxBVBABEhcKE1BST0pFQ1RfS0lORF9TWVNURU0QAhIVChFQUk9KRUNUX0tJTkRfVEVY'
        'VBAD');

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

@$core.Deprecated('Use outputStateDescriptor instead')
const OutputState$json = {
  '1': 'OutputState',
  '2': [
    {'1': 'OUTPUT_STATE_UNSPECIFIED', '2': 0},
    {'1': 'OUTPUT_STATE_UNDRIVEN', '2': 1},
    {'1': 'OUTPUT_STATE_DRIVEN', '2': 2},
    {'1': 'OUTPUT_STATE_ILL_FORMED', '2': 3},
    {'1': 'OUTPUT_STATE_CONFLICT', '2': 4},
  ],
};

/// Descriptor for `OutputState`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List outputStateDescriptor = $convert
    .base64Decode('CgtPdXRwdXRTdGF0ZRIcChhPVVRQVVRfU1RBVEVfVU5TUEVDSUZJRUQQABIZChVPVVRQVVRfU1'
        'RBVEVfVU5EUklWRU4QARIXChNPVVRQVVRfU1RBVEVfRFJJVkVOEAISGwoXT1VUUFVUX1NUQVRF'
        'X0lMTF9GT1JNRUQQAxIZChVPVVRQVVRfU1RBVEVfQ09ORkxJQ1QQBA==');

@$core.Deprecated('Use mappingStatusDescriptor instead')
const MappingStatus$json = {
  '1': 'MappingStatus',
  '2': [
    {'1': 'MAPPING_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'MAPPING_STATUS_DECLARED', '2': 1},
    {'1': 'MAPPING_STATUS_OPEN', '2': 2},
    {'1': 'MAPPING_STATUS_INVALID', '2': 3},
    {'1': 'MAPPING_STATUS_TYPE_VALID', '2': 4},
    {'1': 'MAPPING_STATUS_TEMPORALLY_VALID', '2': 5},
    {'1': 'MAPPING_STATUS_CLOCK_CONSISTENT', '2': 6},
  ],
};

/// Descriptor for `MappingStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List mappingStatusDescriptor = $convert
    .base64Decode('Cg1NYXBwaW5nU3RhdHVzEh4KGk1BUFBJTkdfU1RBVFVTX1VOU1BFQ0lGSUVEEAASGwoXTUFQUE'
        'lOR19TVEFUVVNfREVDTEFSRUQQARIXChNNQVBQSU5HX1NUQVRVU19PUEVOEAISGgoWTUFQUElO'
        'R19TVEFUVVNfSU5WQUxJRBADEh0KGU1BUFBJTkdfU1RBVFVTX1RZUEVfVkFMSUQQBBIjCh9NQV'
        'BQSU5HX1NUQVRVU19URU1QT1JBTExZX1ZBTElEEAUSIwofTUFQUElOR19TVEFUVVNfQ0xPQ0tf'
        'Q09OU0lTVEVOVBAG');

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

@$core.Deprecated('Use actionApplicabilityDescriptor instead')
const ActionApplicability$json = {
  '1': 'ActionApplicability',
  '2': [
    {'1': 'ACTION_APPLICABILITY_UNSPECIFIED', '2': 0},
    {'1': 'ACTION_APPLICABILITY_READY', '2': 1},
    {'1': 'ACTION_APPLICABILITY_NEEDS_CHOICE', '2': 2},
    {'1': 'ACTION_APPLICABILITY_BLOCKED', '2': 3},
  ],
};

/// Descriptor for `ActionApplicability`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List actionApplicabilityDescriptor = $convert
    .base64Decode('ChNBY3Rpb25BcHBsaWNhYmlsaXR5EiQKIEFDVElPTl9BUFBMSUNBQklMSVRZX1VOU1BFQ0lGSU'
        'VEEAASHgoaQUNUSU9OX0FQUExJQ0FCSUxJVFlfUkVBRFkQARIlCiFBQ1RJT05fQVBQTElDQUJJ'
        'TElUWV9ORUVEU19DSE9JQ0UQAhIgChxBQ1RJT05fQVBQTElDQUJJTElUWV9CTE9DS0VEEAM=');

@$core.Deprecated('Use roleHintDescriptor instead')
const RoleHint$json = {
  '1': 'RoleHint',
  '2': [
    {'1': 'ROLE_HINT_UNSPECIFIED', '2': 0},
    {'1': 'ROLE_HINT_INPUT', '2': 1},
    {'1': 'ROLE_HINT_OUTPUT', '2': 2},
    {'1': 'ROLE_HINT_EITHER', '2': 3},
  ],
};

/// Descriptor for `RoleHint`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List roleHintDescriptor = $convert
    .base64Decode('CghSb2xlSGludBIZChVST0xFX0hJTlRfVU5TUEVDSUZJRUQQABITCg9ST0xFX0hJTlRfSU5QVV'
        'QQARIUChBST0xFX0hJTlRfT1VUUFVUEAISFAoQUk9MRV9ISU5UX0VJVEhFUhAD');

@$core.Deprecated('Use deploymentStatusDescriptor instead')
const DeploymentStatus$json = {
  '1': 'DeploymentStatus',
  '2': [
    {'1': 'DEPLOYMENT_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'DEPLOYMENT_STATUS_FEASIBLE', '2': 1},
    {'1': 'DEPLOYMENT_STATUS_INFEASIBLE', '2': 2},
    {'1': 'DEPLOYMENT_STATUS_INCOMPLETE', '2': 3},
  ],
};

/// Descriptor for `DeploymentStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List deploymentStatusDescriptor = $convert
    .base64Decode('ChBEZXBsb3ltZW50U3RhdHVzEiEKHURFUExPWU1FTlRfU1RBVFVTX1VOU1BFQ0lGSUVEEAASHg'
        'oaREVQTE9ZTUVOVF9TVEFUVVNfRkVBU0lCTEUQARIgChxERVBMT1lNRU5UX1NUQVRVU19JTkZF'
        'QVNJQkxFEAISIAocREVQTE9ZTUVOVF9TVEFUVVNfSU5DT01QTEVURRAD');

@$core.Deprecated('Use missingKindDescriptor instead')
const MissingKind$json = {
  '1': 'MissingKind',
  '2': [
    {'1': 'MISSING_KIND_UNSPECIFIED', '2': 0},
    {'1': 'MISSING_KIND_RELATIONSHIP_NOT_CHECKING', '2': 1},
    {'1': 'MISSING_KIND_NOT_CAUSAL', '2': 2},
    {'1': 'MISSING_KIND_NOT_CLOCK_CONSISTENT', '2': 3},
    {'1': 'MISSING_KIND_OUTPUT_NO_DOMAIN', '2': 4},
    {'1': 'MISSING_KIND_OUTPUT_NO_DRIVER', '2': 5},
    {'1': 'MISSING_KIND_OUTPUT_CONNECTION_INVALID', '2': 6},
    {'1': 'MISSING_KIND_OUTPUT_NO_DEVICE', '2': 7},
    {'1': 'MISSING_KIND_DEVICE_NO_OUTPUT', '2': 8},
  ],
};

/// Descriptor for `MissingKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List missingKindDescriptor = $convert
    .base64Decode('CgtNaXNzaW5nS2luZBIcChhNSVNTSU5HX0tJTkRfVU5TUEVDSUZJRUQQABIqCiZNSVNTSU5HX0'
        'tJTkRfUkVMQVRJT05TSElQX05PVF9DSEVDS0lORxABEhsKF01JU1NJTkdfS0lORF9OT1RfQ0FV'
        'U0FMEAISJQohTUlTU0lOR19LSU5EX05PVF9DTE9DS19DT05TSVNURU5UEAMSIQodTUlTU0lOR1'
        '9LSU5EX09VVFBVVF9OT19ET01BSU4QBBIhCh1NSVNTSU5HX0tJTkRfT1VUUFVUX05PX0RSSVZF'
        'UhAFEioKJk1JU1NJTkdfS0lORF9PVVRQVVRfQ09OTkVDVElPTl9JTlZBTElEEAYSIQodTUlTU0'
        'lOR19LSU5EX09VVFBVVF9OT19ERVZJQ0UQBxIhCh1NSVNTSU5HX0tJTkRfREVWSUNFX05PX09V'
        'VFBVVBAI');

@$core.Deprecated('Use portKindDescriptor instead')
const PortKind$json = {
  '1': 'PortKind',
  '2': [
    {'1': 'PORT_KIND_UNSPECIFIED', '2': 0},
    {'1': 'PORT_KIND_REQUIRED', '2': 1},
    {'1': 'PORT_KIND_PROVIDED', '2': 2},
    {'1': 'PORT_KIND_PARAMETER', '2': 3},
  ],
};

/// Descriptor for `PortKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List portKindDescriptor = $convert
    .base64Decode('CghQb3J0S2luZBIZChVQT1JUX0tJTkRfVU5TUEVDSUZJRUQQABIWChJQT1JUX0tJTkRfUkVRVU'
        'lSRUQQARIWChJQT1JUX0tJTkRfUFJPVklERUQQAhIXChNQT1JUX0tJTkRfUEFSQU1FVEVSEAM=');

@$core.Deprecated('Use clockContractKindDescriptor instead')
const ClockContractKind$json = {
  '1': 'ClockContractKind',
  '2': [
    {'1': 'CLOCK_CONTRACT_KIND_UNSPECIFIED', '2': 0},
    {'1': 'CLOCK_CONTRACT_KIND_AGNOSTIC', '2': 1},
    {'1': 'CLOCK_CONTRACT_KIND_PARAMETER', '2': 2},
    {'1': 'CLOCK_CONTRACT_KIND_PRIVATE', '2': 3},
  ],
};

/// Descriptor for `ClockContractKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List clockContractKindDescriptor = $convert
    .base64Decode('ChFDbG9ja0NvbnRyYWN0S2luZBIjCh9DTE9DS19DT05UUkFDVF9LSU5EX1VOU1BFQ0lGSUVEEA'
        'ASIAocQ0xPQ0tfQ09OVFJBQ1RfS0lORF9BR05PU1RJQxABEiEKHUNMT0NLX0NPTlRSQUNUX0tJ'
        'TkRfUEFSQU1FVEVSEAISHwobQ0xPQ0tfQ09OVFJBQ1RfS0lORF9QUklWQVRFEAM=');

@$core.Deprecated('Use localSortDescriptor instead')
const LocalSort$json = {
  '1': 'LocalSort',
  '2': [
    {'1': 'LOCAL_SORT_UNSPECIFIED', '2': 0},
    {'1': 'LOCAL_SORT_DECL', '2': 1},
    {'1': 'LOCAL_SORT_SEM', '2': 2},
    {'1': 'LOCAL_SORT_CLOCK', '2': 3},
    {'1': 'LOCAL_SORT_OUTPUT', '2': 4},
    {'1': 'LOCAL_SORT_DEVICE', '2': 5},
  ],
};

/// Descriptor for `LocalSort`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List localSortDescriptor = $convert
    .base64Decode('CglMb2NhbFNvcnQSGgoWTE9DQUxfU09SVF9VTlNQRUNJRklFRBAAEhMKD0xPQ0FMX1NPUlRfRE'
        'VDTBABEhIKDkxPQ0FMX1NPUlRfU0VNEAISFAoQTE9DQUxfU09SVF9DTE9DSxADEhUKEUxPQ0FM'
        'X1NPUlRfT1VUUFVUEAQSFQoRTE9DQUxfU09SVF9ERVZJQ0UQBQ==');

@$core.Deprecated('Use portStatusKindDescriptor instead')
const PortStatusKind$json = {
  '1': 'PortStatusKind',
  '2': [
    {'1': 'PORT_STATUS_KIND_UNSPECIFIED', '2': 0},
    {'1': 'PORT_STATUS_KIND_PROVIDED', '2': 1},
    {'1': 'PORT_STATUS_KIND_BOUND', '2': 2},
    {'1': 'PORT_STATUS_KIND_EXPORTED', '2': 3},
    {'1': 'PORT_STATUS_KIND_VALUED', '2': 4},
    {'1': 'PORT_STATUS_KIND_OPEN', '2': 5},
  ],
};

/// Descriptor for `PortStatusKind`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List portStatusKindDescriptor = $convert
    .base64Decode('Cg5Qb3J0U3RhdHVzS2luZBIgChxQT1JUX1NUQVRVU19LSU5EX1VOU1BFQ0lGSUVEEAASHQoZUE'
        '9SVF9TVEFUVVNfS0lORF9QUk9WSURFRBABEhoKFlBPUlRfU1RBVFVTX0tJTkRfQk9VTkQQAhId'
        'ChlQT1JUX1NUQVRVU19LSU5EX0VYUE9SVEVEEAMSGwoXUE9SVF9TVEFUVVNfS0lORF9WQUxVRU'
        'QQBBIZChVQT1JUX1NUQVRVU19LSU5EX09QRU4QBQ==');

@$core.Deprecated('Use systemAcceptanceDescriptor instead')
const SystemAcceptance$json = {
  '1': 'SystemAcceptance',
  '2': [
    {'1': 'SYSTEM_ACCEPTANCE_UNSPECIFIED', '2': 0},
    {'1': 'SYSTEM_ACCEPTANCE_INVALID', '2': 1},
    {'1': 'SYSTEM_ACCEPTANCE_OPEN', '2': 2},
    {'1': 'SYSTEM_ACCEPTANCE_EXECUTABLE', '2': 3},
  ],
};

/// Descriptor for `SystemAcceptance`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List systemAcceptanceDescriptor = $convert
    .base64Decode('ChBTeXN0ZW1BY2NlcHRhbmNlEiEKHVNZU1RFTV9BQ0NFUFRBTkNFX1VOU1BFQ0lGSUVEEAASHQ'
        'oZU1lTVEVNX0FDQ0VQVEFOQ0VfSU5WQUxJRBABEhoKFlNZU1RFTV9BQ0NFUFRBTkNFX09QRU4Q'
        'AhIgChxTWVNURU1fQUNDRVBUQU5DRV9FWEVDVVRBQkxFEAM=');

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
    {
      '1': 'start_simulation',
      '3': 23,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.StartSimulationRequest',
      '9': 0,
      '10': 'startSimulation'
    },
    {
      '1': 'step_simulation',
      '3': 24,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.StepSimulationRequest',
      '9': 0,
      '10': 'stepSimulation'
    },
    {
      '1': 'reset_simulation',
      '3': 25,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ResetSimulationRequest',
      '9': 0,
      '10': 'resetSimulation'
    },
    {
      '1': 'list_targets',
      '3': 26,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ListTargetsRequest',
      '9': 0,
      '10': 'listTargets'
    },
    {
      '1': 'analyze_deployment',
      '3': 27,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AnalyzeDeploymentRequest',
      '9': 0,
      '10': 'analyzeDeployment'
    },
    {
      '1': 'analyze_definition_draft',
      '3': 28,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AnalyzeDefinitionDraftRequest',
      '9': 0,
      '10': 'analyzeDefinitionDraft'
    },
    {
      '1': 'discard_definition_draft',
      '3': 29,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DiscardDefinitionDraftRequest',
      '9': 0,
      '10': 'discardDefinitionDraft'
    },
    {
      '1': 'complete_definition_draft',
      '3': 30,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CompleteDefinitionDraftRequest',
      '9': 0,
      '10': 'completeDefinitionDraft'
    },
    {
      '1': 'hover_definition_draft',
      '3': 31,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.HoverDefinitionDraftRequest',
      '9': 0,
      '10': 'hoverDefinitionDraft'
    },
    {
      '1': 'hover_entity',
      '3': 32,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.HoverEntityRequest',
      '9': 0,
      '10': 'hoverEntity'
    },
    {
      '1': 'list_semantic_actions',
      '3': 33,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ListSemanticActionsRequest',
      '9': 0,
      '10': 'listSemanticActions'
    },
    {
      '1': 'get_formula_projection',
      '3': 34,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.GetFormulaProjectionRequest',
      '9': 0,
      '10': 'getFormulaProjection'
    },
    {
      '1': 'get_formula_slot',
      '3': 35,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.GetFormulaSlotRequest',
      '9': 0,
      '10': 'getFormulaSlot'
    },
    {
      '1': 'compose_formula',
      '3': 36,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ComposeFormulaRequest',
      '9': 0,
      '10': 'composeFormula'
    },
    {
      '1': 'list_concept_templates',
      '3': 40,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ListConceptTemplatesRequest',
      '9': 0,
      '10': 'listConceptTemplates'
    },
    {
      '1': 'instantiate_concept_template',
      '3': 41,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.InstantiateConceptTemplateRequest',
      '9': 0,
      '10': 'instantiateConceptTemplate'
    },
    {
      '1': 'init_system_project',
      '3': 50,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.InitSystemProjectRequest',
      '9': 0,
      '10': 'initSystemProject'
    },
    {
      '1': 'get_system',
      '3': 51,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.GetSystemRequest',
      '9': 0,
      '10': 'getSystem'
    },
    {
      '1': 'apply_system_edit',
      '3': 52,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ApplySystemEditRequest',
      '9': 0,
      '10': 'applySystemEdit'
    },
    {
      '1': 'run_system_analysis',
      '3': 53,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RunSystemAnalysisRequest',
      '9': 0,
      '10': 'runSystemAnalysis'
    },
    {
      '1': 'apply_group_edit',
      '3': 54,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ApplyGroupEditRequest',
      '9': 0,
      '10': 'applyGroupEdit'
    },
    {
      '1': 'preview_component_extraction',
      '3': 55,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.PreviewComponentExtractionRequest',
      '9': 0,
      '10': 'previewComponentExtraction'
    },
    {
      '1': 'init_text_project',
      '3': 60,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.InitTextProjectRequest',
      '9': 0,
      '10': 'initTextProject'
    },
    {
      '1': 'reload_project',
      '3': 61,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ReloadProjectRequest',
      '9': 0,
      '10': 'reloadProject'
    },
    {
      '1': 'get_sources',
      '3': 62,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.GetSourcesRequest',
      '9': 0,
      '10': 'getSources'
    },
    {
      '1': 'apply_source_edit',
      '3': 63,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ApplySourceEditRequest',
      '9': 0,
      '10': 'applySourceEdit'
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
        'MSSwoQc3RhcnRfc2ltdWxhdGlvbhgXIAEoCzIeLmJkbC52MS5TdGFydFNpbXVsYXRpb25SZXF1'
        'ZXN0SABSD3N0YXJ0U2ltdWxhdGlvbhJICg9zdGVwX3NpbXVsYXRpb24YGCABKAsyHS5iZGwudj'
        'EuU3RlcFNpbXVsYXRpb25SZXF1ZXN0SABSDnN0ZXBTaW11bGF0aW9uEksKEHJlc2V0X3NpbXVs'
        'YXRpb24YGSABKAsyHi5iZGwudjEuUmVzZXRTaW11bGF0aW9uUmVxdWVzdEgAUg9yZXNldFNpbX'
        'VsYXRpb24SPwoMbGlzdF90YXJnZXRzGBogASgLMhouYmRsLnYxLkxpc3RUYXJnZXRzUmVxdWVz'
        'dEgAUgtsaXN0VGFyZ2V0cxJRChJhbmFseXplX2RlcGxveW1lbnQYGyABKAsyIC5iZGwudjEuQW'
        '5hbHl6ZURlcGxveW1lbnRSZXF1ZXN0SABSEWFuYWx5emVEZXBsb3ltZW50EmEKGGFuYWx5emVf'
        'ZGVmaW5pdGlvbl9kcmFmdBgcIAEoCzIlLmJkbC52MS5BbmFseXplRGVmaW5pdGlvbkRyYWZ0Um'
        'VxdWVzdEgAUhZhbmFseXplRGVmaW5pdGlvbkRyYWZ0EmEKGGRpc2NhcmRfZGVmaW5pdGlvbl9k'
        'cmFmdBgdIAEoCzIlLmJkbC52MS5EaXNjYXJkRGVmaW5pdGlvbkRyYWZ0UmVxdWVzdEgAUhZkaX'
        'NjYXJkRGVmaW5pdGlvbkRyYWZ0EmQKGWNvbXBsZXRlX2RlZmluaXRpb25fZHJhZnQYHiABKAsy'
        'Ji5iZGwudjEuQ29tcGxldGVEZWZpbml0aW9uRHJhZnRSZXF1ZXN0SABSF2NvbXBsZXRlRGVmaW'
        '5pdGlvbkRyYWZ0ElsKFmhvdmVyX2RlZmluaXRpb25fZHJhZnQYHyABKAsyIy5iZGwudjEuSG92'
        'ZXJEZWZpbml0aW9uRHJhZnRSZXF1ZXN0SABSFGhvdmVyRGVmaW5pdGlvbkRyYWZ0Ej8KDGhvdm'
        'VyX2VudGl0eRggIAEoCzIaLmJkbC52MS5Ib3ZlckVudGl0eVJlcXVlc3RIAFILaG92ZXJFbnRp'
        'dHkSWAoVbGlzdF9zZW1hbnRpY19hY3Rpb25zGCEgASgLMiIuYmRsLnYxLkxpc3RTZW1hbnRpY0'
        'FjdGlvbnNSZXF1ZXN0SABSE2xpc3RTZW1hbnRpY0FjdGlvbnMSWwoWZ2V0X2Zvcm11bGFfcHJv'
        'amVjdGlvbhgiIAEoCzIjLmJkbC52MS5HZXRGb3JtdWxhUHJvamVjdGlvblJlcXVlc3RIAFIUZ2'
        'V0Rm9ybXVsYVByb2plY3Rpb24SSQoQZ2V0X2Zvcm11bGFfc2xvdBgjIAEoCzIdLmJkbC52MS5H'
        'ZXRGb3JtdWxhU2xvdFJlcXVlc3RIAFIOZ2V0Rm9ybXVsYVNsb3QSSAoPY29tcG9zZV9mb3JtdW'
        'xhGCQgASgLMh0uYmRsLnYxLkNvbXBvc2VGb3JtdWxhUmVxdWVzdEgAUg5jb21wb3NlRm9ybXVs'
        'YRJbChZsaXN0X2NvbmNlcHRfdGVtcGxhdGVzGCggASgLMiMuYmRsLnYxLkxpc3RDb25jZXB0VG'
        'VtcGxhdGVzUmVxdWVzdEgAUhRsaXN0Q29uY2VwdFRlbXBsYXRlcxJtChxpbnN0YW50aWF0ZV9j'
        'b25jZXB0X3RlbXBsYXRlGCkgASgLMikuYmRsLnYxLkluc3RhbnRpYXRlQ29uY2VwdFRlbXBsYX'
        'RlUmVxdWVzdEgAUhppbnN0YW50aWF0ZUNvbmNlcHRUZW1wbGF0ZRJSChNpbml0X3N5c3RlbV9w'
        'cm9qZWN0GDIgASgLMiAuYmRsLnYxLkluaXRTeXN0ZW1Qcm9qZWN0UmVxdWVzdEgAUhFpbml0U3'
        'lzdGVtUHJvamVjdBI5CgpnZXRfc3lzdGVtGDMgASgLMhguYmRsLnYxLkdldFN5c3RlbVJlcXVl'
        'c3RIAFIJZ2V0U3lzdGVtEkwKEWFwcGx5X3N5c3RlbV9lZGl0GDQgASgLMh4uYmRsLnYxLkFwcG'
        'x5U3lzdGVtRWRpdFJlcXVlc3RIAFIPYXBwbHlTeXN0ZW1FZGl0ElIKE3J1bl9zeXN0ZW1fYW5h'
        'bHlzaXMYNSABKAsyIC5iZGwudjEuUnVuU3lzdGVtQW5hbHlzaXNSZXF1ZXN0SABSEXJ1blN5c3'
        'RlbUFuYWx5c2lzEkkKEGFwcGx5X2dyb3VwX2VkaXQYNiABKAsyHS5iZGwudjEuQXBwbHlHcm91'
        'cEVkaXRSZXF1ZXN0SABSDmFwcGx5R3JvdXBFZGl0Em0KHHByZXZpZXdfY29tcG9uZW50X2V4dH'
        'JhY3Rpb24YNyABKAsyKS5iZGwudjEuUHJldmlld0NvbXBvbmVudEV4dHJhY3Rpb25SZXF1ZXN0'
        'SABSGnByZXZpZXdDb21wb25lbnRFeHRyYWN0aW9uEkwKEWluaXRfdGV4dF9wcm9qZWN0GDwgAS'
        'gLMh4uYmRsLnYxLkluaXRUZXh0UHJvamVjdFJlcXVlc3RIAFIPaW5pdFRleHRQcm9qZWN0EkUK'
        'DnJlbG9hZF9wcm9qZWN0GD0gASgLMhwuYmRsLnYxLlJlbG9hZFByb2plY3RSZXF1ZXN0SABSDX'
        'JlbG9hZFByb2plY3QSPAoLZ2V0X3NvdXJjZXMYPiABKAsyGS5iZGwudjEuR2V0U291cmNlc1Jl'
        'cXVlc3RIAFIKZ2V0U291cmNlcxJMChFhcHBseV9zb3VyY2VfZWRpdBg/IAEoCzIeLmJkbC52MS'
        '5BcHBseVNvdXJjZUVkaXRSZXF1ZXN0SABSD2FwcGx5U291cmNlRWRpdEIJCgdwYXlsb2Fk');

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
    {
      '1': 'simulation',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SimulationResponse',
      '9': 0,
      '10': 'simulation'
    },
    {
      '1': 'targets',
      '3': 16,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TargetsResponse',
      '9': 0,
      '10': 'targets'
    },
    {
      '1': 'deployment',
      '3': 17,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeploymentResponse',
      '9': 0,
      '10': 'deployment'
    },
    {
      '1': 'definition_draft',
      '3': 18,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DefinitionDraftAnalysis',
      '9': 0,
      '10': 'definitionDraft'
    },
    {
      '1': 'draft_completion',
      '3': 19,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DraftCompletionResponse',
      '9': 0,
      '10': 'draftCompletion'
    },
    {
      '1': 'draft_hover',
      '3': 20,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DraftHoverResponse',
      '9': 0,
      '10': 'draftHover'
    },
    {
      '1': 'semantic_actions',
      '3': 21,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SemanticActionsResponse',
      '9': 0,
      '10': 'semanticActions'
    },
    {
      '1': 'concept_templates',
      '3': 30,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ConceptTemplatesResponse',
      '9': 0,
      '10': 'conceptTemplates'
    },
    {
      '1': 'system',
      '3': 40,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SystemResponse',
      '9': 0,
      '10': 'system'
    },
    {
      '1': 'system_edit_applied',
      '3': 41,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SystemEditApplied',
      '9': 0,
      '10': 'systemEditApplied'
    },
    {
      '1': 'system_analysis',
      '3': 42,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SystemAnalysisResponse',
      '9': 0,
      '10': 'systemAnalysis'
    },
    {
      '1': 'extraction_preview',
      '3': 43,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ExtractionPreviewResponse',
      '9': 0,
      '10': 'extractionPreview'
    },
    {
      '1': 'sources',
      '3': 44,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SourcesResponse',
      '9': 0,
      '10': 'sources'
    },
    {
      '1': 'source_edit_applied',
      '3': 45,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SourceEditApplied',
      '9': 0,
      '10': 'sourceEditApplied'
    },
    {
      '1': 'formula_projection',
      '3': 46,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.FormulaProjectionResponse',
      '9': 0,
      '10': 'formulaProjection'
    },
    {
      '1': 'formula_slot',
      '3': 47,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.FormulaSlotResponse',
      '9': 0,
      '10': 'formulaSlot'
    },
    {
      '1': 'compose_formula',
      '3': 48,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ComposeFormulaResponse',
      '9': 0,
      '10': 'composeFormula'
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
        'YWx5c2lzEjwKCnNpbXVsYXRpb24YDyABKAsyGi5iZGwudjEuU2ltdWxhdGlvblJlc3BvbnNlSA'
        'BSCnNpbXVsYXRpb24SMwoHdGFyZ2V0cxgQIAEoCzIXLmJkbC52MS5UYXJnZXRzUmVzcG9uc2VI'
        'AFIHdGFyZ2V0cxI8CgpkZXBsb3ltZW50GBEgASgLMhouYmRsLnYxLkRlcGxveW1lbnRSZXNwb2'
        '5zZUgAUgpkZXBsb3ltZW50EkwKEGRlZmluaXRpb25fZHJhZnQYEiABKAsyHy5iZGwudjEuRGVm'
        'aW5pdGlvbkRyYWZ0QW5hbHlzaXNIAFIPZGVmaW5pdGlvbkRyYWZ0EkwKEGRyYWZ0X2NvbXBsZX'
        'Rpb24YEyABKAsyHy5iZGwudjEuRHJhZnRDb21wbGV0aW9uUmVzcG9uc2VIAFIPZHJhZnRDb21w'
        'bGV0aW9uEj0KC2RyYWZ0X2hvdmVyGBQgASgLMhouYmRsLnYxLkRyYWZ0SG92ZXJSZXNwb25zZU'
        'gAUgpkcmFmdEhvdmVyEkwKEHNlbWFudGljX2FjdGlvbnMYFSABKAsyHy5iZGwudjEuU2VtYW50'
        'aWNBY3Rpb25zUmVzcG9uc2VIAFIPc2VtYW50aWNBY3Rpb25zEk8KEWNvbmNlcHRfdGVtcGxhdG'
        'VzGB4gASgLMiAuYmRsLnYxLkNvbmNlcHRUZW1wbGF0ZXNSZXNwb25zZUgAUhBjb25jZXB0VGVt'
        'cGxhdGVzEjAKBnN5c3RlbRgoIAEoCzIWLmJkbC52MS5TeXN0ZW1SZXNwb25zZUgAUgZzeXN0ZW'
        '0SSwoTc3lzdGVtX2VkaXRfYXBwbGllZBgpIAEoCzIZLmJkbC52MS5TeXN0ZW1FZGl0QXBwbGll'
        'ZEgAUhFzeXN0ZW1FZGl0QXBwbGllZBJJCg9zeXN0ZW1fYW5hbHlzaXMYKiABKAsyHi5iZGwudj'
        'EuU3lzdGVtQW5hbHlzaXNSZXNwb25zZUgAUg5zeXN0ZW1BbmFseXNpcxJSChJleHRyYWN0aW9u'
        'X3ByZXZpZXcYKyABKAsyIS5iZGwudjEuRXh0cmFjdGlvblByZXZpZXdSZXNwb25zZUgAUhFleH'
        'RyYWN0aW9uUHJldmlldxIzCgdzb3VyY2VzGCwgASgLMhcuYmRsLnYxLlNvdXJjZXNSZXNwb25z'
        'ZUgAUgdzb3VyY2VzEksKE3NvdXJjZV9lZGl0X2FwcGxpZWQYLSABKAsyGS5iZGwudjEuU291cm'
        'NlRWRpdEFwcGxpZWRIAFIRc291cmNlRWRpdEFwcGxpZWQSUgoSZm9ybXVsYV9wcm9qZWN0aW9u'
        'GC4gASgLMiEuYmRsLnYxLkZvcm11bGFQcm9qZWN0aW9uUmVzcG9uc2VIAFIRZm9ybXVsYVByb2'
        'plY3Rpb24SQAoMZm9ybXVsYV9zbG90GC8gASgLMhsuYmRsLnYxLkZvcm11bGFTbG90UmVzcG9u'
        'c2VIAFILZm9ybXVsYVNsb3QSSQoPY29tcG9zZV9mb3JtdWxhGDAgASgLMh4uYmRsLnYxLkNvbX'
        'Bvc2VGb3JtdWxhUmVzcG9uc2VIAFIOY29tcG9zZUZvcm11bGFCCQoHcGF5bG9hZA==');

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
  '2': [
    {'1': 'force', '3': 1, '4': 1, '5': 8, '10': 'force'},
  ],
};

/// Descriptor for `SaveProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List saveProjectRequestDescriptor =
    $convert.base64Decode('ChJTYXZlUHJvamVjdFJlcXVlc3QSFAoFZm9yY2UYASABKAhSBWZvcmNl');

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
    {
      '1': 'create_clock_domain',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateClockDomain',
      '9': 0,
      '10': 'createClockDomain'
    },
    {
      '1': 'rename_clock_domain',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameClockDomain',
      '9': 0,
      '10': 'renameClockDomain'
    },
    {
      '1': 'delete_clock_domain',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteClockDomain',
      '9': 0,
      '10': 'deleteClockDomain'
    },
    {
      '1': 'set_mapping_clock',
      '3': 16,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetMappingClock',
      '9': 0,
      '10': 'setMappingClock'
    },
    {
      '1': 'create_output',
      '3': 17,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateOutput',
      '9': 0,
      '10': 'createOutput'
    },
    {
      '1': 'rename_output',
      '3': 18,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameOutput',
      '9': 0,
      '10': 'renameOutput'
    },
    {
      '1': 'set_output_accepts',
      '3': 19,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetOutputAccepts',
      '9': 0,
      '10': 'setOutputAccepts'
    },
    {
      '1': 'set_output_clock',
      '3': 20,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetOutputClock',
      '9': 0,
      '10': 'setOutputClock'
    },
    {
      '1': 'set_output_required',
      '3': 21,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetOutputRequired',
      '9': 0,
      '10': 'setOutputRequired'
    },
    {
      '1': 'delete_output',
      '3': 22,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteOutput',
      '9': 0,
      '10': 'deleteOutput'
    },
    {
      '1': 'set_mapping_drive',
      '3': 23,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetMappingDrive',
      '9': 0,
      '10': 'setMappingDrive'
    },
    {
      '1': 'create_device',
      '3': 24,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateDevice',
      '9': 0,
      '10': 'createDevice'
    },
    {
      '1': 'rename_device',
      '3': 25,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameDevice',
      '9': 0,
      '10': 'renameDevice'
    },
    {
      '1': 'set_device_kind',
      '3': 26,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetDeviceKind',
      '9': 0,
      '10': 'setDeviceKind'
    },
    {
      '1': 'set_device_output',
      '3': 27,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetDeviceOutput',
      '9': 0,
      '10': 'setDeviceOutput'
    },
    {
      '1': 'set_device_pin',
      '3': 28,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetDevicePin',
      '9': 0,
      '10': 'setDevicePin'
    },
    {
      '1': 'delete_device',
      '3': 29,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteDevice',
      '9': 0,
      '10': 'deleteDevice'
    },
    {
      '1': 'set_concept_ordered',
      '3': 30,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetConceptOrdered',
      '9': 0,
      '10': 'setConceptOrdered'
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
        'ZWxldGVNYXBwaW5nEksKE2NyZWF0ZV9jbG9ja19kb21haW4YDSABKAsyGS5iZGwudjEuQ3JlYX'
        'RlQ2xvY2tEb21haW5IAFIRY3JlYXRlQ2xvY2tEb21haW4SSwoTcmVuYW1lX2Nsb2NrX2RvbWFp'
        'bhgOIAEoCzIZLmJkbC52MS5SZW5hbWVDbG9ja0RvbWFpbkgAUhFyZW5hbWVDbG9ja0RvbWFpbh'
        'JLChNkZWxldGVfY2xvY2tfZG9tYWluGA8gASgLMhkuYmRsLnYxLkRlbGV0ZUNsb2NrRG9tYWlu'
        'SABSEWRlbGV0ZUNsb2NrRG9tYWluEkUKEXNldF9tYXBwaW5nX2Nsb2NrGBAgASgLMhcuYmRsLn'
        'YxLlNldE1hcHBpbmdDbG9ja0gAUg9zZXRNYXBwaW5nQ2xvY2sSOwoNY3JlYXRlX291dHB1dBgR'
        'IAEoCzIULmJkbC52MS5DcmVhdGVPdXRwdXRIAFIMY3JlYXRlT3V0cHV0EjsKDXJlbmFtZV9vdX'
        'RwdXQYEiABKAsyFC5iZGwudjEuUmVuYW1lT3V0cHV0SABSDHJlbmFtZU91dHB1dBJIChJzZXRf'
        'b3V0cHV0X2FjY2VwdHMYEyABKAsyGC5iZGwudjEuU2V0T3V0cHV0QWNjZXB0c0gAUhBzZXRPdX'
        'RwdXRBY2NlcHRzEkIKEHNldF9vdXRwdXRfY2xvY2sYFCABKAsyFi5iZGwudjEuU2V0T3V0cHV0'
        'Q2xvY2tIAFIOc2V0T3V0cHV0Q2xvY2sSSwoTc2V0X291dHB1dF9yZXF1aXJlZBgVIAEoCzIZLm'
        'JkbC52MS5TZXRPdXRwdXRSZXF1aXJlZEgAUhFzZXRPdXRwdXRSZXF1aXJlZBI7Cg1kZWxldGVf'
        'b3V0cHV0GBYgASgLMhQuYmRsLnYxLkRlbGV0ZU91dHB1dEgAUgxkZWxldGVPdXRwdXQSRQoRc2'
        'V0X21hcHBpbmdfZHJpdmUYFyABKAsyFy5iZGwudjEuU2V0TWFwcGluZ0RyaXZlSABSD3NldE1h'
        'cHBpbmdEcml2ZRI7Cg1jcmVhdGVfZGV2aWNlGBggASgLMhQuYmRsLnYxLkNyZWF0ZURldmljZU'
        'gAUgxjcmVhdGVEZXZpY2USOwoNcmVuYW1lX2RldmljZRgZIAEoCzIULmJkbC52MS5SZW5hbWVE'
        'ZXZpY2VIAFIMcmVuYW1lRGV2aWNlEj8KD3NldF9kZXZpY2Vfa2luZBgaIAEoCzIVLmJkbC52MS'
        '5TZXREZXZpY2VLaW5kSABSDXNldERldmljZUtpbmQSRQoRc2V0X2RldmljZV9vdXRwdXQYGyAB'
        'KAsyFy5iZGwudjEuU2V0RGV2aWNlT3V0cHV0SABSD3NldERldmljZU91dHB1dBI8Cg5zZXRfZG'
        'V2aWNlX3BpbhgcIAEoCzIULmJkbC52MS5TZXREZXZpY2VQaW5IAFIMc2V0RGV2aWNlUGluEjsK'
        'DWRlbGV0ZV9kZXZpY2UYHSABKAsyFC5iZGwudjEuRGVsZXRlRGV2aWNlSABSDGRlbGV0ZURldm'
        'ljZRJLChNzZXRfY29uY2VwdF9vcmRlcmVkGB4gASgLMhkuYmRsLnYxLlNldENvbmNlcHRPcmRl'
        'cmVkSABSEXNldENvbmNlcHRPcmRlcmVkQgQKAm9w');

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

@$core.Deprecated('Use setConceptOrderedDescriptor instead')
const SetConceptOrdered$json = {
  '1': 'SetConceptOrdered',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'ordered', '3': 2, '4': 1, '5': 8, '10': 'ordered'},
  ],
};

/// Descriptor for `SetConceptOrdered`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setConceptOrderedDescriptor = $convert
    .base64Decode('ChFTZXRDb25jZXB0T3JkZXJlZBIOCgJpZBgBIAEoBFICaWQSGAoHb3JkZXJlZBgCIAEoCFIHb3'
        'JkZXJlZA==');

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

@$core.Deprecated('Use createClockDomainDescriptor instead')
const CreateClockDomain$json = {
  '1': 'CreateClockDomain',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `CreateClockDomain`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createClockDomainDescriptor =
    $convert.base64Decode('ChFDcmVhdGVDbG9ja0RvbWFpbhISCgRuYW1lGAEgASgJUgRuYW1l');

@$core.Deprecated('Use renameClockDomainDescriptor instead')
const RenameClockDomain$json = {
  '1': 'RenameClockDomain',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameClockDomain`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameClockDomainDescriptor = $convert
    .base64Decode('ChFSZW5hbWVDbG9ja0RvbWFpbhIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZQ'
        '==');

@$core.Deprecated('Use deleteClockDomainDescriptor instead')
const DeleteClockDomain$json = {
  '1': 'DeleteClockDomain',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteClockDomain`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteClockDomainDescriptor =
    $convert.base64Decode('ChFEZWxldGVDbG9ja0RvbWFpbhIOCgJpZBgBIAEoBFICaWQ=');

@$core.Deprecated('Use setMappingClockDescriptor instead')
const SetMappingClock$json = {
  '1': 'SetMappingClock',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'clock_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'clockId', '17': true},
  ],
  '8': [
    {'1': '_clock_id'},
  ],
};

/// Descriptor for `SetMappingClock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setMappingClockDescriptor = $convert
    .base64Decode('Cg9TZXRNYXBwaW5nQ2xvY2sSDgoCaWQYASABKARSAmlkEh4KCGNsb2NrX2lkGAIgASgESABSB2'
        'Nsb2NrSWSIAQFCCwoJX2Nsb2NrX2lk');

@$core.Deprecated('Use createOutputDescriptor instead')
const CreateOutput$json = {
  '1': 'CreateOutput',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'accepts', '3': 3, '4': 1, '5': 4, '10': 'accepts'},
    {'1': 'clock_id', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'clockId', '17': true},
  ],
  '8': [
    {'1': '_clock_id'},
  ],
};

/// Descriptor for `CreateOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createOutputDescriptor = $convert
    .base64Decode('CgxDcmVhdGVPdXRwdXQSEgoEbmFtZRgBIAEoCVIEbmFtZRIgCgtkZXNjcmlwdGlvbhgCIAEoCV'
        'ILZGVzY3JpcHRpb24SGAoHYWNjZXB0cxgDIAEoBFIHYWNjZXB0cxIeCghjbG9ja19pZBgEIAEo'
        'BEgAUgdjbG9ja0lkiAEBQgsKCV9jbG9ja19pZA==');

@$core.Deprecated('Use renameOutputDescriptor instead')
const RenameOutput$json = {
  '1': 'RenameOutput',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameOutputDescriptor =
    $convert.base64Decode('CgxSZW5hbWVPdXRwdXQSDgoCaWQYASABKARSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWU=');

@$core.Deprecated('Use setOutputAcceptsDescriptor instead')
const SetOutputAccepts$json = {
  '1': 'SetOutputAccepts',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'accepts', '3': 2, '4': 1, '5': 4, '10': 'accepts'},
  ],
};

/// Descriptor for `SetOutputAccepts`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setOutputAcceptsDescriptor = $convert
    .base64Decode('ChBTZXRPdXRwdXRBY2NlcHRzEg4KAmlkGAEgASgEUgJpZBIYCgdhY2NlcHRzGAIgASgEUgdhY2'
        'NlcHRz');

@$core.Deprecated('Use setOutputClockDescriptor instead')
const SetOutputClock$json = {
  '1': 'SetOutputClock',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'clock_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'clockId', '17': true},
  ],
  '8': [
    {'1': '_clock_id'},
  ],
};

/// Descriptor for `SetOutputClock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setOutputClockDescriptor = $convert
    .base64Decode('Cg5TZXRPdXRwdXRDbG9jaxIOCgJpZBgBIAEoBFICaWQSHgoIY2xvY2tfaWQYAiABKARIAFIHY2'
        'xvY2tJZIgBAUILCglfY2xvY2tfaWQ=');

@$core.Deprecated('Use setOutputRequiredDescriptor instead')
const SetOutputRequired$json = {
  '1': 'SetOutputRequired',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'required', '3': 2, '4': 1, '5': 8, '10': 'required'},
  ],
};

/// Descriptor for `SetOutputRequired`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setOutputRequiredDescriptor = $convert
    .base64Decode('ChFTZXRPdXRwdXRSZXF1aXJlZBIOCgJpZBgBIAEoBFICaWQSGgoIcmVxdWlyZWQYAiABKAhSCH'
        'JlcXVpcmVk');

@$core.Deprecated('Use deleteOutputDescriptor instead')
const DeleteOutput$json = {
  '1': 'DeleteOutput',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteOutputDescriptor =
    $convert.base64Decode('CgxEZWxldGVPdXRwdXQSDgoCaWQYASABKARSAmlk');

@$core.Deprecated('Use setMappingDriveDescriptor instead')
const SetMappingDrive$json = {
  '1': 'SetMappingDrive',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'output_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
  ],
  '8': [
    {'1': '_output_id'},
  ],
};

/// Descriptor for `SetMappingDrive`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setMappingDriveDescriptor = $convert
    .base64Decode('Cg9TZXRNYXBwaW5nRHJpdmUSDgoCaWQYASABKARSAmlkEiAKCW91dHB1dF9pZBgCIAEoBEgAUg'
        'hvdXRwdXRJZIgBAUIMCgpfb3V0cHV0X2lk');

@$core.Deprecated('Use createDeviceDescriptor instead')
const CreateDevice$json = {
  '1': 'CreateDevice',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.DeviceKind', '10': 'kind'},
    {'1': 'output_id', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
  ],
  '8': [
    {'1': '_output_id'},
  ],
};

/// Descriptor for `CreateDevice`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createDeviceDescriptor = $convert
    .base64Decode('CgxDcmVhdGVEZXZpY2USEgoEbmFtZRgBIAEoCVIEbmFtZRImCgRraW5kGAIgASgOMhIuYmRsLn'
        'YxLkRldmljZUtpbmRSBGtpbmQSIAoJb3V0cHV0X2lkGAMgASgESABSCG91dHB1dElkiAEBQgwK'
        'Cl9vdXRwdXRfaWQ=');

@$core.Deprecated('Use renameDeviceDescriptor instead')
const RenameDevice$json = {
  '1': 'RenameDevice',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameDevice`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameDeviceDescriptor =
    $convert.base64Decode('CgxSZW5hbWVEZXZpY2USDgoCaWQYASABKARSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWU=');

@$core.Deprecated('Use setDeviceKindDescriptor instead')
const SetDeviceKind$json = {
  '1': 'SetDeviceKind',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'kind', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.DeviceKind', '10': 'kind'},
  ],
};

/// Descriptor for `SetDeviceKind`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setDeviceKindDescriptor = $convert
    .base64Decode('Cg1TZXREZXZpY2VLaW5kEg4KAmlkGAEgASgEUgJpZBImCgRraW5kGAIgASgOMhIuYmRsLnYxLk'
        'RldmljZUtpbmRSBGtpbmQ=');

@$core.Deprecated('Use setDeviceOutputDescriptor instead')
const SetDeviceOutput$json = {
  '1': 'SetDeviceOutput',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'output_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
  ],
  '8': [
    {'1': '_output_id'},
  ],
};

/// Descriptor for `SetDeviceOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setDeviceOutputDescriptor = $convert
    .base64Decode('Cg9TZXREZXZpY2VPdXRwdXQSDgoCaWQYASABKARSAmlkEiAKCW91dHB1dF9pZBgCIAEoBEgAUg'
        'hvdXRwdXRJZIgBAUIMCgpfb3V0cHV0X2lk');

@$core.Deprecated('Use setDevicePinDescriptor instead')
const SetDevicePin$json = {
  '1': 'SetDevicePin',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'index', '3': 2, '4': 1, '5': 13, '10': 'index'},
    {'1': 'resource', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'resource', '17': true},
  ],
  '8': [
    {'1': '_resource'},
  ],
};

/// Descriptor for `SetDevicePin`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setDevicePinDescriptor = $convert
    .base64Decode('CgxTZXREZXZpY2VQaW4SDgoCaWQYASABKARSAmlkEhQKBWluZGV4GAIgASgNUgVpbmRleBIfCg'
        'hyZXNvdXJjZRgDIAEoCUgAUghyZXNvdXJjZYgBAUILCglfcmVzb3VyY2U=');

@$core.Deprecated('Use deleteDeviceDescriptor instead')
const DeleteDevice$json = {
  '1': 'DeleteDevice',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteDevice`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteDeviceDescriptor =
    $convert.base64Decode('CgxEZWxldGVEZXZpY2USDgoCaWQYASABKARSAmlk');

@$core.Deprecated('Use editOutcomeDescriptor instead')
const EditOutcome$json = {
  '1': 'EditOutcome',
  '2': [
    {'1': 'kind', '3': 1, '4': 1, '5': 14, '6': '.bdl.v1.EditKind', '10': 'kind'},
    {'1': 'invalidates', '3': 2, '4': 3, '5': 14, '6': '.bdl.v1.Invalidation', '10': 'invalidates'},
    {'1': 'origin_decls', '3': 3, '4': 3, '5': 4, '10': 'originDecls'},
    {'1': 'created_concept', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'createdConcept', '17': true},
    {'1': 'created_mapping', '3': 5, '4': 1, '5': 4, '9': 1, '10': 'createdMapping', '17': true},
    {'1': 'created_clock', '3': 6, '4': 1, '5': 4, '9': 2, '10': 'createdClock', '17': true},
    {'1': 'created_output', '3': 7, '4': 1, '5': 4, '9': 3, '10': 'createdOutput', '17': true},
    {'1': 'created_device', '3': 8, '4': 1, '5': 4, '9': 4, '10': 'createdDevice', '17': true},
  ],
  '8': [
    {'1': '_created_concept'},
    {'1': '_created_mapping'},
    {'1': '_created_clock'},
    {'1': '_created_output'},
    {'1': '_created_device'},
  ],
};

/// Descriptor for `EditOutcome`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List editOutcomeDescriptor = $convert
    .base64Decode('CgtFZGl0T3V0Y29tZRIkCgRraW5kGAEgASgOMhAuYmRsLnYxLkVkaXRLaW5kUgRraW5kEjYKC2'
        'ludmFsaWRhdGVzGAIgAygOMhQuYmRsLnYxLkludmFsaWRhdGlvblILaW52YWxpZGF0ZXMSIQoM'
        'b3JpZ2luX2RlY2xzGAMgAygEUgtvcmlnaW5EZWNscxIsCg9jcmVhdGVkX2NvbmNlcHQYBCABKA'
        'RIAFIOY3JlYXRlZENvbmNlcHSIAQESLAoPY3JlYXRlZF9tYXBwaW5nGAUgASgESAFSDmNyZWF0'
        'ZWRNYXBwaW5niAEBEigKDWNyZWF0ZWRfY2xvY2sYBiABKARIAlIMY3JlYXRlZENsb2NriAEBEi'
        'oKDmNyZWF0ZWRfb3V0cHV0GAcgASgESANSDWNyZWF0ZWRPdXRwdXSIAQESKgoOY3JlYXRlZF9k'
        'ZXZpY2UYCCABKARIBFINY3JlYXRlZERldmljZYgBAUISChBfY3JlYXRlZF9jb25jZXB0QhIKEF'
        '9jcmVhdGVkX21hcHBpbmdCEAoOX2NyZWF0ZWRfY2xvY2tCEQoPX2NyZWF0ZWRfb3V0cHV0QhEK'
        'D19jcmVhdGVkX2RldmljZQ==');

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
    {'1': 'clocks', '3': 10, '4': 3, '5': 11, '6': '.bdl.v1.ClockView', '10': 'clocks'},
    {'1': 'outputs', '3': 11, '4': 3, '5': 11, '6': '.bdl.v1.OutputView', '10': 'outputs'},
    {'1': 'devices', '3': 12, '4': 3, '5': 11, '6': '.bdl.v1.DeviceView', '10': 'devices'},
    {'1': 'kind', '3': 13, '4': 1, '5': 14, '6': '.bdl.v1.ProjectKind', '10': 'kind'},
  ],
};

/// Descriptor for `ProjectProjection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectProjectionDescriptor = $convert
    .base64Decode('ChFQcm9qZWN0UHJvamVjdGlvbhIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SEgoEbmFtZR'
        'gCIAEoCVIEbmFtZRIbCglyb290X3BhdGgYAyABKAlSCHJvb3RQYXRoEi8KCGNvbmNlcHRzGAQg'
        'AygLMhMuYmRsLnYxLkNvbmNlcHRWaWV3Ughjb25jZXB0cxIvCghtYXBwaW5ncxgFIAMoCzITLm'
        'JkbC52MS5NYXBwaW5nVmlld1IIbWFwcGluZ3MSJgoGbGF5b3V0GAYgASgLMg4uYmRsLnYxLkxh'
        'eW91dFIGbGF5b3V0EhkKCGNhbl91bmRvGAcgASgIUgdjYW5VbmRvEhkKCGNhbl9yZWRvGAggAS'
        'gIUgdjYW5SZWRvEhQKBWRpcnR5GAkgASgIUgVkaXJ0eRIpCgZjbG9ja3MYCiADKAsyES5iZGwu'
        'djEuQ2xvY2tWaWV3UgZjbG9ja3MSLAoHb3V0cHV0cxgLIAMoCzISLmJkbC52MS5PdXRwdXRWaW'
        'V3UgdvdXRwdXRzEiwKB2RldmljZXMYDCADKAsyEi5iZGwudjEuRGV2aWNlVmlld1IHZGV2aWNl'
        'cxInCgRraW5kGA0gASgOMhMuYmRsLnYxLlByb2plY3RLaW5kUgRraW5k');

@$core.Deprecated('Use clockViewDescriptor instead')
const ClockView$json = {
  '1': 'ClockView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `ClockView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clockViewDescriptor =
    $convert.base64Decode('CglDbG9ja1ZpZXcSDgoCaWQYASABKARSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWU=');

@$core.Deprecated('Use outputViewDescriptor instead')
const OutputView$json = {
  '1': 'OutputView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'accepts', '3': 4, '4': 1, '5': 4, '10': 'accepts'},
    {'1': 'clock_id', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'clockId', '17': true},
    {'1': 'required', '3': 6, '4': 1, '5': 8, '10': 'required'},
  ],
  '8': [
    {'1': '_clock_id'},
  ],
};

/// Descriptor for `OutputView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List outputViewDescriptor = $convert
    .base64Decode('CgpPdXRwdXRWaWV3Eg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiAKC2Rlc2'
        'NyaXB0aW9uGAMgASgJUgtkZXNjcmlwdGlvbhIYCgdhY2NlcHRzGAQgASgEUgdhY2NlcHRzEh4K'
        'CGNsb2NrX2lkGAUgASgESABSB2Nsb2NrSWSIAQESGgoIcmVxdWlyZWQYBiABKAhSCHJlcXVpcm'
        'VkQgsKCV9jbG9ja19pZA==');

@$core.Deprecated('Use deviceViewDescriptor instead')
const DeviceView$json = {
  '1': 'DeviceView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 3, '4': 1, '5': 14, '6': '.bdl.v1.DeviceKind', '10': 'kind'},
    {'1': 'output_id', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
    {'1': 'fixed_pins', '3': 5, '4': 3, '5': 11, '6': '.bdl.v1.DevicePin', '10': 'fixedPins'},
    {
      '1': 'requirements',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.RequirementLabel',
      '10': 'requirements'
    },
  ],
  '8': [
    {'1': '_output_id'},
  ],
};

/// Descriptor for `DeviceView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deviceViewDescriptor = $convert
    .base64Decode('CgpEZXZpY2VWaWV3Eg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiYKBGtpbm'
        'QYAyABKA4yEi5iZGwudjEuRGV2aWNlS2luZFIEa2luZBIgCglvdXRwdXRfaWQYBCABKARIAFII'
        'b3V0cHV0SWSIAQESMAoKZml4ZWRfcGlucxgFIAMoCzIRLmJkbC52MS5EZXZpY2VQaW5SCWZpeG'
        'VkUGlucxI8CgxyZXF1aXJlbWVudHMYBiADKAsyGC5iZGwudjEuUmVxdWlyZW1lbnRMYWJlbFIM'
        'cmVxdWlyZW1lbnRzQgwKCl9vdXRwdXRfaWQ=');

@$core.Deprecated('Use devicePinDescriptor instead')
const DevicePin$json = {
  '1': 'DevicePin',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 13, '10': 'index'},
    {'1': 'resource', '3': 2, '4': 1, '5': 9, '10': 'resource'},
  ],
};

/// Descriptor for `DevicePin`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List devicePinDescriptor = $convert
    .base64Decode('CglEZXZpY2VQaW4SFAoFaW5kZXgYASABKA1SBWluZGV4EhoKCHJlc291cmNlGAIgASgJUghyZX'
        'NvdXJjZQ==');

@$core.Deprecated('Use requirementLabelDescriptor instead')
const RequirementLabel$json = {
  '1': 'RequirementLabel',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 13, '10': 'index'},
    {'1': 'capability', '3': 2, '4': 1, '5': 9, '10': 'capability'},
    {'1': 'label', '3': 3, '4': 1, '5': 9, '10': 'label'},
  ],
};

/// Descriptor for `RequirementLabel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requirementLabelDescriptor = $convert
    .base64Decode('ChBSZXF1aXJlbWVudExhYmVsEhQKBWluZGV4GAEgASgNUgVpbmRleBIeCgpjYXBhYmlsaXR5GA'
        'IgASgJUgpjYXBhYmlsaXR5EhQKBWxhYmVsGAMgASgJUgVsYWJlbA==');

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
    {'1': 'ordered', '3': 5, '4': 1, '5': 8, '10': 'ordered'},
  ],
  '8': [
    {'1': '_representation'},
  ],
};

/// Descriptor for `ConceptView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List conceptViewDescriptor = $convert
    .base64Decode('CgtDb25jZXB0VmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCgtkZX'
        'NjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SQwoOcmVwcmVzZW50YXRpb24YBCABKAsyFi5i'
        'ZGwudjEuUmVwcmVzZW50YXRpb25IAFIOcmVwcmVzZW50YXRpb26IAQESGAoHb3JkZXJlZBgFIA'
        'EoCFIHb3JkZXJlZEIRCg9fcmVwcmVzZW50YXRpb24=');

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
    {'1': 'clock_id', '3': 7, '4': 1, '5': 4, '9': 1, '10': 'clockId', '17': true},
    {'1': 'drives_output_id', '3': 8, '4': 1, '5': 4, '9': 2, '10': 'drivesOutputId', '17': true},
  ],
  '8': [
    {'1': '_definition'},
    {'1': '_clock_id'},
    {'1': '_drives_output_id'},
  ],
};

/// Descriptor for `MappingView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mappingViewDescriptor = $convert
    .base64Decode('CgtNYXBwaW5nVmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCgtkZX'
        'NjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SLwoJc2lnbmF0dXJlGAQgASgLMhEuYmRsLnYx'
        'LlNpZ25hdHVyZVIJc2lnbmF0dXJlEjcKCmRlZmluaXRpb24YBSABKAsyEi5iZGwudjEuRGVmaW'
        '5pdGlvbkgAUgpkZWZpbml0aW9uiAEBEi0KBXN0YXRlGAYgASgOMhcuYmRsLnYxLkFjY2VwdGFu'
        'Y2VTdGF0ZVIFc3RhdGUSHgoIY2xvY2tfaWQYByABKARIAVIHY2xvY2tJZIgBARItChBkcml2ZX'
        'Nfb3V0cHV0X2lkGAggASgESAJSDmRyaXZlc091dHB1dElkiAEBQg0KC19kZWZpbml0aW9uQgsK'
        'CV9jbG9ja19pZEITChFfZHJpdmVzX291dHB1dF9pZA==');

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
    {
      '1': 'reference',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ReferenceDefinition',
      '9': 0,
      '10': 'reference'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Definition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List definitionDescriptor = $convert
    .base64Decode('CgpEZWZpbml0aW9uEhoKB2Zvcm11bGEYASABKAlIAFIHZm9ybXVsYRI7CglyZWZlcmVuY2UYAi'
        'ABKAsyGy5iZGwudjEuUmVmZXJlbmNlRGVmaW5pdGlvbkgAUglyZWZlcmVuY2VCBgoEa2luZA==');

@$core.Deprecated('Use referenceDefinitionDescriptor instead')
const ReferenceDefinition$json = {
  '1': 'ReferenceDefinition',
  '2': [
    {'1': 'target', '3': 1, '4': 1, '5': 4, '10': 'target'},
    {
      '1': 'transport',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TransportSpec',
      '9': 0,
      '10': 'transport',
      '17': true
    },
  ],
  '8': [
    {'1': '_transport'},
  ],
};

/// Descriptor for `ReferenceDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List referenceDefinitionDescriptor = $convert
    .base64Decode('ChNSZWZlcmVuY2VEZWZpbml0aW9uEhYKBnRhcmdldBgBIAEoBFIGdGFyZ2V0EjgKCXRyYW5zcG'
        '9ydBgCIAEoCzIVLmJkbC52MS5UcmFuc3BvcnRTcGVjSABSCXRyYW5zcG9ydIgBAUIMCgpfdHJh'
        'bnNwb3J0');

@$core.Deprecated('Use transportSpecDescriptor instead')
const TransportSpec$json = {
  '1': 'TransportSpec',
  '2': [
    {'1': 'source_clock_id', '3': 1, '4': 1, '5': 4, '10': 'sourceClockId'},
    {'1': 'init', '3': 2, '4': 1, '5': 9, '10': 'init'},
  ],
};

/// Descriptor for `TransportSpec`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List transportSpecDescriptor = $convert
    .base64Decode('Cg1UcmFuc3BvcnRTcGVjEiYKD3NvdXJjZV9jbG9ja19pZBgBIAEoBFINc291cmNlQ2xvY2tJZB'
        'ISCgRpbml0GAIgASgJUgRpbml0');

@$core.Deprecated('Use representationDescriptor instead')
const Representation$json = {
  '1': 'Representation',
  '2': [
    {'1': 'quantity', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Dim', '9': 0, '10': 'quantity'},
    {'1': 'boolean', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'boolean'},
    {'1': 'count', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'count'},
    {
      '1': 'optional',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Representation',
      '9': 0,
      '10': 'optional'
    },
    {'1': 'list', '3': 5, '4': 1, '5': 11, '6': '.bdl.v1.Representation', '9': 0, '10': 'list'},
    {'1': 'pair', '3': 6, '4': 1, '5': 11, '6': '.bdl.v1.PairRepresentation', '9': 0, '10': 'pair'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Representation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List representationDescriptor = $convert
    .base64Decode('Cg5SZXByZXNlbnRhdGlvbhIpCghxdWFudGl0eRgBIAEoCzILLmJkbC52MS5EaW1IAFIIcXVhbn'
        'RpdHkSKAoHYm9vbGVhbhgCIAEoCzIMLmJkbC52MS5Vbml0SABSB2Jvb2xlYW4SJAoFY291bnQY'
        'AyABKAsyDC5iZGwudjEuVW5pdEgAUgVjb3VudBI0CghvcHRpb25hbBgEIAEoCzIWLmJkbC52MS'
        '5SZXByZXNlbnRhdGlvbkgAUghvcHRpb25hbBIsCgRsaXN0GAUgASgLMhYuYmRsLnYxLlJlcHJl'
        'c2VudGF0aW9uSABSBGxpc3QSMAoEcGFpchgGIAEoCzIaLmJkbC52MS5QYWlyUmVwcmVzZW50YX'
        'Rpb25IAFIEcGFpckIGCgRraW5k');

@$core.Deprecated('Use pairRepresentationDescriptor instead')
const PairRepresentation$json = {
  '1': 'PairRepresentation',
  '2': [
    {'1': 'first', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Representation', '10': 'first'},
    {'1': 'second', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Representation', '10': 'second'},
  ],
};

/// Descriptor for `PairRepresentation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pairRepresentationDescriptor = $convert
    .base64Decode('ChJQYWlyUmVwcmVzZW50YXRpb24SLAoFZmlyc3QYASABKAsyFi5iZGwudjEuUmVwcmVzZW50YX'
        'Rpb25SBWZpcnN0Ei4KBnNlY29uZBgCIAEoCzIWLmJkbC52MS5SZXByZXNlbnRhdGlvblIGc2Vj'
        'b25k');

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
    {'1': 'outputs', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.NodePosition', '10': 'outputs'},
    {'1': 'instances', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.NodePosition', '10': 'instances'},
    {'1': 'groups', '3': 5, '4': 3, '5': 11, '6': '.bdl.v1.GroupBox', '10': 'groups'},
    {
      '1': 'components',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ComponentLayout',
      '10': 'components'
    },
    {
      '1': 'viewport',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Viewport',
      '9': 0,
      '10': 'viewport',
      '17': true
    },
  ],
  '8': [
    {'1': '_viewport'},
  ],
};

/// Descriptor for `Layout`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List layoutDescriptor = $convert
    .base64Decode('CgZMYXlvdXQSMAoIY29uY2VwdHMYASADKAsyFC5iZGwudjEuTm9kZVBvc2l0aW9uUghjb25jZX'
        'B0cxIwCghtYXBwaW5ncxgCIAMoCzIULmJkbC52MS5Ob2RlUG9zaXRpb25SCG1hcHBpbmdzEi4K'
        'B291dHB1dHMYAyADKAsyFC5iZGwudjEuTm9kZVBvc2l0aW9uUgdvdXRwdXRzEjIKCWluc3Rhbm'
        'NlcxgEIAMoCzIULmJkbC52MS5Ob2RlUG9zaXRpb25SCWluc3RhbmNlcxIoCgZncm91cHMYBSAD'
        'KAsyEC5iZGwudjEuR3JvdXBCb3hSBmdyb3VwcxI3Cgpjb21wb25lbnRzGAYgAygLMhcuYmRsLn'
        'YxLkNvbXBvbmVudExheW91dFIKY29tcG9uZW50cxIxCgh2aWV3cG9ydBgHIAEoCzIQLmJkbC52'
        'MS5WaWV3cG9ydEgAUgh2aWV3cG9ydIgBAUILCglfdmlld3BvcnQ=');

@$core.Deprecated('Use viewportDescriptor instead')
const Viewport$json = {
  '1': 'Viewport',
  '2': [
    {'1': 'x', '3': 1, '4': 1, '5': 1, '10': 'x'},
    {'1': 'y', '3': 2, '4': 1, '5': 1, '10': 'y'},
    {'1': 'zoom', '3': 3, '4': 1, '5': 1, '10': 'zoom'},
  ],
};

/// Descriptor for `Viewport`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List viewportDescriptor = $convert
    .base64Decode('CghWaWV3cG9ydBIMCgF4GAEgASgBUgF4EgwKAXkYAiABKAFSAXkSEgoEem9vbRgDIAEoAVIEem'
        '9vbQ==');

@$core.Deprecated('Use groupBoxDescriptor instead')
const GroupBox$json = {
  '1': 'GroupBox',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'x', '3': 2, '4': 1, '5': 1, '10': 'x'},
    {'1': 'y', '3': 3, '4': 1, '5': 1, '10': 'y'},
    {'1': 'width', '3': 4, '4': 1, '5': 1, '10': 'width'},
    {'1': 'height', '3': 5, '4': 1, '5': 1, '10': 'height'},
    {'1': 'collapsed', '3': 6, '4': 1, '5': 8, '10': 'collapsed'},
  ],
};

/// Descriptor for `GroupBox`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List groupBoxDescriptor = $convert
    .base64Decode('CghHcm91cEJveBIOCgJpZBgBIAEoBFICaWQSDAoBeBgCIAEoAVIBeBIMCgF5GAMgASgBUgF5Eh'
        'QKBXdpZHRoGAQgASgBUgV3aWR0aBIWCgZoZWlnaHQYBSABKAFSBmhlaWdodBIcCgljb2xsYXBz'
        'ZWQYBiABKAhSCWNvbGxhcHNlZA==');

@$core.Deprecated('Use componentLayoutDescriptor instead')
const ComponentLayout$json = {
  '1': 'ComponentLayout',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'layout', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Layout', '10': 'layout'},
  ],
};

/// Descriptor for `ComponentLayout`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List componentLayoutDescriptor = $convert
    .base64Decode('Cg9Db21wb25lbnRMYXlvdXQSDgoCaWQYASABKARSAmlkEiYKBmxheW91dBgCIAEoCzIOLmJkbC'
        '52MS5MYXlvdXRSBmxheW91dA==');

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

@$core.Deprecated('Use valueDescriptor instead')
const Value$json = {
  '1': 'Value',
  '2': [
    {'1': 'boolean', '3': 1, '4': 1, '5': 8, '9': 0, '10': 'boolean'},
    {'1': 'count', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'count'},
    {'1': 'quantity', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Quantity', '9': 0, '10': 'quantity'},
    {
      '1': 'semantic',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SemanticValue',
      '9': 0,
      '10': 'semantic'
    },
    {'1': 'none', '3': 5, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'none'},
    {'1': 'some', '3': 6, '4': 1, '5': 11, '6': '.bdl.v1.Value', '9': 0, '10': 'some'},
    {'1': 'opaque', '3': 7, '4': 1, '5': 9, '9': 0, '10': 'opaque'},
    {'1': 'list', '3': 8, '4': 1, '5': 11, '6': '.bdl.v1.ValueList', '9': 0, '10': 'list'},
    {'1': 'pair', '3': 9, '4': 1, '5': 11, '6': '.bdl.v1.ValuePair', '9': 0, '10': 'pair'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Value`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List valueDescriptor = $convert
    .base64Decode('CgVWYWx1ZRIaCgdib29sZWFuGAEgASgISABSB2Jvb2xlYW4SFgoFY291bnQYAiABKARIAFIFY2'
        '91bnQSLgoIcXVhbnRpdHkYAyABKAsyEC5iZGwudjEuUXVhbnRpdHlIAFIIcXVhbnRpdHkSMwoI'
        'c2VtYW50aWMYBCABKAsyFS5iZGwudjEuU2VtYW50aWNWYWx1ZUgAUghzZW1hbnRpYxIiCgRub2'
        '5lGAUgASgLMgwuYmRsLnYxLlVuaXRIAFIEbm9uZRIjCgRzb21lGAYgASgLMg0uYmRsLnYxLlZh'
        'bHVlSABSBHNvbWUSGAoGb3BhcXVlGAcgASgJSABSBm9wYXF1ZRInCgRsaXN0GAggASgLMhEuYm'
        'RsLnYxLlZhbHVlTGlzdEgAUgRsaXN0EicKBHBhaXIYCSABKAsyES5iZGwudjEuVmFsdWVQYWly'
        'SABSBHBhaXJCBgoEa2luZA==');

@$core.Deprecated('Use valueListDescriptor instead')
const ValueList$json = {
  '1': 'ValueList',
  '2': [
    {'1': 'items', '3': 1, '4': 3, '5': 11, '6': '.bdl.v1.Value', '10': 'items'},
  ],
};

/// Descriptor for `ValueList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List valueListDescriptor =
    $convert.base64Decode('CglWYWx1ZUxpc3QSIwoFaXRlbXMYASADKAsyDS5iZGwudjEuVmFsdWVSBWl0ZW1z');

@$core.Deprecated('Use valuePairDescriptor instead')
const ValuePair$json = {
  '1': 'ValuePair',
  '2': [
    {'1': 'fst', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Value', '10': 'fst'},
    {'1': 'snd', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Value', '10': 'snd'},
  ],
};

/// Descriptor for `ValuePair`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List valuePairDescriptor = $convert
    .base64Decode('CglWYWx1ZVBhaXISHwoDZnN0GAEgASgLMg0uYmRsLnYxLlZhbHVlUgNmc3QSHwoDc25kGAIgAS'
        'gLMg0uYmRsLnYxLlZhbHVlUgNzbmQ=');

@$core.Deprecated('Use quantityDescriptor instead')
const Quantity$json = {
  '1': 'Quantity',
  '2': [
    {'1': 'dim', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Dim', '10': 'dim'},
    {'1': 'value', '3': 2, '4': 1, '5': 1, '10': 'value'},
  ],
};

/// Descriptor for `Quantity`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quantityDescriptor = $convert
    .base64Decode('CghRdWFudGl0eRIdCgNkaW0YASABKAsyCy5iZGwudjEuRGltUgNkaW0SFAoFdmFsdWUYAiABKA'
        'FSBXZhbHVl');

@$core.Deprecated('Use semanticValueDescriptor instead')
const SemanticValue$json = {
  '1': 'SemanticValue',
  '2': [
    {'1': 'concept_id', '3': 1, '4': 1, '5': 4, '10': 'conceptId'},
    {'1': 'repr', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Value', '10': 'repr'},
  ],
};

/// Descriptor for `SemanticValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List semanticValueDescriptor = $convert
    .base64Decode('Cg1TZW1hbnRpY1ZhbHVlEh0KCmNvbmNlcHRfaWQYASABKARSCWNvbmNlcHRJZBIhCgRyZXByGA'
        'IgASgLMg0uYmRsLnYxLlZhbHVlUgRyZXBy');

@$core.Deprecated('Use simulationInputDescriptor instead')
const SimulationInput$json = {
  '1': 'SimulationInput',
  '2': [
    {'1': 'mapping_id', '3': 1, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'tick', '3': 2, '4': 1, '5': 4, '10': 'tick'},
    {'1': 'value', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Value', '10': 'value'},
  ],
};

/// Descriptor for `SimulationInput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List simulationInputDescriptor = $convert
    .base64Decode('Cg9TaW11bGF0aW9uSW5wdXQSHQoKbWFwcGluZ19pZBgBIAEoBFIJbWFwcGluZ0lkEhIKBHRpY2'
        'sYAiABKARSBHRpY2sSIwoFdmFsdWUYAyABKAsyDS5iZGwudjEuVmFsdWVSBXZhbHVl');

@$core.Deprecated('Use schedulePeriodDescriptor instead')
const SchedulePeriod$json = {
  '1': 'SchedulePeriod',
  '2': [
    {'1': 'clock_id', '3': 1, '4': 1, '5': 4, '10': 'clockId'},
    {'1': 'period', '3': 2, '4': 1, '5': 4, '10': 'period'},
  ],
};

/// Descriptor for `SchedulePeriod`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List schedulePeriodDescriptor = $convert
    .base64Decode('Cg5TY2hlZHVsZVBlcmlvZBIZCghjbG9ja19pZBgBIAEoBFIHY2xvY2tJZBIWCgZwZXJpb2QYAi'
        'ABKARSBnBlcmlvZA==');

@$core.Deprecated('Use startSimulationRequestDescriptor instead')
const StartSimulationRequest$json = {
  '1': 'StartSimulationRequest',
  '2': [
    {'1': 'inputs', '3': 1, '4': 3, '5': 11, '6': '.bdl.v1.SimulationInput', '10': 'inputs'},
    {'1': 'schedule', '3': 2, '4': 3, '5': 11, '6': '.bdl.v1.SchedulePeriod', '10': 'schedule'},
  ],
};

/// Descriptor for `StartSimulationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startSimulationRequestDescriptor = $convert
    .base64Decode('ChZTdGFydFNpbXVsYXRpb25SZXF1ZXN0Ei8KBmlucHV0cxgBIAMoCzIXLmJkbC52MS5TaW11bG'
        'F0aW9uSW5wdXRSBmlucHV0cxIyCghzY2hlZHVsZRgCIAMoCzIWLmJkbC52MS5TY2hlZHVsZVBl'
        'cmlvZFIIc2NoZWR1bGU=');

@$core.Deprecated('Use stepSimulationRequestDescriptor instead')
const StepSimulationRequest$json = {
  '1': 'StepSimulationRequest',
  '2': [
    {'1': 'ticks', '3': 1, '4': 1, '5': 4, '10': 'ticks'},
  ],
};

/// Descriptor for `StepSimulationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List stepSimulationRequestDescriptor =
    $convert.base64Decode('ChVTdGVwU2ltdWxhdGlvblJlcXVlc3QSFAoFdGlja3MYASABKARSBXRpY2tz');

@$core.Deprecated('Use resetSimulationRequestDescriptor instead')
const ResetSimulationRequest$json = {
  '1': 'ResetSimulationRequest',
};

/// Descriptor for `ResetSimulationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resetSimulationRequestDescriptor =
    $convert.base64Decode('ChZSZXNldFNpbXVsYXRpb25SZXF1ZXN0');

@$core.Deprecated('Use simulationResponseDescriptor instead')
const SimulationResponse$json = {
  '1': 'SimulationResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'next_tick', '3': 2, '4': 1, '5': 4, '10': 'nextTick'},
    {'1': 'samples', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.TickSample', '10': 'samples'},
    {
      '1': 'error',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Diagnostic',
      '9': 0,
      '10': 'error',
      '17': true
    },
  ],
  '8': [
    {'1': '_error'},
  ],
};

/// Descriptor for `SimulationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List simulationResponseDescriptor = $convert
    .base64Decode('ChJTaW11bGF0aW9uUmVzcG9uc2USGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEhsKCW5leH'
        'RfdGljaxgCIAEoBFIIbmV4dFRpY2sSLAoHc2FtcGxlcxgDIAMoCzISLmJkbC52MS5UaWNrU2Ft'
        'cGxlUgdzYW1wbGVzEi0KBWVycm9yGAQgASgLMhIuYmRsLnYxLkRpYWdub3N0aWNIAFIFZXJyb3'
        'KIAQFCCAoGX2Vycm9y');

@$core.Deprecated('Use tickSampleDescriptor instead')
const TickSample$json = {
  '1': 'TickSample',
  '2': [
    {'1': 'tick', '3': 1, '4': 1, '5': 4, '10': 'tick'},
    {'1': 'active_clock_ids', '3': 2, '4': 3, '5': 4, '10': 'activeClockIds'},
    {'1': 'values', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.DeclarationSample', '10': 'values'},
  ],
};

/// Descriptor for `TickSample`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tickSampleDescriptor = $convert
    .base64Decode('CgpUaWNrU2FtcGxlEhIKBHRpY2sYASABKARSBHRpY2sSKAoQYWN0aXZlX2Nsb2NrX2lkcxgCIA'
        'MoBFIOYWN0aXZlQ2xvY2tJZHMSMQoGdmFsdWVzGAMgAygLMhkuYmRsLnYxLkRlY2xhcmF0aW9u'
        'U2FtcGxlUgZ2YWx1ZXM=');

@$core.Deprecated('Use declarationSampleDescriptor instead')
const DeclarationSample$json = {
  '1': 'DeclarationSample',
  '2': [
    {'1': 'mapping_id', '3': 1, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.Value', '10': 'value'},
    {'1': 'rendered', '3': 3, '4': 1, '5': 9, '10': 'rendered'},
  ],
};

/// Descriptor for `DeclarationSample`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declarationSampleDescriptor = $convert
    .base64Decode('ChFEZWNsYXJhdGlvblNhbXBsZRIdCgptYXBwaW5nX2lkGAEgASgEUgltYXBwaW5nSWQSIwoFdm'
        'FsdWUYAiABKAsyDS5iZGwudjEuVmFsdWVSBXZhbHVlEhoKCHJlbmRlcmVkGAMgASgJUghyZW5k'
        'ZXJlZA==');

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
    {'1': 'causal', '3': 4, '4': 1, '5': 8, '10': 'causal'},
    {'1': 'clock_consistent', '3': 5, '4': 1, '5': 8, '10': 'clockConsistent'},
    {'1': 'cycles', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.DeclarationCycle', '10': 'cycles'},
    {'1': 'evaluation_order', '3': 7, '4': 3, '5': 4, '10': 'evaluationOrder'},
    {'1': 'outputs', '3': 8, '4': 3, '5': 11, '6': '.bdl.v1.OutputAnalysis', '10': 'outputs'},
    {'1': 'open_outputs', '3': 9, '4': 3, '5': 4, '10': 'openOutputs'},
    {'1': 'output_complete', '3': 10, '4': 1, '5': 8, '10': 'outputComplete'},
  ],
};

/// Descriptor for `ProjectAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectAnalysisDescriptor = $convert
    .base64Decode('Cg9Qcm9qZWN0QW5hbHlzaXMSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEjMKCG1hcHBpbm'
        'dzGAIgAygLMhcuYmRsLnYxLk1hcHBpbmdBbmFseXNpc1IIbWFwcGluZ3MSNAoLZGlhZ25vc3Rp'
        'Y3MYAyADKAsyEi5iZGwudjEuRGlhZ25vc3RpY1ILZGlhZ25vc3RpY3MSFgoGY2F1c2FsGAQgAS'
        'gIUgZjYXVzYWwSKQoQY2xvY2tfY29uc2lzdGVudBgFIAEoCFIPY2xvY2tDb25zaXN0ZW50EjAK'
        'BmN5Y2xlcxgGIAMoCzIYLmJkbC52MS5EZWNsYXJhdGlvbkN5Y2xlUgZjeWNsZXMSKQoQZXZhbH'
        'VhdGlvbl9vcmRlchgHIAMoBFIPZXZhbHVhdGlvbk9yZGVyEjAKB291dHB1dHMYCCADKAsyFi5i'
        'ZGwudjEuT3V0cHV0QW5hbHlzaXNSB291dHB1dHMSIQoMb3Blbl9vdXRwdXRzGAkgAygEUgtvcG'
        'VuT3V0cHV0cxInCg9vdXRwdXRfY29tcGxldGUYCiABKAhSDm91dHB1dENvbXBsZXRl');

@$core.Deprecated('Use outputAnalysisDescriptor instead')
const OutputAnalysis$json = {
  '1': 'OutputAnalysis',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'state', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.OutputState', '10': 'state'},
    {'1': 'driver', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'driver', '17': true},
    {'1': 'claimants', '3': 4, '4': 3, '5': 4, '10': 'claimants'},
  ],
  '8': [
    {'1': '_driver'},
  ],
};

/// Descriptor for `OutputAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List outputAnalysisDescriptor = $convert
    .base64Decode('Cg5PdXRwdXRBbmFseXNpcxIOCgJpZBgBIAEoBFICaWQSKQoFc3RhdGUYAiABKA4yEy5iZGwudj'
        'EuT3V0cHV0U3RhdGVSBXN0YXRlEhsKBmRyaXZlchgDIAEoBEgAUgZkcml2ZXKIAQESHAoJY2xh'
        'aW1hbnRzGAQgAygEUgljbGFpbWFudHNCCQoHX2RyaXZlcg==');

@$core.Deprecated('Use declarationCycleDescriptor instead')
const DeclarationCycle$json = {
  '1': 'DeclarationCycle',
  '2': [
    {'1': 'mapping_ids', '3': 1, '4': 3, '5': 4, '10': 'mappingIds'},
  ],
};

/// Descriptor for `DeclarationCycle`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declarationCycleDescriptor =
    $convert.base64Decode('ChBEZWNsYXJhdGlvbkN5Y2xlEh8KC21hcHBpbmdfaWRzGAEgAygEUgptYXBwaW5nSWRz');

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

@$core.Deprecated('Use analyzeDefinitionDraftRequestDescriptor instead')
const AnalyzeDefinitionDraftRequest$json = {
  '1': 'AnalyzeDefinitionDraftRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'generation', '3': 3, '4': 1, '5': 4, '10': 'generation'},
    {'1': 'source', '3': 4, '4': 1, '5': 9, '10': 'source'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `AnalyzeDefinitionDraftRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List analyzeDefinitionDraftRequestDescriptor = $convert
    .base64Decode('Ch1BbmFseXplRGVmaW5pdGlvbkRyYWZ0UmVxdWVzdBIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aX'
        'Npb24SHQoKbWFwcGluZ19pZBgCIAEoBFIJbWFwcGluZ0lkEh4KCmdlbmVyYXRpb24YAyABKARS'
        'CmdlbmVyYXRpb24SFgoGc291cmNlGAQgASgJUgZzb3VyY2USIQoJY29tcG9uZW50GAUgASgESA'
        'BSCWNvbXBvbmVudIgBAUIMCgpfY29tcG9uZW50');

@$core.Deprecated('Use definitionDraftAnalysisDescriptor instead')
const DefinitionDraftAnalysis$json = {
  '1': 'DefinitionDraftAnalysis',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'generation', '3': 3, '4': 1, '5': 4, '10': 'generation'},
    {'1': 'parse_ok', '3': 4, '4': 1, '5': 8, '10': 'parseOk'},
    {'1': 'analysis', '3': 5, '4': 1, '5': 11, '6': '.bdl.v1.MappingAnalysis', '10': 'analysis'},
    {
      '1': 'projection',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.FormulaProjection',
      '10': 'projection'
    },
  ],
};

/// Descriptor for `DefinitionDraftAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List definitionDraftAnalysisDescriptor = $convert
    .base64Decode('ChdEZWZpbml0aW9uRHJhZnRBbmFseXNpcxIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SHQ'
        'oKbWFwcGluZ19pZBgCIAEoBFIJbWFwcGluZ0lkEh4KCmdlbmVyYXRpb24YAyABKARSCmdlbmVy'
        'YXRpb24SGQoIcGFyc2Vfb2sYBCABKAhSB3BhcnNlT2sSMwoIYW5hbHlzaXMYBSABKAsyFy5iZG'
        'wudjEuTWFwcGluZ0FuYWx5c2lzUghhbmFseXNpcxI5Cgpwcm9qZWN0aW9uGAYgASgLMhkuYmRs'
        'LnYxLkZvcm11bGFQcm9qZWN0aW9uUgpwcm9qZWN0aW9u');

@$core.Deprecated('Use getFormulaProjectionRequestDescriptor instead')
const GetFormulaProjectionRequest$json = {
  '1': 'GetFormulaProjectionRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'component', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `GetFormulaProjectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFormulaProjectionRequestDescriptor = $convert
    .base64Decode('ChtHZXRGb3JtdWxhUHJvamVjdGlvblJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW'
        '9uEh0KCm1hcHBpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIhCgljb21wb25lbnQYAyABKARIAFIJ'
        'Y29tcG9uZW50iAEBQgwKCl9jb21wb25lbnQ=');

@$core.Deprecated('Use formulaProjectionResponseDescriptor instead')
const FormulaProjectionResponse$json = {
  '1': 'FormulaProjectionResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {
      '1': 'projection',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.FormulaProjection',
      '10': 'projection'
    },
  ],
};

/// Descriptor for `FormulaProjectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formulaProjectionResponseDescriptor = $convert
    .base64Decode('ChlGb3JtdWxhUHJvamVjdGlvblJlc3BvbnNlEhoKCHJldmlzaW9uGAEgASgEUghyZXZpc2lvbh'
        'IdCgptYXBwaW5nX2lkGAIgASgEUgltYXBwaW5nSWQSOQoKcHJvamVjdGlvbhgDIAEoCzIZLmJk'
        'bC52MS5Gb3JtdWxhUHJvamVjdGlvblIKcHJvamVjdGlvbg==');

@$core.Deprecated('Use formulaProjectionDescriptor instead')
const FormulaProjection$json = {
  '1': 'FormulaProjection',
  '2': [
    {'1': 'source', '3': 1, '4': 1, '5': 9, '10': 'source'},
    {'1': 'parse_ok', '3': 2, '4': 1, '5': 8, '10': 'parseOk'},
    {'1': 'complete', '3': 3, '4': 1, '5': 8, '10': 'complete'},
    {
      '1': 'root',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.FormulaNode',
      '9': 0,
      '10': 'root',
      '17': true
    },
    {
      '1': 'result',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 1,
      '10': 'result',
      '17': true
    },
    {'1': 'slots', '3': 6, '4': 3, '5': 9, '10': 'slots'},
    {'1': 'unplaced', '3': 7, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'unplaced'},
    {'1': 'draft_generation', '3': 8, '4': 1, '5': 4, '9': 2, '10': 'draftGeneration', '17': true},
  ],
  '8': [
    {'1': '_root'},
    {'1': '_result'},
    {'1': '_draft_generation'},
  ],
};

/// Descriptor for `FormulaProjection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formulaProjectionDescriptor = $convert
    .base64Decode('ChFGb3JtdWxhUHJvamVjdGlvbhIWCgZzb3VyY2UYASABKAlSBnNvdXJjZRIZCghwYXJzZV9vax'
        'gCIAEoCFIHcGFyc2VPaxIaCghjb21wbGV0ZRgDIAEoCFIIY29tcGxldGUSLAoEcm9vdBgEIAEo'
        'CzITLmJkbC52MS5Gb3JtdWxhTm9kZUgAUgRyb290iAEBEi0KBnJlc3VsdBgFIAEoCzIQLmJkbC'
        '52MS5UeXBlVmlld0gBUgZyZXN1bHSIAQESFAoFc2xvdHMYBiADKAlSBXNsb3RzEi4KCHVucGxh'
        'Y2VkGAcgAygLMhIuYmRsLnYxLkRpYWdub3N0aWNSCHVucGxhY2VkEi4KEGRyYWZ0X2dlbmVyYX'
        'Rpb24YCCABKARIAlIPZHJhZnRHZW5lcmF0aW9uiAEBQgcKBV9yb290QgkKB19yZXN1bHRCEwoR'
        'X2RyYWZ0X2dlbmVyYXRpb24=');

@$core.Deprecated('Use formulaNodeDescriptor instead')
const FormulaNode$json = {
  '1': 'FormulaNode',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'range', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.SourceSpan', '10': 'range'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'text', '3': 4, '4': 1, '5': 9, '10': 'text'},
    {'1': 'name', '3': 5, '4': 1, '5': 9, '10': 'name'},
    {'1': 'coordinate', '3': 6, '4': 1, '5': 9, '10': 'coordinate'},
    {'1': 'unit', '3': 7, '4': 1, '5': 9, '10': 'unit'},
    {'1': 'unit_id', '3': 8, '4': 1, '5': 9, '10': 'unitId'},
    {'1': 'equation', '3': 9, '4': 1, '5': 8, '10': 'equation'},
    {
      '1': 'entity',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EntityRef',
      '9': 0,
      '10': 'entity',
      '17': true
    },
    {
      '1': 'actual',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 1,
      '10': 'actual',
      '17': true
    },
    {
      '1': 'expected',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 2,
      '10': 'expected',
      '17': true
    },
    {'1': 'because', '3': 13, '4': 1, '5': 9, '10': 'because'},
    {'1': 'diagnostics', '3': 14, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'diagnostics'},
    {'1': 'children', '3': 15, '4': 3, '5': 11, '6': '.bdl.v1.FormulaNode', '10': 'children'},
    {'1': 'local', '3': 16, '4': 1, '5': 8, '10': 'local'},
    {'1': 'param', '3': 17, '4': 1, '5': 9, '10': 'param'},
    {
      '1': 'param_type',
      '3': 18,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 3,
      '10': 'paramType',
      '17': true
    },
  ],
  '8': [
    {'1': '_entity'},
    {'1': '_actual'},
    {'1': '_expected'},
    {'1': '_param_type'},
  ],
};

/// Descriptor for `FormulaNode`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formulaNodeDescriptor = $convert
    .base64Decode('CgtGb3JtdWxhTm9kZRIOCgJpZBgBIAEoCVICaWQSKAoFcmFuZ2UYAiABKAsyEi5iZGwudjEuU2'
        '91cmNlU3BhblIFcmFuZ2USEgoEa2luZBgDIAEoCVIEa2luZBISCgR0ZXh0GAQgASgJUgR0ZXh0'
        'EhIKBG5hbWUYBSABKAlSBG5hbWUSHgoKY29vcmRpbmF0ZRgGIAEoCVIKY29vcmRpbmF0ZRISCg'
        'R1bml0GAcgASgJUgR1bml0EhcKB3VuaXRfaWQYCCABKAlSBnVuaXRJZBIaCghlcXVhdGlvbhgJ'
        'IAEoCFIIZXF1YXRpb24SLgoGZW50aXR5GAogASgLMhEuYmRsLnYxLkVudGl0eVJlZkgAUgZlbn'
        'RpdHmIAQESLQoGYWN0dWFsGAsgASgLMhAuYmRsLnYxLlR5cGVWaWV3SAFSBmFjdHVhbIgBARIx'
        'CghleHBlY3RlZBgMIAEoCzIQLmJkbC52MS5UeXBlVmlld0gCUghleHBlY3RlZIgBARIYCgdiZW'
        'NhdXNlGA0gASgJUgdiZWNhdXNlEjQKC2RpYWdub3N0aWNzGA4gAygLMhIuYmRsLnYxLkRpYWdu'
        'b3N0aWNSC2RpYWdub3N0aWNzEi8KCGNoaWxkcmVuGA8gAygLMhMuYmRsLnYxLkZvcm11bGFOb2'
        'RlUghjaGlsZHJlbhIUCgVsb2NhbBgQIAEoCFIFbG9jYWwSFAoFcGFyYW0YESABKAlSBXBhcmFt'
        'EjQKCnBhcmFtX3R5cGUYEiABKAsyEC5iZGwudjEuVHlwZVZpZXdIA1IJcGFyYW1UeXBliAEBQg'
        'kKB19lbnRpdHlCCQoHX2FjdHVhbEILCglfZXhwZWN0ZWRCDQoLX3BhcmFtX3R5cGU=');

@$core.Deprecated('Use typeViewDescriptor instead')
const TypeView$json = {
  '1': 'TypeView',
  '2': [
    {'1': 'description', '3': 1, '4': 1, '5': 9, '10': 'description'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'dim', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.Dim', '9': 0, '10': 'dim', '17': true},
    {'1': 'concept_id', '3': 4, '4': 1, '5': 4, '9': 1, '10': 'conceptId', '17': true},
    {
      '1': 'element',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 2,
      '10': 'element',
      '17': true
    },
  ],
  '8': [
    {'1': '_dim'},
    {'1': '_concept_id'},
    {'1': '_element'},
  ],
};

/// Descriptor for `TypeView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List typeViewDescriptor = $convert
    .base64Decode('CghUeXBlVmlldxIgCgtkZXNjcmlwdGlvbhgBIAEoCVILZGVzY3JpcHRpb24SEgoEa2luZBgCIA'
        'EoCVIEa2luZBIiCgNkaW0YAyABKAsyCy5iZGwudjEuRGltSABSA2RpbYgBARIiCgpjb25jZXB0'
        'X2lkGAQgASgESAFSCWNvbmNlcHRJZIgBARIvCgdlbGVtZW50GAUgASgLMhAuYmRsLnYxLlR5cG'
        'VWaWV3SAJSB2VsZW1lbnSIAQFCBgoEX2RpbUINCgtfY29uY2VwdF9pZEIKCghfZWxlbWVudA==');

@$core.Deprecated('Use getFormulaSlotRequestDescriptor instead')
const GetFormulaSlotRequest$json = {
  '1': 'GetFormulaSlotRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'node_id', '3': 4, '4': 1, '5': 9, '10': 'nodeId'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `GetFormulaSlotRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFormulaSlotRequestDescriptor = $convert
    .base64Decode('ChVHZXRGb3JtdWxhU2xvdFJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEh0KCm'
        '1hcHBpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIWCgZzb3VyY2UYAyABKAlSBnNvdXJjZRIXCgdu'
        'b2RlX2lkGAQgASgJUgZub2RlSWQSIQoJY29tcG9uZW50GAUgASgESABSCWNvbXBvbmVudIgBAU'
        'IMCgpfY29tcG9uZW50');

@$core.Deprecated('Use formulaSlotResponseDescriptor instead')
const FormulaSlotResponse$json = {
  '1': 'FormulaSlotResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'node_id', '3': 3, '4': 1, '5': 9, '10': 'nodeId'},
    {
      '1': 'expected',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.TypeView',
      '9': 0,
      '10': 'expected',
      '17': true
    },
    {'1': 'explanation', '3': 5, '4': 1, '5': 9, '10': 'explanation'},
    {'1': 'technical', '3': 6, '4': 1, '5': 9, '10': 'technical'},
    {'1': 'insufficient', '3': 7, '4': 1, '5': 8, '10': 'insufficient'},
    {'1': 'units', '3': 8, '4': 3, '5': 11, '6': '.bdl.v1.UnitCandidate', '10': 'units'},
    {
      '1': 'references',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ReferenceCandidate',
      '10': 'references'
    },
    {
      '1': 'equations',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.EquationCandidate',
      '10': 'equations'
    },
  ],
  '8': [
    {'1': '_expected'},
  ],
};

/// Descriptor for `FormulaSlotResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List formulaSlotResponseDescriptor = $convert
    .base64Decode('ChNGb3JtdWxhU2xvdFJlc3BvbnNlEhoKCHJldmlzaW9uGAEgASgEUghyZXZpc2lvbhIdCgptYX'
        'BwaW5nX2lkGAIgASgEUgltYXBwaW5nSWQSFwoHbm9kZV9pZBgDIAEoCVIGbm9kZUlkEjEKCGV4'
        'cGVjdGVkGAQgASgLMhAuYmRsLnYxLlR5cGVWaWV3SABSCGV4cGVjdGVkiAEBEiAKC2V4cGxhbm'
        'F0aW9uGAUgASgJUgtleHBsYW5hdGlvbhIcCgl0ZWNobmljYWwYBiABKAlSCXRlY2huaWNhbBIi'
        'CgxpbnN1ZmZpY2llbnQYByABKAhSDGluc3VmZmljaWVudBIrCgV1bml0cxgIIAMoCzIVLmJkbC'
        '52MS5Vbml0Q2FuZGlkYXRlUgV1bml0cxI6CgpyZWZlcmVuY2VzGAkgAygLMhouYmRsLnYxLlJl'
        'ZmVyZW5jZUNhbmRpZGF0ZVIKcmVmZXJlbmNlcxI3CgllcXVhdGlvbnMYCiADKAsyGS5iZGwudj'
        'EuRXF1YXRpb25DYW5kaWRhdGVSCWVxdWF0aW9uc0ILCglfZXhwZWN0ZWQ=');

@$core.Deprecated('Use unitCandidateDescriptor instead')
const UnitCandidate$json = {
  '1': 'UnitCandidate',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'symbol', '3': 2, '4': 1, '5': 9, '10': 'symbol'},
    {'1': 'measures', '3': 3, '4': 1, '5': 9, '10': 'measures'},
  ],
};

/// Descriptor for `UnitCandidate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unitCandidateDescriptor = $convert
    .base64Decode('Cg1Vbml0Q2FuZGlkYXRlEg4KAmlkGAEgASgJUgJpZBIWCgZzeW1ib2wYAiABKAlSBnN5bWJvbB'
        'IaCghtZWFzdXJlcxgDIAEoCVIIbWVhc3VyZXM=');

@$core.Deprecated('Use referenceCandidateDescriptor instead')
const ReferenceCandidate$json = {
  '1': 'ReferenceCandidate',
  '2': [
    {'1': 'label', '3': 1, '4': 1, '5': 9, '10': 'label'},
    {'1': 'insert', '3': 2, '4': 1, '5': 9, '10': 'insert'},
    {
      '1': 'entity',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EntityRef',
      '9': 0,
      '10': 'entity',
      '17': true
    },
    {'1': 'produces', '3': 4, '4': 1, '5': 9, '10': 'produces'},
    {'1': 'relevance', '3': 5, '4': 1, '5': 13, '10': 'relevance'},
  ],
  '8': [
    {'1': '_entity'},
  ],
};

/// Descriptor for `ReferenceCandidate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List referenceCandidateDescriptor = $convert
    .base64Decode('ChJSZWZlcmVuY2VDYW5kaWRhdGUSFAoFbGFiZWwYASABKAlSBWxhYmVsEhYKBmluc2VydBgCIA'
        'EoCVIGaW5zZXJ0Ei4KBmVudGl0eRgDIAEoCzIRLmJkbC52MS5FbnRpdHlSZWZIAFIGZW50aXR5'
        'iAEBEhoKCHByb2R1Y2VzGAQgASgJUghwcm9kdWNlcxIcCglyZWxldmFuY2UYBSABKA1SCXJlbG'
        'V2YW5jZUIJCgdfZW50aXR5');

@$core.Deprecated('Use equationCandidateDescriptor instead')
const EquationCandidate$json = {
  '1': 'EquationCandidate',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'shape', '3': 2, '4': 1, '5': 9, '10': 'shape'},
    {'1': 'insert', '3': 3, '4': 1, '5': 9, '10': 'insert'},
    {'1': 'summary', '3': 4, '4': 1, '5': 9, '10': 'summary'},
  ],
};

/// Descriptor for `EquationCandidate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List equationCandidateDescriptor = $convert
    .base64Decode('ChFFcXVhdGlvbkNhbmRpZGF0ZRISCgRuYW1lGAEgASgJUgRuYW1lEhQKBXNoYXBlGAIgASgJUg'
        'VzaGFwZRIWCgZpbnNlcnQYAyABKAlSBmluc2VydBIYCgdzdW1tYXJ5GAQgASgJUgdzdW1tYXJ5');

@$core.Deprecated('Use composeFormulaRequestDescriptor instead')
const ComposeFormulaRequest$json = {
  '1': 'ComposeFormulaRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'action', '3': 4, '4': 1, '5': 11, '6': '.bdl.v1.ComposeAction', '10': 'action'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `ComposeFormulaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeFormulaRequestDescriptor = $convert
    .base64Decode('ChVDb21wb3NlRm9ybXVsYVJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEh0KCm'
        '1hcHBpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIWCgZzb3VyY2UYAyABKAlSBnNvdXJjZRItCgZh'
        'Y3Rpb24YBCABKAsyFS5iZGwudjEuQ29tcG9zZUFjdGlvblIGYWN0aW9uEiEKCWNvbXBvbmVudB'
        'gFIAEoBEgAUgljb21wb25lbnSIAQFCDAoKX2NvbXBvbmVudA==');

@$core.Deprecated('Use composeActionDescriptor instead')
const ComposeAction$json = {
  '1': 'ComposeAction',
  '2': [
    {'1': 'node_id', '3': 1, '4': 1, '5': 9, '10': 'nodeId'},
    {'1': 'fill', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'fill'},
    {
      '1': 'operator',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ComposeOperator',
      '9': 0,
      '10': 'operator'
    },
    {'1': 'call', '3': 4, '4': 1, '5': 11, '6': '.bdl.v1.ComposeCall', '9': 0, '10': 'call'},
    {
      '1': 'set_unit',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ComposeSetUnit',
      '9': 0,
      '10': 'setUnit'
    },
    {'1': 'set_coordinate', '3': 6, '4': 1, '5': 9, '9': 0, '10': 'setCoordinate'},
    {'1': 'remove', '3': 7, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'remove'},
    {'1': 'binder', '3': 8, '4': 1, '5': 11, '6': '.bdl.v1.ComposeBinder', '9': 0, '10': 'binder'},
    {'1': 'range', '3': 9, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'range'},
  ],
  '8': [
    {'1': 'action'},
  ],
};

/// Descriptor for `ComposeAction`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeActionDescriptor = $convert
    .base64Decode('Cg1Db21wb3NlQWN0aW9uEhcKB25vZGVfaWQYASABKAlSBm5vZGVJZBIUCgRmaWxsGAIgASgJSA'
        'BSBGZpbGwSNQoIb3BlcmF0b3IYAyABKAsyFy5iZGwudjEuQ29tcG9zZU9wZXJhdG9ySABSCG9w'
        'ZXJhdG9yEikKBGNhbGwYBCABKAsyEy5iZGwudjEuQ29tcG9zZUNhbGxIAFIEY2FsbBIzCghzZX'
        'RfdW5pdBgFIAEoCzIWLmJkbC52MS5Db21wb3NlU2V0VW5pdEgAUgdzZXRVbml0EicKDnNldF9j'
        'b29yZGluYXRlGAYgASgJSABSDXNldENvb3JkaW5hdGUSJgoGcmVtb3ZlGAcgASgLMgwuYmRsLn'
        'YxLlVuaXRIAFIGcmVtb3ZlEi8KBmJpbmRlchgIIAEoCzIVLmJkbC52MS5Db21wb3NlQmluZGVy'
        'SABSBmJpbmRlchIkCgVyYW5nZRgJIAEoCzIMLmJkbC52MS5Vbml0SABSBXJhbmdlQggKBmFjdG'
        'lvbg==');

@$core.Deprecated('Use composeBinderDescriptor instead')
const ComposeBinder$json = {
  '1': 'ComposeBinder',
  '2': [
    {'1': 'form', '3': 1, '4': 1, '5': 9, '10': 'form'},
  ],
};

/// Descriptor for `ComposeBinder`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeBinderDescriptor =
    $convert.base64Decode('Cg1Db21wb3NlQmluZGVyEhIKBGZvcm0YASABKAlSBGZvcm0=');

@$core.Deprecated('Use composeOperatorDescriptor instead')
const ComposeOperator$json = {
  '1': 'ComposeOperator',
  '2': [
    {'1': 'op', '3': 1, '4': 1, '5': 9, '10': 'op'},
    {'1': 'before', '3': 2, '4': 1, '5': 8, '10': 'before'},
  ],
};

/// Descriptor for `ComposeOperator`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeOperatorDescriptor = $convert
    .base64Decode('Cg9Db21wb3NlT3BlcmF0b3ISDgoCb3AYASABKAlSAm9wEhYKBmJlZm9yZRgCIAEoCFIGYmVmb3'
        'Jl');

@$core.Deprecated('Use composeCallDescriptor instead')
const ComposeCall$json = {
  '1': 'ComposeCall',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'arity', '3': 2, '4': 1, '5': 13, '10': 'arity'},
  ],
};

/// Descriptor for `ComposeCall`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeCallDescriptor = $convert
    .base64Decode('CgtDb21wb3NlQ2FsbBISCgRuYW1lGAEgASgJUgRuYW1lEhQKBWFyaXR5GAIgASgNUgVhcml0eQ'
        '==');

@$core.Deprecated('Use composeSetUnitDescriptor instead')
const ComposeSetUnit$json = {
  '1': 'ComposeSetUnit',
  '2': [
    {'1': 'unit_id', '3': 1, '4': 1, '5': 9, '10': 'unitId'},
    {'1': 'preserve_value', '3': 2, '4': 1, '5': 8, '10': 'preserveValue'},
  ],
};

/// Descriptor for `ComposeSetUnit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeSetUnitDescriptor = $convert
    .base64Decode('Cg5Db21wb3NlU2V0VW5pdBIXCgd1bml0X2lkGAEgASgJUgZ1bml0SWQSJQoOcHJlc2VydmVfdm'
        'FsdWUYAiABKAhSDXByZXNlcnZlVmFsdWU=');

@$core.Deprecated('Use composeFormulaResponseDescriptor instead')
const ComposeFormulaResponse$json = {
  '1': 'ComposeFormulaResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'edits', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.DraftTextEdit', '10': 'edits'},
    {'1': 'select', '3': 5, '4': 1, '5': 9, '10': 'select'},
  ],
};

/// Descriptor for `ComposeFormulaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List composeFormulaResponseDescriptor = $convert
    .base64Decode('ChZDb21wb3NlRm9ybXVsYVJlc3BvbnNlEhoKCHJldmlzaW9uGAEgASgEUghyZXZpc2lvbhIdCg'
        'ptYXBwaW5nX2lkGAIgASgEUgltYXBwaW5nSWQSFgoGc291cmNlGAMgASgJUgZzb3VyY2USKwoF'
        'ZWRpdHMYBCADKAsyFS5iZGwudjEuRHJhZnRUZXh0RWRpdFIFZWRpdHMSFgoGc2VsZWN0GAUgAS'
        'gJUgZzZWxlY3Q=');

@$core.Deprecated('Use draftTextEditDescriptor instead')
const DraftTextEdit$json = {
  '1': 'DraftTextEdit',
  '2': [
    {'1': 'start', '3': 1, '4': 1, '5': 13, '10': 'start'},
    {'1': 'end', '3': 2, '4': 1, '5': 13, '10': 'end'},
    {'1': 'new_text', '3': 3, '4': 1, '5': 9, '10': 'newText'},
  ],
};

/// Descriptor for `DraftTextEdit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List draftTextEditDescriptor = $convert
    .base64Decode('Cg1EcmFmdFRleHRFZGl0EhQKBXN0YXJ0GAEgASgNUgVzdGFydBIQCgNlbmQYAiABKA1SA2VuZB'
        'IZCghuZXdfdGV4dBgDIAEoCVIHbmV3VGV4dA==');

@$core.Deprecated('Use discardDefinitionDraftRequestDescriptor instead')
const DiscardDefinitionDraftRequest$json = {
  '1': 'DiscardDefinitionDraftRequest',
  '2': [
    {'1': 'mapping_id', '3': 1, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'component', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `DiscardDefinitionDraftRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List discardDefinitionDraftRequestDescriptor = $convert
    .base64Decode('Ch1EaXNjYXJkRGVmaW5pdGlvbkRyYWZ0UmVxdWVzdBIdCgptYXBwaW5nX2lkGAEgASgEUgltYX'
        'BwaW5nSWQSIQoJY29tcG9uZW50GAIgASgESABSCWNvbXBvbmVudIgBAUIMCgpfY29tcG9uZW50');

@$core.Deprecated('Use completeDefinitionDraftRequestDescriptor instead')
const CompleteDefinitionDraftRequest$json = {
  '1': 'CompleteDefinitionDraftRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'offset', '3': 4, '4': 1, '5': 13, '10': 'offset'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `CompleteDefinitionDraftRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeDefinitionDraftRequestDescriptor = $convert
    .base64Decode('Ch5Db21wbGV0ZURlZmluaXRpb25EcmFmdFJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldm'
        'lzaW9uEh0KCm1hcHBpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIWCgZzb3VyY2UYAyABKAlSBnNv'
        'dXJjZRIWCgZvZmZzZXQYBCABKA1SBm9mZnNldBIhCgljb21wb25lbnQYBSABKARIAFIJY29tcG'
        '9uZW50iAEBQgwKCl9jb21wb25lbnQ=');

@$core.Deprecated('Use draftCompletionResponseDescriptor instead')
const DraftCompletionResponse$json = {
  '1': 'DraftCompletionResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'items', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.DraftCompletionItem', '10': 'items'},
  ],
};

/// Descriptor for `DraftCompletionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List draftCompletionResponseDescriptor = $convert
    .base64Decode('ChdEcmFmdENvbXBsZXRpb25SZXNwb25zZRIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SHQ'
        'oKbWFwcGluZ19pZBgCIAEoBFIJbWFwcGluZ0lkEjEKBWl0ZW1zGAMgAygLMhsuYmRsLnYxLkRy'
        'YWZ0Q29tcGxldGlvbkl0ZW1SBWl0ZW1z');

@$core.Deprecated('Use draftCompletionItemDescriptor instead')
const DraftCompletionItem$json = {
  '1': 'DraftCompletionItem',
  '2': [
    {'1': 'label', '3': 1, '4': 1, '5': 9, '10': 'label'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'replace_start', '3': 3, '4': 1, '5': 13, '10': 'replaceStart'},
    {'1': 'replace_end', '3': 4, '4': 1, '5': 13, '10': 'replaceEnd'},
    {'1': 'insert', '3': 5, '4': 1, '5': 9, '10': 'insert'},
    {'1': 'resulting_type', '3': 6, '4': 1, '5': 9, '10': 'resultingType'},
    {'1': 'documentation', '3': 7, '4': 1, '5': 9, '10': 'documentation'},
    {'1': 'relevance', '3': 8, '4': 1, '5': 13, '10': 'relevance'},
  ],
};

/// Descriptor for `DraftCompletionItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List draftCompletionItemDescriptor = $convert
    .base64Decode('ChNEcmFmdENvbXBsZXRpb25JdGVtEhQKBWxhYmVsGAEgASgJUgVsYWJlbBISCgRraW5kGAIgAS'
        'gJUgRraW5kEiMKDXJlcGxhY2Vfc3RhcnQYAyABKA1SDHJlcGxhY2VTdGFydBIfCgtyZXBsYWNl'
        'X2VuZBgEIAEoDVIKcmVwbGFjZUVuZBIWCgZpbnNlcnQYBSABKAlSBmluc2VydBIlCg5yZXN1bH'
        'RpbmdfdHlwZRgGIAEoCVINcmVzdWx0aW5nVHlwZRIkCg1kb2N1bWVudGF0aW9uGAcgASgJUg1k'
        'b2N1bWVudGF0aW9uEhwKCXJlbGV2YW5jZRgIIAEoDVIJcmVsZXZhbmNl');

@$core.Deprecated('Use hoverDefinitionDraftRequestDescriptor instead')
const HoverDefinitionDraftRequest$json = {
  '1': 'HoverDefinitionDraftRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'offset', '3': 4, '4': 1, '5': 13, '10': 'offset'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `HoverDefinitionDraftRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List hoverDefinitionDraftRequestDescriptor = $convert
    .base64Decode('ChtIb3ZlckRlZmluaXRpb25EcmFmdFJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW'
        '9uEh0KCm1hcHBpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIWCgZzb3VyY2UYAyABKAlSBnNvdXJj'
        'ZRIWCgZvZmZzZXQYBCABKA1SBm9mZnNldBIhCgljb21wb25lbnQYBSABKARIAFIJY29tcG9uZW'
        '50iAEBQgwKCl9jb21wb25lbnQ=');

@$core.Deprecated('Use draftHoverResponseDescriptor instead')
const DraftHoverResponse$json = {
  '1': 'DraftHoverResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'found', '3': 3, '4': 1, '5': 8, '10': 'found'},
    {
      '1': 'span',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SourceSpan',
      '9': 0,
      '10': 'span',
      '17': true
    },
    {'1': 'concept_id', '3': 5, '4': 1, '5': 4, '9': 1, '10': 'conceptId', '17': true},
    {'1': 'title', '3': 6, '4': 1, '5': 9, '10': 'title'},
    {'1': 'representation', '3': 7, '4': 1, '5': 9, '10': 'representation'},
    {'1': 'status', '3': 8, '4': 1, '5': 9, '10': 'status'},
    {'1': 'details', '3': 9, '4': 3, '5': 11, '6': '.bdl.v1.HoverDetail', '10': 'details'},
    {'1': 'explanation', '3': 10, '4': 1, '5': 9, '10': 'explanation'},
    {
      '1': 'entity',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EntityRef',
      '9': 2,
      '10': 'entity',
      '17': true
    },
    {'1': 'signature', '3': 12, '4': 1, '5': 9, '10': 'signature'},
    {'1': 'open', '3': 13, '4': 1, '5': 8, '10': 'open'},
  ],
  '8': [
    {'1': '_span'},
    {'1': '_concept_id'},
    {'1': '_entity'},
  ],
};

/// Descriptor for `DraftHoverResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List draftHoverResponseDescriptor = $convert
    .base64Decode('ChJEcmFmdEhvdmVyUmVzcG9uc2USGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEh0KCm1hcH'
        'BpbmdfaWQYAiABKARSCW1hcHBpbmdJZBIUCgVmb3VuZBgDIAEoCFIFZm91bmQSKwoEc3BhbhgE'
        'IAEoCzISLmJkbC52MS5Tb3VyY2VTcGFuSABSBHNwYW6IAQESIgoKY29uY2VwdF9pZBgFIAEoBE'
        'gBUgljb25jZXB0SWSIAQESFAoFdGl0bGUYBiABKAlSBXRpdGxlEiYKDnJlcHJlc2VudGF0aW9u'
        'GAcgASgJUg5yZXByZXNlbnRhdGlvbhIWCgZzdGF0dXMYCCABKAlSBnN0YXR1cxItCgdkZXRhaW'
        'xzGAkgAygLMhMuYmRsLnYxLkhvdmVyRGV0YWlsUgdkZXRhaWxzEiAKC2V4cGxhbmF0aW9uGAog'
        'ASgJUgtleHBsYW5hdGlvbhIuCgZlbnRpdHkYCyABKAsyES5iZGwudjEuRW50aXR5UmVmSAJSBm'
        'VudGl0eYgBARIcCglzaWduYXR1cmUYDCABKAlSCXNpZ25hdHVyZRISCgRvcGVuGA0gASgIUgRv'
        'cGVuQgcKBV9zcGFuQg0KC19jb25jZXB0X2lkQgkKB19lbnRpdHk=');

@$core.Deprecated('Use entityRefDescriptor instead')
const EntityRef$json = {
  '1': 'EntityRef',
  '2': [
    {'1': 'project', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Unit', '9': 0, '10': 'project'},
    {'1': 'concept_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'conceptId'},
    {'1': 'mapping_id', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'mappingId'},
    {'1': 'clock_id', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'clockId'},
    {'1': 'output_id', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'outputId'},
    {'1': 'device_id', '3': 6, '4': 1, '5': 4, '9': 0, '10': 'deviceId'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `EntityRef`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List entityRefDescriptor = $convert
    .base64Decode('CglFbnRpdHlSZWYSKAoHcHJvamVjdBgBIAEoCzIMLmJkbC52MS5Vbml0SABSB3Byb2plY3QSHw'
        'oKY29uY2VwdF9pZBgCIAEoBEgAUgljb25jZXB0SWQSHwoKbWFwcGluZ19pZBgDIAEoBEgAUglt'
        'YXBwaW5nSWQSGwoIY2xvY2tfaWQYBCABKARIAFIHY2xvY2tJZBIdCglvdXRwdXRfaWQYBSABKA'
        'RIAFIIb3V0cHV0SWQSHQoJZGV2aWNlX2lkGAYgASgESABSCGRldmljZUlkQgYKBGtpbmQ=');

@$core.Deprecated('Use hoverEntityRequestDescriptor instead')
const HoverEntityRequest$json = {
  '1': 'HoverEntityRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'entity', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EntityRef', '10': 'entity'},
  ],
};

/// Descriptor for `HoverEntityRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List hoverEntityRequestDescriptor = $convert
    .base64Decode('ChJIb3ZlckVudGl0eVJlcXVlc3QSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEikKBmVudG'
        'l0eRgCIAEoCzIRLmJkbC52MS5FbnRpdHlSZWZSBmVudGl0eQ==');

@$core.Deprecated('Use listSemanticActionsRequestDescriptor instead')
const ListSemanticActionsRequest$json = {
  '1': 'ListSemanticActionsRequest',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'entity', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EntityRef', '10': 'entity'},
  ],
};

/// Descriptor for `ListSemanticActionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSemanticActionsRequestDescriptor = $convert
    .base64Decode('ChpMaXN0U2VtYW50aWNBY3Rpb25zUmVxdWVzdBIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb2'
        '4SKQoGZW50aXR5GAIgASgLMhEuYmRsLnYxLkVudGl0eVJlZlIGZW50aXR5');

@$core.Deprecated('Use semanticActionsResponseDescriptor instead')
const SemanticActionsResponse$json = {
  '1': 'SemanticActionsResponse',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'entity', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EntityRef', '10': 'entity'},
    {'1': 'actions', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.SemanticActionView', '10': 'actions'},
  ],
};

/// Descriptor for `SemanticActionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List semanticActionsResponseDescriptor = $convert
    .base64Decode('ChdTZW1hbnRpY0FjdGlvbnNSZXNwb25zZRIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SKQ'
        'oGZW50aXR5GAIgASgLMhEuYmRsLnYxLkVudGl0eVJlZlIGZW50aXR5EjQKB2FjdGlvbnMYAyAD'
        'KAsyGi5iZGwudjEuU2VtYW50aWNBY3Rpb25WaWV3UgdhY3Rpb25z');

@$core.Deprecated('Use semanticActionViewDescriptor instead')
const SemanticActionView$json = {
  '1': 'SemanticActionView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'title', '3': 2, '4': 1, '5': 9, '10': 'title'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {
      '1': 'applicability',
      '3': 4,
      '4': 1,
      '5': 14,
      '6': '.bdl.v1.ActionApplicability',
      '10': 'applicability'
    },
    {'1': 'reason', '3': 5, '4': 1, '5': 9, '10': 'reason'},
    {'1': 'options', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.ActionChoiceView', '10': 'options'},
    {'1': 'explanation', '3': 7, '4': 1, '5': 9, '10': 'explanation'},
    {'1': 'edits', '3': 8, '4': 3, '5': 11, '6': '.bdl.v1.EditOp', '10': 'edits'},
    {'1': 'addresses', '3': 9, '4': 3, '5': 9, '10': 'addresses'},
    {'1': 'invalidation', '3': 10, '4': 1, '5': 9, '10': 'invalidation'},
  ],
};

/// Descriptor for `SemanticActionView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List semanticActionViewDescriptor = $convert
    .base64Decode('ChJTZW1hbnRpY0FjdGlvblZpZXcSDgoCaWQYASABKAlSAmlkEhQKBXRpdGxlGAIgASgJUgV0aX'
        'RsZRISCgRraW5kGAMgASgJUgRraW5kEkEKDWFwcGxpY2FiaWxpdHkYBCABKA4yGy5iZGwudjEu'
        'QWN0aW9uQXBwbGljYWJpbGl0eVINYXBwbGljYWJpbGl0eRIWCgZyZWFzb24YBSABKAlSBnJlYX'
        'NvbhIyCgdvcHRpb25zGAYgAygLMhguYmRsLnYxLkFjdGlvbkNob2ljZVZpZXdSB29wdGlvbnMS'
        'IAoLZXhwbGFuYXRpb24YByABKAlSC2V4cGxhbmF0aW9uEiQKBWVkaXRzGAggAygLMg4uYmRsLn'
        'YxLkVkaXRPcFIFZWRpdHMSHAoJYWRkcmVzc2VzGAkgAygJUglhZGRyZXNzZXMSIgoMaW52YWxp'
        'ZGF0aW9uGAogASgJUgxpbnZhbGlkYXRpb24=');

@$core.Deprecated('Use actionChoiceViewDescriptor instead')
const ActionChoiceView$json = {
  '1': 'ActionChoiceView',
  '2': [
    {'1': 'label', '3': 1, '4': 1, '5': 9, '10': 'label'},
    {'1': 'edit', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EditOp', '10': 'edit'},
  ],
};

/// Descriptor for `ActionChoiceView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List actionChoiceViewDescriptor = $convert
    .base64Decode('ChBBY3Rpb25DaG9pY2VWaWV3EhQKBWxhYmVsGAEgASgJUgVsYWJlbBIiCgRlZGl0GAIgASgLMg'
        '4uYmRsLnYxLkVkaXRPcFIEZWRpdA==');

@$core.Deprecated('Use hoverDetailDescriptor instead')
const HoverDetail$json = {
  '1': 'HoverDetail',
  '2': [
    {'1': 'label', '3': 1, '4': 1, '5': 9, '10': 'label'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `HoverDetail`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List hoverDetailDescriptor = $convert
    .base64Decode('CgtIb3ZlckRldGFpbBIUCgVsYWJlbBgBIAEoCVIFbGFiZWwSFAoFdmFsdWUYAiABKAlSBXZhbH'
        'Vl');

@$core.Deprecated('Use listConceptTemplatesRequestDescriptor instead')
const ListConceptTemplatesRequest$json = {
  '1': 'ListConceptTemplatesRequest',
};

/// Descriptor for `ListConceptTemplatesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConceptTemplatesRequestDescriptor =
    $convert.base64Decode('ChtMaXN0Q29uY2VwdFRlbXBsYXRlc1JlcXVlc3Q=');

@$core.Deprecated('Use conceptTemplatesResponseDescriptor instead')
const ConceptTemplatesResponse$json = {
  '1': 'ConceptTemplatesResponse',
  '2': [
    {
      '1': 'libraries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ConceptLibraryView',
      '10': 'libraries'
    },
    {'1': 'quantities', '3': 2, '4': 3, '5': 11, '6': '.bdl.v1.QuantityView', '10': 'quantities'},
  ],
};

/// Descriptor for `ConceptTemplatesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List conceptTemplatesResponseDescriptor = $convert
    .base64Decode('ChhDb25jZXB0VGVtcGxhdGVzUmVzcG9uc2USOAoJbGlicmFyaWVzGAEgAygLMhouYmRsLnYxLk'
        'NvbmNlcHRMaWJyYXJ5Vmlld1IJbGlicmFyaWVzEjQKCnF1YW50aXRpZXMYAiADKAsyFC5iZGwu'
        'djEuUXVhbnRpdHlWaWV3UgpxdWFudGl0aWVz');

@$core.Deprecated('Use conceptLibraryViewDescriptor instead')
const ConceptLibraryView$json = {
  '1': 'ConceptLibraryView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'schema_version', '3': 3, '4': 1, '5': 13, '10': 'schemaVersion'},
    {'1': 'version', '3': 4, '4': 1, '5': 9, '10': 'version'},
    {
      '1': 'templates',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ConceptTemplateView',
      '10': 'templates'
    },
  ],
};

/// Descriptor for `ConceptLibraryView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List conceptLibraryViewDescriptor = $convert
    .base64Decode('ChJDb25jZXB0TGlicmFyeVZpZXcSDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbW'
        'USJQoOc2NoZW1hX3ZlcnNpb24YAyABKA1SDXNjaGVtYVZlcnNpb24SGAoHdmVyc2lvbhgEIAEo'
        'CVIHdmVyc2lvbhI5Cgl0ZW1wbGF0ZXMYBSADKAsyGy5iZGwudjEuQ29uY2VwdFRlbXBsYXRlVm'
        'lld1IJdGVtcGxhdGVz');

@$core.Deprecated('Use conceptTemplateViewDescriptor instead')
const ConceptTemplateView$json = {
  '1': 'ConceptTemplateView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'default_name', '3': 3, '4': 1, '5': 9, '10': 'defaultName'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
    {'1': 'category', '3': 5, '4': 1, '5': 9, '10': 'category'},
    {'1': 'role_hint', '3': 6, '4': 1, '5': 14, '6': '.bdl.v1.RoleHint', '10': 'roleHint'},
    {
      '1': 'representation',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Representation',
      '9': 0,
      '10': 'representation',
      '17': true
    },
    {'1': 'type_name', '3': 8, '4': 1, '5': 9, '10': 'typeName'},
    {'1': 'unit', '3': 9, '4': 1, '5': 9, '10': 'unit'},
    {'1': 'keywords', '3': 10, '4': 3, '5': 9, '10': 'keywords'},
    {'1': 'icon', '3': 11, '4': 1, '5': 9, '10': 'icon'},
    {'1': 'source_default_name', '3': 12, '4': 1, '5': 9, '10': 'sourceDefaultName'},
    {
      '1': 'display_names',
      '3': 13,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ConceptTemplateView.DisplayNamesEntry',
      '10': 'displayNames'
    },
    {
      '1': 'descriptions',
      '3': 14,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ConceptTemplateView.DescriptionsEntry',
      '10': 'descriptions'
    },
  ],
  '3': [ConceptTemplateView_DisplayNamesEntry$json, ConceptTemplateView_DescriptionsEntry$json],
  '8': [
    {'1': '_representation'},
  ],
};

@$core.Deprecated('Use conceptTemplateViewDescriptor instead')
const ConceptTemplateView_DisplayNamesEntry$json = {
  '1': 'DisplayNamesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use conceptTemplateViewDescriptor instead')
const ConceptTemplateView_DescriptionsEntry$json = {
  '1': 'DescriptionsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `ConceptTemplateView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List conceptTemplateViewDescriptor = $convert
    .base64Decode('ChNDb25jZXB0VGVtcGxhdGVWaWV3Eg4KAmlkGAEgASgJUgJpZBIhCgxkaXNwbGF5X25hbWUYAi'
        'ABKAlSC2Rpc3BsYXlOYW1lEiEKDGRlZmF1bHRfbmFtZRgDIAEoCVILZGVmYXVsdE5hbWUSIAoL'
        'ZGVzY3JpcHRpb24YBCABKAlSC2Rlc2NyaXB0aW9uEhoKCGNhdGVnb3J5GAUgASgJUghjYXRlZ2'
        '9yeRItCglyb2xlX2hpbnQYBiABKA4yEC5iZGwudjEuUm9sZUhpbnRSCHJvbGVIaW50EkMKDnJl'
        'cHJlc2VudGF0aW9uGAcgASgLMhYuYmRsLnYxLlJlcHJlc2VudGF0aW9uSABSDnJlcHJlc2VudG'
        'F0aW9uiAEBEhsKCXR5cGVfbmFtZRgIIAEoCVIIdHlwZU5hbWUSEgoEdW5pdBgJIAEoCVIEdW5p'
        'dBIaCghrZXl3b3JkcxgKIAMoCVIIa2V5d29yZHMSEgoEaWNvbhgLIAEoCVIEaWNvbhIuChNzb3'
        'VyY2VfZGVmYXVsdF9uYW1lGAwgASgJUhFzb3VyY2VEZWZhdWx0TmFtZRJSCg1kaXNwbGF5X25h'
        'bWVzGA0gAygLMi0uYmRsLnYxLkNvbmNlcHRUZW1wbGF0ZVZpZXcuRGlzcGxheU5hbWVzRW50cn'
        'lSDGRpc3BsYXlOYW1lcxJRCgxkZXNjcmlwdGlvbnMYDiADKAsyLS5iZGwudjEuQ29uY2VwdFRl'
        'bXBsYXRlVmlldy5EZXNjcmlwdGlvbnNFbnRyeVIMZGVzY3JpcHRpb25zGj8KEURpc3BsYXlOYW'
        '1lc0VudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaPwoR'
        'RGVzY3JpcHRpb25zRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbH'
        'VlOgI4AUIRCg9fcmVwcmVzZW50YXRpb24=');

@$core.Deprecated('Use quantityViewDescriptor instead')
const QuantityView$json = {
  '1': 'QuantityView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'type_name', '3': 2, '4': 1, '5': 9, '10': 'typeName'},
    {'1': 'unit', '3': 3, '4': 1, '5': 9, '10': 'unit'},
    {'1': 'dim', '3': 4, '4': 1, '5': 11, '6': '.bdl.v1.Dim', '10': 'dim'},
  ],
};

/// Descriptor for `QuantityView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quantityViewDescriptor = $convert
    .base64Decode('CgxRdWFudGl0eVZpZXcSDgoCaWQYASABKAlSAmlkEhsKCXR5cGVfbmFtZRgCIAEoCVIIdHlwZU'
        '5hbWUSEgoEdW5pdBgDIAEoCVIEdW5pdBIdCgNkaW0YBCABKAsyCy5iZGwudjEuRGltUgNkaW0=');

@$core.Deprecated('Use instantiateConceptTemplateRequestDescriptor instead')
const InstantiateConceptTemplateRequest$json = {
  '1': 'InstantiateConceptTemplateRequest',
  '2': [
    {'1': 'base_revision', '3': 1, '4': 1, '5': 4, '10': 'baseRevision'},
    {'1': 'template_id', '3': 2, '4': 1, '5': 9, '10': 'templateId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'name', '17': true},
    {'1': 'component', '3': 4, '4': 1, '5': 4, '9': 1, '10': 'component', '17': true},
    {'1': 'source_name', '3': 5, '4': 1, '5': 9, '9': 2, '10': 'sourceName', '17': true},
  ],
  '8': [
    {'1': '_name'},
    {'1': '_component'},
    {'1': '_source_name'},
  ],
};

/// Descriptor for `InstantiateConceptTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List instantiateConceptTemplateRequestDescriptor = $convert
    .base64Decode('CiFJbnN0YW50aWF0ZUNvbmNlcHRUZW1wbGF0ZVJlcXVlc3QSIwoNYmFzZV9yZXZpc2lvbhgBIA'
        'EoBFIMYmFzZVJldmlzaW9uEh8KC3RlbXBsYXRlX2lkGAIgASgJUgp0ZW1wbGF0ZUlkEhcKBG5h'
        'bWUYAyABKAlIAFIEbmFtZYgBARIhCgljb21wb25lbnQYBCABKARIAVIJY29tcG9uZW50iAEBEi'
        'QKC3NvdXJjZV9uYW1lGAUgASgJSAJSCnNvdXJjZU5hbWWIAQFCBwoFX25hbWVCDAoKX2NvbXBv'
        'bmVudEIOCgxfc291cmNlX25hbWU=');

@$core.Deprecated('Use listTargetsRequestDescriptor instead')
const ListTargetsRequest$json = {
  '1': 'ListTargetsRequest',
};

/// Descriptor for `ListTargetsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTargetsRequestDescriptor =
    $convert.base64Decode('ChJMaXN0VGFyZ2V0c1JlcXVlc3Q=');

@$core.Deprecated('Use targetsResponseDescriptor instead')
const TargetsResponse$json = {
  '1': 'TargetsResponse',
  '2': [
    {'1': 'targets', '3': 1, '4': 3, '5': 11, '6': '.bdl.v1.TargetView', '10': 'targets'},
  ],
};

/// Descriptor for `TargetsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List targetsResponseDescriptor = $convert
    .base64Decode('Cg9UYXJnZXRzUmVzcG9uc2USLAoHdGFyZ2V0cxgBIAMoCzISLmJkbC52MS5UYXJnZXRWaWV3Ug'
        'd0YXJnZXRz');

@$core.Deprecated('Use targetViewDescriptor instead')
const TargetView$json = {
  '1': 'TargetView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'resource_count', '3': 3, '4': 1, '5': 13, '10': 'resourceCount'},
    {'1': 'display_name', '3': 4, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
    {'1': 'family', '3': 6, '4': 1, '5': 9, '10': 'family'},
    {
      '1': 'capabilities',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.CapabilitySummary',
      '10': 'capabilities'
    },
  ],
};

/// Descriptor for `TargetView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List targetViewDescriptor = $convert
    .base64Decode('CgpUYXJnZXRWaWV3Eg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiUKDnJlc2'
        '91cmNlX2NvdW50GAMgASgNUg1yZXNvdXJjZUNvdW50EiEKDGRpc3BsYXlfbmFtZRgEIAEoCVIL'
        'ZGlzcGxheU5hbWUSIAoLZGVzY3JpcHRpb24YBSABKAlSC2Rlc2NyaXB0aW9uEhYKBmZhbWlseR'
        'gGIAEoCVIGZmFtaWx5Ej0KDGNhcGFiaWxpdGllcxgHIAMoCzIZLmJkbC52MS5DYXBhYmlsaXR5'
        'U3VtbWFyeVIMY2FwYWJpbGl0aWVz');

@$core.Deprecated('Use capabilitySummaryDescriptor instead')
const CapabilitySummary$json = {
  '1': 'CapabilitySummary',
  '2': [
    {'1': 'capability', '3': 1, '4': 1, '5': 9, '10': 'capability'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'resource_count', '3': 3, '4': 1, '5': 13, '10': 'resourceCount'},
    {'1': 'shareable', '3': 4, '4': 1, '5': 8, '10': 'shareable'},
  ],
};

/// Descriptor for `CapabilitySummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List capabilitySummaryDescriptor = $convert
    .base64Decode('ChFDYXBhYmlsaXR5U3VtbWFyeRIeCgpjYXBhYmlsaXR5GAEgASgJUgpjYXBhYmlsaXR5EhQKBW'
        'xhYmVsGAIgASgJUgVsYWJlbBIlCg5yZXNvdXJjZV9jb3VudBgDIAEoDVINcmVzb3VyY2VDb3Vu'
        'dBIcCglzaGFyZWFibGUYBCABKAhSCXNoYXJlYWJsZQ==');

@$core.Deprecated('Use analyzeDeploymentRequestDescriptor instead')
const AnalyzeDeploymentRequest$json = {
  '1': 'AnalyzeDeploymentRequest',
  '2': [
    {'1': 'target_id', '3': 1, '4': 1, '5': 9, '10': 'targetId'},
    {'1': 'revision', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'revision', '17': true},
  ],
  '8': [
    {'1': '_revision'},
  ],
};

/// Descriptor for `AnalyzeDeploymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List analyzeDeploymentRequestDescriptor = $convert
    .base64Decode('ChhBbmFseXplRGVwbG95bWVudFJlcXVlc3QSGwoJdGFyZ2V0X2lkGAEgASgJUgh0YXJnZXRJZB'
        'IfCghyZXZpc2lvbhgCIAEoBEgAUghyZXZpc2lvbogBAUILCglfcmV2aXNpb24=');

@$core.Deprecated('Use deploymentResponseDescriptor instead')
const DeploymentResponse$json = {
  '1': 'DeploymentResponse',
  '2': [
    {
      '1': 'deployment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeploymentAnalysis',
      '10': 'deployment'
    },
  ],
};

/// Descriptor for `DeploymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deploymentResponseDescriptor = $convert
    .base64Decode('ChJEZXBsb3ltZW50UmVzcG9uc2USOgoKZGVwbG95bWVudBgBIAEoCzIaLmJkbC52MS5EZXBsb3'
        'ltZW50QW5hbHlzaXNSCmRlcGxveW1lbnQ=');

@$core.Deprecated('Use deploymentAnalysisDescriptor instead')
const DeploymentAnalysis$json = {
  '1': 'DeploymentAnalysis',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'target', '3': 2, '4': 1, '5': 9, '10': 'target'},
    {'1': 'status', '3': 3, '4': 1, '5': 14, '6': '.bdl.v1.DeploymentStatus', '10': 'status'},
    {
      '1': 'requirements',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.RequirementView',
      '10': 'requirements'
    },
    {'1': 'assignment', '3': 5, '4': 3, '5': 11, '6': '.bdl.v1.Placement', '10': 'assignment'},
    {
      '1': 'dead_end',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeadEnd',
      '9': 0,
      '10': 'deadEnd',
      '17': true
    },
    {'1': 'unbound_devices', '3': 7, '4': 3, '5': 4, '10': 'unboundDevices'},
    {'1': 'unrealised_outputs', '3': 8, '4': 3, '5': 4, '10': 'unrealisedOutputs'},
    {'1': 'diagnostics', '3': 9, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'diagnostics'},
    {'1': 'target_display_name', '3': 10, '4': 1, '5': 9, '10': 'targetDisplayName'},
    {'1': 'design_ready', '3': 11, '4': 1, '5': 8, '10': 'designReady'},
    {'1': 'deployable', '3': 12, '4': 1, '5': 8, '10': 'deployable'},
    {'1': 'missing', '3': 13, '4': 3, '5': 11, '6': '.bdl.v1.MissingItem', '10': 'missing'},
    {'1': 'rows', '3': 14, '4': 3, '5': 11, '6': '.bdl.v1.AssignmentRow', '10': 'rows'},
    {
      '1': 'blocker',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Blocker',
      '9': 1,
      '10': 'blocker',
      '17': true
    },
  ],
  '8': [
    {'1': '_dead_end'},
    {'1': '_blocker'},
  ],
};

/// Descriptor for `DeploymentAnalysis`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deploymentAnalysisDescriptor = $convert
    .base64Decode('ChJEZXBsb3ltZW50QW5hbHlzaXMSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEhYKBnRhcm'
        'dldBgCIAEoCVIGdGFyZ2V0EjAKBnN0YXR1cxgDIAEoDjIYLmJkbC52MS5EZXBsb3ltZW50U3Rh'
        'dHVzUgZzdGF0dXMSOwoMcmVxdWlyZW1lbnRzGAQgAygLMhcuYmRsLnYxLlJlcXVpcmVtZW50Vm'
        'lld1IMcmVxdWlyZW1lbnRzEjEKCmFzc2lnbm1lbnQYBSADKAsyES5iZGwudjEuUGxhY2VtZW50'
        'Ugphc3NpZ25tZW50Ei8KCGRlYWRfZW5kGAYgASgLMg8uYmRsLnYxLkRlYWRFbmRIAFIHZGVhZE'
        'VuZIgBARInCg91bmJvdW5kX2RldmljZXMYByADKARSDnVuYm91bmREZXZpY2VzEi0KEnVucmVh'
        'bGlzZWRfb3V0cHV0cxgIIAMoBFIRdW5yZWFsaXNlZE91dHB1dHMSNAoLZGlhZ25vc3RpY3MYCS'
        'ADKAsyEi5iZGwudjEuRGlhZ25vc3RpY1ILZGlhZ25vc3RpY3MSLgoTdGFyZ2V0X2Rpc3BsYXlf'
        'bmFtZRgKIAEoCVIRdGFyZ2V0RGlzcGxheU5hbWUSIQoMZGVzaWduX3JlYWR5GAsgASgIUgtkZX'
        'NpZ25SZWFkeRIeCgpkZXBsb3lhYmxlGAwgASgIUgpkZXBsb3lhYmxlEi0KB21pc3NpbmcYDSAD'
        'KAsyEy5iZGwudjEuTWlzc2luZ0l0ZW1SB21pc3NpbmcSKQoEcm93cxgOIAMoCzIVLmJkbC52MS'
        '5Bc3NpZ25tZW50Um93UgRyb3dzEi4KB2Jsb2NrZXIYDyABKAsyDy5iZGwudjEuQmxvY2tlckgB'
        'UgdibG9ja2VyiAEBQgsKCV9kZWFkX2VuZEIKCghfYmxvY2tlcg==');

@$core.Deprecated('Use missingItemDescriptor instead')
const MissingItem$json = {
  '1': 'MissingItem',
  '2': [
    {'1': 'kind', '3': 1, '4': 1, '5': 14, '6': '.bdl.v1.MissingKind', '10': 'kind'},
    {'1': 'output_id', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
    {'1': 'output_name', '3': 3, '4': 1, '5': 9, '10': 'outputName'},
    {'1': 'device_id', '3': 4, '4': 1, '5': 4, '9': 1, '10': 'deviceId', '17': true},
    {'1': 'device_name', '3': 5, '4': 1, '5': 9, '10': 'deviceName'},
    {'1': 'mapping_id', '3': 6, '4': 1, '5': 4, '9': 2, '10': 'mappingId', '17': true},
    {'1': 'mapping_name', '3': 7, '4': 1, '5': 9, '10': 'mappingName'},
    {'1': 'message', '3': 8, '4': 1, '5': 9, '10': 'message'},
    {'1': 'explanation', '3': 9, '4': 1, '5': 9, '10': 'explanation'},
  ],
  '8': [
    {'1': '_output_id'},
    {'1': '_device_id'},
    {'1': '_mapping_id'},
  ],
};

/// Descriptor for `MissingItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List missingItemDescriptor = $convert
    .base64Decode('CgtNaXNzaW5nSXRlbRInCgRraW5kGAEgASgOMhMuYmRsLnYxLk1pc3NpbmdLaW5kUgRraW5kEi'
        'AKCW91dHB1dF9pZBgCIAEoBEgAUghvdXRwdXRJZIgBARIfCgtvdXRwdXRfbmFtZRgDIAEoCVIK'
        'b3V0cHV0TmFtZRIgCglkZXZpY2VfaWQYBCABKARIAVIIZGV2aWNlSWSIAQESHwoLZGV2aWNlX2'
        '5hbWUYBSABKAlSCmRldmljZU5hbWUSIgoKbWFwcGluZ19pZBgGIAEoBEgCUgltYXBwaW5nSWSI'
        'AQESIQoMbWFwcGluZ19uYW1lGAcgASgJUgttYXBwaW5nTmFtZRIYCgdtZXNzYWdlGAggASgJUg'
        'dtZXNzYWdlEiAKC2V4cGxhbmF0aW9uGAkgASgJUgtleHBsYW5hdGlvbkIMCgpfb3V0cHV0X2lk'
        'QgwKCl9kZXZpY2VfaWRCDQoLX21hcHBpbmdfaWQ=');

@$core.Deprecated('Use assignmentRowDescriptor instead')
const AssignmentRow$json = {
  '1': 'AssignmentRow',
  '2': [
    {'1': 'output_id', '3': 1, '4': 1, '5': 4, '9': 0, '10': 'outputId', '17': true},
    {'1': 'output_name', '3': 2, '4': 1, '5': 9, '10': 'outputName'},
    {'1': 'device_id', '3': 3, '4': 1, '5': 4, '10': 'deviceId'},
    {'1': 'device_name', '3': 4, '4': 1, '5': 9, '10': 'deviceName'},
    {'1': 'device_kind', '3': 5, '4': 1, '5': 14, '6': '.bdl.v1.DeviceKind', '10': 'deviceKind'},
    {'1': 'device_kind_label', '3': 6, '4': 1, '5': 9, '10': 'deviceKindLabel'},
    {'1': 'requirement_index', '3': 7, '4': 1, '5': 13, '10': 'requirementIndex'},
    {'1': 'requirement_label', '3': 8, '4': 1, '5': 9, '10': 'requirementLabel'},
    {'1': 'capability', '3': 9, '4': 1, '5': 9, '10': 'capability'},
    {'1': 'capability_label', '3': 10, '4': 1, '5': 9, '10': 'capabilityLabel'},
    {'1': 'fixed', '3': 11, '4': 1, '5': 9, '9': 1, '10': 'fixed', '17': true},
    {'1': 'resource', '3': 12, '4': 1, '5': 9, '9': 2, '10': 'resource', '17': true},
    {'1': 'resource_label', '3': 13, '4': 1, '5': 9, '9': 3, '10': 'resourceLabel', '17': true},
  ],
  '8': [
    {'1': '_output_id'},
    {'1': '_fixed'},
    {'1': '_resource'},
    {'1': '_resource_label'},
  ],
};

/// Descriptor for `AssignmentRow`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List assignmentRowDescriptor = $convert
    .base64Decode('Cg1Bc3NpZ25tZW50Um93EiAKCW91dHB1dF9pZBgBIAEoBEgAUghvdXRwdXRJZIgBARIfCgtvdX'
        'RwdXRfbmFtZRgCIAEoCVIKb3V0cHV0TmFtZRIbCglkZXZpY2VfaWQYAyABKARSCGRldmljZUlk'
        'Eh8KC2RldmljZV9uYW1lGAQgASgJUgpkZXZpY2VOYW1lEjMKC2RldmljZV9raW5kGAUgASgOMh'
        'IuYmRsLnYxLkRldmljZUtpbmRSCmRldmljZUtpbmQSKgoRZGV2aWNlX2tpbmRfbGFiZWwYBiAB'
        'KAlSD2RldmljZUtpbmRMYWJlbBIrChFyZXF1aXJlbWVudF9pbmRleBgHIAEoDVIQcmVxdWlyZW'
        '1lbnRJbmRleBIrChFyZXF1aXJlbWVudF9sYWJlbBgIIAEoCVIQcmVxdWlyZW1lbnRMYWJlbBIe'
        'CgpjYXBhYmlsaXR5GAkgASgJUgpjYXBhYmlsaXR5EikKEGNhcGFiaWxpdHlfbGFiZWwYCiABKA'
        'lSD2NhcGFiaWxpdHlMYWJlbBIZCgVmaXhlZBgLIAEoCUgBUgVmaXhlZIgBARIfCghyZXNvdXJj'
        'ZRgMIAEoCUgCUghyZXNvdXJjZYgBARIqCg5yZXNvdXJjZV9sYWJlbBgNIAEoCUgDUg1yZXNvdX'
        'JjZUxhYmVsiAEBQgwKCl9vdXRwdXRfaWRCCAoGX2ZpeGVkQgsKCV9yZXNvdXJjZUIRCg9fcmVz'
        'b3VyY2VfbGFiZWw=');

@$core.Deprecated('Use blockerDescriptor instead')
const Blocker$json = {
  '1': 'Blocker',
  '2': [
    {'1': 'device_id', '3': 1, '4': 1, '5': 4, '10': 'deviceId'},
    {'1': 'device_name', '3': 2, '4': 1, '5': 9, '10': 'deviceName'},
    {'1': 'requirement_index', '3': 3, '4': 1, '5': 13, '10': 'requirementIndex'},
    {'1': 'requirement_label', '3': 4, '4': 1, '5': 9, '10': 'requirementLabel'},
    {'1': 'capability', '3': 5, '4': 1, '5': 9, '10': 'capability'},
    {'1': 'capability_label', '3': 6, '4': 1, '5': 9, '10': 'capabilityLabel'},
    {
      '1': 'no_capable_resource',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Unit',
      '9': 0,
      '10': 'noCapableResource'
    },
    {'1': 'fixed_unavailable', '3': 8, '4': 1, '5': 9, '9': 0, '10': 'fixedUnavailable'},
    {
      '1': 'blocked',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.BlockerCandidates',
      '9': 0,
      '10': 'blocked'
    },
    {'1': 'message', '3': 10, '4': 1, '5': 9, '10': 'message'},
    {'1': 'explanation', '3': 11, '4': 1, '5': 9, '10': 'explanation'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `Blocker`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List blockerDescriptor = $convert
    .base64Decode('CgdCbG9ja2VyEhsKCWRldmljZV9pZBgBIAEoBFIIZGV2aWNlSWQSHwoLZGV2aWNlX25hbWUYAi'
        'ABKAlSCmRldmljZU5hbWUSKwoRcmVxdWlyZW1lbnRfaW5kZXgYAyABKA1SEHJlcXVpcmVtZW50'
        'SW5kZXgSKwoRcmVxdWlyZW1lbnRfbGFiZWwYBCABKAlSEHJlcXVpcmVtZW50TGFiZWwSHgoKY2'
        'FwYWJpbGl0eRgFIAEoCVIKY2FwYWJpbGl0eRIpChBjYXBhYmlsaXR5X2xhYmVsGAYgASgJUg9j'
        'YXBhYmlsaXR5TGFiZWwSPgoTbm9fY2FwYWJsZV9yZXNvdXJjZRgHIAEoCzIMLmJkbC52MS5Vbm'
        'l0SABSEW5vQ2FwYWJsZVJlc291cmNlEi0KEWZpeGVkX3VuYXZhaWxhYmxlGAggASgJSABSEGZp'
        'eGVkVW5hdmFpbGFibGUSNQoHYmxvY2tlZBgJIAEoCzIZLmJkbC52MS5CbG9ja2VyQ2FuZGlkYX'
        'Rlc0gAUgdibG9ja2VkEhgKB21lc3NhZ2UYCiABKAlSB21lc3NhZ2USIAoLZXhwbGFuYXRpb24Y'
        'CyABKAlSC2V4cGxhbmF0aW9uQgYKBGtpbmQ=');

@$core.Deprecated('Use blockerCandidatesDescriptor instead')
const BlockerCandidates$json = {
  '1': 'BlockerCandidates',
  '2': [
    {
      '1': 'candidates',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.BlockerCandidate',
      '10': 'candidates'
    },
  ],
};

/// Descriptor for `BlockerCandidates`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List blockerCandidatesDescriptor = $convert
    .base64Decode('ChFCbG9ja2VyQ2FuZGlkYXRlcxI4CgpjYW5kaWRhdGVzGAEgAygLMhguYmRsLnYxLkJsb2NrZX'
        'JDYW5kaWRhdGVSCmNhbmRpZGF0ZXM=');

@$core.Deprecated('Use blockerCandidateDescriptor instead')
const BlockerCandidate$json = {
  '1': 'BlockerCandidate',
  '2': [
    {'1': 'resource', '3': 1, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'resource_label', '3': 2, '4': 1, '5': 9, '10': 'resourceLabel'},
    {'1': 'held_by_device_id', '3': 3, '4': 1, '5': 4, '10': 'heldByDeviceId'},
    {'1': 'held_by_device_name', '3': 4, '4': 1, '5': 9, '10': 'heldByDeviceName'},
    {'1': 'held_by_requirement_index', '3': 5, '4': 1, '5': 13, '10': 'heldByRequirementIndex'},
    {'1': 'held_by_requirement_label', '3': 6, '4': 1, '5': 9, '10': 'heldByRequirementLabel'},
  ],
};

/// Descriptor for `BlockerCandidate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List blockerCandidateDescriptor = $convert
    .base64Decode('ChBCbG9ja2VyQ2FuZGlkYXRlEhoKCHJlc291cmNlGAEgASgJUghyZXNvdXJjZRIlCg5yZXNvdX'
        'JjZV9sYWJlbBgCIAEoCVINcmVzb3VyY2VMYWJlbBIpChFoZWxkX2J5X2RldmljZV9pZBgDIAEo'
        'BFIOaGVsZEJ5RGV2aWNlSWQSLQoTaGVsZF9ieV9kZXZpY2VfbmFtZRgEIAEoCVIQaGVsZEJ5RG'
        'V2aWNlTmFtZRI5ChloZWxkX2J5X3JlcXVpcmVtZW50X2luZGV4GAUgASgNUhZoZWxkQnlSZXF1'
        'aXJlbWVudEluZGV4EjkKGWhlbGRfYnlfcmVxdWlyZW1lbnRfbGFiZWwYBiABKAlSFmhlbGRCeV'
        'JlcXVpcmVtZW50TGFiZWw=');

@$core.Deprecated('Use requirementViewDescriptor instead')
const RequirementView$json = {
  '1': 'RequirementView',
  '2': [
    {'1': 'device_id', '3': 1, '4': 1, '5': 4, '10': 'deviceId'},
    {'1': 'index', '3': 2, '4': 1, '5': 13, '10': 'index'},
    {'1': 'capability', '3': 3, '4': 1, '5': 9, '10': 'capability'},
    {'1': 'fixed', '3': 4, '4': 1, '5': 9, '9': 0, '10': 'fixed', '17': true},
    {'1': 'label', '3': 5, '4': 1, '5': 9, '10': 'label'},
  ],
  '8': [
    {'1': '_fixed'},
  ],
};

/// Descriptor for `RequirementView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requirementViewDescriptor = $convert
    .base64Decode('Cg9SZXF1aXJlbWVudFZpZXcSGwoJZGV2aWNlX2lkGAEgASgEUghkZXZpY2VJZBIUCgVpbmRleB'
        'gCIAEoDVIFaW5kZXgSHgoKY2FwYWJpbGl0eRgDIAEoCVIKY2FwYWJpbGl0eRIZCgVmaXhlZBgE'
        'IAEoCUgAUgVmaXhlZIgBARIUCgVsYWJlbBgFIAEoCVIFbGFiZWxCCAoGX2ZpeGVk');

@$core.Deprecated('Use placementDescriptor instead')
const Placement$json = {
  '1': 'Placement',
  '2': [
    {'1': 'device_id', '3': 1, '4': 1, '5': 4, '10': 'deviceId'},
    {'1': 'index', '3': 2, '4': 1, '5': 13, '10': 'index'},
    {'1': 'resource', '3': 3, '4': 1, '5': 9, '10': 'resource'},
  ],
};

/// Descriptor for `Placement`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List placementDescriptor = $convert
    .base64Decode('CglQbGFjZW1lbnQSGwoJZGV2aWNlX2lkGAEgASgEUghkZXZpY2VJZBIUCgVpbmRleBgCIAEoDV'
        'IFaW5kZXgSGgoIcmVzb3VyY2UYAyABKAlSCHJlc291cmNl');

@$core.Deprecated('Use deadEndDescriptor instead')
const DeadEnd$json = {
  '1': 'DeadEnd',
  '2': [
    {'1': 'device_id', '3': 1, '4': 1, '5': 4, '10': 'deviceId'},
    {'1': 'index', '3': 2, '4': 1, '5': 13, '10': 'index'},
    {
      '1': 'no_capable_resource',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.Unit',
      '9': 0,
      '10': 'noCapableResource'
    },
    {'1': 'fixed_unavailable', '3': 4, '4': 1, '5': 9, '9': 0, '10': 'fixedUnavailable'},
    {
      '1': 'blocked',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.BlockedCandidates',
      '9': 0,
      '10': 'blocked'
    },
    {'1': 'placed', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.Placement', '10': 'placed'},
  ],
  '8': [
    {'1': 'reason'},
  ],
};

/// Descriptor for `DeadEnd`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deadEndDescriptor = $convert
    .base64Decode('CgdEZWFkRW5kEhsKCWRldmljZV9pZBgBIAEoBFIIZGV2aWNlSWQSFAoFaW5kZXgYAiABKA1SBW'
        'luZGV4Ej4KE25vX2NhcGFibGVfcmVzb3VyY2UYAyABKAsyDC5iZGwudjEuVW5pdEgAUhFub0Nh'
        'cGFibGVSZXNvdXJjZRItChFmaXhlZF91bmF2YWlsYWJsZRgEIAEoCUgAUhBmaXhlZFVuYXZhaW'
        'xhYmxlEjUKB2Jsb2NrZWQYBSABKAsyGS5iZGwudjEuQmxvY2tlZENhbmRpZGF0ZXNIAFIHYmxv'
        'Y2tlZBIpCgZwbGFjZWQYBiADKAsyES5iZGwudjEuUGxhY2VtZW50UgZwbGFjZWRCCAoGcmVhc2'
        '9u');

@$core.Deprecated('Use blockedCandidatesDescriptor instead')
const BlockedCandidates$json = {
  '1': 'BlockedCandidates',
  '2': [
    {
      '1': 'candidates',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.BlockedCandidate',
      '10': 'candidates'
    },
  ],
};

/// Descriptor for `BlockedCandidates`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List blockedCandidatesDescriptor = $convert
    .base64Decode('ChFCbG9ja2VkQ2FuZGlkYXRlcxI4CgpjYW5kaWRhdGVzGAEgAygLMhguYmRsLnYxLkJsb2NrZW'
        'RDYW5kaWRhdGVSCmNhbmRpZGF0ZXM=');

@$core.Deprecated('Use blockedCandidateDescriptor instead')
const BlockedCandidate$json = {
  '1': 'BlockedCandidate',
  '2': [
    {'1': 'resource', '3': 1, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'held_by_device_id', '3': 2, '4': 1, '5': 4, '10': 'heldByDeviceId'},
    {'1': 'held_by_index', '3': 3, '4': 1, '5': 13, '10': 'heldByIndex'},
  ],
};

/// Descriptor for `BlockedCandidate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List blockedCandidateDescriptor = $convert
    .base64Decode('ChBCbG9ja2VkQ2FuZGlkYXRlEhoKCHJlc291cmNlGAEgASgJUghyZXNvdXJjZRIpChFoZWxkX2'
        'J5X2RldmljZV9pZBgCIAEoBFIOaGVsZEJ5RGV2aWNlSWQSIgoNaGVsZF9ieV9pbmRleBgDIAEo'
        'DVILaGVsZEJ5SW5kZXg=');

@$core.Deprecated('Use initSystemProjectRequestDescriptor instead')
const InitSystemProjectRequest$json = {
  '1': 'InitSystemProjectRequest',
  '2': [
    {'1': 'root_path', '3': 1, '4': 1, '5': 9, '10': 'rootPath'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `InitSystemProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initSystemProjectRequestDescriptor = $convert
    .base64Decode('ChhJbml0U3lzdGVtUHJvamVjdFJlcXVlc3QSGwoJcm9vdF9wYXRoGAEgASgJUghyb290UGF0aB'
        'ISCgRuYW1lGAIgASgJUgRuYW1l');

@$core.Deprecated('Use initTextProjectRequestDescriptor instead')
const InitTextProjectRequest$json = {
  '1': 'InitTextProjectRequest',
  '2': [
    {'1': 'root_path', '3': 1, '4': 1, '5': 9, '10': 'rootPath'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `InitTextProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initTextProjectRequestDescriptor = $convert
    .base64Decode('ChZJbml0VGV4dFByb2plY3RSZXF1ZXN0EhsKCXJvb3RfcGF0aBgBIAEoCVIIcm9vdFBhdGgSEg'
        'oEbmFtZRgCIAEoCVIEbmFtZQ==');

@$core.Deprecated('Use reloadProjectRequestDescriptor instead')
const ReloadProjectRequest$json = {
  '1': 'ReloadProjectRequest',
};

/// Descriptor for `ReloadProjectRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reloadProjectRequestDescriptor =
    $convert.base64Decode('ChRSZWxvYWRQcm9qZWN0UmVxdWVzdA==');

@$core.Deprecated('Use getSourcesRequestDescriptor instead')
const GetSourcesRequest$json = {
  '1': 'GetSourcesRequest',
};

/// Descriptor for `GetSourcesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSourcesRequestDescriptor =
    $convert.base64Decode('ChFHZXRTb3VyY2VzUmVxdWVzdA==');

@$core.Deprecated('Use sourcesResponseDescriptor instead')
const SourcesResponse$json = {
  '1': 'SourcesResponse',
  '2': [
    {'1': 'sources', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.SourcesView', '10': 'sources'},
  ],
};

/// Descriptor for `SourcesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourcesResponseDescriptor = $convert
    .base64Decode('Cg9Tb3VyY2VzUmVzcG9uc2USLQoHc291cmNlcxgBIAEoCzITLmJkbC52MS5Tb3VyY2VzVmlld1'
        'IHc291cmNlcw==');

@$core.Deprecated('Use sourcesViewDescriptor instead')
const SourcesView$json = {
  '1': 'SourcesView',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'files', '3': 2, '4': 3, '5': 11, '6': '.bdl.v1.SourceFileView', '10': 'files'},
    {
      '1': 'diagnostics',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.SourceDiagnostic',
      '10': 'diagnostics'
    },
  ],
};

/// Descriptor for `SourcesView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourcesViewDescriptor = $convert
    .base64Decode('CgtTb3VyY2VzVmlldxIaCghyZXZpc2lvbhgBIAEoBFIIcmV2aXNpb24SLAoFZmlsZXMYAiADKA'
        'syFi5iZGwudjEuU291cmNlRmlsZVZpZXdSBWZpbGVzEjoKC2RpYWdub3N0aWNzGAMgAygLMhgu'
        'YmRsLnYxLlNvdXJjZURpYWdub3N0aWNSC2RpYWdub3N0aWNz');

@$core.Deprecated('Use sourceFileViewDescriptor instead')
const SourceFileView$json = {
  '1': 'SourceFileView',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'text', '3': 2, '4': 1, '5': 9, '10': 'text'},
    {'1': 'draft', '3': 3, '4': 1, '5': 8, '10': 'draft'},
    {'1': 'anchors', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.SourceAnchor', '10': 'anchors'},
  ],
};

/// Descriptor for `SourceFileView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourceFileViewDescriptor = $convert
    .base64Decode('Cg5Tb3VyY2VGaWxlVmlldxISCgRwYXRoGAEgASgJUgRwYXRoEhIKBHRleHQYAiABKAlSBHRleH'
        'QSFAoFZHJhZnQYAyABKAhSBWRyYWZ0Ei4KB2FuY2hvcnMYBCADKAsyFC5iZGwudjEuU291cmNl'
        'QW5jaG9yUgdhbmNob3Jz');

@$core.Deprecated('Use sourceAnchorDescriptor instead')
const SourceAnchor$json = {
  '1': 'SourceAnchor',
  '2': [
    {'1': 'start', '3': 1, '4': 1, '5': 13, '10': 'start'},
    {'1': 'end', '3': 2, '4': 1, '5': 13, '10': 'end'},
    {'1': 'component', '3': 3, '4': 1, '5': 4, '9': 1, '10': 'component', '17': true},
    {'1': 'concept_id', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'conceptId'},
    {'1': 'mapping_id', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'mappingId'},
    {'1': 'clock_id', '3': 6, '4': 1, '5': 4, '9': 0, '10': 'clockId'},
    {'1': 'output_id', '3': 7, '4': 1, '5': 4, '9': 0, '10': 'outputId'},
    {'1': 'device_id', '3': 8, '4': 1, '5': 4, '9': 0, '10': 'deviceId'},
    {'1': 'component_id', '3': 9, '4': 1, '5': 4, '9': 0, '10': 'componentId'},
    {'1': 'instance_id', '3': 10, '4': 1, '5': 4, '9': 0, '10': 'instanceId'},
    {'1': 'port_id', '3': 11, '4': 1, '5': 4, '9': 0, '10': 'portId'},
  ],
  '8': [
    {'1': 'entity'},
    {'1': '_component'},
  ],
};

/// Descriptor for `SourceAnchor`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourceAnchorDescriptor = $convert
    .base64Decode('CgxTb3VyY2VBbmNob3ISFAoFc3RhcnQYASABKA1SBXN0YXJ0EhAKA2VuZBgCIAEoDVIDZW5kEi'
        'EKCWNvbXBvbmVudBgDIAEoBEgBUgljb21wb25lbnSIAQESHwoKY29uY2VwdF9pZBgEIAEoBEgA'
        'Ugljb25jZXB0SWQSHwoKbWFwcGluZ19pZBgFIAEoBEgAUgltYXBwaW5nSWQSGwoIY2xvY2tfaW'
        'QYBiABKARIAFIHY2xvY2tJZBIdCglvdXRwdXRfaWQYByABKARIAFIIb3V0cHV0SWQSHQoJZGV2'
        'aWNlX2lkGAggASgESABSCGRldmljZUlkEiMKDGNvbXBvbmVudF9pZBgJIAEoBEgAUgtjb21wb2'
        '5lbnRJZBIhCgtpbnN0YW5jZV9pZBgKIAEoBEgAUgppbnN0YW5jZUlkEhkKB3BvcnRfaWQYCyAB'
        'KARIAFIGcG9ydElkQggKBmVudGl0eUIMCgpfY29tcG9uZW50');

@$core.Deprecated('Use sourceDiagnosticDescriptor instead')
const SourceDiagnostic$json = {
  '1': 'SourceDiagnostic',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
    {'1': 'message', '3': 3, '4': 1, '5': 9, '10': 'message'},
    {'1': 'start', '3': 4, '4': 1, '5': 13, '10': 'start'},
    {'1': 'end', '3': 5, '4': 1, '5': 13, '10': 'end'},
    {'1': 'open', '3': 6, '4': 1, '5': 8, '10': 'open'},
  ],
};

/// Descriptor for `SourceDiagnostic`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourceDiagnosticDescriptor = $convert
    .base64Decode('ChBTb3VyY2VEaWFnbm9zdGljEhIKBHBhdGgYASABKAlSBHBhdGgSEgoEY29kZRgCIAEoCVIEY2'
        '9kZRIYCgdtZXNzYWdlGAMgASgJUgdtZXNzYWdlEhQKBXN0YXJ0GAQgASgNUgVzdGFydBIQCgNl'
        'bmQYBSABKA1SA2VuZBISCgRvcGVuGAYgASgIUgRvcGVu');

@$core.Deprecated('Use applySourceEditRequestDescriptor instead')
const ApplySourceEditRequest$json = {
  '1': 'ApplySourceEditRequest',
  '2': [
    {'1': 'base_revision', '3': 1, '4': 1, '5': 4, '10': 'baseRevision'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {'1': 'text', '3': 3, '4': 1, '5': 9, '10': 'text'},
  ],
};

/// Descriptor for `ApplySourceEditRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List applySourceEditRequestDescriptor = $convert
    .base64Decode('ChZBcHBseVNvdXJjZUVkaXRSZXF1ZXN0EiMKDWJhc2VfcmV2aXNpb24YASABKARSDGJhc2VSZX'
        'Zpc2lvbhISCgRwYXRoGAIgASgJUgRwYXRoEhIKBHRleHQYAyABKAlSBHRleHQ=');

@$core.Deprecated('Use sourceEditAppliedDescriptor instead')
const SourceEditApplied$json = {
  '1': 'SourceEditApplied',
  '2': [
    {'1': 'accepted', '3': 1, '4': 1, '5': 8, '10': 'accepted'},
    {'1': 'project', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'project'},
    {'1': 'sources', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.SourcesView', '10': 'sources'},
  ],
};

/// Descriptor for `SourceEditApplied`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sourceEditAppliedDescriptor = $convert
    .base64Decode('ChFTb3VyY2VFZGl0QXBwbGllZBIaCghhY2NlcHRlZBgBIAEoCFIIYWNjZXB0ZWQSMwoHcHJvam'
        'VjdBgCIAEoCzIZLmJkbC52MS5Qcm9qZWN0UHJvamVjdGlvblIHcHJvamVjdBItCgdzb3VyY2Vz'
        'GAMgASgLMhMuYmRsLnYxLlNvdXJjZXNWaWV3Ugdzb3VyY2Vz');

@$core.Deprecated('Use getSystemRequestDescriptor instead')
const GetSystemRequest$json = {
  '1': 'GetSystemRequest',
};

/// Descriptor for `GetSystemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSystemRequestDescriptor =
    $convert.base64Decode('ChBHZXRTeXN0ZW1SZXF1ZXN0');

@$core.Deprecated('Use systemResponseDescriptor instead')
const SystemResponse$json = {
  '1': 'SystemResponse',
  '2': [
    {'1': 'system', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.SystemView', '10': 'system'},
  ],
};

/// Descriptor for `SystemResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemResponseDescriptor = $convert
    .base64Decode('Cg5TeXN0ZW1SZXNwb25zZRIqCgZzeXN0ZW0YASABKAsyEi5iZGwudjEuU3lzdGVtVmlld1IGc3'
        'lzdGVt');

@$core.Deprecated('Use applySystemEditRequestDescriptor instead')
const ApplySystemEditRequest$json = {
  '1': 'ApplySystemEditRequest',
  '2': [
    {'1': 'base_revision', '3': 1, '4': 1, '5': 4, '10': 'baseRevision'},
    {'1': 'op', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.SystemEditOp', '10': 'op'},
  ],
};

/// Descriptor for `ApplySystemEditRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List applySystemEditRequestDescriptor = $convert
    .base64Decode('ChZBcHBseVN5c3RlbUVkaXRSZXF1ZXN0EiMKDWJhc2VfcmV2aXNpb24YASABKARSDGJhc2VSZX'
        'Zpc2lvbhIkCgJvcBgCIAEoCzIULmJkbC52MS5TeXN0ZW1FZGl0T3BSAm9w');

@$core.Deprecated('Use systemEditAppliedDescriptor instead')
const SystemEditApplied$json = {
  '1': 'SystemEditApplied',
  '2': [
    {'1': 'system', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.SystemView', '10': 'system'},
    {'1': 'project', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'project'},
    {'1': 'outcome', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.SystemEditOutcome', '10': 'outcome'},
  ],
};

/// Descriptor for `SystemEditApplied`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemEditAppliedDescriptor = $convert
    .base64Decode('ChFTeXN0ZW1FZGl0QXBwbGllZBIqCgZzeXN0ZW0YASABKAsyEi5iZGwudjEuU3lzdGVtVmlld1'
        'IGc3lzdGVtEjMKB3Byb2plY3QYAiABKAsyGS5iZGwudjEuUHJvamVjdFByb2plY3Rpb25SB3By'
        'b2plY3QSMwoHb3V0Y29tZRgDIAEoCzIZLmJkbC52MS5TeXN0ZW1FZGl0T3V0Y29tZVIHb3V0Y2'
        '9tZQ==');

@$core.Deprecated('Use runSystemAnalysisRequestDescriptor instead')
const RunSystemAnalysisRequest$json = {
  '1': 'RunSystemAnalysisRequest',
};

/// Descriptor for `RunSystemAnalysisRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List runSystemAnalysisRequestDescriptor =
    $convert.base64Decode('ChhSdW5TeXN0ZW1BbmFseXNpc1JlcXVlc3Q=');

@$core.Deprecated('Use systemAnalysisResponseDescriptor instead')
const SystemAnalysisResponse$json = {
  '1': 'SystemAnalysisResponse',
  '2': [
    {'1': 'analysis', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.SystemAnalysisView', '10': 'analysis'},
  ],
};

/// Descriptor for `SystemAnalysisResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemAnalysisResponseDescriptor = $convert
    .base64Decode('ChZTeXN0ZW1BbmFseXNpc1Jlc3BvbnNlEjYKCGFuYWx5c2lzGAEgASgLMhouYmRsLnYxLlN5c3'
        'RlbUFuYWx5c2lzVmlld1IIYW5hbHlzaXM=');

@$core.Deprecated('Use portViewDescriptor instead')
const PortView$json = {
  '1': 'PortView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'kind', '3': 4, '4': 1, '5': 14, '6': '.bdl.v1.PortKind', '10': 'kind'},
    {'1': 'decl', '3': 5, '4': 1, '5': 4, '10': 'decl'},
    {'1': 'contract', '3': 6, '4': 1, '5': 11, '6': '.bdl.v1.PortContractView', '10': 'contract'},
  ],
};

/// Descriptor for `PortView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portViewDescriptor = $convert
    .base64Decode('CghQb3J0VmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCgtkZXNjcm'
        'lwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SJAoEa2luZBgEIAEoDjIQLmJkbC52MS5Qb3J0S2lu'
        'ZFIEa2luZBISCgRkZWNsGAUgASgEUgRkZWNsEjQKCGNvbnRyYWN0GAYgASgLMhguYmRsLnYxLl'
        'BvcnRDb250cmFjdFZpZXdSCGNvbnRyYWN0');

@$core.Deprecated('Use portContractViewDescriptor instead')
const PortContractView$json = {
  '1': 'PortContractView',
  '2': [
    {'1': 'signature', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Signature', '10': 'signature'},
    {'1': 'input_names', '3': 2, '4': 3, '5': 9, '10': 'inputNames'},
    {'1': 'output_name', '3': 3, '4': 1, '5': 9, '10': 'outputName'},
    {'1': 'shared', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.IdPair', '10': 'shared'},
    {
      '1': 'clock_kind',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.bdl.v1.ClockContractKind',
      '10': 'clockKind'
    },
    {'1': 'clock_id', '3': 6, '4': 1, '5': 4, '9': 0, '10': 'clockId', '17': true},
    {'1': 'clock_name', '3': 7, '4': 1, '5': 9, '10': 'clockName'},
    {'1': 'commitments', '3': 8, '4': 3, '5': 9, '10': 'commitments'},
  ],
  '8': [
    {'1': '_clock_id'},
  ],
};

/// Descriptor for `PortContractView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portContractViewDescriptor = $convert
    .base64Decode('ChBQb3J0Q29udHJhY3RWaWV3Ei8KCXNpZ25hdHVyZRgBIAEoCzIRLmJkbC52MS5TaWduYXR1cm'
        'VSCXNpZ25hdHVyZRIfCgtpbnB1dF9uYW1lcxgCIAMoCVIKaW5wdXROYW1lcxIfCgtvdXRwdXRf'
        'bmFtZRgDIAEoCVIKb3V0cHV0TmFtZRImCgZzaGFyZWQYBCADKAsyDi5iZGwudjEuSWRQYWlyUg'
        'ZzaGFyZWQSOAoKY2xvY2tfa2luZBgFIAEoDjIZLmJkbC52MS5DbG9ja0NvbnRyYWN0S2luZFIJ'
        'Y2xvY2tLaW5kEh4KCGNsb2NrX2lkGAYgASgESABSB2Nsb2NrSWSIAQESHQoKY2xvY2tfbmFtZR'
        'gHIAEoCVIJY2xvY2tOYW1lEiAKC2NvbW1pdG1lbnRzGAggAygJUgtjb21taXRtZW50c0ILCglf'
        'Y2xvY2tfaWQ=');

@$core.Deprecated('Use componentViewDescriptor instead')
const ComponentView$json = {
  '1': 'ComponentView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'ports', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.PortView', '10': 'ports'},
    {'1': 'clock_params', '3': 5, '4': 3, '5': 4, '10': 'clockParams'},
    {
      '1': 'shared_concepts',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.IdPair',
      '10': 'sharedConcepts'
    },
    {
      '1': 'external_outputs',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.IdPair',
      '10': 'externalOutputs'
    },
    {'1': 'body', '3': 8, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'body'},
    {'1': 'stamp', '3': 9, '4': 1, '5': 4, '10': 'stamp'},
    {'1': 'interface_stamp', '3': 10, '4': 1, '5': 4, '10': 'interfaceStamp'},
  ],
};

/// Descriptor for `ComponentView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List componentViewDescriptor = $convert
    .base64Decode('Cg1Db21wb25lbnRWaWV3Eg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiAKC2'
        'Rlc2NyaXB0aW9uGAMgASgJUgtkZXNjcmlwdGlvbhImCgVwb3J0cxgEIAMoCzIQLmJkbC52MS5Q'
        'b3J0Vmlld1IFcG9ydHMSIQoMY2xvY2tfcGFyYW1zGAUgAygEUgtjbG9ja1BhcmFtcxI3Cg9zaG'
        'FyZWRfY29uY2VwdHMYBiADKAsyDi5iZGwudjEuSWRQYWlyUg5zaGFyZWRDb25jZXB0cxI5ChBl'
        'eHRlcm5hbF9vdXRwdXRzGAcgAygLMg4uYmRsLnYxLklkUGFpclIPZXh0ZXJuYWxPdXRwdXRzEi'
        '0KBGJvZHkYCCABKAsyGS5iZGwudjEuUHJvamVjdFByb2plY3Rpb25SBGJvZHkSFAoFc3RhbXAY'
        'CSABKARSBXN0YW1wEicKD2ludGVyZmFjZV9zdGFtcBgKIAEoBFIOaW50ZXJmYWNlU3RhbXA=');

@$core.Deprecated('Use idPairDescriptor instead')
const IdPair$json = {
  '1': 'IdPair',
  '2': [
    {'1': 'local', '3': 1, '4': 1, '5': 4, '10': 'local'},
    {'1': 'system', '3': 2, '4': 1, '5': 4, '10': 'system'},
  ],
};

/// Descriptor for `IdPair`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List idPairDescriptor = $convert
    .base64Decode('CgZJZFBhaXISFAoFbG9jYWwYASABKARSBWxvY2FsEhYKBnN5c3RlbRgCIAEoBFIGc3lzdGVt');

@$core.Deprecated('Use componentInstanceViewDescriptor instead')
const ComponentInstanceView$json = {
  '1': 'ComponentInstanceView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'component', '3': 2, '4': 1, '5': 4, '10': 'component'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'clock_bindings', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.IdPair', '10': 'clockBindings'},
    {
      '1': 'parameter_bindings',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ParameterBindingView',
      '10': 'parameterBindings'
    },
  ],
};

/// Descriptor for `ComponentInstanceView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List componentInstanceViewDescriptor = $convert
    .base64Decode('ChVDb21wb25lbnRJbnN0YW5jZVZpZXcSDgoCaWQYASABKARSAmlkEhwKCWNvbXBvbmVudBgCIA'
        'EoBFIJY29tcG9uZW50EhIKBG5hbWUYAyABKAlSBG5hbWUSNQoOY2xvY2tfYmluZGluZ3MYBCAD'
        'KAsyDi5iZGwudjEuSWRQYWlyUg1jbG9ja0JpbmRpbmdzEksKEnBhcmFtZXRlcl9iaW5kaW5ncx'
        'gFIAMoCzIcLmJkbC52MS5QYXJhbWV0ZXJCaW5kaW5nVmlld1IRcGFyYW1ldGVyQmluZGluZ3M=');

@$core.Deprecated('Use parameterBindingViewDescriptor instead')
const ParameterBindingView$json = {
  '1': 'ParameterBindingView',
  '2': [
    {'1': 'port', '3': 1, '4': 1, '5': 4, '10': 'port'},
    {'1': 'source', '3': 2, '4': 1, '5': 9, '10': 'source'},
  ],
};

/// Descriptor for `ParameterBindingView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List parameterBindingViewDescriptor = $convert
    .base64Decode('ChRQYXJhbWV0ZXJCaW5kaW5nVmlldxISCgRwb3J0GAEgASgEUgRwb3J0EhYKBnNvdXJjZRgCIA'
        'EoCVIGc291cmNl');

@$core.Deprecated('Use portRefViewDescriptor instead')
const PortRefView$json = {
  '1': 'PortRefView',
  '2': [
    {'1': 'instance', '3': 1, '4': 1, '5': 4, '10': 'instance'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
    {'1': 'base_decl', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'baseDecl', '17': true},
  ],
  '8': [
    {'1': '_base_decl'},
  ],
};

/// Descriptor for `PortRefView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portRefViewDescriptor = $convert
    .base64Decode('CgtQb3J0UmVmVmlldxIaCghpbnN0YW5jZRgBIAEoBFIIaW5zdGFuY2USEgoEcG9ydBgCIAEoBF'
        'IEcG9ydBIgCgliYXNlX2RlY2wYAyABKARIAFIIYmFzZURlY2yIAQFCDAoKX2Jhc2VfZGVjbA==');

@$core.Deprecated('Use bindingViewDescriptor instead')
const BindingView$json = {
  '1': 'BindingView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'source', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'source'},
    {'1': 'destination', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'destination'},
    {'1': 'transport_init', '3': 4, '4': 1, '5': 9, '9': 0, '10': 'transportInit', '17': true},
  ],
  '8': [
    {'1': '_transport_init'},
  ],
};

/// Descriptor for `BindingView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bindingViewDescriptor = $convert
    .base64Decode('CgtCaW5kaW5nVmlldxIOCgJpZBgBIAEoBFICaWQSKwoGc291cmNlGAIgASgLMhMuYmRsLnYxLl'
        'BvcnRSZWZWaWV3UgZzb3VyY2USNQoLZGVzdGluYXRpb24YAyABKAsyEy5iZGwudjEuUG9ydFJl'
        'ZlZpZXdSC2Rlc3RpbmF0aW9uEioKDnRyYW5zcG9ydF9pbml0GAQgASgJSABSDXRyYW5zcG9ydE'
        'luaXSIAQFCEQoPX3RyYW5zcG9ydF9pbml0');

@$core.Deprecated('Use exportViewDescriptor instead')
const ExportView$json = {
  '1': 'ExportView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'port', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'port'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `ExportView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List exportViewDescriptor = $convert
    .base64Decode('CgpFeHBvcnRWaWV3Eg4KAmlkGAEgASgEUgJpZBInCgRwb3J0GAIgASgLMhMuYmRsLnYxLlBvcn'
        'RSZWZWaWV3UgRwb3J0EhIKBG5hbWUYAyABKAlSBG5hbWU=');

@$core.Deprecated('Use originViewDescriptor instead')
const OriginView$json = {
  '1': 'OriginView',
  '2': [
    {'1': 'flat', '3': 1, '4': 1, '5': 4, '10': 'flat'},
    {'1': 'sort', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.LocalSort', '10': 'sort'},
    {'1': 'instance', '3': 3, '4': 1, '5': 4, '10': 'instance'},
    {'1': 'component', '3': 4, '4': 1, '5': 4, '10': 'component'},
    {'1': 'local', '3': 5, '4': 1, '5': 4, '10': 'local'},
    {'1': 'port', '3': 6, '4': 1, '5': 4, '9': 0, '10': 'port', '17': true},
  ],
  '8': [
    {'1': '_port'},
  ],
};

/// Descriptor for `OriginView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List originViewDescriptor = $convert
    .base64Decode('CgpPcmlnaW5WaWV3EhIKBGZsYXQYASABKARSBGZsYXQSJQoEc29ydBgCIAEoDjIRLmJkbC52MS'
        '5Mb2NhbFNvcnRSBHNvcnQSGgoIaW5zdGFuY2UYAyABKARSCGluc3RhbmNlEhwKCWNvbXBvbmVu'
        'dBgEIAEoBFIJY29tcG9uZW50EhQKBWxvY2FsGAUgASgEUgVsb2NhbBIXCgRwb3J0GAYgASgESA'
        'BSBHBvcnSIAQFCBwoFX3BvcnQ=');

@$core.Deprecated('Use systemViewDescriptor instead')
const SystemView$json = {
  '1': 'SystemView',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'base', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.ProjectProjection', '10': 'base'},
    {'1': 'components', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.ComponentView', '10': 'components'},
    {
      '1': 'instances',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ComponentInstanceView',
      '10': 'instances'
    },
    {'1': 'bindings', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.BindingView', '10': 'bindings'},
    {'1': 'exports', '3': 7, '4': 3, '5': 11, '6': '.bdl.v1.ExportView', '10': 'exports'},
    {'1': 'origins', '3': 8, '4': 3, '5': 11, '6': '.bdl.v1.OriginView', '10': 'origins'},
    {'1': 'is_flat', '3': 9, '4': 1, '5': 8, '10': 'isFlat'},
    {'1': 'groups', '3': 10, '4': 3, '5': 11, '6': '.bdl.v1.BehaviorGroupView', '10': 'groups'},
    {'1': 'authoring_generation', '3': 11, '4': 1, '5': 4, '10': 'authoringGeneration'},
    {
      '1': 'boundaries',
      '3': 12,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.BehaviorGroupBoundaryView',
      '10': 'boundaries'
    },
    {'1': 'dirty', '3': 13, '4': 1, '5': 8, '10': 'dirty'},
    {
      '1': 'definition_drafts',
      '3': 14,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.DefinitionDraftView',
      '10': 'definitionDrafts'
    },
  ],
};

/// Descriptor for `SystemView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemViewDescriptor = $convert
    .base64Decode('CgpTeXN0ZW1WaWV3EhoKCHJldmlzaW9uGAEgASgEUghyZXZpc2lvbhISCgRuYW1lGAIgASgJUg'
        'RuYW1lEi0KBGJhc2UYAyABKAsyGS5iZGwudjEuUHJvamVjdFByb2plY3Rpb25SBGJhc2USNQoK'
        'Y29tcG9uZW50cxgEIAMoCzIVLmJkbC52MS5Db21wb25lbnRWaWV3Ugpjb21wb25lbnRzEjsKCW'
        'luc3RhbmNlcxgFIAMoCzIdLmJkbC52MS5Db21wb25lbnRJbnN0YW5jZVZpZXdSCWluc3RhbmNl'
        'cxIvCghiaW5kaW5ncxgGIAMoCzITLmJkbC52MS5CaW5kaW5nVmlld1IIYmluZGluZ3MSLAoHZX'
        'hwb3J0cxgHIAMoCzISLmJkbC52MS5FeHBvcnRWaWV3UgdleHBvcnRzEiwKB29yaWdpbnMYCCAD'
        'KAsyEi5iZGwudjEuT3JpZ2luVmlld1IHb3JpZ2lucxIXCgdpc19mbGF0GAkgASgIUgZpc0ZsYX'
        'QSMQoGZ3JvdXBzGAogAygLMhkuYmRsLnYxLkJlaGF2aW9yR3JvdXBWaWV3UgZncm91cHMSMQoU'
        'YXV0aG9yaW5nX2dlbmVyYXRpb24YCyABKARSE2F1dGhvcmluZ0dlbmVyYXRpb24SQQoKYm91bm'
        'RhcmllcxgMIAMoCzIhLmJkbC52MS5CZWhhdmlvckdyb3VwQm91bmRhcnlWaWV3Ugpib3VuZGFy'
        'aWVzEhQKBWRpcnR5GA0gASgIUgVkaXJ0eRJIChFkZWZpbml0aW9uX2RyYWZ0cxgOIAMoCzIbLm'
        'JkbC52MS5EZWZpbml0aW9uRHJhZnRWaWV3UhBkZWZpbml0aW9uRHJhZnRz');

@$core.Deprecated('Use definitionDraftViewDescriptor instead')
const DefinitionDraftView$json = {
  '1': 'DefinitionDraftView',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
    {'1': 'mapping_id', '3': 2, '4': 1, '5': 4, '10': 'mappingId'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `DefinitionDraftView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List definitionDraftViewDescriptor = $convert
    .base64Decode('ChNEZWZpbml0aW9uRHJhZnRWaWV3EiEKCWNvbXBvbmVudBgBIAEoBEgAUgljb21wb25lbnSIAQ'
        'ESHQoKbWFwcGluZ19pZBgCIAEoBFIJbWFwcGluZ0lkEhYKBnNvdXJjZRgDIAEoCVIGc291cmNl'
        'QgwKCl9jb21wb25lbnQ=');

@$core.Deprecated('Use behaviorGroupViewDescriptor instead')
const BehaviorGroupView$json = {
  '1': 'BehaviorGroupView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'members', '3': 4, '4': 3, '5': 4, '10': 'members'},
    {'1': 'component', '3': 5, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `BehaviorGroupView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List behaviorGroupViewDescriptor = $convert
    .base64Decode('ChFCZWhhdmlvckdyb3VwVmlldxIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZR'
        'IgCgtkZXNjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SGAoHbWVtYmVycxgEIAMoBFIHbWVt'
        'YmVycxIhCgljb21wb25lbnQYBSABKARIAFIJY29tcG9uZW50iAEBQgwKCl9jb21wb25lbnQ=');

@$core.Deprecated('Use applyGroupEditRequestDescriptor instead')
const ApplyGroupEditRequest$json = {
  '1': 'ApplyGroupEditRequest',
  '2': [
    {'1': 'op', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.GroupEditOp', '10': 'op'},
    {'1': 'base_generation', '3': 2, '4': 1, '5': 4, '9': 0, '10': 'baseGeneration', '17': true},
  ],
  '8': [
    {'1': '_base_generation'},
  ],
};

/// Descriptor for `ApplyGroupEditRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List applyGroupEditRequestDescriptor = $convert
    .base64Decode('ChVBcHBseUdyb3VwRWRpdFJlcXVlc3QSIwoCb3AYASABKAsyEy5iZGwudjEuR3JvdXBFZGl0T3'
        'BSAm9wEiwKD2Jhc2VfZ2VuZXJhdGlvbhgCIAEoBEgAUg5iYXNlR2VuZXJhdGlvbogBAUISChBf'
        'YmFzZV9nZW5lcmF0aW9u');

@$core.Deprecated('Use groupEditOpDescriptor instead')
const GroupEditOp$json = {
  '1': 'GroupEditOp',
  '2': [
    {
      '1': 'create_group',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateGroup',
      '9': 0,
      '10': 'createGroup'
    },
    {
      '1': 'rename_group',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameGroup',
      '9': 0,
      '10': 'renameGroup'
    },
    {
      '1': 'set_group_description',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetGroupDescription',
      '9': 0,
      '10': 'setGroupDescription'
    },
    {
      '1': 'delete_group',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteGroup',
      '9': 0,
      '10': 'deleteGroup'
    },
    {
      '1': 'add_member',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.AddGroupMember',
      '9': 0,
      '10': 'addMember'
    },
    {
      '1': 'remove_member',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RemoveGroupMember',
      '9': 0,
      '10': 'removeMember'
    },
    {
      '1': 'move_member',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.MoveGroupMember',
      '9': 0,
      '10': 'moveMember'
    },
    {
      '1': 'merge_groups',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.MergeGroups',
      '9': 0,
      '10': 'mergeGroups'
    },
    {
      '1': 'split_group',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SplitGroup',
      '9': 0,
      '10': 'splitGroup'
    },
  ],
  '8': [
    {'1': 'op'},
  ],
};

/// Descriptor for `GroupEditOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List groupEditOpDescriptor = $convert
    .base64Decode('CgtHcm91cEVkaXRPcBI4CgxjcmVhdGVfZ3JvdXAYASABKAsyEy5iZGwudjEuQ3JlYXRlR3JvdX'
        'BIAFILY3JlYXRlR3JvdXASOAoMcmVuYW1lX2dyb3VwGAIgASgLMhMuYmRsLnYxLlJlbmFtZUdy'
        'b3VwSABSC3JlbmFtZUdyb3VwElEKFXNldF9ncm91cF9kZXNjcmlwdGlvbhgDIAEoCzIbLmJkbC'
        '52MS5TZXRHcm91cERlc2NyaXB0aW9uSABSE3NldEdyb3VwRGVzY3JpcHRpb24SOAoMZGVsZXRl'
        'X2dyb3VwGAQgASgLMhMuYmRsLnYxLkRlbGV0ZUdyb3VwSABSC2RlbGV0ZUdyb3VwEjcKCmFkZF'
        '9tZW1iZXIYBSABKAsyFi5iZGwudjEuQWRkR3JvdXBNZW1iZXJIAFIJYWRkTWVtYmVyEkAKDXJl'
        'bW92ZV9tZW1iZXIYBiABKAsyGS5iZGwudjEuUmVtb3ZlR3JvdXBNZW1iZXJIAFIMcmVtb3ZlTW'
        'VtYmVyEjoKC21vdmVfbWVtYmVyGAcgASgLMhcuYmRsLnYxLk1vdmVHcm91cE1lbWJlckgAUgpt'
        'b3ZlTWVtYmVyEjgKDG1lcmdlX2dyb3VwcxgIIAEoCzITLmJkbC52MS5NZXJnZUdyb3Vwc0gAUg'
        'ttZXJnZUdyb3VwcxI1CgtzcGxpdF9ncm91cBgJIAEoCzISLmJkbC52MS5TcGxpdEdyb3VwSABS'
        'CnNwbGl0R3JvdXBCBAoCb3A=');

@$core.Deprecated('Use createGroupDescriptor instead')
const CreateGroup$json = {
  '1': 'CreateGroup',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'members', '3': 3, '4': 3, '5': 4, '10': 'members'},
    {'1': 'component', '3': 4, '4': 1, '5': 4, '9': 0, '10': 'component', '17': true},
  ],
  '8': [
    {'1': '_component'},
  ],
};

/// Descriptor for `CreateGroup`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createGroupDescriptor = $convert
    .base64Decode('CgtDcmVhdGVHcm91cBISCgRuYW1lGAEgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW9uGAIgASgJUg'
        'tkZXNjcmlwdGlvbhIYCgdtZW1iZXJzGAMgAygEUgdtZW1iZXJzEiEKCWNvbXBvbmVudBgEIAEo'
        'BEgAUgljb21wb25lbnSIAQFCDAoKX2NvbXBvbmVudA==');

@$core.Deprecated('Use renameGroupDescriptor instead')
const RenameGroup$json = {
  '1': 'RenameGroup',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameGroup`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameGroupDescriptor =
    $convert.base64Decode('CgtSZW5hbWVHcm91cBIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZQ==');

@$core.Deprecated('Use setGroupDescriptionDescriptor instead')
const SetGroupDescription$json = {
  '1': 'SetGroupDescription',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `SetGroupDescription`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setGroupDescriptionDescriptor = $convert
    .base64Decode('ChNTZXRHcm91cERlc2NyaXB0aW9uEg4KAmlkGAEgASgEUgJpZBIgCgtkZXNjcmlwdGlvbhgCIA'
        'EoCVILZGVzY3JpcHRpb24=');

@$core.Deprecated('Use deleteGroupDescriptor instead')
const DeleteGroup$json = {
  '1': 'DeleteGroup',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteGroup`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteGroupDescriptor =
    $convert.base64Decode('CgtEZWxldGVHcm91cBIOCgJpZBgBIAEoBFICaWQ=');

@$core.Deprecated('Use addGroupMemberDescriptor instead')
const AddGroupMember$json = {
  '1': 'AddGroupMember',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
    {'1': 'decl', '3': 2, '4': 1, '5': 4, '10': 'decl'},
  ],
};

/// Descriptor for `AddGroupMember`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List addGroupMemberDescriptor = $convert
    .base64Decode('Cg5BZGRHcm91cE1lbWJlchIUCgVncm91cBgBIAEoBFIFZ3JvdXASEgoEZGVjbBgCIAEoBFIEZG'
        'VjbA==');

@$core.Deprecated('Use removeGroupMemberDescriptor instead')
const RemoveGroupMember$json = {
  '1': 'RemoveGroupMember',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
    {'1': 'decl', '3': 2, '4': 1, '5': 4, '10': 'decl'},
  ],
};

/// Descriptor for `RemoveGroupMember`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List removeGroupMemberDescriptor = $convert
    .base64Decode('ChFSZW1vdmVHcm91cE1lbWJlchIUCgVncm91cBgBIAEoBFIFZ3JvdXASEgoEZGVjbBgCIAEoBF'
        'IEZGVjbA==');

@$core.Deprecated('Use moveGroupMemberDescriptor instead')
const MoveGroupMember$json = {
  '1': 'MoveGroupMember',
  '2': [
    {'1': 'decl', '3': 1, '4': 1, '5': 4, '10': 'decl'},
    {'1': 'to', '3': 2, '4': 1, '5': 4, '10': 'to'},
  ],
};

/// Descriptor for `MoveGroupMember`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List moveGroupMemberDescriptor = $convert
    .base64Decode('Cg9Nb3ZlR3JvdXBNZW1iZXISEgoEZGVjbBgBIAEoBFIEZGVjbBIOCgJ0bxgCIAEoBFICdG8=');

@$core.Deprecated('Use mergeGroupsDescriptor instead')
const MergeGroups$json = {
  '1': 'MergeGroups',
  '2': [
    {'1': 'into', '3': 1, '4': 1, '5': 4, '10': 'into'},
    {'1': 'from', '3': 2, '4': 1, '5': 4, '10': 'from'},
  ],
};

/// Descriptor for `MergeGroups`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mergeGroupsDescriptor = $convert
    .base64Decode('CgtNZXJnZUdyb3VwcxISCgRpbnRvGAEgASgEUgRpbnRvEhIKBGZyb20YAiABKARSBGZyb20=');

@$core.Deprecated('Use splitGroupDescriptor instead')
const SplitGroup$json = {
  '1': 'SplitGroup',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'members', '3': 3, '4': 3, '5': 4, '10': 'members'},
  ],
};

/// Descriptor for `SplitGroup`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List splitGroupDescriptor = $convert
    .base64Decode('CgpTcGxpdEdyb3VwEg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEhgKB21lbW'
        'JlcnMYAyADKARSB21lbWJlcnM=');

@$core.Deprecated('Use behaviorGroupBoundaryViewDescriptor instead')
const BehaviorGroupBoundaryView$json = {
  '1': 'BehaviorGroupBoundaryView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'members', '3': 2, '4': 3, '5': 4, '10': 'members'},
    {'1': 'crossing_in', '3': 3, '4': 3, '5': 4, '10': 'crossingIn'},
    {'1': 'crossing_out', '3': 4, '4': 3, '5': 4, '10': 'crossingOut'},
    {'1': 'open_members', '3': 5, '4': 3, '5': 4, '10': 'openMembers'},
    {'1': 'driven_members', '3': 6, '4': 3, '5': 4, '10': 'drivenMembers'},
    {'1': 'private_candidates', '3': 7, '4': 3, '5': 4, '10': 'privateCandidates'},
    {'1': 'external_inputs', '3': 8, '4': 3, '5': 4, '10': 'externalInputs'},
    {'1': 'external_outputs', '3': 9, '4': 3, '5': 4, '10': 'externalOutputs'},
    {'1': 'clocks', '3': 10, '4': 3, '5': 4, '10': 'clocks'},
    {
      '1': 'internal_edges',
      '3': 11,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.DeclEdge',
      '10': 'internalEdges'
    },
    {
      '1': 'crossing_edges',
      '3': 12,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.DeclEdge',
      '10': 'crossingEdges'
    },
  ],
};

/// Descriptor for `BehaviorGroupBoundaryView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List behaviorGroupBoundaryViewDescriptor = $convert
    .base64Decode('ChlCZWhhdmlvckdyb3VwQm91bmRhcnlWaWV3Eg4KAmlkGAEgASgEUgJpZBIYCgdtZW1iZXJzGA'
        'IgAygEUgdtZW1iZXJzEh8KC2Nyb3NzaW5nX2luGAMgAygEUgpjcm9zc2luZ0luEiEKDGNyb3Nz'
        'aW5nX291dBgEIAMoBFILY3Jvc3NpbmdPdXQSIQoMb3Blbl9tZW1iZXJzGAUgAygEUgtvcGVuTW'
        'VtYmVycxIlCg5kcml2ZW5fbWVtYmVycxgGIAMoBFINZHJpdmVuTWVtYmVycxItChJwcml2YXRl'
        'X2NhbmRpZGF0ZXMYByADKARSEXByaXZhdGVDYW5kaWRhdGVzEicKD2V4dGVybmFsX2lucHV0cx'
        'gIIAMoBFIOZXh0ZXJuYWxJbnB1dHMSKQoQZXh0ZXJuYWxfb3V0cHV0cxgJIAMoBFIPZXh0ZXJu'
        'YWxPdXRwdXRzEhYKBmNsb2NrcxgKIAMoBFIGY2xvY2tzEjcKDmludGVybmFsX2VkZ2VzGAsgAy'
        'gLMhAuYmRsLnYxLkRlY2xFZGdlUg1pbnRlcm5hbEVkZ2VzEjcKDmNyb3NzaW5nX2VkZ2VzGAwg'
        'AygLMhAuYmRsLnYxLkRlY2xFZGdlUg1jcm9zc2luZ0VkZ2Vz');

@$core.Deprecated('Use declEdgeDescriptor instead')
const DeclEdge$json = {
  '1': 'DeclEdge',
  '2': [
    {'1': 'from', '3': 1, '4': 1, '5': 4, '10': 'from'},
    {'1': 'to', '3': 2, '4': 1, '5': 4, '10': 'to'},
  ],
};

/// Descriptor for `DeclEdge`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declEdgeDescriptor =
    $convert.base64Decode('CghEZWNsRWRnZRISCgRmcm9tGAEgASgEUgRmcm9tEg4KAnRvGAIgASgEUgJ0bw==');

@$core.Deprecated('Use previewComponentExtractionRequestDescriptor instead')
const PreviewComponentExtractionRequest$json = {
  '1': 'PreviewComponentExtractionRequest',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
    {'1': 'choices', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ExtractionChoices', '10': 'choices'},
  ],
};

/// Descriptor for `PreviewComponentExtractionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewComponentExtractionRequestDescriptor = $convert
    .base64Decode('CiFQcmV2aWV3Q29tcG9uZW50RXh0cmFjdGlvblJlcXVlc3QSFAoFZ3JvdXAYASABKARSBWdyb3'
        'VwEjMKB2Nob2ljZXMYAiABKAsyGS5iZGwudjEuRXh0cmFjdGlvbkNob2ljZXNSB2Nob2ljZXM=');

@$core.Deprecated('Use extractionChoicesDescriptor instead')
const ExtractionChoices$json = {
  '1': 'ExtractionChoices',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'instance_name', '3': 2, '4': 1, '5': 9, '10': 'instanceName'},
    {'1': 'keep_internal', '3': 3, '4': 3, '5': 4, '10': 'keepInternal'},
    {'1': 'internalize_sinks', '3': 4, '4': 3, '5': 4, '10': 'internalizeSinks'},
  ],
};

/// Descriptor for `ExtractionChoices`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractionChoicesDescriptor = $convert
    .base64Decode('ChFFeHRyYWN0aW9uQ2hvaWNlcxISCgRuYW1lGAEgASgJUgRuYW1lEiMKDWluc3RhbmNlX25hbW'
        'UYAiABKAlSDGluc3RhbmNlTmFtZRIjCg1rZWVwX2ludGVybmFsGAMgAygEUgxrZWVwSW50ZXJu'
        'YWwSKwoRaW50ZXJuYWxpemVfc2lua3MYBCADKARSEGludGVybmFsaXplU2lua3M=');

@$core.Deprecated('Use extractionPreviewResponseDescriptor instead')
const ExtractionPreviewResponse$json = {
  '1': 'ExtractionPreviewResponse',
  '2': [
    {
      '1': 'preview',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ExtractionPreviewView',
      '10': 'preview'
    },
  ],
};

/// Descriptor for `ExtractionPreviewResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractionPreviewResponseDescriptor = $convert
    .base64Decode('ChlFeHRyYWN0aW9uUHJldmlld1Jlc3BvbnNlEjcKB3ByZXZpZXcYASABKAsyHS5iZGwudjEuRX'
        'h0cmFjdGlvblByZXZpZXdWaWV3UgdwcmV2aWV3');

@$core.Deprecated('Use extractionPreviewViewDescriptor instead')
const ExtractionPreviewView$json = {
  '1': 'ExtractionPreviewView',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'instance_name', '3': 3, '4': 1, '5': 9, '10': 'instanceName'},
    {
      '1': 'boundary',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.BehaviorGroupBoundaryView',
      '10': 'boundary'
    },
    {'1': 'required', '3': 5, '4': 3, '5': 11, '6': '.bdl.v1.PreviewPortView', '10': 'required'},
    {'1': 'provided', '3': 6, '4': 3, '5': 11, '6': '.bdl.v1.PreviewPortView', '10': 'provided'},
    {
      '1': 'open_members',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.OpenMemberDecisionView',
      '10': 'openMembers'
    },
    {'1': 'clocks', '3': 8, '4': 3, '5': 4, '10': 'clocks'},
    {'1': 'sinks', '3': 9, '4': 3, '5': 11, '6': '.bdl.v1.SinkDecisionView', '10': 'sinks'},
    {'1': 'private', '3': 10, '4': 3, '5': 4, '10': 'private'},
    {'1': 'warnings', '3': 11, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'warnings'},
  ],
};

/// Descriptor for `ExtractionPreviewView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractionPreviewViewDescriptor = $convert
    .base64Decode('ChVFeHRyYWN0aW9uUHJldmlld1ZpZXcSFAoFZ3JvdXAYASABKARSBWdyb3VwEhIKBG5hbWUYAi'
        'ABKAlSBG5hbWUSIwoNaW5zdGFuY2VfbmFtZRgDIAEoCVIMaW5zdGFuY2VOYW1lEj0KCGJvdW5k'
        'YXJ5GAQgASgLMiEuYmRsLnYxLkJlaGF2aW9yR3JvdXBCb3VuZGFyeVZpZXdSCGJvdW5kYXJ5Ej'
        'MKCHJlcXVpcmVkGAUgAygLMhcuYmRsLnYxLlByZXZpZXdQb3J0Vmlld1IIcmVxdWlyZWQSMwoI'
        'cHJvdmlkZWQYBiADKAsyFy5iZGwudjEuUHJldmlld1BvcnRWaWV3Ughwcm92aWRlZBJBCgxvcG'
        'VuX21lbWJlcnMYByADKAsyHi5iZGwudjEuT3Blbk1lbWJlckRlY2lzaW9uVmlld1ILb3Blbk1l'
        'bWJlcnMSFgoGY2xvY2tzGAggAygEUgZjbG9ja3MSLgoFc2lua3MYCSADKAsyGC5iZGwudjEuU2'
        'lua0RlY2lzaW9uVmlld1IFc2lua3MSGAoHcHJpdmF0ZRgKIAMoBFIHcHJpdmF0ZRIuCgh3YXJu'
        'aW5ncxgLIAMoCzISLmJkbC52MS5EaWFnbm9zdGljUgh3YXJuaW5ncw==');

@$core.Deprecated('Use previewPortViewDescriptor instead')
const PreviewPortView$json = {
  '1': 'PreviewPortView',
  '2': [
    {'1': 'decl', '3': 1, '4': 1, '5': 4, '10': 'decl'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 3, '4': 1, '5': 14, '6': '.bdl.v1.PortKind', '10': 'kind'},
    {'1': 'concept', '3': 4, '4': 1, '5': 4, '10': 'concept'},
  ],
};

/// Descriptor for `PreviewPortView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewPortViewDescriptor = $convert
    .base64Decode('Cg9QcmV2aWV3UG9ydFZpZXcSEgoEZGVjbBgBIAEoBFIEZGVjbBISCgRuYW1lGAIgASgJUgRuYW'
        '1lEiQKBGtpbmQYAyABKA4yEC5iZGwudjEuUG9ydEtpbmRSBGtpbmQSGAoHY29uY2VwdBgEIAEo'
        'BFIHY29uY2VwdA==');

@$core.Deprecated('Use openMemberDecisionViewDescriptor instead')
const OpenMemberDecisionView$json = {
  '1': 'OpenMemberDecisionView',
  '2': [
    {'1': 'decl', '3': 1, '4': 1, '5': 4, '10': 'decl'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'as_input', '3': 3, '4': 1, '5': 8, '10': 'asInput'},
  ],
};

/// Descriptor for `OpenMemberDecisionView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List openMemberDecisionViewDescriptor = $convert
    .base64Decode('ChZPcGVuTWVtYmVyRGVjaXNpb25WaWV3EhIKBGRlY2wYASABKARSBGRlY2wSEgoEbmFtZRgCIA'
        'EoCVIEbmFtZRIZCghhc19pbnB1dBgDIAEoCFIHYXNJbnB1dA==');

@$core.Deprecated('Use sinkDecisionViewDescriptor instead')
const SinkDecisionView$json = {
  '1': 'SinkDecisionView',
  '2': [
    {'1': 'output', '3': 1, '4': 1, '5': 4, '10': 'output'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'drivers', '3': 3, '4': 3, '5': 4, '10': 'drivers'},
    {'1': 'internal', '3': 4, '4': 1, '5': 8, '10': 'internal'},
  ],
};

/// Descriptor for `SinkDecisionView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sinkDecisionViewDescriptor = $convert
    .base64Decode('ChBTaW5rRGVjaXNpb25WaWV3EhYKBm91dHB1dBgBIAEoBFIGb3V0cHV0EhIKBG5hbWUYAiABKA'
        'lSBG5hbWUSGAoHZHJpdmVycxgDIAMoBFIHZHJpdmVycxIaCghpbnRlcm5hbBgEIAEoCFIIaW50'
        'ZXJuYWw=');

@$core.Deprecated('Use systemEditOpDescriptor instead')
const SystemEditOp$json = {
  '1': 'SystemEditOp',
  '2': [
    {'1': 'base', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.EditOp', '9': 0, '10': 'base'},
    {
      '1': 'create_component',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateComponent',
      '9': 0,
      '10': 'createComponent'
    },
    {
      '1': 'rename_component',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameComponent',
      '9': 0,
      '10': 'renameComponent'
    },
    {
      '1': 'set_component_description',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetComponentDescription',
      '9': 0,
      '10': 'setComponentDescription'
    },
    {
      '1': 'delete_component',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteComponent',
      '9': 0,
      '10': 'deleteComponent'
    },
    {
      '1': 'edit_component_body',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EditComponentBody',
      '9': 0,
      '10': 'editComponentBody'
    },
    {
      '1': 'declare_port',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeclarePort',
      '9': 0,
      '10': 'declarePort'
    },
    {
      '1': 'rename_port',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenamePort',
      '9': 0,
      '10': 'renamePort'
    },
    {
      '1': 'retire_port',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RetirePort',
      '9': 0,
      '10': 'retirePort'
    },
    {
      '1': 'set_clock_parameter',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetClockParameter',
      '9': 0,
      '10': 'setClockParameter'
    },
    {
      '1': 'share_concept',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ShareConcept',
      '9': 0,
      '10': 'shareConcept'
    },
    {
      '1': 'externalize_output',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ExternalizeOutput',
      '9': 0,
      '10': 'externalizeOutput'
    },
    {
      '1': 'create_instance',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.CreateInstance',
      '9': 0,
      '10': 'createInstance'
    },
    {
      '1': 'rename_instance',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RenameInstance',
      '9': 0,
      '10': 'renameInstance'
    },
    {
      '1': 'delete_instance',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteInstance',
      '9': 0,
      '10': 'deleteInstance'
    },
    {
      '1': 'set_clock_argument',
      '3': 16,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetClockArgument',
      '9': 0,
      '10': 'setClockArgument'
    },
    {
      '1': 'set_parameter_argument',
      '3': 17,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.SetParameterArgument',
      '9': 0,
      '10': 'setParameterArgument'
    },
    {
      '1': 'bind_ports',
      '3': 18,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.BindPorts',
      '9': 0,
      '10': 'bindPorts'
    },
    {
      '1': 'unbind_ports',
      '3': 19,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.UnbindPorts',
      '9': 0,
      '10': 'unbindPorts'
    },
    {
      '1': 'export_port',
      '3': 20,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ExportPort',
      '9': 0,
      '10': 'exportPort'
    },
    {'1': 'hide_port', '3': 21, '4': 1, '5': 11, '6': '.bdl.v1.HidePort', '9': 0, '10': 'hidePort'},
    {
      '1': 'change_port_contract',
      '3': 22,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ChangePortContract',
      '9': 0,
      '10': 'changePortContract'
    },
    {
      '1': 'rebind_port_declaration',
      '3': 23,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.RebindPortDeclaration',
      '9': 0,
      '10': 'rebindPortDeclaration'
    },
    {
      '1': 'duplicate_component',
      '3': 24,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DuplicateComponent',
      '9': 0,
      '10': 'duplicateComponent'
    },
    {
      '1': 'replace_instance_component',
      '3': 25,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ReplaceInstanceComponent',
      '9': 0,
      '10': 'replaceInstanceComponent'
    },
    {
      '1': 'extract_group_as_component',
      '3': 26,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.ExtractGroupAsComponent',
      '9': 0,
      '10': 'extractGroupAsComponent'
    },
    {
      '1': 'delete_group_with_members',
      '3': 27,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.DeleteGroupWithMembers',
      '9': 0,
      '10': 'deleteGroupWithMembers'
    },
  ],
  '8': [
    {'1': 'op'},
  ],
};

/// Descriptor for `SystemEditOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemEditOpDescriptor = $convert
    .base64Decode('CgxTeXN0ZW1FZGl0T3ASJAoEYmFzZRgBIAEoCzIOLmJkbC52MS5FZGl0T3BIAFIEYmFzZRJECh'
        'BjcmVhdGVfY29tcG9uZW50GAIgASgLMhcuYmRsLnYxLkNyZWF0ZUNvbXBvbmVudEgAUg9jcmVh'
        'dGVDb21wb25lbnQSRAoQcmVuYW1lX2NvbXBvbmVudBgDIAEoCzIXLmJkbC52MS5SZW5hbWVDb2'
        '1wb25lbnRIAFIPcmVuYW1lQ29tcG9uZW50El0KGXNldF9jb21wb25lbnRfZGVzY3JpcHRpb24Y'
        'BCABKAsyHy5iZGwudjEuU2V0Q29tcG9uZW50RGVzY3JpcHRpb25IAFIXc2V0Q29tcG9uZW50RG'
        'VzY3JpcHRpb24SRAoQZGVsZXRlX2NvbXBvbmVudBgFIAEoCzIXLmJkbC52MS5EZWxldGVDb21w'
        'b25lbnRIAFIPZGVsZXRlQ29tcG9uZW50EksKE2VkaXRfY29tcG9uZW50X2JvZHkYBiABKAsyGS'
        '5iZGwudjEuRWRpdENvbXBvbmVudEJvZHlIAFIRZWRpdENvbXBvbmVudEJvZHkSOAoMZGVjbGFy'
        'ZV9wb3J0GAcgASgLMhMuYmRsLnYxLkRlY2xhcmVQb3J0SABSC2RlY2xhcmVQb3J0EjUKC3Jlbm'
        'FtZV9wb3J0GAggASgLMhIuYmRsLnYxLlJlbmFtZVBvcnRIAFIKcmVuYW1lUG9ydBI1CgtyZXRp'
        'cmVfcG9ydBgJIAEoCzISLmJkbC52MS5SZXRpcmVQb3J0SABSCnJldGlyZVBvcnQSSwoTc2V0X2'
        'Nsb2NrX3BhcmFtZXRlchgKIAEoCzIZLmJkbC52MS5TZXRDbG9ja1BhcmFtZXRlckgAUhFzZXRD'
        'bG9ja1BhcmFtZXRlchI7Cg1zaGFyZV9jb25jZXB0GAsgASgLMhQuYmRsLnYxLlNoYXJlQ29uY2'
        'VwdEgAUgxzaGFyZUNvbmNlcHQSSgoSZXh0ZXJuYWxpemVfb3V0cHV0GAwgASgLMhkuYmRsLnYx'
        'LkV4dGVybmFsaXplT3V0cHV0SABSEWV4dGVybmFsaXplT3V0cHV0EkEKD2NyZWF0ZV9pbnN0YW'
        '5jZRgNIAEoCzIWLmJkbC52MS5DcmVhdGVJbnN0YW5jZUgAUg5jcmVhdGVJbnN0YW5jZRJBCg9y'
        'ZW5hbWVfaW5zdGFuY2UYDiABKAsyFi5iZGwudjEuUmVuYW1lSW5zdGFuY2VIAFIOcmVuYW1lSW'
        '5zdGFuY2USQQoPZGVsZXRlX2luc3RhbmNlGA8gASgLMhYuYmRsLnYxLkRlbGV0ZUluc3RhbmNl'
        'SABSDmRlbGV0ZUluc3RhbmNlEkgKEnNldF9jbG9ja19hcmd1bWVudBgQIAEoCzIYLmJkbC52MS'
        '5TZXRDbG9ja0FyZ3VtZW50SABSEHNldENsb2NrQXJndW1lbnQSVAoWc2V0X3BhcmFtZXRlcl9h'
        'cmd1bWVudBgRIAEoCzIcLmJkbC52MS5TZXRQYXJhbWV0ZXJBcmd1bWVudEgAUhRzZXRQYXJhbW'
        'V0ZXJBcmd1bWVudBIyCgpiaW5kX3BvcnRzGBIgASgLMhEuYmRsLnYxLkJpbmRQb3J0c0gAUgli'
        'aW5kUG9ydHMSOAoMdW5iaW5kX3BvcnRzGBMgASgLMhMuYmRsLnYxLlVuYmluZFBvcnRzSABSC3'
        'VuYmluZFBvcnRzEjUKC2V4cG9ydF9wb3J0GBQgASgLMhIuYmRsLnYxLkV4cG9ydFBvcnRIAFIK'
        'ZXhwb3J0UG9ydBIvCgloaWRlX3BvcnQYFSABKAsyEC5iZGwudjEuSGlkZVBvcnRIAFIIaGlkZV'
        'BvcnQSTgoUY2hhbmdlX3BvcnRfY29udHJhY3QYFiABKAsyGi5iZGwudjEuQ2hhbmdlUG9ydENv'
        'bnRyYWN0SABSEmNoYW5nZVBvcnRDb250cmFjdBJXChdyZWJpbmRfcG9ydF9kZWNsYXJhdGlvbh'
        'gXIAEoCzIdLmJkbC52MS5SZWJpbmRQb3J0RGVjbGFyYXRpb25IAFIVcmViaW5kUG9ydERlY2xh'
        'cmF0aW9uEk0KE2R1cGxpY2F0ZV9jb21wb25lbnQYGCABKAsyGi5iZGwudjEuRHVwbGljYXRlQ2'
        '9tcG9uZW50SABSEmR1cGxpY2F0ZUNvbXBvbmVudBJgChpyZXBsYWNlX2luc3RhbmNlX2NvbXBv'
        'bmVudBgZIAEoCzIgLmJkbC52MS5SZXBsYWNlSW5zdGFuY2VDb21wb25lbnRIAFIYcmVwbGFjZU'
        'luc3RhbmNlQ29tcG9uZW50El4KGmV4dHJhY3RfZ3JvdXBfYXNfY29tcG9uZW50GBogASgLMh8u'
        'YmRsLnYxLkV4dHJhY3RHcm91cEFzQ29tcG9uZW50SABSF2V4dHJhY3RHcm91cEFzQ29tcG9uZW'
        '50ElsKGWRlbGV0ZV9ncm91cF93aXRoX21lbWJlcnMYGyABKAsyHi5iZGwudjEuRGVsZXRlR3Jv'
        'dXBXaXRoTWVtYmVyc0gAUhZkZWxldGVHcm91cFdpdGhNZW1iZXJzQgQKAm9w');

@$core.Deprecated('Use extractGroupAsComponentDescriptor instead')
const ExtractGroupAsComponent$json = {
  '1': 'ExtractGroupAsComponent',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
    {'1': 'choices', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ExtractionChoices', '10': 'choices'},
  ],
};

/// Descriptor for `ExtractGroupAsComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractGroupAsComponentDescriptor = $convert
    .base64Decode('ChdFeHRyYWN0R3JvdXBBc0NvbXBvbmVudBIUCgVncm91cBgBIAEoBFIFZ3JvdXASMwoHY2hvaW'
        'NlcxgCIAEoCzIZLmJkbC52MS5FeHRyYWN0aW9uQ2hvaWNlc1IHY2hvaWNlcw==');

@$core.Deprecated('Use deleteGroupWithMembersDescriptor instead')
const DeleteGroupWithMembers$json = {
  '1': 'DeleteGroupWithMembers',
  '2': [
    {'1': 'group', '3': 1, '4': 1, '5': 4, '10': 'group'},
  ],
};

/// Descriptor for `DeleteGroupWithMembers`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteGroupWithMembersDescriptor =
    $convert.base64Decode('ChZEZWxldGVHcm91cFdpdGhNZW1iZXJzEhQKBWdyb3VwGAEgASgEUgVncm91cA==');

@$core.Deprecated('Use changePortContractDescriptor instead')
const ChangePortContract$json = {
  '1': 'ChangePortContract',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
    {'1': 'contract', '3': 3, '4': 1, '5': 11, '6': '.bdl.v1.PortContractView', '10': 'contract'},
  ],
};

/// Descriptor for `ChangePortContract`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List changePortContractDescriptor = $convert
    .base64Decode('ChJDaGFuZ2VQb3J0Q29udHJhY3QSHAoJY29tcG9uZW50GAEgASgEUgljb21wb25lbnQSEgoEcG'
        '9ydBgCIAEoBFIEcG9ydBI0Cghjb250cmFjdBgDIAEoCzIYLmJkbC52MS5Qb3J0Q29udHJhY3RW'
        'aWV3Ughjb250cmFjdA==');

@$core.Deprecated('Use rebindPortDeclarationDescriptor instead')
const RebindPortDeclaration$json = {
  '1': 'RebindPortDeclaration',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
    {'1': 'decl', '3': 3, '4': 1, '5': 4, '10': 'decl'},
  ],
};

/// Descriptor for `RebindPortDeclaration`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rebindPortDeclarationDescriptor = $convert
    .base64Decode('ChVSZWJpbmRQb3J0RGVjbGFyYXRpb24SHAoJY29tcG9uZW50GAEgASgEUgljb21wb25lbnQSEg'
        'oEcG9ydBgCIAEoBFIEcG9ydBISCgRkZWNsGAMgASgEUgRkZWNs');

@$core.Deprecated('Use duplicateComponentDescriptor instead')
const DuplicateComponent$json = {
  '1': 'DuplicateComponent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `DuplicateComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List duplicateComponentDescriptor = $convert
    .base64Decode('ChJEdXBsaWNhdGVDb21wb25lbnQSDgoCaWQYASABKARSAmlkEhIKBG5hbWUYAiABKAlSBG5hbW'
        'U=');

@$core.Deprecated('Use replaceInstanceComponentDescriptor instead')
const ReplaceInstanceComponent$json = {
  '1': 'ReplaceInstanceComponent',
  '2': [
    {'1': 'instance', '3': 1, '4': 1, '5': 4, '10': 'instance'},
    {'1': 'component', '3': 2, '4': 1, '5': 4, '10': 'component'},
  ],
};

/// Descriptor for `ReplaceInstanceComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replaceInstanceComponentDescriptor = $convert
    .base64Decode('ChhSZXBsYWNlSW5zdGFuY2VDb21wb25lbnQSGgoIaW5zdGFuY2UYASABKARSCGluc3RhbmNlEh'
        'wKCWNvbXBvbmVudBgCIAEoBFIJY29tcG9uZW50');

@$core.Deprecated('Use createComponentDescriptor instead')
const CreateComponent$json = {
  '1': 'CreateComponent',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `CreateComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createComponentDescriptor = $convert
    .base64Decode('Cg9DcmVhdGVDb21wb25lbnQSEgoEbmFtZRgBIAEoCVIEbmFtZRIgCgtkZXNjcmlwdGlvbhgCIA'
        'EoCVILZGVzY3JpcHRpb24=');

@$core.Deprecated('Use renameComponentDescriptor instead')
const RenameComponent$json = {
  '1': 'RenameComponent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameComponentDescriptor = $convert
    .base64Decode('Cg9SZW5hbWVDb21wb25lbnQSDgoCaWQYASABKARSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWU=');

@$core.Deprecated('Use setComponentDescriptionDescriptor instead')
const SetComponentDescription$json = {
  '1': 'SetComponentDescription',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `SetComponentDescription`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setComponentDescriptionDescriptor = $convert
    .base64Decode('ChdTZXRDb21wb25lbnREZXNjcmlwdGlvbhIOCgJpZBgBIAEoBFICaWQSIAoLZGVzY3JpcHRpb2'
        '4YAiABKAlSC2Rlc2NyaXB0aW9u');

@$core.Deprecated('Use deleteComponentDescriptor instead')
const DeleteComponent$json = {
  '1': 'DeleteComponent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteComponentDescriptor =
    $convert.base64Decode('Cg9EZWxldGVDb21wb25lbnQSDgoCaWQYASABKARSAmlk');

@$core.Deprecated('Use editComponentBodyDescriptor instead')
const EditComponentBody$json = {
  '1': 'EditComponentBody',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'op', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.EditOp', '10': 'op'},
  ],
};

/// Descriptor for `EditComponentBody`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List editComponentBodyDescriptor = $convert
    .base64Decode('ChFFZGl0Q29tcG9uZW50Qm9keRIcCgljb21wb25lbnQYASABKARSCWNvbXBvbmVudBIeCgJvcB'
        'gCIAEoCzIOLmJkbC52MS5FZGl0T3BSAm9w');

@$core.Deprecated('Use declarePortDescriptor instead')
const DeclarePort$json = {
  '1': 'DeclarePort',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'decl', '3': 2, '4': 1, '5': 4, '10': 'decl'},
    {'1': 'kind', '3': 3, '4': 1, '5': 14, '6': '.bdl.v1.PortKind', '10': 'kind'},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `DeclarePort`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declarePortDescriptor = $convert
    .base64Decode('CgtEZWNsYXJlUG9ydBIcCgljb21wb25lbnQYASABKARSCWNvbXBvbmVudBISCgRkZWNsGAIgAS'
        'gEUgRkZWNsEiQKBGtpbmQYAyABKA4yEC5iZGwudjEuUG9ydEtpbmRSBGtpbmQSEgoEbmFtZRgE'
        'IAEoCVIEbmFtZRIgCgtkZXNjcmlwdGlvbhgFIAEoCVILZGVzY3JpcHRpb24=');

@$core.Deprecated('Use renamePortDescriptor instead')
const RenamePort$json = {
  '1': 'RenamePort',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenamePort`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renamePortDescriptor = $convert
    .base64Decode('CgpSZW5hbWVQb3J0EhwKCWNvbXBvbmVudBgBIAEoBFIJY29tcG9uZW50EhIKBHBvcnQYAiABKA'
        'RSBHBvcnQSEgoEbmFtZRgDIAEoCVIEbmFtZQ==');

@$core.Deprecated('Use retirePortDescriptor instead')
const RetirePort$json = {
  '1': 'RetirePort',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
  ],
};

/// Descriptor for `RetirePort`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retirePortDescriptor = $convert
    .base64Decode('CgpSZXRpcmVQb3J0EhwKCWNvbXBvbmVudBgBIAEoBFIJY29tcG9uZW50EhIKBHBvcnQYAiABKA'
        'RSBHBvcnQ=');

@$core.Deprecated('Use setClockParameterDescriptor instead')
const SetClockParameter$json = {
  '1': 'SetClockParameter',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'clock', '3': 2, '4': 1, '5': 4, '10': 'clock'},
    {'1': 'parameter', '3': 3, '4': 1, '5': 8, '10': 'parameter'},
  ],
};

/// Descriptor for `SetClockParameter`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setClockParameterDescriptor = $convert
    .base64Decode('ChFTZXRDbG9ja1BhcmFtZXRlchIcCgljb21wb25lbnQYASABKARSCWNvbXBvbmVudBIUCgVjbG'
        '9jaxgCIAEoBFIFY2xvY2sSHAoJcGFyYW1ldGVyGAMgASgIUglwYXJhbWV0ZXI=');

@$core.Deprecated('Use shareConceptDescriptor instead')
const ShareConcept$json = {
  '1': 'ShareConcept',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'local', '3': 2, '4': 1, '5': 4, '10': 'local'},
    {'1': 'system', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'system', '17': true},
  ],
  '8': [
    {'1': '_system'},
  ],
};

/// Descriptor for `ShareConcept`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List shareConceptDescriptor = $convert
    .base64Decode('CgxTaGFyZUNvbmNlcHQSHAoJY29tcG9uZW50GAEgASgEUgljb21wb25lbnQSFAoFbG9jYWwYAi'
        'ABKARSBWxvY2FsEhsKBnN5c3RlbRgDIAEoBEgAUgZzeXN0ZW2IAQFCCQoHX3N5c3RlbQ==');

@$core.Deprecated('Use externalizeOutputDescriptor instead')
const ExternalizeOutput$json = {
  '1': 'ExternalizeOutput',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'local', '3': 2, '4': 1, '5': 4, '10': 'local'},
    {'1': 'system', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'system', '17': true},
  ],
  '8': [
    {'1': '_system'},
  ],
};

/// Descriptor for `ExternalizeOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List externalizeOutputDescriptor = $convert
    .base64Decode('ChFFeHRlcm5hbGl6ZU91dHB1dBIcCgljb21wb25lbnQYASABKARSCWNvbXBvbmVudBIUCgVsb2'
        'NhbBgCIAEoBFIFbG9jYWwSGwoGc3lzdGVtGAMgASgESABSBnN5c3RlbYgBAUIJCgdfc3lzdGVt');

@$core.Deprecated('Use createInstanceDescriptor instead')
const CreateInstance$json = {
  '1': 'CreateInstance',
  '2': [
    {'1': 'component', '3': 1, '4': 1, '5': 4, '10': 'component'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `CreateInstance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createInstanceDescriptor = $convert
    .base64Decode('Cg5DcmVhdGVJbnN0YW5jZRIcCgljb21wb25lbnQYASABKARSCWNvbXBvbmVudBISCgRuYW1lGA'
        'IgASgJUgRuYW1l');

@$core.Deprecated('Use renameInstanceDescriptor instead')
const RenameInstance$json = {
  '1': 'RenameInstance',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `RenameInstance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List renameInstanceDescriptor = $convert
    .base64Decode('Cg5SZW5hbWVJbnN0YW5jZRIOCgJpZBgBIAEoBFICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZQ==');

@$core.Deprecated('Use deleteInstanceDescriptor instead')
const DeleteInstance$json = {
  '1': 'DeleteInstance',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
  ],
};

/// Descriptor for `DeleteInstance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteInstanceDescriptor =
    $convert.base64Decode('Cg5EZWxldGVJbnN0YW5jZRIOCgJpZBgBIAEoBFICaWQ=');

@$core.Deprecated('Use setClockArgumentDescriptor instead')
const SetClockArgument$json = {
  '1': 'SetClockArgument',
  '2': [
    {'1': 'instance', '3': 1, '4': 1, '5': 4, '10': 'instance'},
    {'1': 'parameter', '3': 2, '4': 1, '5': 4, '10': 'parameter'},
    {'1': 'clock', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'clock', '17': true},
  ],
  '8': [
    {'1': '_clock'},
  ],
};

/// Descriptor for `SetClockArgument`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setClockArgumentDescriptor = $convert
    .base64Decode('ChBTZXRDbG9ja0FyZ3VtZW50EhoKCGluc3RhbmNlGAEgASgEUghpbnN0YW5jZRIcCglwYXJhbW'
        'V0ZXIYAiABKARSCXBhcmFtZXRlchIZCgVjbG9jaxgDIAEoBEgAUgVjbG9ja4gBAUIICgZfY2xv'
        'Y2s=');

@$core.Deprecated('Use setParameterArgumentDescriptor instead')
const SetParameterArgument$json = {
  '1': 'SetParameterArgument',
  '2': [
    {'1': 'instance', '3': 1, '4': 1, '5': 4, '10': 'instance'},
    {'1': 'port', '3': 2, '4': 1, '5': 4, '10': 'port'},
    {'1': 'value', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'value', '17': true},
  ],
  '8': [
    {'1': '_value'},
  ],
};

/// Descriptor for `SetParameterArgument`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setParameterArgumentDescriptor = $convert
    .base64Decode('ChRTZXRQYXJhbWV0ZXJBcmd1bWVudBIaCghpbnN0YW5jZRgBIAEoBFIIaW5zdGFuY2USEgoEcG'
        '9ydBgCIAEoBFIEcG9ydBIZCgV2YWx1ZRgDIAEoCUgAUgV2YWx1ZYgBAUIICgZfdmFsdWU=');

@$core.Deprecated('Use bindPortsDescriptor instead')
const BindPorts$json = {
  '1': 'BindPorts',
  '2': [
    {'1': 'source', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'source'},
    {'1': 'destination', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'destination'},
    {'1': 'transport_init', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'transportInit', '17': true},
  ],
  '8': [
    {'1': '_transport_init'},
  ],
};

/// Descriptor for `BindPorts`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bindPortsDescriptor = $convert
    .base64Decode('CglCaW5kUG9ydHMSKwoGc291cmNlGAEgASgLMhMuYmRsLnYxLlBvcnRSZWZWaWV3UgZzb3VyY2'
        'USNQoLZGVzdGluYXRpb24YAiABKAsyEy5iZGwudjEuUG9ydFJlZlZpZXdSC2Rlc3RpbmF0aW9u'
        'EioKDnRyYW5zcG9ydF9pbml0GAMgASgJSABSDXRyYW5zcG9ydEluaXSIAQFCEQoPX3RyYW5zcG'
        '9ydF9pbml0');

@$core.Deprecated('Use unbindPortsDescriptor instead')
const UnbindPorts$json = {
  '1': 'UnbindPorts',
  '2': [
    {'1': 'binding', '3': 1, '4': 1, '5': 4, '10': 'binding'},
  ],
};

/// Descriptor for `UnbindPorts`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unbindPortsDescriptor =
    $convert.base64Decode('CgtVbmJpbmRQb3J0cxIYCgdiaW5kaW5nGAEgASgEUgdiaW5kaW5n');

@$core.Deprecated('Use exportPortDescriptor instead')
const ExportPort$json = {
  '1': 'ExportPort',
  '2': [
    {'1': 'port', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'port'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `ExportPort`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List exportPortDescriptor = $convert
    .base64Decode('CgpFeHBvcnRQb3J0EicKBHBvcnQYASABKAsyEy5iZGwudjEuUG9ydFJlZlZpZXdSBHBvcnQSEg'
        'oEbmFtZRgCIAEoCVIEbmFtZQ==');

@$core.Deprecated('Use hidePortDescriptor instead')
const HidePort$json = {
  '1': 'HidePort',
  '2': [
    {'1': 'export', '3': 1, '4': 1, '5': 4, '10': 'export'},
  ],
};

/// Descriptor for `HidePort`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List hidePortDescriptor =
    $convert.base64Decode('CghIaWRlUG9ydBIWCgZleHBvcnQYASABKARSBmV4cG9ydA==');

@$core.Deprecated('Use systemEditOutcomeDescriptor instead')
const SystemEditOutcome$json = {
  '1': 'SystemEditOutcome',
  '2': [
    {'1': 'kind', '3': 1, '4': 1, '5': 14, '6': '.bdl.v1.EditKind', '10': 'kind'},
    {'1': 'invalidates', '3': 2, '4': 3, '5': 14, '6': '.bdl.v1.Invalidation', '10': 'invalidates'},
    {'1': 'origin_decls', '3': 3, '4': 3, '5': 4, '10': 'originDecls'},
    {'1': 'instances', '3': 4, '4': 3, '5': 4, '10': 'instances'},
    {
      '1': 'created_component',
      '3': 5,
      '4': 1,
      '5': 4,
      '9': 0,
      '10': 'createdComponent',
      '17': true
    },
    {'1': 'created_instance', '3': 6, '4': 1, '5': 4, '9': 1, '10': 'createdInstance', '17': true},
    {'1': 'created_port', '3': 7, '4': 1, '5': 4, '9': 2, '10': 'createdPort', '17': true},
    {'1': 'created_binding', '3': 8, '4': 1, '5': 4, '9': 3, '10': 'createdBinding', '17': true},
    {'1': 'created_export', '3': 9, '4': 1, '5': 4, '9': 4, '10': 'createdExport', '17': true},
    {
      '1': 'inner',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.EditOutcome',
      '9': 5,
      '10': 'inner',
      '17': true
    },
    {'1': 'bindings', '3': 11, '4': 3, '5': 4, '10': 'bindings'},
  ],
  '8': [
    {'1': '_created_component'},
    {'1': '_created_instance'},
    {'1': '_created_port'},
    {'1': '_created_binding'},
    {'1': '_created_export'},
    {'1': '_inner'},
  ],
};

/// Descriptor for `SystemEditOutcome`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemEditOutcomeDescriptor = $convert
    .base64Decode('ChFTeXN0ZW1FZGl0T3V0Y29tZRIkCgRraW5kGAEgASgOMhAuYmRsLnYxLkVkaXRLaW5kUgRraW'
        '5kEjYKC2ludmFsaWRhdGVzGAIgAygOMhQuYmRsLnYxLkludmFsaWRhdGlvblILaW52YWxpZGF0'
        'ZXMSIQoMb3JpZ2luX2RlY2xzGAMgAygEUgtvcmlnaW5EZWNscxIcCglpbnN0YW5jZXMYBCADKA'
        'RSCWluc3RhbmNlcxIwChFjcmVhdGVkX2NvbXBvbmVudBgFIAEoBEgAUhBjcmVhdGVkQ29tcG9u'
        'ZW50iAEBEi4KEGNyZWF0ZWRfaW5zdGFuY2UYBiABKARIAVIPY3JlYXRlZEluc3RhbmNliAEBEi'
        'YKDGNyZWF0ZWRfcG9ydBgHIAEoBEgCUgtjcmVhdGVkUG9ydIgBARIsCg9jcmVhdGVkX2JpbmRp'
        'bmcYCCABKARIA1IOY3JlYXRlZEJpbmRpbmeIAQESKgoOY3JlYXRlZF9leHBvcnQYCSABKARIBF'
        'INY3JlYXRlZEV4cG9ydIgBARIuCgVpbm5lchgKIAEoCzITLmJkbC52MS5FZGl0T3V0Y29tZUgF'
        'UgVpbm5lcogBARIaCghiaW5kaW5ncxgLIAMoBFIIYmluZGluZ3NCFAoSX2NyZWF0ZWRfY29tcG'
        '9uZW50QhMKEV9jcmVhdGVkX2luc3RhbmNlQg8KDV9jcmVhdGVkX3BvcnRCEgoQX2NyZWF0ZWRf'
        'YmluZGluZ0IRCg9fY3JlYXRlZF9leHBvcnRCCAoGX2lubmVy');

@$core.Deprecated('Use portStatusViewDescriptor instead')
const PortStatusView$json = {
  '1': 'PortStatusView',
  '2': [
    {'1': 'port', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.PortRefView', '10': 'port'},
    {'1': 'status', '3': 2, '4': 1, '5': 14, '6': '.bdl.v1.PortStatusKind', '10': 'status'},
    {'1': 'binding', '3': 3, '4': 1, '5': 4, '9': 0, '10': 'binding', '17': true},
    {'1': 'export', '3': 4, '4': 1, '5': 4, '9': 1, '10': 'export', '17': true},
  ],
  '8': [
    {'1': '_binding'},
    {'1': '_export'},
  ],
};

/// Descriptor for `PortStatusView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portStatusViewDescriptor = $convert
    .base64Decode('Cg5Qb3J0U3RhdHVzVmlldxInCgRwb3J0GAEgASgLMhMuYmRsLnYxLlBvcnRSZWZWaWV3UgRwb3'
        'J0Ei4KBnN0YXR1cxgCIAEoDjIWLmJkbC52MS5Qb3J0U3RhdHVzS2luZFIGc3RhdHVzEh0KB2Jp'
        'bmRpbmcYAyABKARIAFIHYmluZGluZ4gBARIbCgZleHBvcnQYBCABKARIAVIGZXhwb3J0iAEBQg'
        'oKCF9iaW5kaW5nQgkKB19leHBvcnQ=');

@$core.Deprecated('Use projectedDiagnosticDescriptor instead')
const ProjectedDiagnostic$json = {
  '1': 'ProjectedDiagnostic',
  '2': [
    {'1': 'diagnostic', '3': 1, '4': 1, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'diagnostic'},
    {
      '1': 'origin',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.bdl.v1.OriginView',
      '9': 0,
      '10': 'origin',
      '17': true
    },
    {'1': 'label', '3': 3, '4': 1, '5': 9, '10': 'label'},
  ],
  '8': [
    {'1': '_origin'},
  ],
};

/// Descriptor for `ProjectedDiagnostic`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectedDiagnosticDescriptor = $convert
    .base64Decode('ChNQcm9qZWN0ZWREaWFnbm9zdGljEjIKCmRpYWdub3N0aWMYASABKAsyEi5iZGwudjEuRGlhZ2'
        '5vc3RpY1IKZGlhZ25vc3RpYxIvCgZvcmlnaW4YAiABKAsyEi5iZGwudjEuT3JpZ2luVmlld0gA'
        'UgZvcmlnaW6IAQESFAoFbGFiZWwYAyABKAlSBWxhYmVsQgkKB19vcmlnaW4=');

@$core.Deprecated('Use systemAnalysisViewDescriptor instead')
const SystemAnalysisView$json = {
  '1': 'SystemAnalysisView',
  '2': [
    {'1': 'revision', '3': 1, '4': 1, '5': 4, '10': 'revision'},
    {'1': 'analysis', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ProjectAnalysis', '10': 'analysis'},
    {'1': 'composition', '3': 3, '4': 3, '5': 11, '6': '.bdl.v1.Diagnostic', '10': 'composition'},
    {'1': 'ports', '3': 4, '4': 3, '5': 11, '6': '.bdl.v1.PortStatusView', '10': 'ports'},
    {
      '1': 'acceptance',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.bdl.v1.SystemAcceptance',
      '10': 'acceptance'
    },
    {
      '1': 'projected',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ProjectedDiagnostic',
      '10': 'projected'
    },
    {
      '1': 'components',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ComponentStatusView',
      '10': 'components'
    },
    {
      '1': 'groups',
      '3': 8,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.BehaviorGroupBoundaryView',
      '10': 'groups'
    },
    {
      '1': 'component_analyses',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.bdl.v1.ComponentAnalysisView',
      '10': 'componentAnalyses'
    },
  ],
};

/// Descriptor for `SystemAnalysisView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemAnalysisViewDescriptor = $convert
    .base64Decode('ChJTeXN0ZW1BbmFseXNpc1ZpZXcSGgoIcmV2aXNpb24YASABKARSCHJldmlzaW9uEjMKCGFuYW'
        'x5c2lzGAIgASgLMhcuYmRsLnYxLlByb2plY3RBbmFseXNpc1IIYW5hbHlzaXMSNAoLY29tcG9z'
        'aXRpb24YAyADKAsyEi5iZGwudjEuRGlhZ25vc3RpY1ILY29tcG9zaXRpb24SLAoFcG9ydHMYBC'
        'ADKAsyFi5iZGwudjEuUG9ydFN0YXR1c1ZpZXdSBXBvcnRzEjgKCmFjY2VwdGFuY2UYBSABKA4y'
        'GC5iZGwudjEuU3lzdGVtQWNjZXB0YW5jZVIKYWNjZXB0YW5jZRI5Cglwcm9qZWN0ZWQYBiADKA'
        'syGy5iZGwudjEuUHJvamVjdGVkRGlhZ25vc3RpY1IJcHJvamVjdGVkEjsKCmNvbXBvbmVudHMY'
        'ByADKAsyGy5iZGwudjEuQ29tcG9uZW50U3RhdHVzVmlld1IKY29tcG9uZW50cxI5CgZncm91cH'
        'MYCCADKAsyIS5iZGwudjEuQmVoYXZpb3JHcm91cEJvdW5kYXJ5Vmlld1IGZ3JvdXBzEkwKEmNv'
        'bXBvbmVudF9hbmFseXNlcxgJIAMoCzIdLmJkbC52MS5Db21wb25lbnRBbmFseXNpc1ZpZXdSEW'
        'NvbXBvbmVudEFuYWx5c2Vz');

@$core.Deprecated('Use componentAnalysisViewDescriptor instead')
const ComponentAnalysisView$json = {
  '1': 'ComponentAnalysisView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'analysis', '3': 2, '4': 1, '5': 11, '6': '.bdl.v1.ProjectAnalysis', '10': 'analysis'},
  ],
};

/// Descriptor for `ComponentAnalysisView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List componentAnalysisViewDescriptor = $convert
    .base64Decode('ChVDb21wb25lbnRBbmFseXNpc1ZpZXcSDgoCaWQYASABKARSAmlkEjMKCGFuYWx5c2lzGAIgAS'
        'gLMhcuYmRsLnYxLlByb2plY3RBbmFseXNpc1IIYW5hbHlzaXM=');

@$core.Deprecated('Use componentStatusViewDescriptor instead')
const ComponentStatusView$json = {
  '1': 'ComponentStatusView',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    {'1': 'realizes', '3': 2, '4': 1, '5': 8, '10': 'realizes'},
  ],
};

/// Descriptor for `ComponentStatusView`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List componentStatusViewDescriptor = $convert
    .base64Decode('ChNDb21wb25lbnRTdGF0dXNWaWV3Eg4KAmlkGAEgASgEUgJpZBIaCghyZWFsaXplcxgCIAEoCF'
        'IIcmVhbGl6ZXM=');
