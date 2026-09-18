//! Deterministic printer for [`crate::ast`]: four-space indentation, one
//! item per blank-line-separated block, every compound sub-expression
//! parenthesised so precedence never depends on the printer's judgment.
//! The same AST always prints the same bytes.

use crate::ast::*;

pub fn module(m: &Module) -> String {
    let mut p = Printer::default();
    for d in &m.doc {
        if d.is_empty() {
            p.line("//!");
        } else {
            p.line(&format!("//! {d}"));
        }
    }
    if !m.doc.is_empty() {
        p.blank();
    }
    for a in &m.inner_attrs {
        p.line(&format!("#![{a}]"));
    }
    if !m.inner_attrs.is_empty() {
        p.blank();
    }
    let mut first = true;
    for it in &m.items {
        let is_use = matches!(it, Item::Use(_) | Item::ExternCrate(_));
        if !(first || (is_use && p.last_was_use)) {
            p.blank();
        }
        first = false;
        p.item(it);
        p.last_was_use = is_use;
    }
    p.out
}

#[derive(Default)]
struct Printer {
    out: String,
    indent: usize,
    last_was_use: bool,
}

impl Printer {
    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }
    fn blank(&mut self) {
        self.out.push('\n');
    }
    fn docs(&mut self, doc: &[String]) {
        for d in doc {
            self.line(&format!("/// {d}"));
        }
    }

    fn item(&mut self, it: &Item) {
        match it {
            Item::Comment(lines) => {
                for l in lines {
                    self.line(&format!("// {l}"));
                }
            }
            Item::ExternCrate(c) => self.line(&format!("extern crate {c};")),
            Item::Use(p) => self.line(&format!("use {p};")),
            Item::Const {
                doc,
                name,
                ty,
                value,
            } => {
                self.docs(doc);
                self.line(&format!(
                    "pub const {name}: {} = {};",
                    ty_str(ty),
                    expr_str(value)
                ));
            }
            Item::Struct {
                doc,
                derives,
                name,
                fields,
            } => {
                self.docs(doc);
                if !derives.is_empty() {
                    self.line(&format!("#[derive({})]", derives.join(", ")));
                }
                match fields {
                    Fields::Named(fs) if fs.is_empty() => {
                        self.line(&format!("pub struct {name} {{}}"))
                    }
                    Fields::Named(fs) => {
                        self.line(&format!("pub struct {name} {{"));
                        self.indent += 1;
                        for (f, t) in fs {
                            self.line(&format!("pub {f}: {},", ty_str(t)));
                        }
                        self.indent -= 1;
                        self.line("}");
                    }
                    Fields::Tuple(ts) => {
                        let inner: Vec<String> =
                            ts.iter().map(|t| format!("pub {}", ty_str(t))).collect();
                        self.line(&format!("pub struct {name}({});", inner.join(", ")));
                    }
                }
            }
            Item::Fn(f) => self.function(f, false),
            Item::Impl {
                trait_,
                target,
                items,
            } => {
                match trait_ {
                    Some(t) => self.line(&format!("impl {t} for {} {{", ty_str(target))),
                    None => self.line(&format!("impl {} {{", ty_str(target))),
                }
                self.indent += 1;
                let mut first = true;
                for it in items {
                    if !first {
                        self.blank();
                    }
                    first = false;
                    match it {
                        ImplItem::Type(n, t) => self.line(&format!("type {n} = {};", ty_str(t))),
                        ImplItem::Const(n, t, e) => {
                            self.line(&format!("const {n}: {} = {};", ty_str(t), expr_str(e)))
                        }
                        ImplItem::Fn(f) => self.function(f, true),
                    }
                }
                self.indent -= 1;
                self.line("}");
            }
        }
    }

    fn function(&mut self, f: &Function, in_impl: bool) {
        self.docs(&f.doc);
        for a in &f.attrs {
            self.line(&format!("#[{a}]"));
        }
        let vis = if f.public && !in_impl { "pub " } else { "" };
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {}", ty_str(t)))
            .collect();
        let ret = match &f.ret {
            Some(t) => format!(" -> {}", ty_str(t)),
            None => String::new(),
        };
        self.line(&format!(
            "{vis}fn {}({}){ret} {{",
            f.name,
            params.join(", ")
        ));
        self.indent += 1;
        self.block_body(&f.body);
        self.indent -= 1;
        self.line("}");
    }

    fn block_body(&mut self, b: &Block) {
        for s in &b.stmts {
            match s {
                Stmt::Comment(c) => self.line(&format!("// {c}")),
                Stmt::Let {
                    name,
                    mutable,
                    ty,
                    value,
                } => {
                    let m = if *mutable { "mut " } else { "" };
                    let t = match ty {
                        Some(t) => format!(": {}", ty_str(t)),
                        None => String::new(),
                    };
                    self.line(&format!("let {m}{name}{t} = {};", expr_str(value)));
                }
                Stmt::Expr(e) => {
                    let s = expr_str(e);
                    if matches!(e, Expr::If { .. } | Expr::Match { .. } | Expr::Block(_)) {
                        self.line(&s);
                    } else {
                        self.line(&format!("{s};"));
                    }
                }
            }
        }
        if let Some(t) = &b.tail {
            let s = expr_str(t);
            self.line(&s);
        }
    }
}

