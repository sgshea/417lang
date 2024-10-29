use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Error while parsing!")]
pub struct ParseError {
    #[source_code]
    src: NamedSource<String>,

    label: String,

    #[label("{label} here")]
    bad_bit: SourceSpan,
}

impl ParseError {
    pub fn new(source_name: &str, src: &str, span: (usize, usize), label: &str) -> Self {
        let err = Self {
            src: NamedSource::new(source_name, src.to_string()),
            label: label.to_string(),
            bad_bit: span.into(),
        };
        err
    }

    pub fn change_label(&mut self, new_label: &str) {
        self.label = new_label.to_string()
    }
}