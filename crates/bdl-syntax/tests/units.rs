//! The unit expression after a number (docs/spec/textual-syntax.md §5.2):
//! `per`, `*` and `^n` parse into factors; `/` stays value division; the
//! boundary between a unit and the formula's own arithmetic is explicit;
//! malformed spellings are named; the formatter writes the canonical
//! spacing.

#![allow(clippy::unwrap_used)]

use bdl_syntax::format::format_module;
use bdl_syntax::{formula, BinaryOp, ExprKind, SurfaceExpr, SurfaceUnitFactor};

type Factors = Vec<(String, i64)>;

fn unit_of(e: &SurfaceExpr) -> Option<(String, Factors, Factors)> {
    match &e.kind {
        ExprKind::Number { unit: Some(u), .. } => Some((
            u.name.clone(),
            u.numerator
                .iter()
                .map(|f: &SurfaceUnitFactor| (f.symbol.clone(), f.exponent))
                .collect(),
            u.denominator
                .iter()
                .map(|f| (f.symbol.clone(), f.exponent))
                .collect(),
        )),
        _ => None,
    }
}

fn f(s: &str) -> Factors {
    s.split(',')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (sym, e) = p.split_once('^').unwrap_or((p, "1"));
            (sym.trim().to_owned(), e.parse().unwrap())
        })
        .collect()
}

#[test]
fn per_star_and_powers_parse_into_factors() {
    let cases = [
        ("90 deg", "deg", "deg", ""),
        ("180 deg per s", "deg per s", "deg", "s"),
        ("9.81 m per s^2", "m per s^2", "m", "s^2"),
        ("1 N * m", "N * m", "N,m", ""),
        ("1 kg * m per s^2", "kg * m per s^2", "kg,m", "s^2"),
        ("3 s^-1", "s^-1", "s^-1", ""),
        ("2 m^3", "m^3", "m^3", ""),
        ("1 rad ^ 2", "rad^2", "rad^2", ""),
        ("5 turn per min", "turn per min", "turn", "min"),
        // whitespace inside the unit is free; the spelling is normalised
        ("1  m   per   s", "m per s", "m", "s"),
    ];
    for (src, spelling, num, den) in cases {
        let e = formula(src).unwrap_or_else(|e| panic!("{src}: {e:?}"));
        let (name, n, d) = unit_of(&e).unwrap_or_else(|| panic!("{src}: no unit"));
        assert_eq!(name, spelling, "{src}");
        assert_eq!(n, f(num), "{src}");
        assert_eq!(d, f(den), "{src}");
    }
}

#[test]
fn slash_stays_value_division_and_star_continues_a_unit_only_before_a_registered_symbol() {
    // `10 m / s`: a length divided by a *reference* named `s`
    let e = formula("10 m / s").unwrap();
    let ExprKind::Binary { op, lhs, rhs } = &e.kind else {
        panic!("{:?}", e.kind);
    };
    assert_eq!(*op, BinaryOp::Div);
    assert_eq!(unit_of(lhs).unwrap().0, "m");
    assert!(matches!(&rhs.kind, ExprKind::Name(n) if n == "s"));
    // `90 deg * gain`: a quantity times a value — `gain` is no unit
    let e = formula("90 deg * gain").unwrap();
    let ExprKind::Binary { op, lhs, .. } = &e.kind else {
        panic!("{:?}", e.kind);
    };
    assert_eq!(*op, BinaryOp::Mul);
    assert_eq!(unit_of(lhs).unwrap().0, "deg");
    // `2 m * width`, `level * 2`: untouched
    assert!(matches!(
        formula("2 m * width").unwrap().kind,
        ExprKind::Binary {
            op: BinaryOp::Mul,
            ..
        }
    ));
    assert!(matches!(
        formula("level * 2").unwrap().kind,
        ExprKind::Binary {
            op: BinaryOp::Mul,
            ..
        }
    ));
    // `1 N * m`: `m` is a registered symbol, so the unit continues
    assert_eq!(unit_of(&formula("1 N * m").unwrap()).unwrap().0, "N * m");
    // after the unit, arithmetic resumes: `(1 N * m) / torque`
    let e = formula("1 N * m / torque").unwrap();
    let ExprKind::Binary { op, lhs, .. } = &e.kind else {
        panic!("{:?}", e.kind);
    };
    assert_eq!(*op, BinaryOp::Div);
    assert_eq!(unit_of(lhs).unwrap().0, "N * m");
    // a plain `per` elsewhere is an identifier
    assert!(matches!(formula("per").unwrap().kind, ExprKind::Name(n) if n == "per"));
    assert!(matches!(
        formula("per * 2").unwrap().kind,
        ExprKind::Binary { .. }
    ));
}

#[test]
fn a_unit_ends_where_arithmetic_begins() {
    // everything after `per` is unit syntax: `width` is a (future) unit
    // fault, never a reference
    let (name, _, d) = unit_of(&formula("10 m per width").unwrap()).unwrap();
    assert_eq!(name, "m per width");
    assert_eq!(d, f("width"));
    // `+`, `-`, comparison: the literal is done
    let e = formula("10 m per s + 2 m per s").unwrap();
    assert!(matches!(
        e.kind,
        ExprKind::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    // a call after a number is not a unit: the parse is refused, as before
    assert!(formula("2 f(x)").is_err());
}

#[test]
fn malformed_units_are_named() {
    let cases: &[(&str, &str)] = &[
        ("10 m per", "expected a unit after `per`"),
        ("10 per s", "cannot start with `per`"),
        ("10 m^", "whole-number exponent"),
        ("10 m^1.5", "whole-number exponent"),
        ("10 m^^2", "whole-number exponent"),
        ("10 m per per s", "one `per`"),
        ("10 m * per s", "expected a unit after `*`"),
        ("10 m^99999999999999999999", "whole-number exponent"),
    ];
    for (src, expected) in cases {
        let errors = formula(src).err().unwrap_or_else(|| panic!("{src} parsed"));
        assert!(
            errors
                .iter()
                .any(|e| e.code == bdl_syntax::SyntaxErrorCode::MalformedUnit
                    && e.message.contains(expected)),
            "{src}: {errors:?}"
        );
    }
}

#[test]
fn the_formatter_writes_canonical_unit_spacing() {
    let src = "concept A : Angle\nmapping f : () -> A\nf() = 180   deg   per   s + 9.81 m per s ^ 2 + 1 N*m + 2 s ^ - 1\n";
    let out = format_module(src).unwrap();
    assert!(
        out.contains("180 deg per s + 9.81 m per s^2 + 1 N * m + 2 s^-1"),
        "{out}"
    );
    // idempotent
    assert_eq!(format_module(&out).unwrap(), out);
}
