use crate::diag::Span;

use super::expr::{Block, Expr};
use super::ty::TypeRepr;

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

impl Name {
    pub fn new(text: impl Into<String>, span: Span) -> Self {
        Self {
            text: text.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub attrs: Vec<Attribute>,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Fn(FnDef),
    Object(ObjectDef),
    Trait(TraitDef),
    Error(ErrorDef),
    Enum(EnumDef),
    Import(ImportDef),
    Const(ConstDef),
    Test(TestDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRepr>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Name,
    pub ty: TypeRepr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: Name,
    pub ty: TypeRepr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectDef {
    pub name: Name,
    pub generics: Vec<Name>,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<FnDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRepr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: Name,
    pub methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorDef {
    pub name: Name,
    pub variants: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Name,
    pub payload: Vec<TypeRepr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: Name,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDef {
    pub path: Vec<Name>,
    pub children: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    pub name: Name,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDef {
    pub name: String,
    pub body: Block,
}
