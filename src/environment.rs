use std::collections::HashMap;

use crate::functions::{add, sub};
use crate::interpreter::{Expr, InterpError};
use crate::functions::Function::RFunc;

/// Environment of running interpreter
pub struct Environment {
    stack: HashMap<String, Expr>,
}

impl Environment {
    pub fn default_environment() -> Self {
        let mut stack = HashMap::new();
        stack.insert("x".to_string(), Expr::Integer(10));
        stack.insert("v".to_string(), Expr::Integer(5));
        stack.insert("i".to_string(), Expr::Integer(1));
        stack.insert("add".to_string(), Expr::Function(RFunc(add)));
        stack.insert("sub".to_string(), Expr::Function(RFunc(sub)));

        Self {
            stack,
        }
    }

    pub fn bind(&mut self, binding: &str, value: Expr) -> Result<Expr, InterpError> {
        self.stack.insert(binding.to_string(), value);
        Ok(Expr::Symbol(binding.to_string()))
    }

    pub fn lookup(&mut self, binding: &str) -> Option<Expr> {
        self.stack.get(binding).cloned()
    }
}