//! The kernel's types (`Base.lean`), plus the empty product.
//!
//! # A relationship's type
//!
//! A mapping `(A₁, …, Aₙ) -> B` over concepts has one **canonical type**,
//! the function type from its domain to its output:
//!
//! ```text
//! domain([])          = ()                     (the empty product, `Ty::Unit`)
//! domain([A])         = A
//! domain([A, B, …])   = A × (B × …)            (right-nested, like the surface tuple)
//! mapping type        = domain(inputs) -> B
//! ```
//!
//! so `mapping f : B` — shorthand for `mapping f : () -> B` — has type
//! `() -> B`, not `B` ([`Ty::of_signature`]).  The kernel's `Ty` has no unit
//! and binary arrows only, so a declaration's interface type
//! ([`Ty::kernel_of_signature`], Lean's `expectedType`) is the canonical
//! type under two isomorphisms that the checker, the evaluator and the
//! generated code work with directly:
//!
//! ```text
//! (A × B) -> C  ≅  A -> (B -> C)      currying
//! () -> B       ≅  B                  unit elimination: the unique argument is erased
//! Product([])   ≅  ()                 the empty product is the unit
//! ```
//!
//! The two encodings are a bijection over signatures ([`Ty::canonical_mapping_ty`]
//! recovers the canonical type from the kernel one), so a relationship
//! without inputs is not a category of its own: it is the `()`-domain case of
//! the one rule, represented — like every zero-argument function in the
//! generated core — by its value at the unique point.

use bdl_model::{ConceptId, Dim};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum Ty {
    Bool,
    Nat,
    /// The empty product `()`: the unique value carries no information.  The
    /// domain of a relationship without inputs (`() -> B`).  Never a
    /// concept's representation and never the type of a Core term: the
    /// kernel encodes `() -> B` as `B` (unit elimination, module docs).
    Unit,
    Arr {
        dom: Box<Ty>,
        cod: Box<Ty>,
    },
    /// `sem C`, "a Sem of `C`": the nominal type a concept names — a
    /// template; a Sem block is its instance.  Two distinct ids are
    /// distinct types regardless of representation.
    Sem {
        id: ConceptId,
    },
    /// Physical quantity of dimension `dim`.
    Q {
        dim: Dim,
    },
    /// Optional value; `opt τ` streams are single-domain occurrences.
    Opt {
        inner: Box<Ty>,
    },
    /// Ordinary list data (Phase 9a).  A list is data exactly when its
    /// elements are; the cross-domain window of occurrences is one.
    List {
        elem: Box<Ty>,
    },
    /// A product (Phase 9b): value-level composition only.  Never a
    /// component interface, an output bundle or a system boundary.
    Prod {
        fst: Box<Ty>,
        snd: Box<Ty>,
    },
}

impl Ty {
    pub fn arr(dom: Ty, cod: Ty) -> Ty {
        Ty::Arr {
            dom: Box::new(dom),
            cod: Box::new(cod),
        }
    }
    pub fn opt(inner: Ty) -> Ty {
        Ty::Opt {
            inner: Box::new(inner),
        }
    }
    pub fn list(elem: Ty) -> Ty {
        Ty::List {
            elem: Box::new(elem),
        }
    }
    pub fn prod(fst: Ty, snd: Ty) -> Ty {
        Ty::Prod {
            fst: Box::new(fst),
            snd: Box::new(snd),
        }
    }
    pub fn sem(id: ConceptId) -> Ty {
        Ty::Sem { id }
    }
    pub fn q(dim: Dim) -> Ty {
        Ty::Q { dim }
    }
    /// Build `a₁ → … → aₙ → b`.
    pub fn arrows(inputs: impl IntoIterator<Item = Ty>, output: Ty) -> Ty {
        let inputs: Vec<Ty> = inputs.into_iter().collect();
        inputs
            .into_iter()
            .rev()
            .fold(output, |cod, dom| Ty::arr(dom, cod))
    }

