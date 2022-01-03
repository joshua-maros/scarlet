use itertools::Itertools;

use crate::{
    parser::{phrase::Phrase, util::collect_comma_list, Node, NodeChild, ParseContext},
    phrase,
    shared::indented,
};

fn vomit(pc: &ParseContext, src: &Node) -> String {
    let child = NodeChild::Node(src.clone());
    let list = collect_comma_list(&child);
    let list = list.into_iter().map(|entry| entry.vomit(pc)).collect_vec();
    if list.iter().map(|x| x.len() + 2).sum::<usize>() >= 40 {
        let mut result = String::new();
        for entry in list {
            result.push_str(&format!("\n    {}", indented(&entry)));
        }
        result.push_str("\n");
        result
    } else {
        list.join("  ")
    }
}

pub fn phrase() -> Phrase {
    phrase!(
        "multiple constructs",
        None,
        vomit,
        255 => 255, r",", 255
    )
}
