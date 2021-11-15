use super::structure::{AnnotatedConstruct, ConstructId, Environment};
use crate::{
    stage1::structure::Module,
    stage2::construct::{constructs::CUnresolved, Construct},
};

pub fn ingest<'x>(src: &'x Module<'x>) -> (Environment<'x>, ConstructId<'x>) {
    let mut env = Environment::new();
    let root = env.push_item(AnnotatedConstruct::<'x> {
        cached_reduction: None,
        definition: Some(Box::new(CUnresolved(src.self_content.clone()))),
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
