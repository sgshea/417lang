---
marp: true
theme: gaia
_class: invert
---

# CSC417 Language Project
## Sammy Shea

Implementation language:
- Rust: compiled, statically typed, systems language

Cool things:
- Custom parser, Wasm compilation (demo)

---

# Architecture Overview
- interpreter crate
	- `environment.rs`
	- `functions.rs`
	- `error.rs`
	- `interpreter.rs`
	- `lib.rs` and `main.rs`


---
## Expr
[interpreter.rs](interpreter/src/interpreter.rs)

- Language Expression type defined as an enumeration

```rust
#[derive(PartialEq, Eq, Clone)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    String(String),
    List(Vec<Expr>),
    Function(Function),
}
```

---

## Eval
```rust
pub fn eval(val: &serde_json::Value, env: &mut Environment) -> Result<Expr, InterpError> {
    match val {
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| InterpError::TypeError {
                expected: "i64".to_string(),
                found: num.to_string(),
            })
            .map(|i| Expr::Integer(i)),
        Value::Bool(bool) => Ok(Expr::Boolean(*bool)),
        Value::String(string) => {
            return Ok(Expr::String(string.to_string()));
        }
        Value::Array(arr) => Ok(Expr::List(
            arr.into_iter()
                .map(|val| Expr::eval(val, env))
                .collect::<Result<Vec<Expr>, InterpError>>()?,
        )),
        Value::Object(obj) => interpret_object(obj, env),
        _ => Err(InterpError::ParseError {
            message: format!(
                "{} is not an implemented type! It is of JSON type {:?}",
                val, val
            ),
        }),
    }
}
```

---

# WebAssembly (Wasm)
- WebAssembly is low level programming language that can be compiled to
	- Open web standard, included in all browsers
	- Faster than JavaScript

Raw Wasm example (adding two numbers)
- From mdn web docs
```wasm
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    ;; load `10` and `3` onto the stack
    i32.const 10
    i32.const 3

    i32.add ;; add up both numbers
    call $log ;; log the result
  )
  (start $main)
)
```

- Rust has great support for Wasm!
	- `wasm-bindgen` library and tooling like `wasm-pack`

Demo: https://pages.github.ncsu.edu/sgshea/417-interp/
- Also includes my custom parser

---

Questions