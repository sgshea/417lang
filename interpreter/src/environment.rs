use std::collections::HashMap;

use crate::functions::{add, dbg, eq, print, println, sub, zero};
use crate::interpreter::Expr;
use crate::functions::Function::RFunc;

/// Environment of running interpreter
#[derive(Debug, Clone)]
pub struct Environment {
    // Stack of environments, deepest is default, next is global, then local, etc.
    stack: Vec<HashMap<String, Expr>>,
}

impl Environment {
    /// Default environment of the interpreter with all builtins
    pub fn default_environment() -> Self {
        let mut stack = HashMap::new();
        stack.insert("x".to_string(), Expr::Integer(10));
        stack.insert("v".to_string(), Expr::Integer(5));
        stack.insert("i".to_string(), Expr::Integer(1));
        stack.insert("true".to_string(), Expr::Boolean(true));
        stack.insert("false".to_string(), Expr::Boolean(false));
        stack.insert("eq".to_string(), Expr::Function(RFunc{name: "eq".to_string(), func: eq}));
        stack.insert("print".to_string(), Expr::Function(RFunc{name: "print".to_string(), func: print}));
        stack.insert("println".to_string(), Expr::Function(RFunc{name: "println".to_string(), func: println}));
        stack.insert("dbg".to_string(), Expr::Function(RFunc{name: "dbg".to_string(), func: dbg}));
        stack.insert("add".to_string(), Expr::Function(RFunc{name: "add".to_string(), func: add}));
        stack.insert("sub".to_string(), Expr::Function(RFunc{name: "sub".to_string(), func: sub}));
        stack.insert("zero?".to_string(), Expr::Function(RFunc{name: "zero?".to_string(), func: zero}));

        Self {
            stack: vec![stack],
        }
    }


    /// Bind a group of bindings to expressions that are passed in as a tuple pair
    /// Adds to a new local environment
    pub fn bind(&mut self, pairs: Vec<(&String, &Expr)>) {
        let mut local_env: HashMap<String, Expr> = HashMap::with_capacity(pairs.len());
        for (binding, expr) in pairs {
            local_env.insert(binding.to_string(), expr.clone().clone());
        }
        self.stack.push(local_env);
    }

    /// Removes the most local environment, returining it
    pub fn pop_top_env(&mut self) -> Option<HashMap<String, Expr>> {
        self.stack.pop()
    }

    /// Look for binding in local (top) environment first, then search deeper
    pub fn lookup(&mut self, binding: &str) -> Option<Expr> {
        for env in (&self.stack).into_iter().rev() {
            if let Some(expr) = env.get(binding) {
                return Some(expr.clone())
            }
        }
        None
    }
}