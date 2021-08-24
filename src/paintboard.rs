use crate::from_32;

use crate::cookies::CookiesList;
use crate::node::NodeOpt;
use crate::Config;
use crate::ScriptError;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use reqwest::header;
use reqwest::header::HeaderMap;

/// 画板
pub struct PaintBoard {
    pub color: Arc<Mutex<Vec<Vec<usize>>>>,
    pub gol_color: Arc<Mutex<VecDeque<NodeOpt>>>,
    pub wait_check: Arc<Mutex<VecDeque<(NodeOpt, std::time::Instant)>>>,
}

/// 获取画板状态
pub fn get_board(config: &Config) -> Option<String> {
    let mut headers = HeaderMap::new();
    headers.insert(header::REFERER, config.board_addr.parse().unwrap());
    let client = reqwest::blocking::Client::new();
    // try 3 times to send request
    for i in 1..4 {
        let rep = client.get(&format!("{}/board", config.board_addr)).send();
        match rep {
            Ok(res) => {
                return Some(res.text().unwrap());
            }
            Err(_err) => {
                log::warn!("Get board failed! {} retries remaining.", 3 - i);
            }
        }
    }
    log::error!("All retries to get board failed!");
    None
}

impl CookiesList {
    pub fn get_cookie(&self, wait_time: u64) -> String {
        let mut list = self.cookies.lock().unwrap();
        let mut cur_cookie = list.pop_front().unwrap();
        let res = cur_cookie.cookies.clone();
        while std::time::Instant::now() - cur_cookie.last_time
            <= std::time::Duration::from_secs(wait_time)
        {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        cur_cookie.last_time = std::time::Instant::now();
        list.push_back(cur_cookie);
        res
    }
}

// TODO: Refactor
impl PaintBoard {
    /// 测试指定点颜色
    pub fn check(&self, opt: &NodeOpt) -> bool {
        self.color.lock().unwrap()[opt.x][opt.y] == opt.color
    }
    pub fn get_update(&self) -> Option<NodeOpt> {
        log::debug!("Start to get work{:?}", std::time::Instant::now());
        let mut queue = self.gol_color.lock().unwrap();
        let mut wait_check = self.wait_check.lock().unwrap();
        while wait_check.is_empty() == false {
            let time = wait_check.front().unwrap().1;
            if std::time::Instant::now() - time >= std::time::Duration::from_secs(5) {
                let cur = wait_check.pop_front().unwrap().0;
                queue.push_back(cur);
            } else if queue.len() != 0 {
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let cur = rng.gen_range(0..2);
        let mut cnt = 0;
        let tot = queue.len();
        loop {
            if cnt > tot {
                break;
            }
            cnt += 1;
            if cur == 0 {
                let front = queue.pop_front().unwrap();
                if self.check(&front) == false {
                    wait_check.push_back((front.clone(), std::time::Instant::now()));
                    return Some(front);
                } else {
                    queue.push_back(front);
                }
            } else {
                let back = queue.pop_back().unwrap();
                if self.check(&back) == false {
                    wait_check.push_back((back.clone(), std::time::Instant::now()));
                    return Some(back);
                } else {
                    queue.push_front(back);
                }
            }
        }
        None
    }
    pub fn start_daemon(self, cookies_list: Arc<CookiesList>, config: Arc<Config>) {
        //use tokio::runtime::Runtime;
        let board = Arc::from(self);

        let handle_ws;
        let handle_board;
        {
            let board = board.clone();
            let config = Arc::clone(&config);
            handle_ws = std::thread::spawn(move || loop {
                if let Err(err) = board.websocket_daemon(&config) {
                    log::error!("{:?}", err);
                }
            });
        }
        {
            let board = board.clone();
            let config = Arc::clone(&config);
            handle_board = std::thread::spawn(move || {
                log::info!("Start auto refresh daemon");
                loop {
                    board.refresh_board(&config);
                    std::thread::sleep(std::time::Duration::from_secs(120));
                }
            });
        }
        for i in 0..4 {
            let board = board.clone();
            let cookies_list = cookies_list.clone();
            let config = Arc::clone(&config);
            std::thread::spawn(move || {
                log::info!("Thread {} started", i);
                loop {
                    let cookies = cookies_list.get_cookie(config.wait_time);
                    if let Some(opt) = board.get_update() {
                        log::info!("Thread {}: get work {:?}", i, opt);
                        if let Err(err) = opt.update(cookies, &config) {
                            log::error!("Failed paint: {:?}", err);
                        }
                    } else {
                        log::info!("Thread {}: There is nothing to do", i);
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });
        }
        handle_board.join().unwrap();
        handle_ws.join().unwrap();
    }
    fn websocket_daemon(&self, config: &Config) -> Result<(), ScriptError> {
        use websocket::{ClientBuilder, Message};
        let mut client = ClientBuilder::new(&config.websocket_addr).unwrap();
        let mut client = client.connect_secure(None)?;
        client.send_message(&Message::text(
            "{\"type\":\"join_channel\",\"channel\":\"paintboard\"}",
        ))?;
        log::info!("Websocket conn est, wait a recv");
        let mut first_req = false;
        for message in client.incoming_messages() {
            if first_req == false {
                first_req = true;
                continue;
            }
            log::trace!("Update recv: {:?}", message);
            if let websocket::OwnedMessage::Text(message) = message? {
                if let Ok(update) = serde_json::from_str::<NodeOpt>(&message) {
                    self.color.lock().unwrap()[update.x][update.y] = update.color;
                }
            }
        }
        Ok(())
    }
    fn refresh_board(&self, config: &Config) {
        let raw_board = get_board(config);
        match raw_board {
            None => {
                log::error!("Failed to refresh board!");
                ()
            } // just log and skip if the process failed to get board from remote server
            Some(raw_board) => {
                let mut color = self.color.lock().unwrap();
                for (i, line) in raw_board.lines().enumerate() {
                    for (j, chr) in line.chars().enumerate() {
                        color[i][j] = from_32(chr);
                    }
                }
                ()
            }
        }
    }
}
