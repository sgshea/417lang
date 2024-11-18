/// Main function for usage on native tagets with command line
#[cfg(not(target_arch = "wasm32"))] // Cannot compile code needed std::io on wasm
pub fn main() {
    use std::env;

    let args: Vec<String> = env::args().collect();
    // Lexical scope is default, dynamic scope only if argument specified
    // Like '-- dynamic_scope
    let lexical_scope = !args.contains(&"dynamic_scope".to_string());

    // The normal run without any features, reads in input (expecting JSON) and interprets
    #[cfg(not(feature = "parser"))]
    {
        use std::io;
        use interpreter::interpret_string;

        let input = io::read_to_string(io::stdin()).expect("Error reading from stdin.");

        match interpret_string(&input, lexical_scope, false) {
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            },
            Ok(expr) => {
                println!("{}", expr.0);
                std::process::exit(0);
            }
        }
    }

    // Reads in input, parses it using the parser crate, and then interprets
    #[cfg(feature = "parser")]
    {
        use std::io;
        use parser::parse;
        use interpreter::interpret_default;

        let input = io::read_to_string(io::stdin());
        match parse("stdio", &input.expect("Error reading input.")) {
            Err(e) => {
                eprintln!("{:?}", e.as_diagnostic());
                std::process::exit(1);
            },
            Ok(ast) => {
                match interpret_default(ast, lexical_scope, false) {
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                    Ok(expr) => {
                        println!("{}", expr.0);
                    }
                }
            }
        }
    }

}