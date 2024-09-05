#[derive(PartialEq, Eq, Debug)]
pub enum Function {
    // Function defined by user in language
    LFunc(String),
    // Internal Rust function (holds a function pointer)
    RFunc(fn()),
}