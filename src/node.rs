use super::{Config, ScriptError};

use serde::{Deserialize, Serialize};

use reqwest::header;
use reqwest::header::HeaderMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// 单个点的信息
/// (x,y) = color
pub struct NodeOpt {
    pub x: usize,
    pub y: usize,
    pub color: usize,
}

// #[derive(Deserialize)]
// /// Luogu 返回的状态
// pub struct Status {
//     status: u32,
// }

impl NodeOpt {
    pub fn update(&self, cookies: &str, config: &Config) -> Result<(), ScriptError> {
        let mut headers = HeaderMap::new();
        headers.insert(header::REFERER, config.board_addr.parse().unwrap());
        // headers.insert(header::COOKIE, cookies.parse().unwrap());
        let client = reqwest::blocking::Client::new();
        let cookies = cookies.replace(":", "%3A");
        let mut params = std::collections::HashMap::new();
        params.insert("x", self.x.to_string());
        params.insert("y", self.y.to_string());
        params.insert("color", self.color.to_string());
        let rep = client
            .post(&format!("{}/paint?token={}", config.board_addr, cookies))
            .headers(headers)
            .form(&params)
            .send()
            .unwrap();
        let rep_content = rep.text()?;
        log::debug!("{:?} send to server, get {}", params, rep_content);

        if rep_content == "[]" {
            return Ok(());
        }

        log::warn!("Request failed {}", rep_content);
        Err(ScriptError::FailedRequest)
    }
}
