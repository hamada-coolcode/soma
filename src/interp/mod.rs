//! Development interpreter for `soma run` (owned by task 15).
//!
//! This is NOT the zero-runtime-cost backend; it exists so semantics can be
//! tested without codegen.

pub mod eval;
pub mod value;

pub use value::Value;

use crate::diag::{Diagnostic, ErrorKind, Span};

#[derive(Debug, Default)]
pub struct RunOutput {
    pub stdout: String,
}

pub fn run_source(name: &str, src: &str) -> Result<RunOutput, Diagnostic> {
    let parsed = crate::driver::parse_source(name, src)?;
    let lowered = crate::desugar::lower(parsed.items);

    let mut ctx = eval::Context::new();
    ctx.globals.insert("output".to_string(), Value::BuiltinFn("output".to_string()));
    ctx.globals.insert("input".to_string(), Value::BuiltinFn("input".to_string()));
    ctx.globals.insert("env.args".to_string(), Value::BuiltinFn("env.args".to_string()));

    let env = eval::Env::new();

    let mut main_fn = None;

    for item in lowered {
        match item.kind {
            crate::ast::item::ItemKind::Fn(f) => {
                if f.name.text == "main" {
                    main_fn = Some(f.clone());
                }
                ctx.globals.insert(f.name.text.clone(), Value::Fn(f));
            },
            crate::ast::item::ItemKind::Object(obj) => {
                ctx.objects.insert(obj.name.text.clone(), obj);
            },
            crate::ast::item::ItemKind::Const(c) => {
                if let Ok(v) = eval::eval_expr(&mut ctx, &env, &c.value) {
                    ctx.globals.insert(c.name.text.clone(), v);
                }
            },
            crate::ast::item::ItemKind::Error(_) => {
            },
            _ => {}
        }
    }

    if let Some(m) = main_fn {
        match eval::eval_block(&mut ctx, &env, &m.body) {
            Ok(_) => Ok(ctx.output),
            Err(eval::ControlFlow::Return(_)) => Ok(ctx.output),
            Err(eval::ControlFlow::Error(e)) => Err(e),
            Err(_) => Err(Diagnostic::new(ErrorKind::Other, "unexpected control flow", Span::default())),
        }
    } else {
        Ok(ctx.output)
    }
}

pub fn run_file(path: &str) -> Result<RunOutput, Diagnostic> {
    let src = std::fs::read_to_string(path).map_err(|e| {
        Diagnostic::new(ErrorKind::Io, format!("{path}: {e}"), Span::default())
    })?;
    run_source(path, &src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_hello() {
        let src = r#"
fn
  main -> !
    output("Hello, world!")
"#;
        let out = run_source("hello.soma", src).unwrap();
        assert_eq!(out.stdout, "Hello, world!\n");
    }

    #[test]
    fn test_fib() {
        let src = r#"
fn
  fib n: int -> int
    if n <= 1
      return n
    return (fib(n - 1) + fib(n - 2))

  main -> !
    r = fib(10)
    output(r)
"#;
        let out = run_source("fib.soma", src).unwrap();
        assert_eq!(out.stdout, "55\n");
    }
}
