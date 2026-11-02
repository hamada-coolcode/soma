pub mod eval;
pub mod value;

pub use value::Value;

use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::driver;
use crate::desugar;

#[derive(Debug, Default)]
pub struct RunOutput {
    pub stdout: String,
}

pub fn run_source(name: &str, src: &str) -> Result<RunOutput, Diagnostic> {
    let parsed = driver::parse_source(name, src)?;
    let items = desugar::lower(parsed.items);

    let mut evaluator = eval::Evaluator::new();
    evaluator.run_items(&items)?;

    if let Err(mut diag) = evaluator.call_main() {
        if !diag.message.starts_with("E0600") {
            diag.message = format!("E0600: {}", diag.message);
        }
        return Err(diag);
    }

    Ok(RunOutput {
        stdout: evaluator.stdout,
    })
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
    fn test_hello_world() {
        let src = r#"import
  lib:std
    io
      term
        output

fn
  main
    output("Hello, world!")
"#;
        let output = run_source("hello.soma", src).unwrap();
        assert_eq!(output.stdout, "Hello, world!\n");
    }

    #[test]
    fn test_fibonacci() {
        let src = std::fs::read_to_string("examples/fibonacci-n-step/main.soma").unwrap();
        let output = run_source("fibonacci.soma", &src).unwrap();
        let expected = "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n---\n0\n0\n1\n1\n2\n4\n7\n13\n24\n44\n";
        assert_eq!(output.stdout, expected);
    }

    #[test]
    fn test_div_by_zero() {
        let src = "fn\n  main\n    a = 1 / 0\n";
        let res = run_source("div.soma", src);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("division by zero"));
    }

    #[test]
    fn test_runaway_loop() {
        let src = "fn\n  main\n    while true\n      x = 1\n";
        let res = run_source("loop.soma", src);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("runaway loop"));
    }
}
