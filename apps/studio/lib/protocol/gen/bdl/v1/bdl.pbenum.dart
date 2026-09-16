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

/// What a device binding needs from the board is derived from its kind
/// (docs/HARDWARE_MODEL.md); the kind is the whole of what the surface says.
class DeviceKind extends $pb.ProtobufEnum {
  static const DeviceKind DEVICE_KIND_UNSPECIFIED =
      DeviceKind._(0, _omitEnumNames ? '' : 'DEVICE_KIND_UNSPECIFIED');
  static const DeviceKind DEVICE_KIND_PWM_CHANNEL =
      DeviceKind._(1, _omitEnumNames ? '' : 'DEVICE_KIND_PWM_CHANNEL');
  static const DeviceKind DEVICE_KIND_DIGITAL_OUTPUT =
      DeviceKind._(2, _omitEnumNames ? '' : 'DEVICE_KIND_DIGITAL_OUTPUT');
  static const DeviceKind DEVICE_KIND_H_BRIDGE_CHANNEL =
      DeviceKind._(3, _omitEnumNames ? '' : 'DEVICE_KIND_H_BRIDGE_CHANNEL');
  static const DeviceKind DEVICE_KIND_I2C_SENSOR =
      DeviceKind._(4, _omitEnumNames ? '' : 'DEVICE_KIND_I2C_SENSOR');
  static const DeviceKind DEVICE_KIND_QUADRATURE_ENCODER =
      DeviceKind._(5, _omitEnumNames ? '' : 'DEVICE_KIND_QUADRATURE_ENCODER');
  static const DeviceKind DEVICE_KIND_UART =
      DeviceKind._(6, _omitEnumNames ? '' : 'DEVICE_KIND_UART');

  static const $core.List<DeviceKind> values = <DeviceKind>[
    DEVICE_KIND_UNSPECIFIED,
    DEVICE_KIND_PWM_CHANNEL,
    DEVICE_KIND_DIGITAL_OUTPUT,
    DEVICE_KIND_H_BRIDGE_CHANNEL,
    DEVICE_KIND_I2C_SENSOR,
    DEVICE_KIND_QUADRATURE_ENCODER,
    DEVICE_KIND_UART,
  ];

  static final $core.List<DeviceKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 6);
  static DeviceKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const DeviceKind._(super.value, super.name);
}

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

class ProjectKind extends $pb.ProtobufEnum {
  static const ProjectKind PROJECT_KIND_UNSPECIFIED =
      ProjectKind._(0, _omitEnumNames ? '' : 'PROJECT_KIND_UNSPECIFIED');
  static const ProjectKind PROJECT_KIND_FLAT =
      ProjectKind._(1, _omitEnumNames ? '' : 'PROJECT_KIND_FLAT');
  static const ProjectKind PROJECT_KIND_SYSTEM =
      ProjectKind._(2, _omitEnumNames ? '' : 'PROJECT_KIND_SYSTEM');

  static const $core.List<ProjectKind> values = <ProjectKind>[
    PROJECT_KIND_UNSPECIFIED,
    PROJECT_KIND_FLAT,
    PROJECT_KIND_SYSTEM,
  ];

  static final $core.List<ProjectKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 2);
  static ProjectKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ProjectKind._(super.value, super.name);
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

class OutputState extends $pb.ProtobufEnum {
  static const OutputState OUTPUT_STATE_UNSPECIFIED =
      OutputState._(0, _omitEnumNames ? '' : 'OUTPUT_STATE_UNSPECIFIED');
  static const OutputState OUTPUT_STATE_UNDRIVEN =
      OutputState._(1, _omitEnumNames ? '' : 'OUTPUT_STATE_UNDRIVEN');
  static const OutputState OUTPUT_STATE_DRIVEN =
      OutputState._(2, _omitEnumNames ? '' : 'OUTPUT_STATE_DRIVEN');
  static const OutputState OUTPUT_STATE_ILL_FORMED =
      OutputState._(3, _omitEnumNames ? '' : 'OUTPUT_STATE_ILL_FORMED');
  static const OutputState OUTPUT_STATE_CONFLICT =
      OutputState._(4, _omitEnumNames ? '' : 'OUTPUT_STATE_CONFLICT');

  static const $core.List<OutputState> values = <OutputState>[
    OUTPUT_STATE_UNSPECIFIED,
    OUTPUT_STATE_UNDRIVEN,
    OUTPUT_STATE_DRIVEN,
    OUTPUT_STATE_ILL_FORMED,
    OUTPUT_STATE_CONFLICT,
  ];

