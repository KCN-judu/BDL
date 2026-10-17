//! Measured curves as terms.  The kernel has no `ln` or `pow`, and a
//! datasheet gives a curve as points anyway: a profile's transducer is a
//! piecewise-linear table, lowered to nested `if … then … else`, so the
//! board evaluates only comparisons, subtractions, products and sums.
//!
//! Every table here is a curve of a *named, common part*, sampled at the
//! points listed; its provenance is said at each constant.  Replacing a
//! table with a datasheet's own points changes one constant.

use bdl_ir::{Expr, Prim, Scalar, Ty};
use bdl_model::Dim;

/// What a piecewise curve does outside its first and last point.
#[derive(Clone, Copy, Debug)]
pub enum Tails {
    /// Hold the end value (a sensor reading saturates).
    Hold,
    /// Continue with this slope (an out-of-range command stays out of
    /// range, so the sink's policy refuses it rather than a table hiding it).
    Slope(f64),
}

fn lit(dim: Dim, value: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim,
        value: Scalar(value),
    })
}

fn bin(p: Prim, a: Expr, b: Expr) -> Expr {
    Expr::apps(Expr::prim(p), [a, b])
}

fn num(v: f64) -> Expr {
    lit(Dim::ZERO, v)
}

fn add(a: Expr, b: Expr) -> Expr {
    bin(Prim::Add { dim: Dim::ZERO }, a, b)
}

fn sub(a: Expr, b: Expr) -> Expr {
    bin(Prim::Sub { dim: Dim::ZERO }, a, b)
}

fn mul(a: Expr, b: Expr) -> Expr {
    bin(
        Prim::Mul {
            d1: Dim::ZERO,
            d2: Dim::ZERO,
        },
        a,
        b,
    )
}

fn lt(a: Expr, b: Expr) -> Expr {
    bin(Prim::Lt { dim: Dim::ZERO }, a, b)
}

fn ite(c: Expr, t: Expr, e: Expr) -> Expr {
    Expr::apps(
        Expr::prim(Prim::Ite {
            ty: Ty::q(Dim::ZERO),
        }),
        [c, t, e],
    )
}

/// `y` of the line through two points at `x`.
fn lerp(x: &Expr, (xa, ya): (f64, f64), (xb, yb): (f64, f64)) -> Expr {
    let slope = (yb - ya) / (xb - xa);
    add(num(ya), mul(sub(x.clone(), num(xa)), num(slope)))
}

/// The piecewise-linear curve through `points` (strictly increasing in
/// `x`), as a dimensionless term of `x`.
pub fn piecewise(x: Expr, points: &[(f64, f64)], tails: Tails) -> Expr {
    assert!(points.len() >= 2, "a curve needs two points");
    let first = points[0];
    let last = points[points.len() - 1];
    let below = match tails {
        Tails::Hold => num(first.1),
        Tails::Slope(s) => add(num(first.1), mul(sub(x.clone(), num(first.0)), num(s))),
    };
    let above = match tails {
        Tails::Hold => num(last.1),
        Tails::Slope(s) => add(num(last.1), mul(sub(x.clone(), num(last.0)), num(s))),
    };
    // innermost: past the last point; each step outward adds one segment
    let mut term = above;
    for w in points.windows(2).rev() {
        term = ite(lt(x.clone(), num(w[1].0)), lerp(&x, w[0], w[1]), term);
    }
    ite(lt(x, num(first.0)), below, term)
}

/// A dimensionless term read as a quantity of `dim` (the unit-1 product).
pub fn as_quantity(term: Expr, dim: Dim) -> Expr {
    bin(
        Prim::Mul {
            d1: Dim::ZERO,
            d2: dim,
        },
        term,
        lit(dim, 1.0),
    )
}

/// 10 kΩ NTC thermistor, β = 3950 K, R₂₅ = 10 kΩ, wired from the pin to
/// ground with a 10 kΩ resistor to 5 V (ADC counts of a 10-bit reading on
/// a 5 V reference) → kelvin.  Sampled from −10 °C to 110 °C every 5 °C
/// from R(T) = R₂₅·exp(β(1/T − 1/T₂₅)) and the divider; the β model is
/// good to about ±1 K over this span.
pub const NTC_10K_B3950: &[(f64, f64)] = &[
    (51.41, 383.15),
    (58.49, 378.15),
    (66.7, 373.15),
    (76.25, 368.15),
    (87.35, 363.15),
    (100.27, 358.15),
    (115.31, 353.15),
    (132.79, 348.15),
    (153.09, 343.15),
    (176.59, 338.15),
    (203.69, 333.15),
    (234.77, 328.15),
    (270.14, 323.15),
    (310.02, 318.15),
    (354.44, 313.15),
    (403.21, 308.15),
    (455.84, 303.15),
    (511.5, 298.15),
    (569.05, 293.15),
    (627.06, 288.15),
    (683.97, 283.15),
    (738.24, 278.15),
    (788.48, 273.15),
    (833.65, 268.15),
    (873.1, 263.15),
];

/// GL5528 photoresistor (R₁₀ₗₓ ≈ 15 kΩ, γ ≈ 0.7: R = R₁₀·(lux/10)^−γ), wired
/// from 5 V to the pin with a 10 kΩ resistor to ground (brighter reads
/// higher), ADC counts → lux, sampled at 1…2000 lx.
pub const LDR_GL5528: &[(f64, f64)] = &[
    (120.1, 1.0),
    (181.78, 2.0),
    (297.66, 5.0),
    (409.2, 10.0),
    (531.88, 20.0),
    (688.33, 50.0),
    (787.35, 100.0),
    (863.85, 200.0),
    (932.54, 500.0),
    (965.35, 1000.0),
    (986.73, 2000.0),
];

/// Sharp GP2Y0A21YK0F infrared ranger on a 5 V reference: the output
/// voltage against distance as read off the datasheet's graph
/// (10 cm → 2.50 V … 80 cm → 0.40 V), ADC counts → metres.  Nearer than
/// 10 cm the part's output folds back and is not trustworthy; below 0.4 V
/// reads as 0.8 m.
pub const SHARP_GP2Y0A21: &[(f64, f64)] = &[
    (81.84, 0.8),
    (90.02, 0.7),
    (102.3, 0.6),
    (118.67, 0.5),
    (143.22, 0.4),
    (184.14, 0.3),
    (265.98, 0.2),
    (337.59, 0.15),
    (511.5, 0.1),
];

/// Gamma 2.2 for an LED: a level 0–100 % → duty 0–255, `255·(n/100)^2.2`
/// sampled every 5 %.
pub const GAMMA22: &[(f64, f64)] = &[
    (0.0, 0.0),
    (5.0, 0.35),
    (10.0, 1.609),
    (15.0, 3.926),
    (20.0, 7.393),
    (25.0, 12.078),
    (30.0, 18.039),
    (35.0, 25.322),
    (40.0, 33.968),
    (45.0, 44.016),
    (50.0, 55.498),
    (55.0, 68.444),
    (60.0, 82.884),
    (65.0, 98.844),
    (70.0, 116.347),
    (75.0, 135.418),
    (80.0, 156.077),
    (85.0, 178.345),
    (90.0, 202.243),
    (95.0, 227.789),
    (100.0, 255.0),
];
