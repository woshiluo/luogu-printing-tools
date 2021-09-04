use super::Config;

use std::collections::VecDeque;
use std::sync::Mutex;

use serde::Deserialize;

#[derive(Deserialize)]
/// 原始 Cookie
pub struct RawCookie {
    pub cookie: String,
}

#[derive(Debug)]
/// 提供了时间检测的 Cookie
pub struct Cookie {
    cookie: String,
    last_update: std::time::Instant,
}

impl Cookie {
    pub fn new(raw_cookie: RawCookie) -> Cookie {
        Cookie {
            cookie: raw_cookie.cookie,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn cookie(&self) -> &str {
        &self.cookie
    }
    pub fn last_update(&self) -> std::time::Instant {
        self.last_update
    }

    pub fn update(&mut self) {
        self.last_update = std::time::Instant::now();
    }
}

/// Cookies 列表
pub struct CookieList {
    list: Mutex<VecDeque<Cookie>>,
}

impl CookieList {
    pub fn new(list: VecDeque<Cookie>) -> CookieList {
        CookieList {
            list: Mutex::new(list),
        }
    }
    pub fn get_cookie(&self, config: &Config) -> String {
        let mut list = self.list.lock().unwrap();
        let mut cur_cookie = list.pop_front().unwrap();

        while std::time::Instant::now() - cur_cookie.last_update()
            <= std::time::Duration::from_secs(config.wait_time)
        {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        cur_cookie.update();
        let cookie = cur_cookie.cookie().to_string();
        list.push_back(cur_cookie);

        cookie
    }
    pub fn remove_cookie(&self, cookie: &str) {
        let mut list = self.list.lock().unwrap();
        for i in 0..list.len() {
            if list.get(i).unwrap().cookie == cookie {
                list.remove(i);
                break;
            }
        }
    }
}