  static final $core.List<OutputState?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 4);
  static OutputState? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const OutputState._(super.value, super.name);
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

class ActionApplicability extends $pb.ProtobufEnum {
  static const ActionApplicability ACTION_APPLICABILITY_UNSPECIFIED =
      ActionApplicability._(0, _omitEnumNames ? '' : 'ACTION_APPLICABILITY_UNSPECIFIED');

  /// `edits` can be applied as they are.
  static const ActionApplicability ACTION_APPLICABILITY_READY =
      ActionApplicability._(1, _omitEnumNames ? '' : 'ACTION_APPLICABILITY_READY');

  /// The designer picks one of `options`; the tool never guesses.
  static const ActionApplicability ACTION_APPLICABILITY_NEEDS_CHOICE =
      ActionApplicability._(2, _omitEnumNames ? '' : 'ACTION_APPLICABILITY_NEEDS_CHOICE');

  /// The language cannot express the fix yet; `reason` says why.
  static const ActionApplicability ACTION_APPLICABILITY_BLOCKED =
      ActionApplicability._(3, _omitEnumNames ? '' : 'ACTION_APPLICABILITY_BLOCKED');

  static const $core.List<ActionApplicability> values = <ActionApplicability>[
    ACTION_APPLICABILITY_UNSPECIFIED,
    ACTION_APPLICABILITY_READY,
    ACTION_APPLICABILITY_NEEDS_CHOICE,
    ACTION_APPLICABILITY_BLOCKED,
  ];

  static final $core.List<ActionApplicability?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static ActionApplicability? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ActionApplicability._(super.value, super.name);
}

class RoleHint extends $pb.ProtobufEnum {
  static const RoleHint ROLE_HINT_UNSPECIFIED =
      RoleHint._(0, _omitEnumNames ? '' : 'ROLE_HINT_UNSPECIFIED');
  static const RoleHint ROLE_HINT_INPUT = RoleHint._(1, _omitEnumNames ? '' : 'ROLE_HINT_INPUT');
  static const RoleHint ROLE_HINT_OUTPUT = RoleHint._(2, _omitEnumNames ? '' : 'ROLE_HINT_OUTPUT');
  static const RoleHint ROLE_HINT_EITHER = RoleHint._(3, _omitEnumNames ? '' : 'ROLE_HINT_EITHER');

  static const $core.List<RoleHint> values = <RoleHint>[
    ROLE_HINT_UNSPECIFIED,
    ROLE_HINT_INPUT,
    ROLE_HINT_OUTPUT,
    ROLE_HINT_EITHER,
  ];

  static final $core.List<RoleHint?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 3);
  static RoleHint? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const RoleHint._(super.value, super.name);
}

class DeploymentStatus extends $pb.ProtobufEnum {
  static const DeploymentStatus DEPLOYMENT_STATUS_UNSPECIFIED =
      DeploymentStatus._(0, _omitEnumNames ? '' : 'DEPLOYMENT_STATUS_UNSPECIFIED');
  static const DeploymentStatus DEPLOYMENT_STATUS_FEASIBLE =
      DeploymentStatus._(1, _omitEnumNames ? '' : 'DEPLOYMENT_STATUS_FEASIBLE');
  static const DeploymentStatus DEPLOYMENT_STATUS_INFEASIBLE =
      DeploymentStatus._(2, _omitEnumNames ? '' : 'DEPLOYMENT_STATUS_INFEASIBLE');

  /// Placement succeeds but an output has no device, or a device no output.
  static const DeploymentStatus DEPLOYMENT_STATUS_INCOMPLETE =
      DeploymentStatus._(3, _omitEnumNames ? '' : 'DEPLOYMENT_STATUS_INCOMPLETE');

  static const $core.List<DeploymentStatus> values = <DeploymentStatus>[
    DEPLOYMENT_STATUS_UNSPECIFIED,
    DEPLOYMENT_STATUS_FEASIBLE,
    DEPLOYMENT_STATUS_INFEASIBLE,
    DEPLOYMENT_STATUS_INCOMPLETE,
  ];

  static final $core.List<DeploymentStatus?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static DeploymentStatus? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const DeploymentStatus._(super.value, super.name);
}

