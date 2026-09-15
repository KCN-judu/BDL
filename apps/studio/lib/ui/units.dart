// Unit presets offered when a concept is a quantity.
//
// The semantic fact is the dimension (`pb.Dim`, eight SI exponents); the
// designer chooses it by the name of the quantity kind, with the unit symbol
// in its own column.  The first entry is the dedicated *no unit* choice — a
// dimensionless quantity is still a quantity.  Every base dimension the model
// carries is listed (including luminous intensity and amount of substance),
// then a few common derived ones.

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// One selectable preset: the quantity kind, its unit symbol, its dimension.
typedef UnitPreset = ({String name, String symbol, pb.Dim dim});

/// Ordered presets: no unit, the seven SI base dimensions plus angle, then
/// derived quantities.
final List<UnitPreset> unitPresets = [
  (name: 'no unit', symbol: 'dimensionless', dim: pb.Dim()),
  (name: 'angle', symbol: 'rad', dim: pb.Dim(angle: 1)),
  (name: 'length', symbol: 'm', dim: pb.Dim(length: 1)),
  (name: 'time', symbol: 's', dim: pb.Dim(time: 1)),
  (name: 'mass', symbol: 'kg', dim: pb.Dim(mass: 1)),
  (name: 'temperature', symbol: 'K', dim: pb.Dim(temperature: 1)),
  (name: 'current', symbol: 'A', dim: pb.Dim(current: 1)),
  (name: 'luminous intensity', symbol: 'cd', dim: pb.Dim(luminous: 1)),
  (name: 'amount of substance', symbol: 'mol', dim: pb.Dim(amount: 1)),
  (name: 'angular rate', symbol: 'rad/s', dim: pb.Dim(angle: 1, time: -1)),
  (name: 'speed', symbol: 'm/s', dim: pb.Dim(length: 1, time: -1)),
];

/// The preset whose dimension equals [dim], if any.
UnitPreset? unitPresetFor(pb.Dim dim) => unitPresets.where((p) => p.dim == dim).firstOrNull;
