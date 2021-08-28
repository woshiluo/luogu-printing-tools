use crate::from_32;

use crate::cookie::CookieList;
use crate::node::NodeOpt;
use crate::Config;

use std::collections::VecDeque;
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

    pub fn get_target(&self, paint_board: &PaintBoard) -> NodeOpt {
        let targets = self.targets.lock().unwrap();
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let mut pos = rng.gen_range(0..targets.len());
        loop {
            // TODO: 可以使用线段树二分来优化，是否有必要?

            // TODO: this var should from config file
            for _i in 0..50 {
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
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

pub struct ColorArray {
    array: Mutex<Vec<Vec<usize>>>,
}

impl ColorArray {
    // TODO: set by config
    pub fn new() -> ColorArray {
        ColorArray {
            array: Mutex::from(vec![vec![1; 600]; 1000]),
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

// TODO: Refactor
impl PaintBoard {
    /// 测试指定点颜色
    pub fn get_update(&self) -> NodeOpt {
        log::debug!("Start to get work{:?}", std::time::Instant::now());
        self.targets.get_target(self)
    }
    pub fn start_daemon(self, cookie_list: Arc<CookieList>, config: Arc<Config>) {
        //use tokio::runtime::Runtime;
        let board = Arc::from(self);

        let handle_board;
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
            let cookie_list = cookie_list.clone();
            let config = Arc::clone(&config);
            std::thread::spawn(move || {
                log::info!("Thread {} started", i);
                loop {
                    let cookies = cookie_list.get_cookie(&config);
                    let opt = board.get_update();
                    if let Err(err) = opt.update(cookies, &config) {
                        log::error!("Failed paint: {}", err);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });
        }
        handle_board.join().unwrap();
    }
    fn refresh_board(&self, config: &Config) {
        let raw_board = get_board(config);
        match raw_board {
            None => {
                log::error!("Failed to refresh board!");
                ()
            } // just log and skip if the process failed to get board from remote server
            Some(raw_board) => {
                for (i, line) in raw_board.lines().enumerate() {
                    for (j, chr) in line.chars().enumerate() {
                        self.color.set_color(i, j, from_32(chr));
                    }
                }
                ()
            }
        }
    }
}
