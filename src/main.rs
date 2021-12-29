use draw_script::cookie::CookieList;
use draw_script::init;
use draw_script::paintboard::{ColorArray, PaintBoard, TargetList};
use draw_script::Config;

use std::process;
use std::sync::Arc;

fn main() {
    pretty_env_logger::init();
    let config = Arc::new(
        Config::new("config.toml".to_string()).unwrap_or_else(|err| {
            eprintln!("Error parsing the config file: {}", err);
            process::exit(1);
        }),
    );
    let cookie_list = CookieList::new(
        init::get_cookie_from_dir(&config.cookie_dir).unwrap_or_else(|err| {
            eprintln!("Error getting cookies: {}", err);
            process::exit(1);
        }),
    );
    let paint_board = PaintBoard {
        color: ColorArray::new(Arc::clone(&config)),
        targets: TargetList::new(
            Arc::clone(&config),
            init::get_node(&config.node_file).unwrap_or_else(|err| {
                eprintln!("Error getting nodes: {}", err);
                process::exit(1);
            }),
        ),
    };
    paint_board.start_daemon(Arc::from(cookie_list), Arc::clone(&config));
}