class MissingKind extends $pb.ProtobufEnum {
  static const MissingKind MISSING_KIND_UNSPECIFIED =
      MissingKind._(0, _omitEnumNames ? '' : 'MISSING_KIND_UNSPECIFIED');

  /// Semantic (about the design itself).
  static const MissingKind MISSING_KIND_RELATIONSHIP_NOT_CHECKING =
      MissingKind._(1, _omitEnumNames ? '' : 'MISSING_KIND_RELATIONSHIP_NOT_CHECKING');
  static const MissingKind MISSING_KIND_NOT_CAUSAL =
      MissingKind._(2, _omitEnumNames ? '' : 'MISSING_KIND_NOT_CAUSAL');
  static const MissingKind MISSING_KIND_NOT_CLOCK_CONSISTENT =
      MissingKind._(3, _omitEnumNames ? '' : 'MISSING_KIND_NOT_CLOCK_CONSISTENT');
  static const MissingKind MISSING_KIND_OUTPUT_NO_DOMAIN =
      MissingKind._(4, _omitEnumNames ? '' : 'MISSING_KIND_OUTPUT_NO_DOMAIN');
  static const MissingKind MISSING_KIND_OUTPUT_NO_DRIVER =
      MissingKind._(5, _omitEnumNames ? '' : 'MISSING_KIND_OUTPUT_NO_DRIVER');
  static const MissingKind MISSING_KIND_OUTPUT_CONNECTION_INVALID =
      MissingKind._(6, _omitEnumNames ? '' : 'MISSING_KIND_OUTPUT_CONNECTION_INVALID');

  /// Deployment configuration (about devices on this target).
  static const MissingKind MISSING_KIND_OUTPUT_NO_DEVICE =
      MissingKind._(7, _omitEnumNames ? '' : 'MISSING_KIND_OUTPUT_NO_DEVICE');
  static const MissingKind MISSING_KIND_DEVICE_NO_OUTPUT =
      MissingKind._(8, _omitEnumNames ? '' : 'MISSING_KIND_DEVICE_NO_OUTPUT');

  static const $core.List<MissingKind> values = <MissingKind>[
    MISSING_KIND_UNSPECIFIED,
    MISSING_KIND_RELATIONSHIP_NOT_CHECKING,
    MISSING_KIND_NOT_CAUSAL,
    MISSING_KIND_NOT_CLOCK_CONSISTENT,
    MISSING_KIND_OUTPUT_NO_DOMAIN,
    MISSING_KIND_OUTPUT_NO_DRIVER,
    MISSING_KIND_OUTPUT_CONNECTION_INVALID,
    MISSING_KIND_OUTPUT_NO_DEVICE,
    MISSING_KIND_DEVICE_NO_OUTPUT,
  ];

  static final $core.List<MissingKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 8);
  static MissingKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const MissingKind._(super.value, super.name);
}

class PortKind extends $pb.ProtobufEnum {
  static const PortKind PORT_KIND_UNSPECIFIED =
      PortKind._(0, _omitEnumNames ? '' : 'PORT_KIND_UNSPECIFIED');
  static const PortKind PORT_KIND_REQUIRED =
      PortKind._(1, _omitEnumNames ? '' : 'PORT_KIND_REQUIRED');
  static const PortKind PORT_KIND_PROVIDED =
      PortKind._(2, _omitEnumNames ? '' : 'PORT_KIND_PROVIDED');
  static const PortKind PORT_KIND_PARAMETER =
      PortKind._(3, _omitEnumNames ? '' : 'PORT_KIND_PARAMETER');

  static const $core.List<PortKind> values = <PortKind>[
    PORT_KIND_UNSPECIFIED,
    PORT_KIND_REQUIRED,
    PORT_KIND_PROVIDED,
    PORT_KIND_PARAMETER,
  ];

  static final $core.List<PortKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 3);
  static PortKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const PortKind._(super.value, super.name);
}

class LocalSort extends $pb.ProtobufEnum {
  static const LocalSort LOCAL_SORT_UNSPECIFIED =
      LocalSort._(0, _omitEnumNames ? '' : 'LOCAL_SORT_UNSPECIFIED');
  static const LocalSort LOCAL_SORT_DECL = LocalSort._(1, _omitEnumNames ? '' : 'LOCAL_SORT_DECL');
  static const LocalSort LOCAL_SORT_SEM = LocalSort._(2, _omitEnumNames ? '' : 'LOCAL_SORT_SEM');
  static const LocalSort LOCAL_SORT_CLOCK =
      LocalSort._(3, _omitEnumNames ? '' : 'LOCAL_SORT_CLOCK');
  static const LocalSort LOCAL_SORT_OUTPUT =
      LocalSort._(4, _omitEnumNames ? '' : 'LOCAL_SORT_OUTPUT');
  static const LocalSort LOCAL_SORT_DEVICE =
      LocalSort._(5, _omitEnumNames ? '' : 'LOCAL_SORT_DEVICE');

