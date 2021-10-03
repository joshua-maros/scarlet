use self::context::Context;
use super::structure::Environment;
use crate::stage3::ingest::dereference::ItemBeingDereferenced;

mod context;
mod dereference;
// mod ingest_structures;

pub fn ingest(
    input: &mut crate::stage2::structure::Environment,
    root: crate::stage2::structure::Item,
) -> Environment {
    let mut env = Environment::new();
    let mut ctx = Context::new(input, &mut env);
    let result = ctx.dereference(ItemBeingDereferenced::from(root));
    println!("{:?}\nbecomes\n{:#?}\n", root, result);
    env
}