pub fn ty_str(t: &Type) -> String {
    match t {
        Type::Path(p) => p.clone(),
        Type::Option(i) => format!("Option<{}>", ty_str(i)),
        Type::Vec(i) => format!("Vec<{}>", ty_str(i)),
        Type::Ref { mutable, inner } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, ty_str(inner))
        }
        Type::Tuple(ts) => format!("({})", ts.iter().map(ty_str).collect::<Vec<_>>().join(", ")),
    }
}

fn lit_str(l: &Lit) -> String {
    match l {
        Lit::Bool(b) => b.to_string(),
        Lit::F64(v) => {
            let s = format!("{v:?}");
            if v.is_sign_negative() {
                format!("({s}_f64)")
            } else {
                format!("{s}_f64")
            }
        }
        Lit::U64(v) => format!("{v}_u64"),
        Lit::U16(v) => format!("{v}_u16"),
        Lit::Usize(v) => format!("{v}_usize"),
        Lit::Str(s) => format!("{s:?}"),
    }
}

/// Single-line rendering of an expression; blocks are rendered inline
/// with `{ … }` and `;`-separated statements.
pub fn expr_str(e: &Expr) -> String {
    match e {
        Expr::Lit(l) => lit_str(l),
        Expr::Path(p) => p.clone(),
        Expr::Call { func, args } => format!("{}({})", expr_str(func), args_str(args)),
        Expr::MethodCall { recv, method, args } => {
            format!("{}.{method}({})", paren(recv), args_str(args))
        }
        Expr::Field { base, name } => format!("{}.{name}", paren(base)),
        Expr::Binary { op, l, r } => {
            let o = match op {
                BinOp::Lt => "<",
                BinOp::Eq => "==",
            };
            format!("({} {o} {})", expr_str(l), expr_str(r))
        }
        Expr::Not(x) => format!("(!{})", expr_str(x)),
        Expr::Try(x) => format!("{}?", paren(x)),
        Expr::Ref { mutable, e } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, expr_str(e))
        }
        Expr::Struct { path, fields } => {
            if fields.is_empty() {
                format!("{path} {{}}")
            } else {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(n, v)| format!("{n}: {}", expr_str(v)))
                    .collect();
                format!("{path} {{ {} }}", fs.join(", "))
            }
        }
        Expr::Tuple(es) => format!("({})", args_str(es)),
        Expr::If { cond, then, else_ } => {
            let mut s = format!("if {} {}", expr_str(cond), block_str(then));
            if let Some(b) = else_ {
                s.push_str(" else ");
                s.push_str(&block_str(b));
            }
            s
        }
        Expr::Match { scrutinee, arms } => {
            let arms: Vec<String> = arms
                .iter()
                .map(|(p, e)| format!("{p} => {}", expr_str(e)))
                .collect();
            format!("match {} {{ {} }}", expr_str(scrutinee), arms.join(", "))
        }
        Expr::Block(b) => block_str(b),
        Expr::Assign { target, value } => format!("{} = {}", expr_str(target), expr_str(value)),
        Expr::Cast { e, ty } => format!("({} as {})", expr_str(e), ty_str(ty)),
        Expr::VecMacro(es) => format!("vec![{}]", args_str(es)),
        Expr::Closure { params, ret, body } => match ret {
            None => format!("|{}| {}", params.join(", "), expr_str(body)),
            Some(t) => format!(
                "|{}| -> {} {{ {} }}",
                params.join(", "),
                ty_str(t),
                expr_str(body)
            ),
        },
    }
}

