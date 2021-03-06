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
    assert_eq!(node.children.len(), 5);
    assert_eq!(node.children[1], NodeChild::Text("AS_LANGUAGE_ITEM"));
    assert_eq!(node.children[2], NodeChild::Text("("));
    assert_eq!(node.children[4], NodeChild::Text(")"));
    let base = node.children[0].as_construct_dyn_scope(pc, env, scope)?;
    let name = node.children[3].as_node().as_ident()?;
    env.define_language_item(name, base.ptr_clone());
    Ok(base)
}

fn uncreate<'a>(
    _env: &mut Environment,
    _ctx: &mut VomitContext<'a, '_>,
    _uncreate: ItemPtr,
) -> UncreateResult<'a> {
    Ok(None)
}

fn vomit(_pc: &ParseContext, src: &Node) -> String {
    format!("{:#?}", src)
}

pub fn phrase() -> Phrase {
    phrase!(
        "as language item",
        128, 128,
        Some((create, uncreate)),
        vomit,
        4 => 4, r"AS_LANGUAGE_ITEM", r"\(", 255, r"\)"
    )
}
