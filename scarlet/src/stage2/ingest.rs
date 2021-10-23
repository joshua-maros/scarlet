mod from_tree;
mod top_level;
mod util;

use super::{
    flatten,
    structure::{Environment, ItemId},
};
use crate::stage1::structure::Module;

pub fn ingest<'x>(src: &'x Module) -> (Environment<'x>, ItemId<'x>) {
    let (mut env, root) = top_level::ingest(src);
    println!("{:#?}", env);
    env.show_all();
    flatten::flatten(&mut env);
    let root = env.reduce(root);
    env.get_deps(root);
    (env, root)
}
