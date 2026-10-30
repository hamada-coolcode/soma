use crate::ast::expr::{Expr, ExprKind};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::lexer::{Op, TokKind};

use super::ParseError;
use super::util::Parser;

/// Parse a type: `int`, `T[]`, `int[10]`, `T?`, `(int) int`, `Stack[T]`,
/// `MathError!int`, or bare `!`.
pub fn type_repr(p: &mut Parser<'_>) -> Result<TypeRepr, ParseError> {
    parse_error_union(p)
}

fn parse_error_union(p: &mut Parser<'_>) -> Result<TypeRepr, ParseError> {
    if p.check_op(Op::Bang) {
        let span = p.expect_op(Op::Bang)?;
        return Ok(TypeRepr {
            kind: TypeKind::Never,
            span,
        });
    }

    let mut left = parse_postfix(p)?;

    if p.eat_op(Op::Bang) {
        let ok = parse_postfix(p)?;
        let span = left.span.join(ok.span);
        left = TypeRepr {
            kind: TypeKind::ErrorUnion {
                err: Box::new(left),
                ok: Box::new(ok),
            },
            span,
        };
    }

    Ok(left)
}

fn parse_postfix(p: &mut Parser<'_>) -> Result<TypeRepr, ParseError> {
    let mut ty = parse_primary(p)?;

    loop {
        if p.check_op(Op::Question) {
            let span = p.expect_op(Op::Question)?;
            ty = TypeRepr {
                kind: TypeKind::Optional(Box::new(ty.clone())),
                span: ty.span.join(span),
            };
        } else if p.check_op(Op::LBracket) {
            let _lbracket_span = p.expect_op(Op::LBracket)?;
            let mut size = None;
            if let Some(TokKind::Int(v)) = p.peek_kind() {
                let v = *v;
                let span = p.peek_span();
                p.bump();
                size = Some(Expr {
                    kind: ExprKind::Int(v),
                    span,
                });
            }
            let rbracket_span = p.expect_op(Op::RBracket)?;
            ty = TypeRepr {
                kind: TypeKind::Array {
                    elem: Box::new(ty.clone()),
                    size,
                },
                span: ty.span.join(rbracket_span),
            };
        } else {
            break;
        }
    }

    Ok(ty)
}

