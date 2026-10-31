use soma::desugar::psoma::{emit_with_map, emit};
use soma::ast::item::{Item, ItemKind, FnDef, Name, Param};
use soma::ast::expr::{Block, Stmt, StmtKind, Expr, ExprKind, BinOp};
use soma::ast::ty::{TypeRepr, TypeKind};
use soma::diag::{Span, sourcemap::SourceMap};

fn s(start: usize, end: usize) -> Span {
    Span::new(start, end)
}

fn n(text: &str) -> Name {
    Name::new(text, s(0, text.len()))
}

fn int_ty() -> TypeRepr {
    TypeRepr {
        kind: TypeKind::Named {
            name: n("int"),
            generics: vec![],
        },
        span: s(10, 13),
    }
}

#[test]
fn test_span_map() {
    let mut sm = SourceMap::new();
    // We mock some items

    // a + b
    let e = Expr {
        kind: ExprKind::Binary {
            op: BinOp::Add,
            lhs: Box::new(Expr {
                kind: ExprKind::Name(n("a")),
                span: s(20, 21),
            }),
            rhs: Box::new(Expr {
                kind: ExprKind::Name(n("b")),
                span: s(24, 25),
            }),
        },
        span: s(20, 25),
    };

    let f = FnDef {
        name: n("add"),
        generics: vec![],
        params: vec![
            Param { name: n("a"), ty: int_ty() },
            Param { name: n("b"), ty: int_ty() },
        ],
        ret: Some(int_ty()),
        body: Block {
            stmts: vec![Stmt {
                kind: StmtKind::Return(Some(e)),
                span: s(15, 26),
            }],
            span: s(14, 28),
        },
    };

    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(f),
        span: s(0, 29),
    };

    let items = vec![item];
    let (text, mapping) = emit_with_map(&items);

    // Check mapping
    // offset 0 is item
    let r = SourceMap::map_psoma_span(&mapping, 0);
    assert_eq!(r, Some(s(0, 29)));

    let text_len = text.len();
    assert!(mapping.len() > 0);
}
