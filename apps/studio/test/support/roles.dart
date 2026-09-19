/// Fixture relationships with the role the daemon would state.
///
/// Every `MappingView` bdld sends carries `role` (protocol 0.20), derived
/// in Rust from the authored shape and realization state; Studio never
/// re-derives it.  A fixture stands in for the daemon, so it states the
/// role the same way — here, for fixtures only, mirroring
/// `bdl_model::RelationshipRole`.  A test that exercises a role the shape
/// alone does not give (a base relationship a binding realises, which the
/// system view states as a Value) passes `role:` explicitly.
library;

import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';

/// What bdld states for a relationship that reads [reads] inputs and
/// has ([realized]) or lacks a realization.
pb.RelationshipRole statedRole({required bool reads, required bool realized}) => reads
    ? pb.RelationshipRole.RELATIONSHIP_ROLE_RULE
    : realized
    ? pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE
    : pb.RelationshipRole.RELATIONSHIP_ROLE_SOURCE;

/// A `MappingView` as the daemon sends it: with its role.
pb.MappingView mappingView({
  Int64? id,
  String? name,
  String? description,
  pb.Signature? signature,
  pb.Definition? definition,
  pb.AcceptanceState? state,
  Int64? clockId,
  Int64? drivesOutputId,
  pb.RelationshipRole? role,
}) {
  final m = pb.MappingView(
    id: id,
    name: name,
    description: description,
    signature: signature,
    definition: definition,
    state: state,
    clockId: clockId,
    drivesOutputId: drivesOutputId,
  );
  m.role =
      role ??
      statedRole(reads: signature?.inputs.isNotEmpty ?? false, realized: definition != null);
  return m;
}

/// An analysis as the daemon sends it: with `applied_by` on every
/// relationship — the inverse of the fixture's `references` (the compiler
/// inverts its own dependency graph; a fixture stands in for it).  A
/// relationship named only as a reference gets an entry of its own.
pb.ProjectAnalysis withAppliedBy(pb.ProjectAnalysis a) {
  final appliedBy = <Int64, List<Int64>>{};
  for (final m in a.mappings) {
    for (final d in m.references) {
      appliedBy.putIfAbsent(d, () => []).add(m.id);
    }
  }
  final present = {for (final m in a.mappings) m.id};
  for (final m in a.mappings) {
    m.appliedBy
      ..clear()
      ..addAll(appliedBy[m.id] ?? const []);
  }
  for (final e in appliedBy.entries) {
    if (!present.contains(e.key)) {
      a.mappings.add(pb.MappingAnalysis(id: e.key, appliedBy: e.value));
    }
  }
  return a;
}
