//! The unit registry and the composite unit algebra live in `bdl-model`
//! (`bdl_model::units`) so the parser can consult the atom symbols; this
//! module keeps the elaborator's path to them.

pub use bdl_model::units::*;
