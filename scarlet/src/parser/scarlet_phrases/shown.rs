use crate::{
    diagnostic::Diagnostic,
    environment::{vomit::VomitContext, Environment},
    item::ItemPtr,
    parser::{
        phrase::{Phrase, UncreateResult},
        Node, NodeChild, ParseContext,
    },
    phrase,
    scope::Scope,
};

fn create(
    pc: &ParseContext,
    env: &mut Environment,
    scope: Box<dyn Scope>,
    node: &Node,
) -> Result<ItemPtr, Diagnostic> {
    assert_eq!(node.children.len(), 2);
    assert_eq!(node.children[1], NodeChild::Text("SHOWN"));
    let base = node.children[0].as_construct_dyn_scope(pc, env, scope)?;
    base.borrow_mut().show = true;
    Ok(base)
}

fn uncreate<'a>(
    _env: &mut Environment,
    _ctx: &mut VomitContext<'a, '_>,
    _uncreate: ItemPtr,
) -> UncreateResult<'a> {
    Ok(None)
}

fn vomit(pc: &ParseContext, src: &Node) -> String {
    format!("{} SHOWN", src.children[0].vomit(pc))
}

pub fn phrase() -> Phrase {
    phrase!(
        "shown",
        128, 128,
        Some((create, uncreate)),
        vomit,
        4 => 4, r"\bSHOWN\b"
    )
}
