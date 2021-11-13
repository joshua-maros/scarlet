use super::structure::{Environment, Item, ItemId};
use crate::{stage1::structure::Module, stage2::structure::Definition};

pub fn ingest<'x>(src: &'x Module) -> (Environment<'x>, ItemId<'x>) {
    let mut env = Environment::new();
    let root = env.push_item(Item {
        cached_reduction: None,
        definition: Some(Definition::Unresolved(src.self_content.clone())),
        dependencies: None,
        parent_scope: None,
        shown_from: vec![],
    });

    let root = env.reduce(root);
    env.get_deps(root);

    let mut next_id = env.items.first();
    while let Some(id) = next_id {
        env.check(id);
        next_id = env.items.next(id);
    }
    (env, root)
}
