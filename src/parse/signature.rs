use crate::ast::item::{Name, Param};
use crate::ast::ty::TypeRepr;

use super::ParseError;
use super::util::Parser;

/// A parsed declaration signature: `name [generics] params -> ret`.
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRepr>,
}

use crate::lexer::Op;
use crate::lexer::TokKind;

/// Parse `name [G] p: T p: T -> R` (the caller has already consumed nothing).
/// TODO(task 07).
pub fn signature(p: &mut Parser<'_>) -> Result<Signature, ParseError> {
    let name = p.expect_ident()?;

    let mut generics = Vec::new();
    if p.eat_op(Op::LBracket) {
        while !p.check_op(Op::RBracket) && !p.at_eof() {
            generics.push(p.expect_ident()?);
        }
        p.expect_op(Op::RBracket)?;
    }

    let mut params = Vec::new();
    while matches!(p.peek_kind(), Some(TokKind::Ident(_))) {
        let param_name = p.expect_ident()?;
        p.expect_op(Op::Colon)?;
        let ty = super::types::type_repr(p)?;
        params.push(Param {
            name: param_name,
            ty,
        });
    }

    let mut ret = None;
    if p.eat_op(Op::Arrow) {
        ret = Some(super::types::type_repr(p)?);
    }

    Ok(Signature {
        name,
        generics,
        params,
        ret,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parse::util::Parser;

    #[test]
    fn test_signature_name_only() {
        let toks = vec![Token::ident("main")];
        let mut p = Parser::new(&toks);
        let sig = signature(&mut p).unwrap();
        assert_eq!(sig.name.text, "main");
        assert!(sig.generics.is_empty());
        assert!(sig.params.is_empty());
        assert!(sig.ret.is_none());
        assert!(p.at_eof());
    }

    #[test]
    fn test_signature_name_only_print() {
        let toks = vec![Token::ident("print")];
        let mut p = Parser::new(&toks);
        let sig = signature(&mut p).unwrap();
        assert_eq!(sig.name.text, "print");
        assert!(sig.generics.is_empty());
        assert!(sig.params.is_empty());
        assert!(sig.ret.is_none());
        assert!(p.at_eof());
    }

    #[test]
    fn test_signature_generics_only() {
        let toks = vec![
            Token::ident("foo"),
            Token::op(Op::LBracket),
            Token::ident("T"),
            Token::op(Op::RBracket),
        ];
        let mut p = Parser::new(&toks);
        let sig = signature(&mut p).unwrap();
        assert_eq!(sig.name.text, "foo");
        assert_eq!(sig.generics.len(), 1);
        assert_eq!(sig.generics[0].text, "T");
        assert!(sig.params.is_empty());
        assert!(sig.ret.is_none());
        assert!(p.at_eof());
    }

    #[test]
    fn test_signature_multiple_generics() {
        let toks = vec![
            Token::ident("foo"),
            Token::op(Op::LBracket),
            Token::ident("T"),
            Token::ident("U"),
            Token::op(Op::RBracket),
        ];
        let mut p = Parser::new(&toks);
        let sig = signature(&mut p).unwrap();
        assert_eq!(sig.name.text, "foo");
        assert_eq!(sig.generics.len(), 2);
        assert_eq!(sig.generics[0].text, "T");
        assert_eq!(sig.generics[1].text, "U");
        assert!(sig.params.is_empty());
        assert!(sig.ret.is_none());
        assert!(p.at_eof());
    }

    #[test]
    fn test_signature_missing_colon() {
        let toks = vec![Token::ident("add"), Token::ident("a")];
        let mut p = Parser::new(&toks);
        let err = signature(&mut p).unwrap_err();
        assert!(err.message.contains("expected Colon"));
    }
}