fn parse_primary(p: &mut Parser<'_>) -> Result<TypeRepr, ParseError> {
    if p.check_op(Op::LParen) {
        let lparen_span = p.expect_op(Op::LParen)?;
        let mut params = Vec::new();
        while !p.check_op(Op::RParen) && !p.at_eof() {
            params.push(type_repr(p)?);
        }
        let _rparen_span = p.expect_op(Op::RParen)?;
        let ret = type_repr(p)?;
        let span = lparen_span.join(ret.span);
        return Ok(TypeRepr {
            kind: TypeKind::Fn {
                params,
                ret: Box::new(ret),
            },
            span,
        });
    }

    let name = p.expect_ident()?;
    let mut generics = Vec::new();
    let mut span = name.span;

    // Distinguish between T[] (array) and Stack[T] (generics)
    // If the next token after [ is ] or an int, it's an array, so we don't parse generics here.
    if p.check_op(Op::LBracket) {
        let is_generics = {
            if p.pos + 1 < p.toks.len() {
                let next_kind = &p.toks[p.pos + 1].kind;
                match next_kind {
                    TokKind::Op(Op::RBracket) => false,
                    TokKind::Int(_) => false,
                    _ => true,
                }
            } else {
                false
            }
        };

        if is_generics {
            p.expect_op(Op::LBracket)?;
            while !p.check_op(Op::RBracket) && !p.at_eof() {
                generics.push(type_repr(p)?);
            }
            let rbracket_span = p.expect_op(Op::RBracket)?;
            span = span.join(rbracket_span);
        }
    }

    Ok(TypeRepr {
        kind: TypeKind::Named { name, generics },
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{Expr, ExprKind};
    use crate::ast::item::Name;
    use crate::ast::ty::{TypeKind, TypeRepr};
    use crate::diag::Span;
    use crate::lexer::{Op, Token};

    fn parse_ty(toks: &[Token]) -> Result<TypeRepr, ParseError> {
        let mut p = Parser::new(toks);
        type_repr(&mut p)
    }

    #[test]
    fn test_named_type() {
        let toks = vec![Token::ident("int")];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Named {
                    name: Name::new("int".to_string(), Span::default()),
                    generics: vec![],
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_generic_type() {
        let toks = vec![
            Token::ident("Stack"),
            Token::op(Op::LBracket),
            Token::ident("int"),
            Token::op(Op::RBracket),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Named {
                    name: Name::new("Stack".to_string(), Span::default()),
                    generics: vec![TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("int".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }],
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_array_type() {
        let toks = vec![
            Token::ident("T"),
            Token::op(Op::LBracket),
            Token::op(Op::RBracket),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Array {
                    elem: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("T".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                    size: None,
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_fixed_array_type() {
        let toks = vec![
            Token::ident("int"),
            Token::op(Op::LBracket),
            Token::int(10),
            Token::op(Op::RBracket),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Array {
                    elem: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("int".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                    size: Some(Expr {
                        kind: ExprKind::Int(10),
                        span: Span::default(),
                    }),
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_optional_type() {
        let toks = vec![Token::ident("T"), Token::op(Op::Question)];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Optional(Box::new(TypeRepr {
                    kind: TypeKind::Named {
                        name: Name::new("T".to_string(), Span::default()),
                        generics: vec![],
                    },
                    span: Span::default(),
                })),
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_fn_type() {
        let toks = vec![
            Token::op(Op::LParen),
            Token::ident("int"),
            Token::op(Op::RParen),
            Token::ident("int"),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Fn {
                    params: vec![TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("int".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }],
                    ret: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("int".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_error_union_type() {
        let toks = vec![
            Token::ident("MathError"),
            Token::op(Op::Bang),
            Token::ident("int"),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::ErrorUnion {
                    err: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("MathError".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                    ok: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("int".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_never_type() {
        let toks = vec![Token::op(Op::Bang)];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Never,
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_nested_generic_array() {
        let toks = vec![
            Token::ident("Stack"),
            Token::op(Op::LBracket),
            Token::ident("int"),
            Token::op(Op::LBracket),
            Token::op(Op::RBracket),
            Token::op(Op::RBracket),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Named {
                    name: Name::new("Stack".to_string(), Span::default()),
                    generics: vec![TypeRepr {
                        kind: TypeKind::Array {
                            elem: Box::new(TypeRepr {
                                kind: TypeKind::Named {
                                    name: Name::new("int".to_string(), Span::default()),
                                    generics: vec![],
                                },
                                span: Span::default(),
                            }),
                            size: None,
                        },
                        span: Span::default(),
                    }],
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_fn_type_multi_params() {
        let toks = vec![
            Token::op(Op::LParen),
            Token::ident("int"),
            Token::ident("int"),
            Token::op(Op::RParen),
            Token::ident("bool"),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Fn {
                    params: vec![
                        TypeRepr {
                            kind: TypeKind::Named {
                                name: Name::new("int".to_string(), Span::default()),
                                generics: vec![],
                            },
                            span: Span::default(),
                        },
                        TypeRepr {
                            kind: TypeKind::Named {
                                name: Name::new("int".to_string(), Span::default()),
                                generics: vec![],
                            },
                            span: Span::default(),
                        }
                    ],
                    ret: Box::new(TypeRepr {
                        kind: TypeKind::Named {
                            name: Name::new("bool".to_string(), Span::default()),
                            generics: vec![],
                        },
                        span: Span::default(),
                    }),
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_optional_array() {
        // T?[] - Array of Optional T
        let toks = vec![
            Token::ident("T"),
            Token::op(Op::Question),
            Token::op(Op::LBracket),
            Token::op(Op::RBracket),
        ];
        let ty = parse_ty(&toks).unwrap();
        assert_eq!(
            ty,
            TypeRepr {
                kind: TypeKind::Array {
                    elem: Box::new(TypeRepr {
                        kind: TypeKind::Optional(Box::new(TypeRepr {
                            kind: TypeKind::Named {
                                name: Name::new("T".to_string(), Span::default()),
                                generics: vec![],
                            },
                            span: Span::default(),
                        })),
                        span: Span::default(),
                    }),
                    size: None,
                },
                span: Span::default(),
            }
        );
    }

    #[test]
    fn test_invalid_array_start() {
        let toks = vec![Token::op(Op::LBracket), Token::op(Op::RBracket)];
        let res = parse_ty(&toks);
        assert!(res.is_err());
    }
}
