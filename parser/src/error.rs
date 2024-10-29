use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
#[error("Error while parsing!")]
pub struct ParseError {
    #[source_code]
    src: NamedSource<String>,

    label: String,

    #[label("{label} here")]
    main_span: SourceSpan,

    #[label(collection, "related to this")]
    other_spans: Vec<LabeledSpan>,

    #[help]
    help: Option<String>,
}

impl ParseError {
    pub fn new(source_name: &str, src: &str, span: (usize, usize), label: &str) -> Self {
        let err = Self {
            src: NamedSource::new(source_name, src.to_string()),
            label: label.to_string(),
            main_span: span.into(),
            other_spans: vec![],
            help: None
        };
        err
    }

    pub fn new_full(source_name: &str, src: &str, span: (usize, usize), label: &str, help: Option<String>, other_spans: Vec<LabeledSpan>) -> Self {
        let err = Self {
            src: NamedSource::new(source_name, src.to_string()),
            label: label.to_string(),
            main_span: span.into(),
            other_spans,
            help
        };
        err
    }

    pub fn change_label(&mut self, new_label: &str) {
        self.label = new_label.to_string()
    }

    pub fn add_spans(&mut self, additional_spans: &mut Vec<LabeledSpan>) {
        self.other_spans.append(additional_spans);
    }

    pub fn add_help(&mut self, help: &str) {
        self.help = Some(help.to_string())
    }
}