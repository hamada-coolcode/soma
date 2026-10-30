use crate::diag::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Local,
    Param,
    Field,
    Function,
    Type,
    Module,
    Const,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    /// `false` for parameters: assigning to them is an error.
    pub mutable: bool,
}
