use std::{collections::VecDeque, sync::Arc};

use draw_script::{
    cookie::{Cookie, CookieList, RawCookie},
    node::NodeOpt,
    paintboard::{ColorArray, PaintBoard, TargetList},
    Config,
};
use rand::Rng;

fn generate_cookie_list() -> CookieList {
    let mut list = VecDeque::new();
    for i in 0..10 {
        list.push_back({
            Cookie::new(RawCookie {
                cookie: format!("_uid={};__client_id={}", i, i),
            })
        });
    }
    CookieList::new(list)
}

fn generate_nodes() -> VecDeque<NodeOpt> {
    let mut list = VecDeque::new();
    for x in 0..10 {
        for y in 0..10 {
            list.push_back(NodeOpt {
                x: x,
                y: y,
                color: rand::thread_rng().gen_range(0..32),
            })
        }
    }
    list
}

#[test]
fn test() {
    // use local board server https://github.com/ouuan/fake-luogu-paintboard-server
    // do not forget to set the cd to 0 before starting the server
    let config = Arc::new(Config {
        board_addr: "http://localhost:3000/paintBoard".to_string(),
        websocket_addr: "ws://localhost:4000/ws".to_string(),
        cookie_dir: "".to_string(),
        node_file: "".to_string(),
        wait_time: 0,
        thread_num: 8,
        board_width: 1000,
        board_height: 600,
        node_retry_times: 50,
    });
    let cookie_list = generate_cookie_list();
    let paint_board = PaintBoard {
        color: ColorArray::new(Arc::clone(&config)),
        targets: TargetList::new(generate_nodes()),
    };
    paint_board.start_daemon(Arc::from(cookie_list), Arc::clone(&config));
}
