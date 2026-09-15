// Unit presets offered when a concept is a quantity.
//
// The semantic fact is the dimension (`pb.Dim`, eight SI exponents); the
// designer chooses it by the name of the quantity kind, with the unit symbol
// in its own column.  The first entry is the dedicated *no unit* choice — a
// dimensionless quantity is still a quantity.
//
// The list is the daemon's shared quantity vocabulary (`bdl_model::quantity`,
// served with the concept libraries), so the picker, the Library rows and
// the textual syntax name the same kinds.  Until the daemon has answered, a
// built-in fallback covers the base dimensions.

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// One selectable preset: the quantity kind, its unit symbol, its dimension.
typedef UnitPreset = ({String name, String symbol, pb.Dim dim});

final UnitPreset noUnit = (name: 'no unit', symbol: 'dimensionless', dim: pb.Dim());

/// Fallback presets: no unit, the seven SI base dimensions plus angle.
final List<UnitPreset> builtinUnitPresets = [
  noUnit,
  (name: 'angle', symbol: 'rad', dim: pb.Dim(angle: 1)),
  (name: 'length', symbol: 'm', dim: pb.Dim(length: 1)),
  (name: 'time', symbol: 's', dim: pb.Dim(time: 1)),
  (name: 'mass', symbol: 'kg', dim: pb.Dim(mass: 1)),
  (name: 'temperature', symbol: 'K', dim: pb.Dim(temperature: 1)),
  (name: 'current', symbol: 'A', dim: pb.Dim(current: 1)),
  (name: 'luminous intensity', symbol: 'cd', dim: pb.Dim(luminous: 1)),
  (name: 'amount of substance', symbol: 'mol', dim: pb.Dim(amount: 1)),
];

/// The picker's presets from the daemon's quantity vocabulary: *no unit*
/// first, then every named quantity but the dimensionless one, in the
/// daemon's order (base dimensions, then derived).  The built-in list when
/// [quantities] is empty.
List<UnitPreset> unitPresetsFrom(Iterable<pb.QuantityView> quantities) {
  final served = [
    for (final q in quantities)
      if (q.dim != pb.Dim()) (name: quantityWord(q.typeName), symbol: q.unit, dim: q.dim),
  ];
  return served.isEmpty ? builtinUnitPresets : [noUnit, ...served];
}

/// `AngularVelocity` → `angular velocity`, `Luminous` → `luminous intensity`.
String quantityWord(String typeName) {
  if (typeName == 'Luminous') return 'luminous intensity';
  if (typeName == 'Amount') return 'amount of substance';
  return typeName.replaceAllMapped(RegExp(r'(?<=[a-z])([A-Z])'), (m) => ' ${m[1]}').toLowerCase();
}

/// The preset whose dimension equals [dim], if any.
UnitPreset? unitPresetFor(List<UnitPreset> presets, pb.Dim dim) =>
    presets.where((p) => p.dim == dim).firstOrNull;
