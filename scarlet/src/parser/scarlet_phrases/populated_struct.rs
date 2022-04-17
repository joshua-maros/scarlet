use typed_arena::Arena;

use crate::{
    environment::{vomit::VomitContext, Environment},
    item::{
        definitions::structt::{DPopulatedStruct, SField, SFieldAndRest},
        ItemDefinition, ItemPtr,
    },
    parser::{
        phrase::{Phrase, UncreateResult},
        util, Node, NodeChild, ParseContext,
    },
    phrase,
    scope::Scope,
};

fn create(pc: &ParseContext, env: &mut Environment, scope: Box<dyn Scope>, node: &Node) -> ItemPtr {
    assert!(node.children.len() == 4);
    assert_eq!(node.children[0], NodeChild::Text("POPULATED_STRUCT"));
    assert_eq!(node.children[1], NodeChild::Text("["));
    assert_eq!(node.children[3], NodeChild::Text("]"));
    let args = util::collect_comma_list(&node.children[2]);
    assert_eq!(args.len(), 3);
    let this = crate::item::Item::placeholder_with_scope(scope);

    let label = args[0].as_ident().to_owned();
    let value = args[1].as_construct(pc, env, SFieldAndRest(this));
    let rest = args[2].as_construct(pc, env, SField(this));
    this.redefine(DPopulatedStruct::new(label, value, rest).clone_into_box());
    this
}

fn uncreate<'a>(
    env: &mut Environment,
    ctx: &mut VomitContext<'a, '_>,
    uncreate: ItemPtr,
) -> UncreateResult<'a> {
    Ok(None)
}

fn vomit(_pc: &ParseContext, src: &Node) -> String {
    format!("{:#?}", src)
}

pub fn phrase() -> Phrase {
    phrase!(
        "populated struct",
        128, 128,
        Some((create, uncreate)),
        vomit,
        0 => r"\bPOPULATED_STRUCT\b" , r"\[", 255, r"\]"
    )
}
