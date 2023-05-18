use crate::parser::*;
use crate::source::Source;
use miette::NamedSource;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct TomlParser;

impl Parser for TomlParser {
    fn parse<'de, T: DeserializeOwned>(
        &self,
        content: &'de str,
        source: &Source,
    ) -> Result<T, ParserError> {
        let de = toml::Deserializer::new(&content);

        serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.to_string(), content.to_owned()),
            path: error.path().to_string(),
            span: error.inner().span().map(|s| s.into()),
            error: error.inner().message().to_owned(),
        })
    }
}
