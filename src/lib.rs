use serde::{Deserialize, Serialize};

use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use reqwest::header;
use reqwest::header::HeaderMap;

// static MAX_X: usize = 1000;
// static MAX_Y: usize = 600;
static WAIT_TIME: u64 = 30;

static BOARD_ADDR: &str = "https://www.luogu.com.cn";
static WEBSOCKET_ADDR: &str = "wss://ws.luogu.com.cn/ws";

pub fn to_32(cur: usize) -> char {
    let cur = u8::try_from(cur).unwrap();
    if cur <= 9 {
        return (b'0' + cur) as char;
    } else {
        return (cur - 9 + b'a') as char;
    }
}

pub fn from_32(cur: char) -> usize {
    cur.to_digit(33).unwrap() as usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeOpt {
    pub x: usize,
    pub y: usize,
    pub color: usize,
}

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

pub struct PaintBoard {
    pub color: Arc<Mutex<Vec<Vec<usize>>>>,
    pub gol_color: Arc<Mutex<VecDeque<NodeOpt>>>,
    pub wait_check: Arc<Mutex<VecDeque<(NodeOpt, std::time::Instant)>>>,
}

#[derive(Deserialize)]
pub struct Status {
    status: u32,
}

impl NodeOpt {
    pub fn update(&self, cookies: String) -> bool {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::REFERER,
            "https://www.luogu.com.cn/paintBoard".parse().unwrap(),
        );
        headers.insert(header::COOKIE, cookies.parse().unwrap());
        let client = reqwest::blocking::Client::new();
        let mut params = std::collections::HashMap::new();
        params.insert("x", self.x.to_string());
        params.insert("y", self.y.to_string());
        params.insert("color", self.color.to_string());
        let rep = client
            .post(&format!("{}/paintBoard/paint", BOARD_ADDR))
            .form(&params)
            .headers(headers)
            .send()
            .unwrap();
        let rep_content = rep.text().unwrap();
        log::debug!("{:?} send to server, get {}", params, rep_content);
        let status: Status = serde_json::from_str(&rep_content).unwrap();
        let status = status.status;
        if status == 401 {
            log::warn!("{} is logouted", cookies);
        }
        status == 200
    }
}

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
        let cur_cookie = list.pop_front().unwrap();
        let res = cur_cookie.cookies.clone();
        while std::time::Instant::now() - cur_cookie.last_time
            <= std::time::Duration::from_secs(WAIT_TIME)
        {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        list.push_back(cur_cookie);
        res
    }
}

impl PaintBoard {
    // TODO: Use random get.
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
                    queue.push_back(back);
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
            handle_ws = std::thread::spawn(move || {
                use websocket::{ClientBuilder, Message};
                let mut client = ClientBuilder::new(WEBSOCKET_ADDR).unwrap();
                let mut client = client.connect_secure(None).unwrap();
                client
                    .send_message(&Message::text(
                        "{\"type\":\"join_channel\",\"channel\":\"paintboard\"}",
                    ))
                    .unwrap();
                log::info!("Websocket conn est, wait a recv");
                let mut first_req = false;
                for message in client.incoming_messages() {
                    if first_req == false {
                        first_req = true;
                        continue;
                    }
                    log::trace!("Update recv: {:?}", message);
                    let board = board.clone();
                    std::thread::spawn(move || {
                        if message.is_err() {
                            return;
                        }
                        if let websocket::OwnedMessage::Text(message) = message.unwrap() {
                            let update: NodeOpt = serde_json::from_str(&message).unwrap();
                            board.color.lock().unwrap()[update.x][update.y] = update.color;
                        }
                    })
                    .join();
                }
            });
        }
        {
            let board = board.clone();
            handle_board = std::thread::spawn(move || {
                log::info!("Start auto refresh daemon");
                loop {
                    let raw_board = get_board();
                    let mut color = board.color.lock().unwrap();
                    let mut i = 0;
                    for line in raw_board.lines() {
                        let mut j = 0;
                        for chr in line.chars() {
                            color[i][j] = from_32(chr);
                            j += 1;
                        }
                        i += 1;
                    }
                    drop(color);
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
                        std::thread::spawn(move || {
                            opt.update(cookies);
                        })
                        .join();
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
}
