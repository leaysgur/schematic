use crate::error::ConfigError;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{self, Display};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Code { code: String },
    Defaults,
    Env,
    File { path: PathBuf },
    Url { url: String },
}

impl Source {
    pub fn new(value: &str, parent_source: Option<&Source>) -> Result<Source, ConfigError> {
        // Extending from a URL is allowed from any parent source
        if is_url_like(value) {
            return Source::url(value);
        }

        // Extending from a file is only allowed from file parent sources
        if is_file_like(value) {
            let value = if let Some(stripped) = value.strip_prefix("file://") {
                stripped
            } else {
                value
            };

            if parent_source.is_none() {
                return Source::file(value);
            }

            if let Source::File {
                path: parent_path, ..
            } = parent_source.unwrap()
            {
                let mut path = PathBuf::from(value);

                // Not absolute, so prefix with parent
                if !path.has_root() {
                    path = parent_path.parent().unwrap().join(path);
                }

                return Source::file(path);
            } else {
                return Err(ConfigError::ExtendsFromParentFileOnly);
            }
        }

        Source::code(value)
    }

    pub fn code<T: TryInto<String>>(code: T) -> Result<Source, ConfigError> {
        let code: String = code.try_into().map_err(|_| ConfigError::InvalidCode)?;

        Ok(Source::Code { code })
    }

    pub fn file<T: TryInto<PathBuf>>(path: T) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;

        Ok(Source::File { path })
    }

    pub fn url<T: TryInto<String>>(url: T) -> Result<Source, ConfigError> {
        let url: String = url.try_into().map_err(|_| ConfigError::InvalidUrl)?;

        if !url.starts_with("https://") {
            return Err(ConfigError::HttpsOnly);
        }

        Ok(Source::Url { url })
    }

    pub fn parse<D>(&self, format: SourceFormat, label: &str) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        let result = match self {
            Source::Code { code } => format.parse(code.to_owned(), "code"),
            Source::File { path } => {
                if !path.exists() {
                    return Err(ConfigError::MissingFile(path.to_path_buf()));
                }

                format.parse(fs::read_to_string(path)?, path.to_str().unwrap())
            }
            Source::Url { url } => format.parse(reqwest::blocking::get(url)?.text()?, url),
            _ => unreachable!(),
        };

        result.map_err(|error| ConfigError::Parser {
            config: label.to_owned(),
            content: error.content,
            error: error.error,
            path: error.path,
            span: error.span,
        })
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Code { .. } => write!(f, "code"),
            Source::Defaults => write!(f, "defaults"),
            Source::Env => write!(f, "env"),
            Source::File { path } => write!(f, "{}", path.display()),
            Source::Url { url } => write!(f, "{}", url),
        }
    }
}

pub fn is_file_like(value: &str) -> bool {
    value.starts_with("file://")
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with('.')
        || value.contains('/')
        || value.contains('\\')
        || value.ends_with(".json")
        || value.ends_with(".toml")
        || value.ends_with(".yaml")
        || value.ends_with(".yml")
}

pub fn is_url_like(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("www")
}
