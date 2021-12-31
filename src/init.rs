use std::collections::VecDeque;

use crate::{
    cookie::{Cookie, RawCookie},
    node::NodeOpt,
    ScriptError,
};

pub fn get_cookie_from_dir<T>(dir: &T) -> Result<VecDeque<Cookie>, ScriptError>
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

pub fn get_node<T>(file: &T) -> Result<VecDeque<NodeOpt>, ScriptError>
where
    T: AsRef<std::path::Path>,
{
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    let mut queue = VecDeque::new();
    let mut dot_draw: Vec<[usize; 3]> =
        serde_json::from_str(&std::fs::read_to_string(file.as_ref())?)?;
    // dot_draw.shuffle(&mut rng);

    for node in dot_draw {
        queue.push_back(NodeOpt {
            x: node[0],
            y: node[1],
            color: node[2],
        });
    }
    Ok(queue)
}
