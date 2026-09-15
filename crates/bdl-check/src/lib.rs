//! Checking the Reactive Core IR against the kernel's rules
//! (`BDL_FV/BDL/Core/Typing.lean`, `Decl.lean`).
//!
//! * [`infer`] — syntax-directed type inference for `Expr`, consulting the
//!   design only through `DesignIr::ty_view` (declarations) and
//!   `DesignIr::representation_of` (concepts), under a [`Grant`].
//! * [`check_realization`] — a declaration's body against its interface:
//!   typed under `Grant::of(expected_type)`, and equal to the expected type.
//!
//! Dimension correctness is not a separate pass: it falls out of typing,
//! because the dimension algebra lives in `Prim::ty`.
//!
//! Everything is pure.  Errors carry an [`ExprPath`] so an elaborator that
//! kept a path → span map can point at the source.

#![forbid(unsafe_code)]

pub mod pretty;
pub mod typing;

pub use typing::{check_realization, infer, ExprPath, Grant, TypeError, TypeErrorKind};
