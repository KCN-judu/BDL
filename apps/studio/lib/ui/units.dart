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

import '../l10n/l10n.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// One selectable preset: the quantity kind, its unit symbol, its dimension.
typedef UnitPreset = ({String name, String symbol, pb.Dim dim});

UnitPreset noUnit([AppLocalizations? l10n]) =>
    (name: (l10n ?? kEnglish).noUnit, symbol: 'dimensionless', dim: pb.Dim());

/// Fallback presets: no unit, the seven SI base dimensions plus angle.
List<UnitPreset> builtinUnitPresets([AppLocalizations? l10n]) {
  l10n ??= kEnglish;
  return [
    noUnit(l10n),
    (name: quantityWord('Angle', l10n), symbol: 'rad', dim: pb.Dim(angle: 1)),
    (name: quantityWord('Length', l10n), symbol: 'm', dim: pb.Dim(length: 1)),
    (name: quantityWord('Time', l10n), symbol: 's', dim: pb.Dim(time: 1)),
    (name: quantityWord('Mass', l10n), symbol: 'kg', dim: pb.Dim(mass: 1)),
    (name: quantityWord('Temperature', l10n), symbol: 'K', dim: pb.Dim(temperature: 1)),
    (name: quantityWord('Current', l10n), symbol: 'A', dim: pb.Dim(current: 1)),
    (name: quantityWord('Luminous', l10n), symbol: 'cd', dim: pb.Dim(luminous: 1)),
    (name: quantityWord('Amount', l10n), symbol: 'mol', dim: pb.Dim(amount: 1)),
  ];
}

/// The picker's presets from the daemon's quantity vocabulary: *no unit*
/// first, then every named quantity but the dimensionless one, in the
/// daemon's order (base dimensions, then derived).  The built-in list when
/// [quantities] is empty.
List<UnitPreset> unitPresetsFrom(Iterable<pb.QuantityView> quantities, [AppLocalizations? l10n]) {
  l10n ??= kEnglish;
  final served = [
    for (final q in quantities)
      if (q.dim != pb.Dim()) (name: quantityWord(q.typeName, l10n), symbol: q.unit, dim: q.dim),
  ];
  return served.isEmpty ? builtinUnitPresets(l10n) : [noUnit(l10n), ...served];
}

/// The designer's word for a quantity kind.  The base dimensions and the
/// few kinds whose type name is not their word are localized; a derived
/// kind (`AngularVelocity`) is spelled out from its type name in English,
/// the shared vocabulary's own language (docs/project/localization-style.md).
String quantityWord(String typeName, [AppLocalizations? l10n]) {
  l10n ??= kEnglish;
  return switch (typeName) {
    'Scalar' => l10n.noUnit,
    'Angle' => l10n.dimAngle,
    'Length' => l10n.dimLength,
    'Time' => l10n.dimTime,
    'Mass' => l10n.dimMass,
    'Temperature' => l10n.dimTemperature,
    'Current' => l10n.dimCurrent,
    'Luminous' => l10n.luminousIntensity,
    'Amount' => l10n.amountOfSubstance,
    _ => typeName.replaceAllMapped(RegExp(r'(?<=[a-z])([A-Z])'), (m) => ' ${m[1]}').toLowerCase(),
  };
}

/// The preset whose dimension equals [dim], if any.
UnitPreset? unitPresetFor(List<UnitPreset> presets, pb.Dim dim) =>
    presets.where((p) => p.dim == dim).firstOrNull;
