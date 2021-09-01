use draw_script::cookie::{Cookie, CookieList, RawCookie};
use draw_script::node::NodeOpt;
use draw_script::paintboard::{ColorArray, PaintBoard, TargetList};
use draw_script::Config;
use draw_script::ScriptError;

use std::collections::VecDeque;
use std::process;
use std::sync::Arc;

fn get_cookie_from_dir<T>(dir: &T) -> Result<VecDeque<Cookie>, ScriptError>
where
    T: AsRef<std::path::Path>,
{
    let mut queue = VecDeque::new();
    let cookies = std::fs::read_dir(dir.as_ref())?;
    for cookie in cookies {
        let content = std::fs::read_to_string(cookie?.path())?;
        let cookie: RawCookie = serde_json::from_str(&content)?;
        queue.push_back(Cookie::new(cookie));
    }
    Ok(queue)
}

fn get_node<T>(file: &T) -> Result<VecDeque<NodeOpt>, ScriptError>
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
    let config = Arc::new(
        Config::new("config.toml".to_string()).unwrap_or_else(|err| {
            eprintln!("Error parsing the config file: {}", err);
            process::exit(1);
        }),
    );
    let cookie_list = CookieList::new(get_cookie_from_dir(&config.cookie_dir).unwrap_or_else(
        |err| {
            eprintln!("Error getting cookies: {}", err);
            process::exit(1);
        },
    ));
    let paint_board = PaintBoard {
        color: ColorArray::new(),
        targets: TargetList::new(get_node(&config.node_file).unwrap_or_else(|err| {
            eprintln!("Error getting nodes: {}", err);
            process::exit(1);
        })),
    };
    paint_board.start_daemon(Arc::from(cookie_list), Arc::clone(&config));
}
