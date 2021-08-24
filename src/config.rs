use super::ScriptError;

use serde::{Deserialize, Serialize};

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
        Ok(config)
    }
}