    /// The domain of a relationship over the concepts `inputs`: `()` for
    /// none, the concept for one, a right-nested product otherwise (module
    /// docs).
    pub fn domain_of(inputs: &[ConceptId]) -> Ty {
        match inputs {
            [] => Ty::Unit,
            [a] => Ty::sem(*a),
            [a, rest @ ..] => Ty::prod(Ty::sem(*a), Ty::domain_of(rest)),
        }
    }

    /// The canonical type of a relationship: `domain(inputs) -> output`.
    /// `mapping f : B` and `mapping f : () -> B` both give `() -> B`.
    pub fn of_signature(inputs: &[ConceptId], output: ConceptId) -> Ty {
        Ty::arr(Ty::domain_of(inputs), Ty::sem(output))
    }

    /// The kernel encoding of the same relationship: curried, with the
    /// unit domain eliminated — `sem A₁ → … → sem Aₙ → sem B`, and `sem B`
    /// when there are no inputs.  Lean's `expectedType`; what a
    /// realization is checked against and what `declRef` has as a term.
    pub fn kernel_of_signature(inputs: &[ConceptId], output: ConceptId) -> Ty {
        Ty::arrows(inputs.iter().map(|c| Ty::sem(*c)), Ty::sem(output))
    }

    /// The canonical type back from a kernel interface type: a curried
    /// arrow is uncurried into a product domain, anything else is the value
    /// at the unique point of `()`.  Inverse of [`Ty::kernel_of_signature`]
    /// on signatures over concepts.
    pub fn canonical_mapping_ty(&self) -> Ty {
        let (inputs, output) = self.uncurry();
        let domain = match inputs.as_slice() {
            [] => Ty::Unit,
            [a] => (*a).clone(),
            [a, rest @ ..] => rest
                .iter()
                .rev()
                .fold(None, |acc: Option<Ty>, t| {
                    Some(match acc {
                        None => (*t).clone(),
                        Some(acc) => Ty::prod((*t).clone(), acc),
                    })
                })
                .map(|tail| Ty::prod((*a).clone(), tail))
                .unwrap_or_else(|| (*a).clone()),
        };
        Ty::arr(domain, output.clone())
    }

    /// `a₁ → … → aₙ → b` as `([a₁, …, aₙ], b)`.
    pub fn uncurry(&self) -> (Vec<&Ty>, &Ty) {
        let mut inputs = Vec::new();
        let mut t = self;
        while let Ty::Arr { dom, cod } = t {
            inputs.push(dom.as_ref());
            t = cod;
        }
        (inputs, t)
    }

