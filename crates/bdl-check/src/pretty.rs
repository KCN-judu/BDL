//! Product-language rendering of types for diagnostics, and the kernel
//! rendering for the technical line.

use bdl_ir::{DesignIr, Ty};
use bdl_model::Dim;

/// "Brightness", "an angle", "a dimensionless level", "true or false",
/// "a count", "Tilt → Brightness".
pub fn describe(ir: &DesignIr, ty: &Ty) -> String {
    match ty {
        Ty::Bool => "true or false".into(),
        Ty::Nat => "a count".into(),
        Ty::Unit => "()".into(),
        Ty::Sem { id } => ir
            .concepts
            .get(id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("concept {id}")),
        Ty::Q { dim } => describe_dim(*dim),
        Ty::Opt { inner } => format!("an occurrence of {}", describe(ir, inner)),
        Ty::List { elem } => format!("a collection of {}", describe(ir, elem)),
        Ty::Prod { fst, snd } => {
            format!(
                "a grouped value ({} and {})",
                describe(ir, fst),
                describe(ir, snd)
            )
        }
        Ty::Arr { .. } => {
            let mut parts = Vec::new();
            let mut t = ty;
            while let Ty::Arr { dom, cod } = t {
                parts.push(describe(ir, dom));
                t = cod;
            }
            format!("({}) → {}", parts.join(", "), describe(ir, t))
        }
    }
}

/// A relationship's canonical type in the designer's spelling — `() ->
/// RoomTemp`, `Angle -> Brightness`, `(Angle, Time) -> Speed` — from its
/// signature (`bdl_ir::ty`: the domain of no inputs is `()`, the empty
/// product; never the word "unit", which names a measurement unit here).
pub fn mapping_type(
    ir: &DesignIr,
    inputs: &[bdl_model::SemanticId],
    output: bdl_model::SemanticId,
) -> String {
    let name = |id: &bdl_model::SemanticId| {
        ir.concepts
            .get(id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("concept {id}"))
    };
    let domain = match inputs {
        [] => "()".to_string(),
        [a] => name(a),
        many => format!("({})", many.iter().map(name).collect::<Vec<_>>().join(", ")),
    };
    format!("{domain} -> {}", name(&output))
}

/// "an angle", "a length", "a dimensionless quantity", "a quantity of m·s⁻²".
pub fn describe_dim(dim: Dim) -> String {
    let named = [
        (Dim::ZERO, "a dimensionless quantity"),
        (Dim::ANGLE, "an angle"),
        (Dim::LENGTH, "a length"),
        (Dim::TIME, "a time"),
        (Dim::MASS, "a mass"),
        (Dim::TEMPERATURE, "a temperature"),
        (Dim::CURRENT, "a current"),
        (Dim::ANGLE - Dim::TIME, "an angular rate"),
        (Dim::LENGTH - Dim::TIME, "a speed"),
    ];
    for (d, name) in named {
        if d == dim {
            return name.into();
        }
    }
    format!("a quantity of {}", symbol(dim))
}

/// `m·s^-2`-style symbol.
pub fn symbol(d: Dim) -> String {
    let mut parts = Vec::new();
    for (sym, e) in [
        ("m", d.length),
        ("kg", d.mass),
        ("s", d.time),
        ("A", d.current),
        ("K", d.temperature),
        ("mol", d.amount),
        ("cd", d.luminous),
        ("rad", d.angle),
    ] {
        if e == 1 {
            parts.push(sym.to_string());
        } else if e != 0 {
            parts.push(format!("{sym}^{e}"));
        }
    }
    if parts.is_empty() {
        "1".into()
    } else {
        parts.join("·")
    }
}

/// Kernel notation: `sem#1`, `q[rad]`, `bool`, `nat`, `q[rad] → sem#1`.
pub fn kernel(ty: &Ty) -> String {
    match ty {
        Ty::Bool => "bool".into(),
        Ty::Nat => "nat".into(),
        Ty::Unit => "()".into(),
        Ty::Sem { id } => id.to_string(),
        Ty::Q { dim } => format!("q[{}]", symbol(*dim)),
        Ty::Opt { inner } => format!("opt {}", kernel(inner)),
        Ty::List { elem } => format!("list {}", kernel(elem)),
        Ty::Prod { fst, snd } => format!("({} × {})", kernel(fst), kernel(snd)),
        Ty::Arr { dom, cod } => {
            let d = match **dom {
                Ty::Arr { .. } => format!("({})", kernel(dom)),
                _ => kernel(dom),
            };
            format!("{d} → {}", kernel(cod))
        }
    }
}

