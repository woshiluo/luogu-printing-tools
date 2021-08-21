use draw_script::cookies::{Cookies, CookiesList, RawCookies};
use draw_script::node::NodeOpt;
use draw_script::paintboard::PaintBoard;
use draw_script::ScriptError;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn get_cookie_from_dir<T>(dir: T) -> Result<VecDeque<Cookies>, ScriptError>
where
    T: AsRef<std::path::Path>,
{
    let mut queue = VecDeque::new();
    let cookies = std::fs::read_dir(dir.as_ref())?;
    for cookie in cookies {
        let content = std::fs::read_to_string(cookie?.path())?;
        let cookies: RawCookies = serde_json::from_str(&content)?;
        queue.push_back(Cookies {
            cookies: cookies.cookie,
            last_time: std::time::Instant::now(),
        });
    }
    Ok(queue)
}

fn get_node<T>(file: T) -> Result<VecDeque<NodeOpt>, ScriptError>
where
    T: AsRef<std::path::Path>,
{
    let mut queue = VecDeque::new();
    let dot_draw: Vec<[usize; 3]> = serde_json::from_str(&std::fs::read_to_string(file.as_ref())?)?;
    for node in dot_draw {
        queue.push_back(NodeOpt {
            x: node[0],
            y: node[1],
            color: node[2],
        });
    }
    Ok(queue)
}

fn main() {
    pretty_env_logger::init();
    let cookies_list = CookiesList {
        cookies: Arc::from(Mutex::from(get_cookie_from_dir("/home/user/data").unwrap())),
    };
    let paint_board = PaintBoard {
        color: Arc::from(Mutex::from(vec![vec![1; 600]; 1000])),
        gol_color: Arc::from(Mutex::from(get_node("/opt/cp_editor").unwrap())),
        wait_check: Arc::from(Mutex::from(VecDeque::new())),
    };
    paint_board.start_daemon(Arc::from(cookies_list));
}
