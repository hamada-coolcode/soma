//! Builtin function registry (owned by task 08).

use crate::ast::ty::TypeRepr;

#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    /// Fully qualified path, e.g. `io.term.output`.
    pub path: &'static str,
    /// Signature in soma syntax, e.g. `s: string -> !`.
    pub sig: &'static str,
}

pub const BUILTINS: &[Builtin] = &[
    Builtin {
        path: "io.term.output",
        sig: "(string) !",
    },
    Builtin {
        path: "io.term.input",
        sig: "(string) T",
    },
    Builtin {
        path: "fs.read",
        sig: "() string[]",
    },
    Builtin {
        path: "fs.write",
        sig: "(string) !",
    },
    Builtin {
        path: "fs.exists",
        sig: "() bool",
    },
    Builtin {
        path: "string.len",
        sig: "() int",
    },
    Builtin {
        path: "string.contains",
        sig: "(string) bool",
    },
    Builtin {
        path: "string.lower",
        sig: "() string",
    },
    Builtin {
        path: "string.split",
        sig: "(string) string[]",
    },
    Builtin {
        path: "list.len",
        sig: "() int",
    },
    Builtin {
        path: "list.add",
        sig: "(T) !",
    },
    Builtin {
        path: "list.extend",
        sig: "(T[]) !",
    },
    Builtin {
        path: "list.map",
        sig: "((T) T) T[]",
    },
    Builtin {
        path: "env.args",
        sig: "() string[]",
    },
    Builtin {
        path: "env.BoolVar",
        sig: "(string) bool",
    },
];

pub fn find(path: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.path == path)
}

/// Parse a type written in soma syntax, using the real type parser.
pub fn type_from_str(src: &str) -> Option<TypeRepr> {
    let toks = crate::lexer::scan(src).ok()?;
    let mut p = crate::parse::util::Parser::new(&toks);
    let ty = crate::parse::types::type_repr(&mut p).ok()?;
    if !p.at_eof() {
        return None;
    }
    Some(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtins_surface() {
        let expected = vec![
            "io.term.output",
            "io.term.input",
            "fs.read",
            "fs.write",
            "fs.exists",
            "string.len",
            "string.contains",
            "string.lower",
            "string.split",
            "list.len",
            "list.add",
            "list.extend",
            "list.map",
            "env.args",
            "env.BoolVar",
        ];

        let mut actual: Vec<_> = BUILTINS.iter().map(|b| b.path).collect();
        actual.sort();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();

        assert_eq!(actual, expected_sorted, "BUILTINS paths mismatch");

        for &path in &expected {
            assert!(find(path).is_some(), "Path not found: {}", path);
        }

        for builtin in BUILTINS {
            let ty = type_from_str(builtin.sig);
            assert!(
                ty.is_some(),
                "Failed to parse signature for {}: {}",
                builtin.path,
                builtin.sig
            );
        }
    }
}
