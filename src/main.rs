use draw_script::{Cookies, CookiesList, NodeOpt, PaintBoard, RawCookies};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

static COLOR: [usize; 18] = [
    28, 28, 1, 1, 13, 13, 8, 8, 26, 26, 22, 22, 10, 10, 11, 11, 17, 17,
];

fn main() {
    pretty_env_logger::init();
    let mut queue = VecDeque::new();
    let files = std::fs::read_dir("/home/woshiluo/data").unwrap();
    for file in files {
        let file_content = std::fs::read_to_string(file.unwrap().path()).unwrap();
        let cookies: RawCookies = serde_json::from_str(&file_content).unwrap();
        queue.push_back(Cookies {
            cookies: cookies.cookie,
            last_time: std::time::Instant::now(),
        });
    }
    let cookies_list = CookiesList {
        cookies: Arc::from(Mutex::from(queue)),
    };
    let mut queue = VecDeque::new();
    for i in 0..18 {
        for j in 0..600 {
            queue.push_back(NodeOpt {
                x: i,
                y: j,
                color: COLOR[i],
            });
        }
    }
    let paint_board = PaintBoard {
        color: Arc::from(Mutex::from(vec![vec![1; 600]; 1000])),
        gol_color: Arc::from(Mutex::from(queue)),
        wait_check: Arc::from(Mutex::from(VecDeque::new())),
    };
    paint_board.start_daemon(Arc::from(cookies_list));
}