/// Kernel notation for a Core term, for the explanation view:
/// `λ(sem#0). mk sem#1 (div[rad,rad] (rep #0) 1.5708[rad])`.
pub fn expr(e: &bdl_ir::Expr) -> String {
    use bdl_ir::{Expr, Prim};
    fn prim(p: &Prim) -> String {
        match p {
            Prim::Lit { dim, value } => format!("{}[{}]", value.0, symbol(*dim)),
            Prim::Add { dim } => format!("add[{}]", symbol(*dim)),
            Prim::Sub { dim } => format!("sub[{}]", symbol(*dim)),
            Prim::Mul { d1, d2 } => format!("mul[{},{}]", symbol(*d1), symbol(*d2)),
            Prim::Div { d1, d2 } => format!("div[{},{}]", symbol(*d1), symbol(*d2)),
            Prim::Lt { dim } => format!("lt[{}]", symbol(*dim)),
            Prim::Eq { ty } => format!("eq[{}]", kernel(ty)),
            Prim::Not => "not".into(),
            Prim::And => "and".into(),
            Prim::Or => "or".into(),
            Prim::Ite { ty } => format!("ite[{}]", kernel(ty)),
            Prim::None { ty } => format!("none[{}]", kernel(ty)),
            Prim::Some { ty } => format!("some[{}]", kernel(ty)),
            Prim::IsSome { ty } => format!("isSome[{}]", kernel(ty)),
            Prim::GetD { ty } => format!("getD[{}]", kernel(ty)),
            Prim::Nil { ty } => format!("nil[{}]", kernel(ty)),
            Prim::Cons { ty } => format!("cons[{}]", kernel(ty)),
            Prim::Length { ty } => format!("length[{}]", kernel(ty)),
            Prim::Take { ty } => format!("take[{}]", kernel(ty)),
            Prim::Drop { ty } => format!("drop[{}]", kernel(ty)),
            Prim::Reverse { ty } => format!("reverse[{}]", kernel(ty)),
            Prim::Head { ty } => format!("head[{}]", kernel(ty)),
            Prim::ToList { ty } => format!("toList[{}]", kernel(ty)),
            Prim::Pair { fst, snd } => format!("pair[{},{}]", kernel(fst), kernel(snd)),
            Prim::Fst { fst, snd } => format!("fst[{},{}]", kernel(fst), kernel(snd)),
            Prim::Snd { fst, snd } => format!("snd[{},{}]", kernel(fst), kernel(snd)),
        }
    }
    fn go(e: &Expr, out: &mut String) {
        match e {
            Expr::Var { index } => out.push_str(&format!("#{index}")),
            Expr::BoolLit { value } => out.push_str(&value.to_string()),
            Expr::NatLit { value } => out.push_str(&value.to_string()),
            Expr::Lam { dom, body } => {
                out.push_str(&format!("λ({}). ", kernel(dom)));
                go(body, out);
            }
            Expr::App { .. } => {
                // flatten f a b c
                let mut args = Vec::new();
                let mut f = e;
                while let Expr::App { f: g, a } = f {
                    args.push(a);
                    f = g;
                }
                args.reverse();
                out.push('(');
                go(f, out);
                for a in args {
                    out.push(' ');
                    go(a, out);
                }
                out.push(')');
            }
            Expr::DeclRef { id } => out.push_str(&id.to_string()),
            Expr::Rep { e } => {
                out.push_str("(rep ");
                go(e, out);
                out.push(')');
            }
            Expr::Mk { s, e } => {
                out.push_str(&format!("(mk {s} "));
                go(e, out);
                out.push(')');
            }
            Expr::Prim { p } => out.push_str(&prim(p)),
            Expr::Delay { init, e } => {
                out.push_str("(delay ");
                go(init, out);
                out.push(' ');
                go(e, out);
                out.push(')');
            }
            Expr::Sync { src, init, e } => {
                out.push_str(&format!("(sync {src} "));
                go(init, out);
                out.push(' ');
                go(e, out);
                out.push(')');
            }
            Expr::Fold { f, z, l } => {
                out.push_str("(fold ");
                go(f, out);
                out.push(' ');
                go(z, out);
                out.push(' ');
                go(l, out);
                out.push(')');
            }
        }
    }
    let mut s = String::new();
    go(e, &mut s);
    s
}
