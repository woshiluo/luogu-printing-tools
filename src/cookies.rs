use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawCookies {
    pub cookie: String,
}

#[derive(Debug)]
pub struct Cookies {
    pub cookies: String,
    pub last_time: std::time::Instant,
}

pub struct CookiesList {
    pub cookies: Arc<Mutex<VecDeque<Cookies>>>,
}
