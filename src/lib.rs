pub mod ast;
pub mod builtin;
pub mod cli;
pub mod desugar;
pub mod diag;
pub mod driver;
pub mod emit_json;
pub mod fmt;
pub mod interp;
pub mod lexer;
pub mod loader;
pub mod parse;
pub mod resolve;
pub mod testrun;
pub mod types;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
