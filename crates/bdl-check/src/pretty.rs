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
        Ty::Sem { id } => ir
            .concepts
            .get(id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("concept {id}")),
        Ty::Q { dim } => describe_dim(*dim),
        Ty::Opt { inner } => format!("an occurrence of {}", describe(ir, inner)),
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
        Ty::Sem { id } => id.to_string(),
        Ty::Q { dim } => format!("q[{}]", symbol(*dim)),
        Ty::Opt { inner } => format!("opt {}", kernel(inner)),
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
            Prim::Eq { dim } => format!("eq[{}]", symbol(*dim)),
            Prim::Not => "not".into(),
            Prim::And => "and".into(),
            Prim::Or => "or".into(),
            Prim::Ite { ty } => format!("ite[{}]", kernel(ty)),
            Prim::None { ty } => format!("none[{}]", kernel(ty)),
            Prim::Some { ty } => format!("some[{}]", kernel(ty)),
            Prim::IsSome { ty } => format!("isSome[{}]", kernel(ty)),
            Prim::GetD { ty } => format!("getD[{}]", kernel(ty)),
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
        }
    }
    let mut s = String::new();
    go(e, &mut s);
    s
}
