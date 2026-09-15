//! The BDL formula language, v0: what a designer types into a mapping's
//! definition field.
//!
//! ```text
//! expr    := if expr then expr else expr | or
//! or      := and ( "||" and )*
//! and     := cmp ( "&&" cmp )*
//! cmp     := add ( ("==" | "!=" | "<" | "<=" | ">" | ">=") add )?
//! add     := mul ( ("+" | "-") mul )*
//! mul     := unary ( ("*" | "/") unary )*
//! unary   := ("!" | "-") unary | primary
//! primary := name | number unit? | true | false | "(" expr ")"
//! ```
//!
//! Deliberately small: no loops, no mutation, no user functions.  The AST
//! ([`ast::SurfaceExpr`]) records what was written, with spans; it is not
//! the Core IR and never leaves the elaborator.

#![forbid(unsafe_code)]

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{BinaryOp, ExprKind, SurfaceExpr, UnaryOp};
pub use parser::{parse, ParseError};