  static const $core.List<LocalSort> values = <LocalSort>[
    LOCAL_SORT_UNSPECIFIED,
    LOCAL_SORT_DECL,
    LOCAL_SORT_SEM,
    LOCAL_SORT_CLOCK,
    LOCAL_SORT_OUTPUT,
    LOCAL_SORT_DEVICE,
  ];

  static final $core.List<LocalSort?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 5);
  static LocalSort? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const LocalSort._(super.value, super.name);
}

class PortStatusKind extends $pb.ProtobufEnum {
  static const PortStatusKind PORT_STATUS_KIND_UNSPECIFIED =
      PortStatusKind._(0, _omitEnumNames ? '' : 'PORT_STATUS_KIND_UNSPECIFIED');
  static const PortStatusKind PORT_STATUS_KIND_PROVIDED =
      PortStatusKind._(1, _omitEnumNames ? '' : 'PORT_STATUS_KIND_PROVIDED');
  static const PortStatusKind PORT_STATUS_KIND_BOUND =
      PortStatusKind._(2, _omitEnumNames ? '' : 'PORT_STATUS_KIND_BOUND');
  static const PortStatusKind PORT_STATUS_KIND_EXPORTED =
      PortStatusKind._(3, _omitEnumNames ? '' : 'PORT_STATUS_KIND_EXPORTED');
  static const PortStatusKind PORT_STATUS_KIND_VALUED =
      PortStatusKind._(4, _omitEnumNames ? '' : 'PORT_STATUS_KIND_VALUED');

  /// An unbound required port: an ordinary open declaration, never an error.
  static const PortStatusKind PORT_STATUS_KIND_OPEN =
      PortStatusKind._(5, _omitEnumNames ? '' : 'PORT_STATUS_KIND_OPEN');

  static const $core.List<PortStatusKind> values = <PortStatusKind>[
    PORT_STATUS_KIND_UNSPECIFIED,
    PORT_STATUS_KIND_PROVIDED,
    PORT_STATUS_KIND_BOUND,
    PORT_STATUS_KIND_EXPORTED,
    PORT_STATUS_KIND_VALUED,
    PORT_STATUS_KIND_OPEN,
  ];

  static final $core.List<PortStatusKind?> _byValue = $pb.ProtobufEnum.$_initByValueList(values, 5);
  static PortStatusKind? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const PortStatusKind._(super.value, super.name);
}

class SystemAcceptance extends $pb.ProtobufEnum {
  static const SystemAcceptance SYSTEM_ACCEPTANCE_UNSPECIFIED =
      SystemAcceptance._(0, _omitEnumNames ? '' : 'SYSTEM_ACCEPTANCE_UNSPECIFIED');
  static const SystemAcceptance SYSTEM_ACCEPTANCE_INVALID =
      SystemAcceptance._(1, _omitEnumNames ? '' : 'SYSTEM_ACCEPTANCE_INVALID');
  static const SystemAcceptance SYSTEM_ACCEPTANCE_OPEN =
      SystemAcceptance._(2, _omitEnumNames ? '' : 'SYSTEM_ACCEPTANCE_OPEN');
  static const SystemAcceptance SYSTEM_ACCEPTANCE_EXECUTABLE =
      SystemAcceptance._(3, _omitEnumNames ? '' : 'SYSTEM_ACCEPTANCE_EXECUTABLE');

  static const $core.List<SystemAcceptance> values = <SystemAcceptance>[
    SYSTEM_ACCEPTANCE_UNSPECIFIED,
    SYSTEM_ACCEPTANCE_INVALID,
    SYSTEM_ACCEPTANCE_OPEN,
    SYSTEM_ACCEPTANCE_EXECUTABLE,
  ];

  static final $core.List<SystemAcceptance?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static SystemAcceptance? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const SystemAcceptance._(super.value, super.name);
}

const $core.bool _omitEnumNames = $core.bool.fromEnvironment('protobuf.omit_enum_names');
