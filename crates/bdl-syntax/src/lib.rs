//! The BDL textual syntax (`docs/TEXTUAL_SYNTAX.md`).
//!
//! ```text
//! source ──lexer──▶ tokens ──parser──▶ events ──sink──▶ Rowan CST
//!                                                    │
//!                                          ast (typed views) ──lower──▶ surface tree
//! ```
//!
//! * [`parse_module`] parses a whole file; [`parse_formula`] parses one
//!   expression (the canvas formula field).  Both are total: any input
//!   yields a lossless tree (`tree.text() == source`) plus diagnostics.
//! * [`ast`] are thin typed wrappers over the tree.
//! * [`lower`] turns an error-free tree into the id-free surface tree that
//!   `bdl-elab` elaborates; [`parse`] / [`formula`] are the shortcuts the
//!   formula path uses.
//!
//! What is *not* here: semantic ids, grants, dimensions, clocks, or any
//! machine number — a literal is its spelling ([`literal::NumberLiteral`]).

#![forbid(unsafe_code)]

pub mod ast;
pub mod format;
pub mod kind;
pub mod lexer;
pub mod literal;
pub mod lower;
pub mod parser;
pub mod syntax;

pub use kind::{BdlLanguage, SyntaxKind};
pub use literal::{Decimal, NumberLiteral};
pub use lower::{
    lower_formula, lower_module, BinaryOp, ExprKind, PatternKind, SurfaceExpr, SurfaceModule,
    SurfacePattern, SurfaceType, TypeKind, UnaryOp, Unit,
};
pub use parser::{parse_formula, parse_module};
pub use syntax::{Parse, SyntaxElement, SyntaxError, SyntaxErrorCode, SyntaxNode, SyntaxToken};

/// A syntax error, under the name the formula path has always used.
pub type ParseError = SyntaxError;

/// Parse and lower one formula, reporting every syntax error.
pub fn formula(src: &str) -> Result<SurfaceExpr, Vec<SyntaxError>> {
    lower_formula(&parse_formula(src))
}

/// Parse and lower one formula, reporting the first syntax error.
pub fn parse(src: &str) -> Result<SurfaceExpr, ParseError> {
    formula(src).map_err(|errors| {
        errors.into_iter().next().unwrap_or_else(|| {
            SyntaxError::new(
                SyntaxErrorCode::Empty,
                bdl_diagnostics::Span::new(0, 0),
                "the formula is empty",
                "nothing",
            )
        })
    })
}
