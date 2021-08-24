pub mod config;
pub mod cookies;
pub mod node;
pub mod paintboard;

pub use self::config::*;

use std::convert::TryFrom;

pub enum ScriptError {
    FailedReadFile(std::io::Error),
    FailedParseToml(toml::de::Error),
    FailedParseJson(serde_json::Error),
    FailedConnecntWebsocket(websocket::WebSocketError),
    FailedParseUrl(url::ParseError),
    FailedProcessRequest(reqwest::Error),
    UnexpectedUrl(UrlError),
}

pub enum UrlError {
    InvalidHTTPUrl,
    InvalidWSUrl,
}

impl From<std::io::Error> for ScriptError {
    fn from(error: std::io::Error) -> Self {
        ScriptError::FailedReadFile(error)
    }
}

impl From<serde_json::Error> for ScriptError {
    fn from(error: serde_json::Error) -> Self {
        ScriptError::FailedParseJson(error)
    }
}

impl From<websocket::WebSocketError> for ScriptError {
    fn from(error: websocket::WebSocketError) -> Self {
        ScriptError::FailedConnecntWebsocket(error)
    }
}

impl From<toml::de::Error> for ScriptError {
    fn from(error: toml::de::Error) -> Self {
        ScriptError::FailedParseToml(error)
    }
}

impl From<url::ParseError> for ScriptError {
    fn from(error: url::ParseError) -> Self {
        ScriptError::FailedParseUrl(error)
    }
}

impl From<reqwest::Error> for ScriptError {
    fn from(error: reqwest::Error) -> Self {
        ScriptError::FailedProcessRequest(error)
    }
}

impl std::fmt::Display for UrlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UrlError::InvalidHTTPUrl => {
                write!(formatter, "Invalid HTTP URL!")
            }
            UrlError::InvalidWSUrl => {
                write!(formatter, "Invalid WebSocket URL!")
            }
        }
    }
}

impl From<UrlError> for ScriptError {
    fn from(error: UrlError) -> Self {
        ScriptError::UnexpectedUrl(error)
    }
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScriptError::FailedReadFile(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::FailedParseToml(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::FailedParseJson(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::FailedConnecntWebsocket(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::FailedParseUrl(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::FailedProcessRequest(err) => formatter.write_str(&format!("{}", err)),
            ScriptError::UnexpectedUrl(err) => formatter.write_str(&format!("{}", err)),
        }
    }
}

// TODO: 抽象
pub fn to_32(cur: usize) -> char {
    let cur = u8::try_from(cur).unwrap();
    if cur <= 9 {
        (b'0' + cur) as char
    } else {
        (cur - 9 + b'a') as char
    }
}

pub fn from_32(cur: char) -> usize {
    cur.to_digit(33).unwrap() as usize
}
