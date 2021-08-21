use crate::BOARD_ADDR;
use crate::WEBSOCKET_ADDR;

use crate::from_32;

use crate::cookies::CookiesList;
use crate::node::NodeOpt;
use crate::ScriptError;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use reqwest::header;
use reqwest::header::HeaderMap;

static WAIT_TIME: u64 = 30;

/// 画板
pub struct PaintBoard {
    pub color: Arc<Mutex<Vec<Vec<usize>>>>,
    pub gol_color: Arc<Mutex<VecDeque<NodeOpt>>>,
    pub wait_check: Arc<Mutex<VecDeque<(NodeOpt, std::time::Instant)>>>,
}

/// 获取画板状态
pub fn get_board() -> String {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::REFERER,
        "https://www.luogu.com.cn/paintBoard".parse().unwrap(),
    );
    let client = reqwest::blocking::Client::new();
    let rep = client
        .get(&format!("{}/paintBoard/board", BOARD_ADDR))
        .send()
        .unwrap();
    rep.text().unwrap()
}

impl CookiesList {
    pub fn get_cookie(&self) -> String {
        let mut list = self.cookies.lock().unwrap();
        let mut cur_cookie = list.pop_front().unwrap();
        let res = cur_cookie.cookies.clone();
        while std::time::Instant::now() - cur_cookie.last_time
            <= std::time::Duration::from_secs(WAIT_TIME)
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
    pub fn start_daemon(self, cookies_list: Arc<CookiesList>) {
        //use tokio::runtime::Runtime;
        let board = Arc::from(self);

        let handle_ws;
        let handle_board;
        {
            let board = board.clone();
            handle_ws = std::thread::spawn(move || loop {
                if let Err(err) = board.websocket_daemon() {
                    log::error!("{:?}", err);
                }
            });
        }
        {
            let board = board.clone();
            handle_board = std::thread::spawn(move || {
                log::info!("Start auto refresh daemon");
                loop {
                    if let Err(err) = board.refresh_board() {
                        log::error!("Failed refresh board: {:?}", err);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(120));
                }
            });
        }
        for i in 0..4 {
            let board = board.clone();
            let cookies_list = cookies_list.clone();
            std::thread::spawn(move || {
                log::info!("Thread {} started", i);
                loop {
                    let cookies = cookies_list.get_cookie();
                    if let Some(opt) = board.get_update() {
                        log::info!("Thread {}: get work {:?}", i, opt);
                        if let Err(err) = opt.update(cookies) {
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
    fn websocket_daemon(&self) -> Result<(), ScriptError> {
        use websocket::{ClientBuilder, Message};
        let mut client = ClientBuilder::new(WEBSOCKET_ADDR).unwrap();
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
    fn refresh_board(&self) -> Result<(), ScriptError> {
        let raw_board = get_board();
        let mut color = self.color.lock().unwrap();
        for (i, line) in raw_board.lines().enumerate() {
            for (j, chr) in line.chars().enumerate() {
                color[i][j] = from_32(chr);
            }
        }
        Ok(())
    }
}
