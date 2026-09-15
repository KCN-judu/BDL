//! The subset of Rust the backend generates, as owned data.  No parser, no
//! macro hygiene, no generics beyond `Option<T>`: exactly what a lowered
//! design needs.  Printing is [`crate::print`].

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    /// Leading `//!` lines.
    pub doc: Vec<String>,
    /// Inner attributes without the `#![…]`, e.g. `no_std`.
    pub inner_attrs: Vec<String>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A `//` comment line (or several) between items.
    Comment(Vec<String>),
    Use(String),
    Const {
        doc: Vec<String>,
        name: String,
        ty: Type,
        value: Expr,
    },
    Struct {
        doc: Vec<String>,
        derives: Vec<String>,
        name: String,
        fields: Fields,
    },
    Fn(Function),
    Impl {
        trait_: Option<String>,
        target: Type,
        items: Vec<ImplItem>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fields {
    Named(Vec<(String, Type)>),
    Tuple(Vec<Type>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub doc: Vec<String>,
    pub attrs: Vec<String>,
    pub public: bool,
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Option<Type>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImplItem {
    Type(String, Type),
    Const(String, Type, Expr),
    Fn(Function),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

impl Block {
    pub fn expr(e: Expr) -> Block {
        Block {
            stmts: Vec::new(),
            tail: Some(Box::new(e)),
        }
    }
    pub fn new(stmts: Vec<Stmt>, tail: Option<Expr>) -> Block {
        Block {
            stmts,
            tail: tail.map(Box::new),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Comment(String),
    Let {
        name: String,
        mutable: bool,
        ty: Option<Type>,
        value: Expr,
    },
    /// An expression statement (`expr;`); `if` blocks print without `;`.
    Expr(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Path(String),
    Option(Box<Type>),
    Ref { mutable: bool, inner: Box<Type> },
    Tuple(Vec<Type>),
}

impl Type {
    pub fn path(p: impl Into<String>) -> Type {
        Type::Path(p.into())
    }
    pub fn option(t: Type) -> Type {
        Type::Option(Box::new(t))
    }
    pub fn reference(t: Type, mutable: bool) -> Type {
        Type::Ref {
            mutable,
            inner: Box::new(t),
        }
    }
    pub fn unit() -> Type {
        Type::Tuple(Vec::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    Bool(bool),
    /// Printed with the shortest round-tripping representation and an
    /// `_f64` suffix; never NaN or ±∞ (the elaborator rejects those).
    F64(f64),
    U64(u64),
    U16(u16),
    Usize(usize),
    Str(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinOp {
    Lt,
    Eq,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Path(String),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        recv: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: String,
    },
    Binary {
        op: BinOp,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    Not(Box<Expr>),
    /// `expr?`
    Try(Box<Expr>),
    Ref {
        mutable: bool,
        e: Box<Expr>,
    },
    Struct {
        path: String,
        fields: Vec<(String, Expr)>,
    },
    Tuple(Vec<Expr>),
    If {
        cond: Box<Expr>,
        then: Block,
        else_: Option<Block>,
    },
    /// `match scrutinee { pat => expr, … }` with textual patterns.
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<(String, Expr)>,
    },
    Block(Block),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    /// `value as Type`
    Cast {
        e: Box<Expr>,
        ty: Type,
    },
    /// `vec![…]` — host bridge only, never in the core.
    VecMacro(Vec<Expr>),
    /// A closure with one parameter: `|name| body`.
    Closure {
        param: String,
        body: Box<Expr>,
    },
}

impl Expr {
    pub fn path(p: impl Into<String>) -> Expr {
        Expr::Path(p.into())
    }
    pub fn call(func: impl Into<String>, args: impl IntoIterator<Item = Expr>) -> Expr {
        Expr::Call {
            func: Box::new(Expr::Path(func.into())),
            args: args.into_iter().collect(),
        }
    }
    pub fn method(
        recv: Expr,
        method: impl Into<String>,
        args: impl IntoIterator<Item = Expr>,
    ) -> Expr {
        Expr::MethodCall {
            recv: Box::new(recv),
            method: method.into(),
            args: args.into_iter().collect(),
        }
    }
    pub fn field(base: Expr, name: impl Into<String>) -> Expr {
        Expr::Field {
            base: Box::new(base),
            name: name.into(),
        }
    }
    pub fn try_(e: Expr) -> Expr {
        Expr::Try(Box::new(e))
    }
    pub fn f64(v: f64) -> Expr {
        Expr::Lit(Lit::F64(v))
    }
    pub fn u64(v: u64) -> Expr {
        Expr::Lit(Lit::U64(v))
    }
    pub fn bool(v: bool) -> Expr {
        Expr::Lit(Lit::Bool(v))
    }
    pub fn str(s: impl Into<String>) -> Expr {
        Expr::Lit(Lit::Str(s.into()))
    }
    pub fn some(e: Expr) -> Expr {
        Expr::call("Some", [e])
    }
    pub fn none() -> Expr {
        Expr::path("None")
    }
    pub fn assign(target: Expr, value: Expr) -> Expr {
        Expr::Assign {
            target: Box::new(target),
            value: Box::new(value),
        }
    }
    pub fn if_(cond: Expr, then: Block, else_: Option<Block>) -> Expr {
        Expr::If {
            cond: Box::new(cond),
            then,
            else_,
        }
    }
    pub fn if_else(cond: Expr, then: Expr, else_: Expr) -> Expr {
        Expr::if_(cond, Block::expr(then), Some(Block::expr(else_)))
    }
    pub fn strukt(
        path: impl Into<String>,
        fields: impl IntoIterator<Item = (String, Expr)>,
    ) -> Expr {
        Expr::Struct {
            path: path.into(),
            fields: fields.into_iter().collect(),
        }
    }
}
