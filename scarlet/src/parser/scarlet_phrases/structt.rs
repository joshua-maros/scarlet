use typed_arena::Arena;

use crate::{
    constructs::{
        downcast_construct,
        structt::{CPopulatedStruct, SField, SFieldAndRest},
        ConstructId,
    },
    environment::Environment,
    parser::{phrase::Phrase, util, Node, NodeChild, ParseContext},
    phrase,
    scope::Scope,
    shared::TripleBool,
};

fn struct_from_fields<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    mut fields: Vec<(Option<&str>, &Node<'x>)>,
    scope: Box<dyn Scope>,
) -> ConstructId {
    if fields.is_empty() {
        env.get_language_item("void")
    } else {
        let (label, field) = fields.remove(0);
        let label = label.unwrap_or("").to_owned();
        let this = env.push_placeholder(scope);
        let field = field.as_construct(pc, env, SFieldAndRest(this));
        let rest = struct_from_fields(pc, env, fields, Box::new(SField(this)));
        let this_def = CPopulatedStruct::new(label, field, rest);
        env.define_construct(this, this_def);
        this
    }
}

fn create<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 3);
    assert_eq!(node.children[0], NodeChild::Text("{"));
    assert_eq!(node.children[2], NodeChild::Text("}"));
    let fields = util::collect_comma_list(&node.children[1]);
    let fields = fields
        .into_iter()
        .map(|field| {
            if field.phrase == "is" {
                (
                    Some(field.children[0].as_node().as_ident()),
                    field.children[2].as_node(),
                )
            } else {
                (None, field)
            }
        })
        .collect();
    struct_from_fields(pc, env, fields, scope)
}

fn uncreate<'a>(
    pc: &ParseContext,
    env: &mut Environment,
    code_arena: &'a Arena<String>,
    uncreate: ConstructId,
    from: &dyn Scope,
) -> Option<Node<'a>> {
    let mut maybe_structt = uncreate;
    let mut fields = Vec::new();
    while let Some(structt) =
        downcast_construct::<CPopulatedStruct>(&**env.get_reduced_construct_definition(uncreate))
    {
        let label = code_arena.alloc(structt.get_label().to_owned());
        let value = structt.get_value();
        maybe_structt = structt.get_rest();
        fields.push(NodeChild::Node(Node {
            phrase: "is",
            children: vec![
                NodeChild::Text(label),
                NodeChild::Text("is"),
                NodeChild::Node(env.vomit(255, true, pc, code_arena, value, from)),
            ],
        }));
    }
    if env.is_def_equal(maybe_structt, env.get_language_item("void")) == TripleBool::True {
        Some(Node {
            phrase: "struct",
            children: [
                vec![NodeChild::Text("{")],
                fields,
                vec![NodeChild::Text("}")],
            ]
            .concat(),
        })
    } else {
        None
    }
}

fn vomit(_pc: &ParseContext, src: &Node) -> String {
    format!("{:#?}", src)
}

pub fn phrase() -> Phrase {
    phrase!(
        "struct",
        Some((create, uncreate)),
        vomit,
        0 => r"\{", 255, r"\}"
    )
}
