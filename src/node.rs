use super::paintboard::PaintBoard;
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

#[derive(Deserialize)]
/// Luogu 返回的状态
pub struct Status {
    status: u32,
}

impl NodeOpt {
    pub fn check(&self, paint_board: &PaintBoard) -> bool {
        paint_board.check(self.x, self.y, self.color)
    }

    pub fn update(&self, cookies: String, config: &Config) -> Result<(), ScriptError> {
        let mut headers = HeaderMap::new();
        headers.insert(header::REFERER, config.board_addr.parse().unwrap());
        headers.insert(header::COOKIE, cookies.parse().unwrap());
        let client = reqwest::blocking::Client::new();
        let mut params = std::collections::HashMap::new();
        params.insert("x", self.x.to_string());
        params.insert("y", self.y.to_string());
        params.insert("color", self.color.to_string());
        let rep = client
            .post(&format!("{}/paint", config.board_addr))
            .form(&params)
            .headers(headers)
            .send()
            .unwrap();
        let rep_content = rep.text()?;
        log::debug!("{:?} send to server, get {}", params, rep_content);
        let status: Status = serde_json::from_str(&rep_content)?;
        let status = status.status;
        if status == 401 {
            log::warn!("{} is logouted", cookies);
        }
        Ok(())
    }
}
