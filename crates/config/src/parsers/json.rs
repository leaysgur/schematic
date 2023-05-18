use crate::parser::*;
use crate::source::Source;
use miette::NamedSource;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct JsonParser;

impl Parser for JsonParser {
    fn parse<'de, T: DeserializeOwned>(
        &self,
        content: &'de str,
        source: &Source,
    ) -> Result<T, ParserError> {
        let de = &mut serde_json::Deserializer::from_str(&content);

        serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.to_string(), content.to_owned()),
            path: error.path().to_string(),
            span: Some(create_span(
                &content,
                error.inner().line(),
                error.inner().column(),
            )),
            error: error.inner().to_string(),
        })
    }
}
