use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::Deserialize;

#[derive(Deserialize)]
/// 原始 Cookies
pub struct RawCookies {
    pub cookie: String,
}

#[derive(Debug)]
/// 提供了时间检测的 Cookies
/// TODO: 抽象
pub struct Cookies {
    pub cookies: String,
    pub last_time: std::time::Instant,
}

/// Cookies 列表
pub struct CookiesList {
    pub cookies: Arc<Mutex<VecDeque<Cookies>>>,
}
