use crate::diag::Span;
use crate::lexer::token::FPart;

use super::item::{Item, Name};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    /// `x = e`, `x.f = e`, `x[i] = e`. `target` is a Name/Field/Index expr.
    Assign { target: Expr, value: Expr },
    While { cond: Expr, body: Block },
    For { var: Name, iter: Expr, body: Block },
    Loop { body: Block },
    Break,
    Continue,
    Return(Option<Expr>),
    Match { scrutinee: Expr, arms: Vec<Arm> },
    /// Nested item blocks (import trees, error blocks) appearing in a body.
    Item(Item),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: Stmt,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub name: Name,
    /// Bound names, e.g. `Num(v)` binds `[v]`.
    pub binds: Vec<Name>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Str(String),
    FStr(Vec<FPart>),
    Bool(bool),
    Null,
    Name(Name),
    SelfExpr,
    List(Vec<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: Name,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Range {
        from: Option<Box<Expr>>,
        to: Option<Box<Expr>>,
    },
    Lambda {
        params: Vec<Name>,
        body: Box<Expr>,
    },
    Comprehension {
        expr: Box<Expr>,
        var: Name,
        iter: Box<Expr>,
    },
    /// `expr!` — propagate on error.
    Propagate(Box<Expr>),
    If {
        cond: Box<Expr>,
        then: Block,
        elifs: Vec<(Expr, Block)>,
        else_: Option<Block>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// `Some` for named arguments: `Circle(radius = 2.0)`.
    pub name: Option<Name>,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}
