use std::collections::HashMap;

use super::symbol::Symbol;

#[derive(Debug, Default)]
pub struct Scope {
    frames: Vec<HashMap<String, Symbol>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Returns the symbol it shadowed, if any.
    pub fn define(&mut self, sym: Symbol) -> Option<Symbol> {
        self.frames
            .last_mut()
            .expect("scope always has a frame")
            .insert(sym.name.clone(), sym)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.frames.iter().rev().find_map(|f| f.get(name))
    }

    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.frames.last().and_then(|f| f.get(name))
    }
}
