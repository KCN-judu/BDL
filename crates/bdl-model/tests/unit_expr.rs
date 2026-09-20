//! The composite unit algebra (`bdl_model::units::UnitExpr`): the laws of
//! §13–§15 of the brief as properties over the registry, the canonical
//! conversions at the `f64` boundary, physical equivalence, normalisation,
//! and the affine boundary.  What is proved formally is the registry's
//! `Unit K` model (FV Phase 10); these are the production evidence that
//! the derived expressions obey it.

#![allow(clippy::unwrap_used)]

use bdl_model::quantity;
use bdl_model::units::{self, UnitError, UnitExpr, UNITS};
use bdl_model::Dim;
use proptest::prelude::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

fn u(s: &str) -> UnitExpr {
    UnitExpr::parse_canonical(s).unwrap_or_else(|e| panic!("{s}: {e}"))
}

fn dim_of(name: &str) -> Dim {
    quantity::by_type_name(name).unwrap().dim
}

fn atom_index() -> impl Strategy<Value = usize> {
    0..UNITS.len()
}

fn arb_unit() -> impl Strategy<Value = UnitExpr> {
    // one to three atoms with small exponents
    prop::collection::vec((atom_index(), -3i64..=3), 1..=3).prop_map(|fs| {
        let mut acc = UnitExpr::one();
        for (i, e) in fs {
            acc = acc
                .mul(&UnitExpr::atom(&UNITS[i]).unwrap().pow(e).unwrap())
                .unwrap();
        }
        acc
    })
}

fn scale_dim(d: Dim, n: i8) -> Dim {
    Dim {
        length: d.length * n,
        mass: d.mass * n,
        time: d.time * n,
        current: d.current * n,
        temperature: d.temperature * n,
        amount: d.amount * n,
        luminous: d.luminous * n,
        angle: d.angle * n,
    }
}

proptest! {
    #[test]
    fn multiplication_adds_dimensions_and_multiplies_scales(a in arb_unit(), b in arb_unit()) {
        let p = a.mul(&b).unwrap();
        prop_assert_eq!(p.dim(), a.dim() + b.dim());
        prop_assert!(close(p.scale(), a.scale() * b.scale()));
        // commutative
        prop_assert_eq!(b.mul(&a).unwrap(), p);
    }

    #[test]
    fn per_subtracts_dimensions_and_divides_scales(a in arb_unit(), b in arb_unit()) {
        let q = a.per(&b).unwrap();
        prop_assert_eq!(q.dim(), a.dim() - b.dim());
        prop_assert!(close(q.scale(), a.scale() / b.scale()));
    }

    #[test]
    fn powers_scale_dimensions_and_exponentiate_scales(a in arb_unit(), n in -4i8..=4) {
        let p = a.pow(i64::from(n)).unwrap();
        prop_assert_eq!(p.dim(), scale_dim(a.dim(), n));
        prop_assert!(close(p.scale(), a.scale().powi(i32::from(n))));
    }

    #[test]
    fn u_per_u_is_dimensionless_with_scale_one(a in arb_unit()) {
        let q = a.per(&a).unwrap();
        prop_assert!(q.is_dimensionless());
        prop_assert_eq!(q.scale(), 1.0);
        prop_assert_eq!(q, UnitExpr::one());
    }

    #[test]
    fn inverse_cancels(a in arb_unit(), b in arb_unit()) {
        let q = a.per(&b).unwrap().mul(&b.per(&a).unwrap()).unwrap();
        prop_assert_eq!(q, UnitExpr::one());
    }

    #[test]
    fn normalisation_is_idempotent_and_spelling_round_trips(a in arb_unit()) {
        let again = a.mul(&UnitExpr::one()).unwrap();
        prop_assert_eq!(&again, &a);
        let spelled = UnitExpr::parse_canonical(&a.source()).unwrap();
        prop_assert_eq!(&spelled, &a);
        prop_assert_eq!(spelled.source(), a.source());
        // the display renders the same factors in the same order
        let strip = |s: &str| s.chars().filter(|c| c.is_ascii_alphabetic()).collect::<String>();
        prop_assert_eq!(strip(&a.display()), strip(&a.source()).replace("per", ""));
    }

    #[test]
    fn cancellation_preserves_physical_meaning(a in arb_unit(), b in arb_unit(), x in -1e6f64..1e6) {
        // (x in a*b) per b is x in a: the same canonical magnitude
        let ab = a.mul(&b).unwrap();
        let back = ab.per(&b).unwrap();
        prop_assert_eq!(&back, &a);
        prop_assert!(close(back.to_canonical(x), a.to_canonical(x)));
    }

    #[test]
    fn conversion_between_equivalent_expressions_composes(a in arb_unit(), x in -1e6f64..1e6) {
        // to canonical and back is the identity; two spellings of one unit
        // give one magnitude
        prop_assert!(close(a.from_canonical(a.to_canonical(x)), x));
        let via = UnitExpr::parse_canonical(&a.source()).unwrap();
        prop_assert!(close(via.to_canonical(x), a.to_canonical(x)));
    }
}

