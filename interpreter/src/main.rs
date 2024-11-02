#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    #[cfg(feature = "parser")]
    {
        use std::io;
        use parser::parse;
        use interpreter::interpret_default;

        let input = io::read_to_string(io::stdin());
        use interpreter::interpret_with_parser_to_string;
        match parse("stdio", &input.expect("Error reading input.")) {
            Err(e) => {
                eprintln!("{:?}", e.as_diagnostic());
                std::process::exit(1);
            },
            Ok(ast) => {
                match interpret_default(ast) {
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                    Ok(expr) => {
                        println!("{}", expr);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "parser"))]
    {
        use std::io;
        use interpreter::interpret_string;

        let input = io::read_to_string(io::stdin()).expect("Error reading from stdin.");

        match interpret_string(&input) {
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            },
            Ok(expr) => {
                println!("{}", expr);
                std::process::exit(0);
            }
        }
    }
}