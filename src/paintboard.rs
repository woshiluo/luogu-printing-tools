use crate::from_32;

use crate::cookie::CookieList;
use crate::node::NodeOpt;
use crate::Config;

use std::collections::VecDeque;
use std::process;
use std::sync::{Arc, Mutex};

use reqwest::header;
use reqwest::header::HeaderMap;

pub struct TargetList {
    targets: Mutex<VecDeque<NodeOpt>>,
}

impl TargetList {
    pub fn new(list: VecDeque<NodeOpt>) -> TargetList {
        TargetList {
            targets: Mutex::new(list),
        }
    }

    pub fn get_target(&self, config: &Config, paint_board: &PaintBoard) -> NodeOpt {
        let targets = self.targets.lock().unwrap();
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let mut pos = rng.gen_range(0..targets.len());
        loop {
            // TODO: 可以使用线段树二分来优化，是否有必要?

            for _i in 0..config.node_retry_times {
                if !targets[pos].check(paint_board) {
                    return targets[pos].clone();
                }
                pos = rng.gen_range(0..targets.len());
            }
            for i in 0..targets.len() {
                if !targets[i].check(paint_board) {
                    return targets[i].clone();
                }
            }

            log::info!("There is nothing to do.");

            #[cfg(test)]
            {
                eprintln!("Test complete!");
                process::exit(0);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
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
    pub fn get_update(&self, config: &Config) -> NodeOpt {
        log::debug!("Start to get work{:?}", std::time::Instant::now());
        self.targets.get_target(config, self)
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
                use tungstenite::{client, protocol::Message};
                // TODO: What to do if init connect failed?
                let mut client = client::connect(&config.websocket_addr).unwrap().0;
                client
                    .write_message(Message::text(
                        "{\"type\":\"join_channel\",\"channel\":\"paintboard\"}",
                    ))
                    .unwrap();
                log::info!("Websocket conn est, wait for messages");
                let mut first_req = false;
                loop {
                    let message = client.read_message();
                    if first_req == false {
                        first_req = true;
                        continue;
                    }
                    log::info!("Update recv: {:?}", message);
                    match message {
                        Ok(message) => {
                            if let Message::Text(message) = message {
                                if let Ok(update) = serde_json::from_str::<NodeOpt>(&message) {
                                    board.color.set_color(update.x, update.y, update.color);
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
                let opt = board.get_update(&config);
                let cookie = cookie_list.get_cookie(&config);
                if let Err(err) = opt.update(&cookie, &config) {
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
                        self.color.set_color(i, j, from_32(chr));
                    }
                }
            }
        }
    }
}
