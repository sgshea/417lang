use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::InterpError;
use crate::functions::Function::CoreFunction;
use crate::functions::{
    add, concat, contains, dbg, div, eq, mul, print, println, rem, sub, to_lowercase, to_uppercase,
    zero,
};
use crate::interpreter::Expr;

/// Environment of running interpreter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEnvironment {
    // Stack of environments, deepest is default, next is global, then local, etc.
    variables: HashMap<String, Expr>,
    parent: Option<Rc<RefCell<LocalEnvironment>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    // Flag for whether to enable lexical scope or not (default true)
    pub lexical_scope: bool,
    // Flag for whether to store output instead of directly outputing it
    pub store_output: bool,
    // All output stored, able to be used for environments that do not support printing normally (WASM)
    pub output: Vec<String>,
}

impl LocalEnvironment {
    /// Default environment of the interpreter with all builtins
    pub fn default_environment() -> Self {
        let mut env = Self {
            variables: HashMap::new(),
            parent: None,
        };

        env.add_builtin_func("print", print);
        env.add_builtin_func("println", println);
        env.add_builtin_func("dbg", dbg);
        env.add_builtin_func("eq", eq);
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

    pub fn from_parent(parent: Rc<RefCell<Self>>) -> Rc<RefCell<LocalEnvironment>> {
        Rc::new(RefCell::new(Self {
            variables: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn return_parent(&self) -> Option<Rc<RefCell<LocalEnvironment>>> {
        self.parent.clone()
    }

    /// Adds builtin item
    fn add_builtin(&mut self, name: &str, expr: Expr) {
        self.variables.insert(name.to_string(), expr);
    }

    /// Adds function to builtins (bottom of stack)
    fn add_builtin_func(
        &mut self,
        name: &str,
        func: fn(&[Expr], &mut LocalEnvironment, &mut Environment) -> Result<Expr, InterpError>,
    ) {
        self.variables.insert(
            name.to_string(),
            Expr::Function(CoreFunction {
                name: name.to_string(),
                func,
            }),
        );
    }

    /// Bind a group of bindings to expressions that are passed in as a tuple pair
    /// Adds to top environment of the stack
    pub fn bind(&mut self, pairs: Vec<(&String, &Expr)>) {
        let local_env: &mut HashMap<String, Expr> = &mut self.variables;
        for (binding, expr) in pairs {
            local_env.insert(binding.to_string(), expr.clone());
        }
    }

    /// Look for binding in local (top) environment first, then search deeper
    pub fn lookup(&self, binding: &str) -> Option<Expr> {
        if let Some(expr) = self.variables.get(binding) {
            return Some(expr.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().lookup(binding);
        }

        None
    }
}

impl Environment {
    pub fn add_output(&mut self, output: &str) {
        self.output.push(output.to_string());
    }
}