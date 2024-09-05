use std::collections::HashMap;

use crate::interpreter::Expr;

/// Environment of running interpreter
pub struct Environment {
    stack: HashMap<String, Expr>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: HashMap::new()
        }
    }

    pub fn push(&mut self, binding: &str, value: Expr) {
        self.stack.insert(binding.to_string(), value);
    }
}