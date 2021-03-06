use super::ScriptError;
use super::UrlError;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub board_addr: String,
    pub websocket_addr: String,
    pub cookie_dir: String,
    pub node_file: String,
    pub wait_time: u64,
    pub thread_num: usize,
    pub board_width: usize,
    pub board_height: usize,
}

impl Config {
    fn from_toml(raw_config: &str) -> Result<Config, ScriptError> {
        let config: Config = toml::from_str(raw_config)?;
        Ok(config)
    }
    fn check(&self) -> Result<(), ScriptError> {
        // check if the board_addr and websocket_addr is what we are expected
        // to avoid fill in a http URL in websocket_addr
        let board_addr = Url::parse(&self.board_addr)?;
        let websocket_addr = Url::parse(&self.websocket_addr)?;
        if board_addr.scheme() != "http" && board_addr.scheme() != "https" {
            return Err(ScriptError::UnexpectedUrl(UrlError::InvalidHTTPUrl));
        }
        if websocket_addr.scheme() != "ws" && websocket_addr.scheme() != "wss" {
            return Err(ScriptError::UnexpectedUrl(UrlError::InvalidWSUrl));
        }
        Ok(())
    }

    pub fn new<T>(filename: T) -> Result<Config, ScriptError>
    where
        T: AsRef<std::path::Path>,
    {
        let config = Config::from_toml(&std::fs::read_to_string(&filename.as_ref())?)?;
        config.check()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_board_address() {
        let config = Config {
            board_addr: "ws://qwq.com".to_string(),
            websocket_addr: "wss://qwq.com".to_string(),
            cookie_dir: "/home".to_string(),
            node_file: "/home/node.file".to_string(),
            wait_time: 30,
            thread_num: 8,
            board_width: 1000,
            board_height: 600,
        };

        if let Err(ScriptError::UnexpectedUrl(UrlError::InvalidHTTPUrl)) = config.check() {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn check_websocket_address() {
        let config = Config {
            board_addr: "http://qwq.com".to_string(),
            websocket_addr: "https://qwq.com".to_string(),
            cookie_dir: "/home".to_string(),
            node_file: "/home/node.file".to_string(),
            wait_time: 30,
            thread_num: 8,
            board_width: 1000,
            board_height: 600,
        };

        if let Err(ScriptError::UnexpectedUrl(UrlError::InvalidWSUrl)) = config.check() {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
