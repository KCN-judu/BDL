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

import 'package:protobuf/protobuf.dart' as $pb;

class EditKind extends $pb.ProtobufEnum {
  static const EditKind EDIT_KIND_UNSPECIFIED =
      EditKind._(0, _omitEnumNames ? '' : 'EDIT_KIND_UNSPECIFIED');
  static const EditKind EDIT_KIND_REFINEMENT =
      EditKind._(1, _omitEnumNames ? '' : 'EDIT_KIND_REFINEMENT');
  static const EditKind EDIT_KIND_EDIT = EditKind._(2, _omitEnumNames ? '' : 'EDIT_KIND_EDIT');

  static const $core.List<EditKind> values = <EditKind>[
    EDIT_KIND_UNSPECIFIED,
    EDIT_KIND_REFINEMENT,
    EDIT_KIND_EDIT,
  ];

  static final $core.List<EditKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 2);
  static EditKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const EditKind._(super.value, super.name);
}

class Invalidation extends $pb.ProtobufEnum {
  static const Invalidation INVALIDATION_UNSPECIFIED =
      Invalidation._(0, _omitEnumNames ? '' : 'INVALIDATION_UNSPECIFIED');
  static const Invalidation INVALIDATION_INTERFACE =
      Invalidation._(1, _omitEnumNames ? '' : 'INVALIDATION_INTERFACE');
  static const Invalidation INVALIDATION_REALIZATION =
      Invalidation._(2, _omitEnumNames ? '' : 'INVALIDATION_REALIZATION');
  static const Invalidation INVALIDATION_SEMANTIC =
      Invalidation._(3, _omitEnumNames ? '' : 'INVALIDATION_SEMANTIC');
  static const Invalidation INVALIDATION_REACTIVE =
      Invalidation._(4, _omitEnumNames ? '' : 'INVALIDATION_REACTIVE');
  static const Invalidation INVALIDATION_CLOCK =
      Invalidation._(5, _omitEnumNames ? '' : 'INVALIDATION_CLOCK');
  static const Invalidation INVALIDATION_OUTPUT =
      Invalidation._(6, _omitEnumNames ? '' : 'INVALIDATION_OUTPUT');
  static const Invalidation INVALIDATION_DEPLOYMENT =
      Invalidation._(7, _omitEnumNames ? '' : 'INVALIDATION_DEPLOYMENT');

  static const $core.List<Invalidation> values = <Invalidation>[
    INVALIDATION_UNSPECIFIED,
    INVALIDATION_INTERFACE,
    INVALIDATION_REALIZATION,
    INVALIDATION_SEMANTIC,
    INVALIDATION_REACTIVE,
    INVALIDATION_CLOCK,
    INVALIDATION_OUTPUT,
    INVALIDATION_DEPLOYMENT,
  ];

  static final $core.List<Invalidation?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 7);
  static Invalidation? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Invalidation._(super.value, super.name);
}

