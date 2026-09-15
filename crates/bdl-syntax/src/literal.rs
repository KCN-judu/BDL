//! Number literals as *spellings* with an exact decimal reading.
//!
//! `bdl-syntax` never produces an `f64`: `0.1` stays `"0.1"`, and
//! [`NumberLiteral::decimal`] reads it as the exact `1 × 10⁻¹`.  Machine
//! conversion (`to_f64`) exists only for the elaboration boundary and is
//! recorded as technical debt against the planned exact/symbolic layer
//! (`docs/TEXTUAL_SYNTAX.md` §2.3, §11; DI-1).

use serde::{Deserialize, Serialize};

/// The source text of a number token.  Equality is spelling equality:
/// `1.0` ≠ `1`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NumberLiteral {
    pub text: String,
}

/// An exact decimal: `digits × 10^exponent`, `digits` a non-empty run of
/// ASCII digits without leading zeros (`"0"` for zero).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Decimal {
    pub digits: String,
    pub exponent: i64,
}

impl NumberLiteral {
    pub fn new(text: impl Into<String>) -> NumberLiteral {
        NumberLiteral { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// `true` when the spelling has no fraction and no exponent (`42`,
    /// not `4.2e1`), the class literal patterns accept.
    pub fn is_integer_spelling(&self) -> bool {
        !self.text.contains(['.', 'e', 'E'])
    }

    /// The exact value.  `None` only when the exponent does not fit an
    /// `i64` (a spelling no product will ever contain) or the text is not a
    /// number token's text.
    pub fn decimal(&self) -> Option<Decimal> {
        let t = self.text.as_str();
        let (mantissa, exp) = match t.find(['e', 'E']) {
            Some(i) => (&t[..i], t[i + 1..].parse::<i64>().ok()?),
            None => (t, 0i64),
        };
        let (int_part, frac_part) = match mantissa.find('.') {
            Some(i) => (&mantissa[..i], &mantissa[i + 1..]),
            None => (mantissa, ""),
        };
        if !int_part.bytes().all(|b| b.is_ascii_digit())
            || !frac_part.bytes().all(|b| b.is_ascii_digit())
            || (int_part.is_empty() && frac_part.is_empty())
        {
            return None;
        }
        let mut digits = String::with_capacity(int_part.len() + frac_part.len());
        digits.push_str(int_part);
        digits.push_str(frac_part);
        let digits = digits.trim_start_matches('0');
        let exponent = exp.checked_sub(frac_part.len() as i64)?;
        Some(if digits.is_empty() {
            Decimal {
                digits: "0".to_owned(),
                exponent: 0,
            }
        } else {
            Decimal {
                digits: digits.to_owned(),
                exponent,
            }
        })
    }

    /// Machine conversion for the elaboration boundary (DI-1).  `None` when
    /// the value is not finite in `f64`.
    pub fn to_f64(&self) -> Option<f64> {
        self.text.parse::<f64>().ok().filter(|v| v.is_finite())
    }
}

impl Decimal {
    /// Canonical form: trailing zeros of the digits folded into the
    /// exponent, so `1.0` and `1` and `10e-1` compare equal.
    pub fn normalized(&self) -> Decimal {
        if self.digits == "0" {
            return Decimal {
                digits: "0".to_owned(),
                exponent: 0,
            };
        }
        let trimmed = self.digits.trim_end_matches('0');
        let zeros = (self.digits.len() - trimmed.len()) as i64;
        Decimal {
            digits: trimmed.to_owned(),
            exponent: self.exponent.saturating_add(zeros),
        }
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}e{}", self.digits, self.exponent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        NumberLiteral::new(s).decimal().expect("valid literal")
    }

    #[test]
    fn exact_readings() {
        assert_eq!(
            dec("0.1"),
            Decimal {
                digits: "1".into(),
                exponent: -1
            }
        );
        assert_eq!(
            dec("90"),
            Decimal {
                digits: "90".into(),
                exponent: 0
            }
        );
        assert_eq!(
            dec("1.5e3"),
            Decimal {
                digits: "15".into(),
                exponent: 2
            }
        );
        assert_eq!(
            dec(".5"),
            Decimal {
                digits: "5".into(),
                exponent: -1
            }
        );
        assert_eq!(
            dec("0.0"),
            Decimal {
                digits: "0".into(),
                exponent: 0
            }
        );
        assert_eq!(
            dec("007"),
            Decimal {
                digits: "7".into(),
                exponent: 0
            }
        );
        assert_eq!(
            dec("2E-3"),
            Decimal {
                digits: "2".into(),
                exponent: -3
            }
        );
        assert_eq!(
            dec("1e999"),
            Decimal {
                digits: "1".into(),
                exponent: 999
            }
        );
    }

    #[test]
    fn spelling_is_preserved_and_normalization_is_value_equality() {
        let a = NumberLiteral::new("1.0");
        let b = NumberLiteral::new("1");
        assert_ne!(a, b);
        assert_ne!(dec("1.0"), dec("1"));
        assert_eq!(dec("1.0").normalized(), dec("1").normalized());
        assert_eq!(dec("10e-1").normalized(), dec("1").normalized());
        assert_eq!(
            dec("100").normalized(),
            Decimal {
                digits: "1".into(),
                exponent: 2
            }
        );
    }

    #[test]
    fn machine_boundary() {
        assert_eq!(NumberLiteral::new("0.1").to_f64(), Some(0.1));
        assert_eq!(NumberLiteral::new("1e999").to_f64(), None);
        assert!(NumberLiteral::new("1e99999999999999999999")
            .decimal()
            .is_none());
        assert!(NumberLiteral::new("abc").decimal().is_none());
        assert!(NumberLiteral::new("42").is_integer_spelling());
        assert!(!NumberLiteral::new("4.2").is_integer_spelling());
    }
}