#[test]
fn the_required_examples_have_the_named_dimensions() {
    let cases = [
        ("rad per s", "AngularVelocity"),
        ("deg per s", "AngularVelocity"),
        ("turn per s", "AngularVelocity"),
        ("deg per min", "AngularVelocity"),
        ("m per s", "Speed"),
        ("mm per s", "Speed"),
        ("km per h", "Speed"),
        ("m per s^2", "Acceleration"),
        ("mm per s^2", "Acceleration"),
        ("N * m", "Torque"),
        ("kg * m per s^2", "Force"),
    ];
    for (spelling, name) in cases {
        assert_eq!(u(spelling).dim(), dim_of(name), "{spelling}");
    }
    // N per m is a force per length: the algebra, no name
    assert_eq!(u("N per m").dim(), dim_of("Force") - Dim::LENGTH);
    assert_eq!(u("kg * m").dim(), Dim::MASS + Dim::LENGTH);
    assert_eq!(u("s^-2").dim(), scale_dim(Dim::TIME, -2));
    assert!(close(u("s^-2").scale(), 1.0));
    assert!(close(u("ms^-2").scale(), 1e6));
    assert_eq!(u("m^2").dim(), scale_dim(Dim::LENGTH, 2));
    assert_eq!(u("m^3").dim(), scale_dim(Dim::LENGTH, 3));
    assert_eq!(u("rad^2").dim(), scale_dim(Dim::ANGLE, 2));
}

#[test]
fn angle_stays_a_base_dimension() {
    assert_eq!(u("rad per s").dim(), Dim::ANGLE - Dim::TIME);
    assert_ne!(u("rad per s").dim(), u("Hz").dim());
    assert!(!u("rad per s").equivalent(&u("Hz")));
    assert!(u("Hz").equivalent(&u("s^-1")));
    assert!(close(
        u("deg per s").scale() / u("rad per s").scale(),
        std::f64::consts::PI / 180.0
    ));
}

#[test]
fn angular_velocity_speed_and_acceleration_convert_at_the_boundary() {
    let pi = std::f64::consts::PI;
    // 180 deg per s = π rad per s
    assert!(close(
        u("deg per s").to_canonical(180.0),
        u("rad per s").to_canonical(pi)
    ));
    // 1 turn per s = 2π rad per s
    assert!(close(
        u("turn per s").to_canonical(1.0),
        u("rad per s").to_canonical(2.0 * pi)
    ));
    // 60 deg per min = 1 deg per s
    assert!(close(
        u("deg per min").to_canonical(60.0),
        u("deg per s").to_canonical(1.0)
    ));
    // 1000 mm per s = 1 m per s
    assert!(close(
        u("mm per s").to_canonical(1000.0),
        u("m per s").to_canonical(1.0)
    ));
    // 36 km per h = 10 m per s
    assert!(close(
        u("km per h").to_canonical(36.0),
        u("m per s").to_canonical(10.0)
    ));
    // 60 m per min = 1 m per s
    assert!(close(
        u("m per min").to_canonical(60.0),
        u("m per s").to_canonical(1.0)
    ));
    // 9.81 m per s^2 = 9810 mm per s^2, both L T⁻²
    let a = u("m per s^2");
    let b = u("mm per s^2");
    assert_eq!(a.dim(), dim_of("Acceleration"));
    assert_eq!(b.dim(), dim_of("Acceleration"));
    assert!(close(a.to_canonical(9.81), b.to_canonical(9810.0)));
    assert!(close(b.from_canonical(a.to_canonical(9.81)), 9810.0));
}

