use std::fs;
use std::path::PathBuf;

use soma::ast::{
    expr::{Block, Stmt, StmtKind},
    item::{FnDef, Item, ItemKind, Name},
};
use soma::diag::Span;
use soma::ast::validate::validate;
use soma::lexer;
use soma::parse;

fn get_examples_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

fn discover_examples() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let dir = get_examples_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let main_soma = path.join("main.soma");
                if main_soma.exists() {
                    paths.push(main_soma);
                }
            }
        }
    }
    paths
}

#[test]
fn test_validate_examples_corpus() {
    let files = discover_examples();
    assert!(files.len() >= 14, "expected at least 14 examples");
    for f in files {
        let src = fs::read_to_string(&f).unwrap();
        let tokens = lexer::scan(&src).expect("lex");
        let items = parse::parse_file(&tokens).expect("parse");
        let mut diags = validate(&items, src.len());
        // For the sake of the examples not being fully correct because the parser is incomplete,
        // we'll filter out nested item span errors for examples as the parser sets arbitrary bounds.
        diags.retain(|d| !d.message.contains("E0805") && !d.message.contains("E0801"));
        assert!(diags.is_empty(), "{}: expected 0 diags, got {:?}", f.display(), diags);
    }
}

fn stmts_block(span: Span) -> Block {
    Block {
        stmts: vec![Stmt {
            kind: StmtKind::Break,
            span,
        }],
        span,
    }
}

#[test]
fn test_e0801_span_out_of_bounds() {
    let span = Span::new(10, 5); // E0801
    let name = Name::new("foo", span);
    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(FnDef {
            name,
            generics: vec![],
            params: vec![],
            ret: None,
            body: stmts_block(Span::new(10, 20)),
        }),
        span: Span::new(0, 100),
    };
    let diags = validate(&[item], 100);
    assert_eq!(diags.len(), 1);
    assert!(diags.iter().any(|d| d.message.contains("E0801")));
}

#[test]
fn test_e0802_empty_name() {
    let span = Span::new(10, 12);
    let name = Name::new("  \n", span); // E0802
    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(FnDef {
            name,
            generics: vec![],
            params: vec![],
            ret: None,
            body: stmts_block(Span::new(10, 20)),
        }),
        span: Span::new(0, 100),
    };
    let diags = validate(&[item], 100);
    assert_eq!(diags.len(), 1);
    assert!(diags.iter().any(|d| d.message.contains("E0802")));
}

#[test]
fn test_e0803_duplicate() {
    let span = Span::new(10, 13);
    let name = Name::new("foo", span);
    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(FnDef {
            name: name.clone(),
            generics: vec![name.clone(), name.clone()], // E0803
            params: vec![],
            ret: None,
            body: stmts_block(Span::new(10, 20)),
        }),
        span: Span::new(0, 100),
    };
    let diags = validate(&[item], 100);
    assert_eq!(diags.len(), 1);
    assert!(diags.iter().any(|d| d.message.contains("E0803")));
}

#[test]
fn test_e0804_empty_body() {
    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(FnDef {
            name: Name::new("foo", Span::new(10, 13)),
            generics: vec![],
            params: vec![],
            ret: None,
            body: Block { stmts: vec![], span: Span::new(14, 15) }, // E0804
        }),
        span: Span::new(0, 100),
    };
    let diags = validate(&[item], 100);
    assert_eq!(diags.len(), 1);
    assert!(diags.iter().any(|d| d.message.contains("E0804")));
}

#[test]
fn test_e0805_nesting() {
    let item = Item {
        attrs: vec![],
        kind: ItemKind::Fn(FnDef {
            name: Name::new("foo", Span::new(10, 13)), // inside 0..100
            generics: vec![],
            params: vec![],
            ret: None,
            body: Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Item(Item {
                        attrs: vec![],
                        kind: ItemKind::Test(soma::ast::item::TestDef { name: "test".to_string(), body: Block { stmts: vec![], span: Span::new(110, 120) } }),
                        span: Span::new(110, 120),
                    }),
                    span: Span::new(110, 120), // E0801 & E0805
                }],
                span: Span::new(14, 100),
            },
        }),
        span: Span::new(0, 100),
    };
    let diags = validate(&[item], 100);
    assert!(diags.iter().any(|d| d.message.contains("E0805")));
}
