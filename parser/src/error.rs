use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Error while parsing!")]
#[diagnostic()]
pub struct ParseError {
    #[source_code]
    src: NamedSource<String>, // we should figure out how to share these

    #[label("This bit right here")]
    bad_bit: SourceSpan,
}

impl ParseError {
    pub fn new(source_name: &str, src: &str, span: (usize, usize)) -> Self {
        let err = Self {
            src: NamedSource::new(source_name, src.to_string()),
            bad_bit: span.into()
        };
        err
    }
}