//! The kernel's types (`Base.lean`).

use bdl_model::{Dim, SemanticId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "ty", rename_all = "snake_case")]
pub enum Ty {
    Bool,
    Nat,
    Arr {
        dom: Box<Ty>,
        cod: Box<Ty>,
    },
    /// Nominal semantic type: two distinct ids are distinct types regardless
    /// of representation.
    Sem {
        id: SemanticId,
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
    pub fn sem(id: SemanticId) -> Ty {
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

    /// `Ty.SemFree`: mentions no semantic concept.  Required of every
    /// concept representation.
    pub fn is_sem_free(&self) -> bool {
        match self {
            Ty::Sem { .. } => false,
            Ty::Arr { dom, cod } => dom.is_sem_free() && cod.is_sem_free(),
            Ty::Opt { inner } | Ty::List { elem: inner } => inner.is_sem_free(),
            Ty::Prod { fst, snd } => fst.is_sem_free() && snd.is_sem_free(),
            Ty::Bool | Ty::Nat | Ty::Q { .. } => true,
        }
    }

    /// `Ty.Data`: no function type inside.  Only data may be delayed.
    pub fn is_data(&self) -> bool {
        match self {
            Ty::Arr { .. } => false,
            Ty::Opt { inner } | Ty::List { elem: inner } => inner.is_data(),
            Ty::Prod { fst, snd } => fst.is_data() && snd.is_data(),
            Ty::Bool | Ty::Nat | Ty::Sem { .. } | Ty::Q { .. } => true,
        }
    }

    /// Every semantic concept the type mentions, in traversal order.
    pub fn concepts(&self) -> Vec<SemanticId> {
        fn go(t: &Ty, out: &mut Vec<SemanticId>) {
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
                Ty::Bool | Ty::Nat | Ty::Q { .. } => {}
            }
        }
        let mut out = Vec::new();
        go(self, &mut out);
        out
    }

    /// `Ty.grant`: the concepts in result position of a signature — exactly
    /// the concepts a realization of this type may construct with `mk`.
    /// Note `opt (sem s)` grants nothing, as in the Lean definition.
    pub fn grant(&self) -> Vec<SemanticId> {
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

    #[test]
    fn grant_follows_result_position() {
        let tilt = SemanticId::from_raw(1);
        let bright = SemanticId::from_raw(2);
        let sig = Ty::arrows([Ty::sem(tilt)], Ty::sem(bright));
        assert_eq!(sig.grant(), vec![bright]);
        assert_eq!(Ty::opt(Ty::sem(bright)).grant(), Vec::<SemanticId>::new());
    }

    #[test]
    fn predicates() {
        assert!(Ty::q(Dim::ANGLE).is_sem_free());
        assert!(!Ty::sem(SemanticId::from_raw(0)).is_sem_free());
        assert!(Ty::opt(Ty::Bool).is_data());
        assert!(!Ty::arr(Ty::Bool, Ty::Bool).is_data());
        // lists and pairs are data exactly when their parts are
        assert!(Ty::list(Ty::q(Dim::ZERO)).is_data());
        assert!(!Ty::list(Ty::arr(Ty::Bool, Ty::Bool)).is_data());
        assert!(Ty::prod(Ty::Bool, Ty::opt(Ty::Nat)).is_data());
        assert!(!Ty::prod(Ty::Bool, Ty::arr(Ty::Bool, Ty::Bool)).is_data());
        assert!(Ty::list(Ty::q(Dim::ZERO)).is_sem_free());
        assert!(!Ty::prod(Ty::Bool, Ty::sem(SemanticId::from_raw(0))).is_sem_free());
        // a pair grants nothing: only result position of a signature does
        assert_eq!(
            Ty::prod(Ty::sem(SemanticId::from_raw(1)), Ty::Bool).grant(),
            Vec::<SemanticId>::new()
        );
    }

    #[test]
    fn arrows_are_right_nested() {
        let t = Ty::arrows([Ty::Bool, Ty::Nat], Ty::Bool);
        assert_eq!(t, Ty::arr(Ty::Bool, Ty::arr(Ty::Nat, Ty::Bool)));
    }
}
