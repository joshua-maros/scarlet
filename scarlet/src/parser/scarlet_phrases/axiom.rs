use crate::{
    diagnostic::Diagnostic,
    environment::{vomit::VomitContext, Environment},
    item::{definitions::axiom::DAxiom, Item, ItemDefinition, ItemPtr},
    parser::{
        phrase::{Phrase, UncreateResult},
        Node, NodeChild, ParseContext,
    },
    phrase,
    scope::Scope,
};

fn create(
    _pc: &ParseContext,
    env: &mut Environment,
    scope: Box<dyn Scope>,
    node: &Node,
) -> Result<ItemPtr, Diagnostic> {
    assert_eq!(node.children.len(), 4);
    assert_eq!(node.children[0], NodeChild::Text("AXIOM"));
    assert_eq!(node.children[1], NodeChild::Text("("));
    assert_eq!(node.children[3], NodeChild::Text(")"));
    let name_node = node.children[2].as_node();
    let name = name_node.as_ident()?;
    let con = DAxiom::from_name(env, name).ok_or_else(|| {
        Diagnostic::new()
            .with_text_error(format!("{} is not a valid axiom:", name))
            .with_source_code_block_error(name_node.position)
    })?;
    Ok(Item::new_boxed(con.clone_into_box(), scope))
}

fn uncreate<'a>(
    env: &mut Environment,
    _ctx: &mut VomitContext<'a, '_>,
    uncreate: ItemPtr,
) -> UncreateResult<'a> {
    if let Some(cax) = uncreate.downcast_definition::<DAxiom>() {
        let cax = cax.clone();
        let statement = cax.get_statement(env);
        let statement = &statement[..statement.len()]; // - "_statement".len()];
        Ok(Some(Node {
            phrase: "axiom",
            children: vec![
                NodeChild::Text("AXIOM"),
                NodeChild::Text("("),
                NodeChild::Text(statement),
                NodeChild::Text(")"),
            ],
            ..Default::default()
        }))
    } else {
        Ok(None)
    }
}

fn vomit(_pc: &ParseContext, src: &Node) -> String {
    format!("AXIOM({})", src.children[2].as_text())
}

pub fn phrase() -> Phrase {
    phrase!(
        "axiom",
        128, 128,
        Some((create, uncreate)),
        vomit,
        4 => r"\bAXIOM\b", r"\(", 255, r"\)"
    )
}
