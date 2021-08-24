pub mod config;
pub mod cookies;
pub mod node;
pub mod paintboard;

pub use self::config::*;

use std::convert::TryFrom;

pub enum ScriptError {
    FailedReadFile(std::io::Error),
    FailedReadConfig(toml::de::Error),
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

impl From<toml::de::Error> for ScriptError {
    fn from(error: toml::de::Error) -> Self {
        ScriptError::FailedReadConfig(error)
    }
}

impl std::fmt::Debug for ScriptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScriptError::FailedReadFile(err) => {
                formatter.write_str(&format!("无法访问文件: {:?}", err))
            }
            ScriptError::FailedReadConfig(err) => {
                formatter.write_str(&format!("读取配置文件失败: {:?}", err))
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
