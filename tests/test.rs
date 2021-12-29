use std::{process, sync::Arc, thread};

use draw_script::{
    cookie::CookieList,
    init,
    paintboard::{ColorArray, PaintBoard, TargetList},
    Config,
};

#[test]
fn test() {
    pretty_env_logger::init();
    let config = Arc::new(
        Config::new("config.toml".to_string()).unwrap_or_else(|err| {
            panic!("Error parsing the config file: {}", err);
        }),
    );
    let cookie_list = CookieList::new(
        init::get_cookie_from_dir(&config.cookie_dir).unwrap_or_else(|err| {
            panic!("Error getting cookies: {}", err);
        }),
    );
    let paint_board = PaintBoard {
        color: ColorArray::new(Arc::clone(&config)),
        targets: TargetList::new(
            Arc::clone(&config),
            init::get_node(&config.node_file).unwrap_or_else(|err| {
                panic!("Error getting nodes: {}", err);
            }),
        ),
    };
    let paint_board = Arc::new(paint_board);
    let monitor = Arc::clone(&paint_board);
    thread::spawn(move || {
        PaintBoard::start_daemon_arc(paint_board, Arc::from(cookie_list), Arc::clone(&config))
    });

    loop {
        if monitor.targets.queue_empty() {
            eprintln!("Test complete!");
            process::exit(0);
        }
    }
}
