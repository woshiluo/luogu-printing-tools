use crate::from_32;

use crate::cookie::CookieList;
use crate::node::NodeOpt;
use crate::Config;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use reqwest::header;
use reqwest::header::HeaderMap;

// TODO: use Option
const FAILED_COLOR: usize = 63;

pub struct TargetList {
    targets: Mutex<VecDeque<NodeOpt>>,
    array: ColorArray,
}

impl TargetList {
    pub fn new(config: Arc<Config>, list: VecDeque<NodeOpt>) -> TargetList {
        let array = ColorArray::new(config.clone());
        for i in 0..config.board_width {
            for j in 0..config.board_height {
                array.set_color(i, j, FAILED_COLOR);
            }
        }
        for node in &list {
            array.set_color(node.x, node.y, node.color);
        }
        TargetList {
            targets: Mutex::new(VecDeque::from(list)),
            array,
        }
    }

    pub fn get_target(&self, paint_board: &PaintBoard) -> NodeOpt {
        loop {
            // 避免 targets 堵塞，只在查找时 lock
            {
                let mut targets = self.targets.lock().unwrap();
                while targets.len() > 0 && targets.front().unwrap().check(paint_board) {
                    targets.pop_front();
                }
                if targets.len() > 0 {
                    let node = targets.front().unwrap().clone();
                    targets.pop_front();
                    return node;
                }
            }

            log::info!("There is nothing to do.");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    pub fn color(&self, x: usize, y: usize) -> usize {
        self.array.color(x, y)
    }
    pub fn add_list(&self, x: usize, y: usize) {
        let mut targets = self.targets.lock().unwrap();
        targets.push_back(NodeOpt {
            x,
            y,
            color: self.array.color(x, y),
        });
    }
}

pub struct ColorArray {
    array: Mutex<Vec<Vec<usize>>>,
}

impl ColorArray {
    pub fn new(config: Arc<Config>) -> ColorArray {
        ColorArray {
            array: Mutex::from(vec![vec![1; config.board_height]; config.board_width]),
        }
    }

    pub fn color(&self, x: usize, y: usize) -> usize {
        self.array.lock().unwrap()[x][y]
    }
    pub fn set_color(&self, x: usize, y: usize, color: usize) {
        self.array.lock().unwrap()[x][y] = color
    }
}

/// 画板
pub struct PaintBoard {
    pub color: ColorArray,
    pub targets: TargetList,
}

/// 获取画板状态
pub fn get_board(config: &Config) -> Option<String> {
    let mut headers = HeaderMap::new();
    headers.insert(header::REFERER, config.board_addr.parse().unwrap());
    let client = reqwest::blocking::Client::new();
    // try 3 times to send request
    for i in 0..3 {
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

impl PaintBoard {
    pub fn get_update(&self) -> NodeOpt {
        log::debug!("Start to get work{:?}", std::time::Instant::now());
        self.targets.get_target(self)
    }
    pub fn check(&self, x: usize, y: usize) -> bool {
        let target = self.targets.color(x, y);
        return target == FAILED_COLOR || target == self.color.color(x, y);
    }
    pub fn set_color(&self, x: usize, y: usize, color: usize) {
        self.color.set_color(x, y, color);
        if !self.check(x, y) {
            self.targets.add_list(x, y);
        }
    }
    pub fn start_daemon(self, cookie_list: Arc<CookieList>, config: Arc<Config>) {
        use threadpool::ThreadPool;
        let board = Arc::from(self);
        let pool = ThreadPool::new(config.thread_num);

        {
            let board = board.clone();
            let config = config.clone();
            pool.execute(move || {
                log::info!("Start auto refresh daemon");
                loop {
                    board.refresh_board(&config);
                    std::thread::sleep(std::time::Duration::from_secs(120));
                }
            });
        }
        {
            let board = board.clone();
            let config = config.clone();
            pool.execute(move || {
                log::info!("Start websocket update daemon");
                use websocket::{ClientBuilder, Message};
                // TODO: What to do if init connect failed?
                let mut client = ClientBuilder::new(&config.websocket_addr).unwrap();
                let mut client = client.connect(None).unwrap();
                client
                    .send_message(&Message::text(
                        "{\"type\":\"join_channel\",\"channel\":\"paintboard\"}",
                    ))
                    .unwrap();
                log::info!("Websocket conn est, wait for messages");
                let mut first_req = false;
                for message in client.incoming_messages() {
                    if first_req == false {
                        first_req = true;
                        continue;
                    }
                    log::trace!("Update recv: {:?}", message);
                    match message {
                        Ok(message) => {
                            if let websocket::OwnedMessage::Text(message) = message {
                                if let Ok(update) = serde_json::from_str::<NodeOpt>(&message) {
                                    board.set_color(update.x, update.y, update.color);
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Failed Recv websocket: {:?}", err);
                        }
                    }
                }
            });
        }
        loop {
            let cookie_list = cookie_list.clone();
            let board = board.clone();
            let config = config.clone();
            while pool.max_count() <= pool.active_count() {
                // TODO: Set with config
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            pool.execute(move || {
                use crate::ScriptError;
                let opt = board.get_update();
                let cookie = cookie_list.get_cookie(&config);
                if let Err(err) = opt.update(&cookie, &config) {
                    board.set_color(opt.x, opt.y, FAILED_COLOR);
                    if let ScriptError::CookieOutdated = err {
                        cookie_list.remove_cookie(&cookie);
                    }
                    log::warn!("Failed update node {}", err);
                }
            });
        }
    }
    fn refresh_board(&self, config: &Config) {
        let raw_board = get_board(config);
        match raw_board {
            None => {
                log::error!("Failed to refresh board!");
            } // just log and skip if the process failed to get board from remote server
            Some(raw_board) => {
                for (i, line) in raw_board.lines().enumerate() {
                    for (j, chr) in line.chars().enumerate() {
                        self.set_color(i, j, from_32(chr));
                    }
                }
            }
        }
    }
}
