use crate::parser::*;
use crate::source::Source;
use miette::NamedSource;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct YamlParser;

impl Parser for YamlParser {
    fn parse<'de, T: DeserializeOwned>(
        &self,
        content: &'de str,
        source: &Source,
    ) -> Result<T, ParserError> {
        use serde::de::IntoDeserializer;

        // First pass, convert string to value
        let de = serde_yaml::Deserializer::from_str(&content);
        let mut result: serde_yaml::Value =
            serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                content: NamedSource::new(source.to_string(), content.to_owned()),
                path: error.path().to_string(),
                span: error
                    .inner()
                    .location()
                    .map(|s| create_span(&content, s.line(), s.column())),
                error: error.inner().to_string(),
            })?;

        // Applies anchors/aliases/references
        result.apply_merge().map_err(|error| ParserError {
            content: NamedSource::new(source.to_string(), content.to_owned()),
            path: String::new(),
            span: error.location().map(|s| (s.line(), s.column()).into()),
            error: error.to_string(),
        })?;

        // Second pass, convert value to struct
        let de = result.into_deserializer();

        serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.to_string(), content.to_owned()),
            path: error.path().to_string(),
            span: error
                .inner()
                .location()
                .map(|s| create_span(&content, s.line(), s.column())),
            error: error.inner().to_string(),
        })
    }
}