fn args_str(args: &[Expr]) -> String {
    args.iter().map(expr_str).collect::<Vec<_>>().join(", ")
}

/// Wrap in parentheses unless atomic, for use as a receiver / base.
fn paren(e: &Expr) -> String {
    match e {
        Expr::Path(_)
        | Expr::Call { .. }
        | Expr::MethodCall { .. }
        | Expr::Field { .. }
        | Expr::Try(_) => expr_str(e),
        _ => format!("({})", expr_str(e)),
    }
}

fn block_str(b: &Block) -> String {
    let mut parts: Vec<String> = Vec::new();
    for s in &b.stmts {
        match s {
            Stmt::Comment(_) => {}
            Stmt::Let {
                name,
                mutable,
                ty,
                value,
            } => {
                let m = if *mutable { "mut " } else { "" };
                let t = match ty {
                    Some(t) => format!(": {}", ty_str(t)),
                    None => String::new(),
                };
                parts.push(format!("let {m}{name}{t} = {};", expr_str(value)));
            }
            Stmt::Expr(e) => parts.push(format!("{};", expr_str(e))),
        }
    }
    if let Some(t) = &b.tail {
        parts.push(expr_str(t));
    }
    if parts.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_round_trip_and_negatives_are_parenthesised() {
        assert_eq!(expr_str(&Expr::f64(0.1)), "0.1_f64");
        assert_eq!(expr_str(&Expr::f64(1e308)), "1e308_f64");
        assert_eq!(expr_str(&Expr::f64(-2.5)), "(-2.5_f64)");
        assert_eq!(expr_str(&Expr::f64(-0.0)), "(-0.0_f64)");
        assert_eq!(expr_str(&Expr::f64(3.0)), "3.0_f64");
        assert_eq!(expr_str(&Expr::u64(7)), "7_u64");
        assert_eq!(expr_str(&Expr::str("a\"b")), "\"a\\\"b\"");
    }

    #[test]
    fn compound_expressions_are_fully_parenthesised() {
        let e = Expr::Binary {
            op: BinOp::Lt,
            l: Box::new(Expr::try_(Expr::call(
                "num::add",
                [Expr::f64(1.0), Expr::path("x"), Expr::u64(0)],
            ))),
            r: Box::new(Expr::Not(Box::new(Expr::path("b")))),
        };
        assert_eq!(expr_str(&e), "(num::add(1.0_f64, x, 0_u64)? < (!b))");
        let m = Expr::Match {
            scrutinee: Box::new(Expr::field(Expr::path("prev"), "cell_0")),
            arms: vec![
                ("Some(v)".into(), Expr::path("v")),
                ("None".into(), Expr::f64(0.0)),
            ],
        };
        assert_eq!(
            expr_str(&m),
            "match prev.cell_0 { Some(v) => v, None => 0.0_f64 }"
        );
        let f = Expr::field(Expr::Block(Block::expr(Expr::path("x"))), "0");
        assert_eq!(expr_str(&f), "({ x }).0");
    }

    #[test]
    fn modules_print_deterministically() {
        let m = Module {
            doc: vec!["hello".into(), String::new()],
            inner_attrs: vec!["no_std".into()],
            items: vec![
                Item::Use("a::b".into()),
                Item::Use("a::c".into()),
                Item::Struct {
                    doc: vec![],
                    derives: vec!["Clone".into()],
                    name: "S".into(),
                    fields: Fields::Named(vec![("x".into(), Type::option(Type::path("f64")))]),
                },
                Item::Fn(Function {
                    doc: vec![],
                    attrs: vec![],
                    public: true,
                    name: "f".into(),
                    params: vec![("s".into(), Type::reference(Type::path("S"), true))],
                    ret: Some(Type::path("f64")),
                    body: Block::new(
                        vec![Stmt::Let {
                            name: "y".into(),
                            mutable: false,
                            ty: None,
                            value: Expr::f64(1.0),
                        }],
                        Some(Expr::path("y")),
                    ),
                }),
            ],
        };
        let a = module(&m);
        assert_eq!(a, module(&m));
        assert_eq!(
            a,
            "//! hello\n//!\n\n#![no_std]\n\nuse a::b;\nuse a::c;\n\n#[derive(Clone)]\npub struct S {\n    pub x: Option<f64>,\n}\n\npub fn f(s: &mut S) -> f64 {\n    let y = 1.0_f64;\n    y\n}\n"
        );
    }
}
