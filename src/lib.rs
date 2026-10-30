pub mod ast;
pub mod builtin;
pub mod cli;
pub mod codegen;
pub mod desugar;
pub mod diag;
pub mod driver;
pub mod interp;
pub mod emit_json;
pub mod fmt;
pub mod loader;
pub mod testrun;
pub mod lexer;
pub mod mem;
pub mod parse;
pub mod resolve;
pub mod types;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
