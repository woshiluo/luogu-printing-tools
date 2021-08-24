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
}

impl Config {
    pub fn new<T>(filename: T) -> Result<Config, ScriptError>
    where
        T: AsRef<std::path::Path>,
    {
        let config: Config = toml::from_str(&std::fs::read_to_string(&filename.as_ref())?)?;
        // check if the board_addr and websocket_addr is what we are expected
        // to avoid fill in a http URL in websocket_addr
        let board_addr = Url::parse(&config.board_addr)?;
        let websocket_addr = Url::parse(&config.websocket_addr)?;
        if board_addr.scheme() != "http" && board_addr.scheme() != "https" {
            return Err(ScriptError::UnexpectedUrl(UrlError::InvalidHTTPUrl));
        }
        if websocket_addr.scheme() != "ws" && websocket_addr.scheme() != "wss" {
            return Err(ScriptError::UnexpectedUrl(UrlError::InvalidWSUrl));
        }
        Ok(config)
    }
}
