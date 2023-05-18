use crate::source::Source;
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use serde::de::DeserializeOwned;
use starbase_styles::{Style, Stylize};
use thiserror::Error;

pub fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}

#[derive(Error, Debug, Diagnostic)]
#[error("Invalid setting {}", .path.style(Style::Id))]
#[diagnostic(severity(Error))]
pub struct ParserError {
    #[source_code]
    pub content: NamedSource,

    pub error: String,

    pub path: String,

    #[label("{}", .error)]
    pub span: Option<SourceSpan>,
}

pub trait Parser: Sized {
    fn parse<'de, T: DeserializeOwned>(
        &self,
        content: &'de str,
        source: &Source,
    ) -> Result<T, ParserError>;
}
