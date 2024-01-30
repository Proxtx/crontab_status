use std::{error::Error, fmt};

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    ReadFileError(std::io::Error),
    TomlParseError(toml::de::Error),
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ReadFileError(v) => {
                write!(f, "Unable to read config file: {}", v)
            }
            ConfigError::TomlParseError(v) => {
                write!(f, "Unable to parse Toml: {}", v)
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::ReadFileError(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::TomlParseError(value)
    }
}
