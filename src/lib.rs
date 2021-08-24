pub mod cookies;
pub mod node;
pub mod paintboard;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::error::Error;
use std::fs;
use toml;
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub board_addr: String,
    pub websocket_addr: String,
    pub cookie_dir: String,
    pub node_file: String,
    pub wait_time: u64,
}

pub enum InvalidUrlError {
    InvalidHTTPError,
    InvalidWSError,
}

impl std::fmt::Display for InvalidUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InvalidUrlError::InvalidHTTPError => {
                write!(f, "Invalid HTTP URL!")
            }
            InvalidUrlError::InvalidWSError => {
                write!(f, "Invalid WebSocket URL!")
            }
        }
    }
}

impl std::fmt::Debug for InvalidUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InvalidUrlError::InvalidHTTPError => {
                write!(f, "Invalid HTTP URL!")
            }
            InvalidUrlError::InvalidWSError => {
                write!(f, "Invalid WebSocket URL!")
            }
        }
    }
}

impl Error for InvalidUrlError {}

impl Config {
    pub fn new(filename: String) -> Result<Config, Box<dyn Error>> {
        let config: Config = toml::from_str(&fs::read_to_string(&filename)?)?;
        // check if the board_addr and websocket_addr are valid
        let board_addr = Url::parse(&config.board_addr)?;
        let websocket_addr = Url::parse(&config.websocket_addr)?;
        if board_addr.scheme() != "http" && board_addr.scheme() != "https" {
            return Err(Box::new(InvalidUrlError::InvalidHTTPError));
        }
        if websocket_addr.scheme() != "ws" && websocket_addr.scheme() != "wss" {
            return Err(Box::new(InvalidUrlError::InvalidWSError));
        }
        Ok(config)
    }
}

pub enum ScriptError {
    FailedReadFile(std::io::Error),
    FailedParseString(serde_json::Error),
    FailedConnecntWebsocket(websocket::WebSocketError),
}

impl From<std::io::Error> for ScriptError {
    fn from(error: std::io::Error) -> Self {
        ScriptError::FailedReadFile(error)
    }
}

impl From<serde_json::Error> for ScriptError {
    fn from(error: serde_json::Error) -> Self {
        ScriptError::FailedParseString(error)
    }
}

impl From<websocket::WebSocketError> for ScriptError {
    fn from(error: websocket::WebSocketError) -> Self {
        ScriptError::FailedConnecntWebsocket(error)
    }
}

impl std::fmt::Debug for ScriptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScriptError::FailedReadFile(err) => {
                formatter.write_str(&format!("无法访问文件: {:?}", err))
            }
            ScriptError::FailedParseString(err) => {
                formatter.write_str(&format!("无法解析文件: {:?}", err))
            }
            ScriptError::FailedConnecntWebsocket(err) => {
                formatter.write_str(&format!("无法建立 Websocket 链接: {:?}", err))
            }
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
