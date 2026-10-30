pub mod ast;
pub mod cli;
pub mod desugar;
pub mod diag;
pub mod lexer;
pub mod parse;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