/// The workspace states of the paper's interaction model.  Only the first two
/// are produced before the checker exists; the rest are reserved.
class AcceptanceState extends $pb.ProtobufEnum {
  static const AcceptanceState ACCEPTANCE_STATE_UNSPECIFIED =
      AcceptanceState._(0, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_UNSPECIFIED');
  static const AcceptanceState ACCEPTANCE_STATE_DECLARED =
      AcceptanceState._(1, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_DECLARED');
  static const AcceptanceState ACCEPTANCE_STATE_DEFINED =
      AcceptanceState._(2, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_DEFINED');
  static const AcceptanceState ACCEPTANCE_STATE_TYPE_VALID =
      AcceptanceState._(3, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_TYPE_VALID');
  static const AcceptanceState ACCEPTANCE_STATE_TEMPORALLY_VALID =
      AcceptanceState._(4, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_TEMPORALLY_VALID');
  static const AcceptanceState ACCEPTANCE_STATE_CLOCK_CONSISTENT =
      AcceptanceState._(5, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_CLOCK_CONSISTENT');
  static const AcceptanceState ACCEPTANCE_STATE_OUTPUT_COMPLETE =
      AcceptanceState._(6, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_OUTPUT_COMPLETE');
  static const AcceptanceState ACCEPTANCE_STATE_HARDWARE_FEASIBLE =
      AcceptanceState._(7, _omitEnumNames ? '' : 'ACCEPTANCE_STATE_HARDWARE_FEASIBLE');

  static const $core.List<AcceptanceState> values = <AcceptanceState>[
    ACCEPTANCE_STATE_UNSPECIFIED,
    ACCEPTANCE_STATE_DECLARED,
    ACCEPTANCE_STATE_DEFINED,
    ACCEPTANCE_STATE_TYPE_VALID,
    ACCEPTANCE_STATE_TEMPORALLY_VALID,
    ACCEPTANCE_STATE_CLOCK_CONSISTENT,
    ACCEPTANCE_STATE_OUTPUT_COMPLETE,
    ACCEPTANCE_STATE_HARDWARE_FEASIBLE,
  ];

  static final $core.List<AcceptanceState?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 7);
  static AcceptanceState? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const AcceptanceState._(super.value, super.name);
}

class MappingStatus extends $pb.ProtobufEnum {
  static const MappingStatus MAPPING_STATUS_UNSPECIFIED =
      MappingStatus._(0, _omitEnumNames ? '' : 'MAPPING_STATUS_UNSPECIFIED');

  /// Named and typed; no definition.
  static const MappingStatus MAPPING_STATUS_DECLARED =
      MappingStatus._(1, _omitEnumNames ? '' : 'MAPPING_STATUS_DECLARED');

  /// Defined, but something it needs is still open (not an error).
  static const MappingStatus MAPPING_STATUS_OPEN =
      MappingStatus._(2, _omitEnumNames ? '' : 'MAPPING_STATUS_OPEN');

  /// Defined and does not check.
  static const MappingStatus MAPPING_STATUS_INVALID =
      MappingStatus._(3, _omitEnumNames ? '' : 'MAPPING_STATUS_INVALID');

  /// Defined and produces what the signature promises.
  static const MappingStatus MAPPING_STATUS_TYPE_VALID =
      MappingStatus._(4, _omitEnumNames ? '' : 'MAPPING_STATUS_TYPE_VALID');

  /// Type-valid and on no instantaneous loop: has a value at every activation.
  static const MappingStatus MAPPING_STATUS_TEMPORALLY_VALID =
      MappingStatus._(5, _omitEnumNames ? '' : 'MAPPING_STATUS_TEMPORALLY_VALID');

  /// Temporally valid and every read stays in its domain or is transported.
  static const MappingStatus MAPPING_STATUS_CLOCK_CONSISTENT =
      MappingStatus._(6, _omitEnumNames ? '' : 'MAPPING_STATUS_CLOCK_CONSISTENT');

  static const $core.List<MappingStatus> values = <MappingStatus>[
    MAPPING_STATUS_UNSPECIFIED,
    MAPPING_STATUS_DECLARED,
    MAPPING_STATUS_OPEN,
    MAPPING_STATUS_INVALID,
    MAPPING_STATUS_TYPE_VALID,
    MAPPING_STATUS_TEMPORALLY_VALID,
    MAPPING_STATUS_CLOCK_CONSISTENT,
  ];

  static final $core.List<MappingStatus?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 6);
  static MappingStatus? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const MappingStatus._(super.value, super.name);
}

class DiagnosticSeverity extends $pb.ProtobufEnum {
  static const DiagnosticSeverity DIAGNOSTIC_SEVERITY_UNSPECIFIED =
      DiagnosticSeverity._(0, _omitEnumNames ? '' : 'DIAGNOSTIC_SEVERITY_UNSPECIFIED');
  static const DiagnosticSeverity DIAGNOSTIC_SEVERITY_ERROR =
      DiagnosticSeverity._(1, _omitEnumNames ? '' : 'DIAGNOSTIC_SEVERITY_ERROR');
  static const DiagnosticSeverity DIAGNOSTIC_SEVERITY_WARNING =
      DiagnosticSeverity._(2, _omitEnumNames ? '' : 'DIAGNOSTIC_SEVERITY_WARNING');
  static const DiagnosticSeverity DIAGNOSTIC_SEVERITY_INFO =
      DiagnosticSeverity._(3, _omitEnumNames ? '' : 'DIAGNOSTIC_SEVERITY_INFO');

  static const $core.List<DiagnosticSeverity> values = <DiagnosticSeverity>[
    DIAGNOSTIC_SEVERITY_UNSPECIFIED,
    DIAGNOSTIC_SEVERITY_ERROR,
    DIAGNOSTIC_SEVERITY_WARNING,
    DIAGNOSTIC_SEVERITY_INFO,
  ];

  static final $core.List<DiagnosticSeverity?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static DiagnosticSeverity? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const DiagnosticSeverity._(super.value, super.name);
}

const $core.bool _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