    /// `Ty.SemFree`: mentions no concept.  Required of every
    /// concept representation.
    pub fn is_sem_free(&self) -> bool {
        match self {
            Ty::Sem { .. } => false,
            Ty::Arr { dom, cod } => dom.is_sem_free() && cod.is_sem_free(),
            Ty::Opt { inner } | Ty::List { elem: inner } => inner.is_sem_free(),
            Ty::Prod { fst, snd } => fst.is_sem_free() && snd.is_sem_free(),
            Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } => true,
        }
    }

    /// `Ty.Data`: no function type inside.  Only data may be delayed.
    pub fn is_data(&self) -> bool {
        match self {
            Ty::Arr { .. } => false,
            Ty::Opt { inner } | Ty::List { elem: inner } => inner.is_data(),
            Ty::Prod { fst, snd } => fst.is_data() && snd.is_data(),
            Ty::Bool | Ty::Nat | Ty::Unit | Ty::Sem { .. } | Ty::Q { .. } => true,
        }
    }

    /// Every concept the type mentions, in traversal order.
    pub fn concepts(&self) -> Vec<ConceptId> {
        fn go(t: &Ty, out: &mut Vec<ConceptId>) {
            match t {
                Ty::Sem { id } => out.push(*id),
                Ty::Arr { dom, cod } => {
                    go(dom, out);
                    go(cod, out);
                }
                Ty::Opt { inner } | Ty::List { elem: inner } => go(inner, out),
                Ty::Prod { fst, snd } => {
                    go(fst, out);
                    go(snd, out);
                }
                Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } => {}
            }
        }
        let mut out = Vec::new();
        go(self, &mut out);
        out
    }

    /// `Ty.grant`: the concepts in result position of a signature — exactly
    /// the concepts a realization of this type may construct with `mk`.
    /// Note `opt (sem s)` grants nothing, as in the Lean definition.
    pub fn grant(&self) -> Vec<ConceptId> {
        match self {
            Ty::Sem { id } => vec![*id],
            Ty::Arr { cod, .. } => cod.grant(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `mapping f : B` and `mapping f : () -> B` have the one canonical
    /// type `() -> B`; the kernel encodes it as `B`, and the two encodings
    /// are inverse over signatures — with one, two and no inputs.
    #[test]
    fn a_relationship_without_inputs_has_the_unit_domain_and_the_encodings_are_inverse() {
        let (a, b, c) = (
            ConceptId::from_raw(1),
            ConceptId::from_raw(2),
            ConceptId::from_raw(3),
        );
        assert_eq!(Ty::of_signature(&[], c), Ty::arr(Ty::Unit, Ty::sem(c)));
        assert_eq!(Ty::kernel_of_signature(&[], c), Ty::sem(c));
        assert_eq!(Ty::of_signature(&[a], c), Ty::arr(Ty::sem(a), Ty::sem(c)));
        assert_eq!(
            Ty::of_signature(&[a, b], c),
            Ty::arr(Ty::prod(Ty::sem(a), Ty::sem(b)), Ty::sem(c))
        );
        assert_eq!(
            Ty::kernel_of_signature(&[a, b], c),
            Ty::arr(Ty::sem(a), Ty::arr(Ty::sem(b), Ty::sem(c)))
        );
        for inputs in [vec![], vec![a], vec![a, b], vec![a, b, c]] {
            assert_eq!(
                Ty::kernel_of_signature(&inputs, c).canonical_mapping_ty(),
                Ty::of_signature(&inputs, c),
                "{inputs:?}"
            );
        }
        assert!(Ty::Unit.is_data() && Ty::Unit.is_sem_free());
        assert!(Ty::Unit.grant().is_empty());
    }

    #[test]
    fn grant_follows_result_position() {
        let tilt = ConceptId::from_raw(1);
        let bright = ConceptId::from_raw(2);
        let sig = Ty::arrows([Ty::sem(tilt)], Ty::sem(bright));
        assert_eq!(sig.grant(), vec![bright]);
        assert_eq!(Ty::opt(Ty::sem(bright)).grant(), Vec::<ConceptId>::new());
    }

    #[test]
    fn predicates() {
        assert!(Ty::q(Dim::ANGLE).is_sem_free());
        assert!(!Ty::sem(ConceptId::from_raw(0)).is_sem_free());
        assert!(Ty::opt(Ty::Bool).is_data());
        assert!(!Ty::arr(Ty::Bool, Ty::Bool).is_data());
        // lists and pairs are data exactly when their parts are
        assert!(Ty::list(Ty::q(Dim::ZERO)).is_data());
        assert!(!Ty::list(Ty::arr(Ty::Bool, Ty::Bool)).is_data());
        assert!(Ty::prod(Ty::Bool, Ty::opt(Ty::Nat)).is_data());
        assert!(!Ty::prod(Ty::Bool, Ty::arr(Ty::Bool, Ty::Bool)).is_data());
        assert!(Ty::list(Ty::q(Dim::ZERO)).is_sem_free());
        assert!(!Ty::prod(Ty::Bool, Ty::sem(ConceptId::from_raw(0))).is_sem_free());
        // a pair grants nothing: only result position of a signature does
        assert_eq!(
            Ty::prod(Ty::sem(ConceptId::from_raw(1)), Ty::Bool).grant(),
            Vec::<ConceptId>::new()
        );
    }

    #[test]
    fn arrows_are_right_nested() {
        let t = Ty::arrows([Ty::Bool, Ty::Nat], Ty::Bool);
        assert_eq!(t, Ty::arr(Ty::Bool, Ty::arr(Ty::Nat, Ty::Bool)));
    }
}
