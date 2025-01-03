use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::InterpError;
use crate::functions::Function::CoreFunction;
use crate::functions::{
    add, as_list, concat, contains, dbg, div, eq, get, greater, length, less, mul, print, println,
    rem, set, sort, sub, to_lowercase, to_uppercase, zero,
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
        env.add_builtin_func("equal?", eq);
        env.add_builtin_func("greater?", greater);
        env.add_builtin_func("less?", less);
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
        env.add_builtin_func("length", length);
        env.add_builtin_func("as_list", as_list);
        env.add_builtin_func("get", get);
        env.add_builtin_func("set", set);
        env.add_builtin_func("sort", sort);
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
        func: fn(&[Expr], &mut Environment) -> Result<Expr, InterpError>,
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

    /// Look for binding and change it if possible, else return an error
    /// Returns the new value if it was successful
    pub fn assignment(&mut self, identifier: &str, new_value: &Expr) -> Result<Expr, InterpError> {
        // Try to do in this local environment
        if let Some(expr_mut) = self.variables.get_mut(identifier) {
            *expr_mut = new_value.clone();
            return Ok(new_value.clone());
        }

        // Try recursively through parent environments
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assignment(identifier, new_value);
        }

        // Base case, identifier was never found and there is no parent of this environment
        Err(InterpError::UndefinedError {
            symbol: identifier.to_string(),
        })
    }
}

impl Environment {
    pub fn add_output(&mut self, output: &str) {
        self.output.push(output.to_string());
    }
}