#[test]
fn physical_equivalence_is_dimension_and_scale_not_factorisation() {
    let torque = u("N * m");
    let base = u("kg * m^2 per s^2");
    assert_ne!(torque, base, "two factorisations");
    assert!(torque.equivalent(&base));
    assert_eq!(torque.dim(), dim_of("Torque"));
    assert!(u("N per m").equivalent(&u("kg per s^2")));
    assert!(u("Hz").equivalent(&u("s^-1")));
    assert!(!u("Hz").equivalent(&u("rad per s")));
    // normalisation: cancellation and exponent combination
    assert_eq!(u("m * s per s"), u("m"));
    assert_eq!(u("m^2 per m"), u("m"));
    assert_eq!(u("rad per rad"), UnitExpr::one());
    assert_eq!(u("deg per deg"), UnitExpr::one());
    assert_eq!(u("deg per deg").scale(), 1.0);
    assert_eq!(u("m * m"), u("m^2"));
}

#[test]
fn canonical_spellings_and_displays() {
    for (src, display) in [
        ("rad per s", "rad/s"),
        ("m per s^2", "m/s²"),
        ("kg * m per s^2", "kg·m/s²"),
        ("N * m", "N·m"),
        ("s^-1", "s⁻¹"),
        ("deg", "deg"),
        ("1", "1"),
    ] {
        let e = u(src);
        assert_eq!(e.source(), src);
        assert_eq!(e.display(), display);
    }
    // the vocabulary's preferred units are canonical spellings of their
    // own dimension, and every curated candidate parses to the quantity
    for q in quantity::QUANTITIES {
        for spelling in quantity::preferred_units(q.id) {
            let e = u(spelling);
            assert_eq!(e.dim(), q.dim, "{}: {spelling}", q.id);
            assert_eq!(e.source(), *spelling, "{spelling} is not canonical");
        }
    }
    let angular = units::candidates_for(dim_of("AngularVelocity"));
    let spelled: Vec<String> = angular.iter().map(|e| e.source()).collect();
    assert_eq!(
        spelled,
        ["rad per s", "deg per s", "turn per s", "deg per min"]
    );
    assert_eq!(
        units::preferred_for(dim_of("Speed")).unwrap().source(),
        "m per s"
    );
    assert_eq!(units::preferred_for(Dim::ANGLE).unwrap().source(), "rad");
    assert!(
        units::candidates_for(Dim::LENGTH + Dim::LENGTH).is_empty(),
        "area: nothing curated"
    );
}

#[test]
fn affine_atoms_never_enter_the_algebra() {
    for def in units::AFFINE_CHARTS {
        assert_eq!(
            UnitExpr::atom(def).unwrap_err(),
            UnitError::AffineAtom {
                symbol: def.symbol.to_owned()
            }
        );
        assert!(matches!(
            UnitExpr::symbol(def.symbol).unwrap_err(),
            UnitError::AffineAtom { .. }
        ));
    }
    // by spelling, before any atom exists: `°C per s`, `°F * m`, `°C^2`
    for text in ["°C per s", "°F * m", "°C^2"] {
        assert!(
            matches!(
                UnitExpr::parse_canonical(text),
                Err(UnitError::AffineAtom { .. })
            ),
            "{text}"
        );
    }
    assert!(matches!(
        UnitExpr::parse_canonical("furlong per s"),
        Err(UnitError::UnknownAtom { .. })
    ));
}

#[test]
fn exponents_are_bounded_never_wrapped() {
    let m = u("m");
    assert!(matches!(m.pow(1000), Err(UnitError::ExponentRange { .. })));
    assert!(matches!(m.pow(128), Err(UnitError::ExponentRange { .. })));
    assert!(m.pow(127).is_ok() && m.pow(-127).is_ok());
    assert!(matches!(
        UnitExpr::parse_canonical("m^1000"),
        Err(UnitError::ExponentRange { .. })
    ));
    // combining two in-range factors past the bound is refused too
    let big = m.pow(100).unwrap();
    assert!(matches!(
        big.mul(&big),
        Err(UnitError::ExponentRange { .. })
    ));
    assert_eq!(m.pow(0).unwrap(), UnitExpr::one());
    // build: the faulting factor is indexed, numerator first
    let err = UnitExpr::build(&[("m".into(), 1)], &[("furlong".into(), 1)]).unwrap_err();
    assert_eq!(err.0, 1);
}
