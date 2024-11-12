use std::collections::HashMap;

use crate::functions::{add, concat, contains, dbg, div, eq, mul, print, println, rem, sub, to_lowercase, to_uppercase, zero};
use crate::interpreter::Expr;
use crate::functions::Function::CoreFunction;
use crate::error::InterpError;

/// Environment of running interpreter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    // Stack of environments, deepest is default, next is global, then local, etc.
    stack: Vec<HashMap<String, Expr>>,
    // Flag for whether to enable lexical scope or not (default true)
    pub lexical_scope: bool,
    // Flag for whether to store output instead of directly outputing it
    pub store_output: bool,
    // All output stored, able to be used for environments that do not support printing normally (WASM)
    output: Vec<String>,
}

impl Environment {
    /// Default environment of the interpreter with all builtins
    pub fn default_environment(lexical_scope: bool, store_output: bool) -> Self {
        let mut env = Self {
            stack: vec![HashMap::new()],
            lexical_scope,
            store_output,
            output: Vec::new()
        };

        env.add_builtin_func("print", print);
        env.add_builtin_func("println", println);
        env.add_builtin_func("dbg", dbg);
        env.add_builtin_func("=", eq);
        env.add_builtin_func("add", add);
        env.add_builtin_func("sub", sub);
        env.add_builtin_func("mul", mul);
        env.add_builtin_func("div", div);
        env.add_builtin_func("rem", rem);
        env.add_builtin_func("zero?", zero);
        env.add_builtin_func("to_uppercase", to_uppercase);
        env.add_builtin_func("to_lowercase", to_lowercase);
        env.add_builtin_func("concat", concat);
        env.add_builtin_func("contains", contains);
        env.add_builtin("x", Expr::Integer(10));
        env.add_builtin("v", Expr::Integer(5));
        env.add_builtin("i", Expr::Integer(1));
        env.add_builtin("true", Expr::Boolean(true));
        env.add_builtin("false", Expr::Boolean(false));

        env
    }

    /// Adds builtin item
    fn add_builtin(&mut self, name: &str, expr: Expr) {
        self.stack.first_mut().expect("Stack should be initialized!").insert(name.to_string(), expr);
    }

    /// Adds function to builtins (bottom of stack)
    fn add_builtin_func(&mut self, name: &str, func: fn(&[Expr], &mut Environment) -> Result<Expr, InterpError>) {
        self.stack.first_mut().expect("Stack should be initialized!").insert(name.to_string(), Expr::Function(CoreFunction { name: name.to_string(), func }));
    }

    /// Bind a group of bindings to expressions that are passed in as a tuple pair
    /// Adds to top environment of the stack
    pub fn bind(&mut self, pairs: Vec<(&String, &Expr)>) {
        let local_env: &mut HashMap<String, Expr> = self.stack.last_mut().expect("Environment should not be empty!");
        for (binding, expr) in pairs {
            local_env.insert(binding.to_string(), expr.clone());
        }
    }

    /// Creates a new local environment on the top of the stack
    pub fn create_local_env(&mut self) {
        self.stack.push(HashMap::new());
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

    /// Add new element to output
    pub fn add_output(&mut self, output: &str) {
        self.output.push(output.to_string());
    }

    /// Get the output
    pub fn get_output(&self) -> &Vec<String> {
        &self.output
    }

    // Get the output concatenated together as a String
    pub fn get_output_string(&self) -> String {
        self.output.concat()
    }
}
