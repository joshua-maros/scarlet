use itertools::Itertools;

use self::NodeChild::*;
use super::{node::Node, ParseContext};
use crate::{
    constructs::{
        equal::CEqual,
        if_then_else::CIfThenElse,
        is_populated_struct::CIsPopulatedStruct,
        shown::CShown,
        structt::{
            AtomicStructMember, CAtomicStructMember, CPopulatedStruct, SField, SFieldAndRest,
        },
        unique::CUnique,
        variable::SVariableInvariants,
        ConstructId,
    },
    environment::Environment,
    parser::node::NodeChild,
    resolvable::{RIdentifier, RSubstitution, RVariable},
    scope::{SPlain, Scope},
};

fn collect_comma_list<'a, 'n>(list: &'a NodeChild<'n>) -> Vec<&'a Node<'n>> {
    if let NodeChild::Node(list) = list {
        if list.phrase == "multiple constructs" {
            assert_eq!(list.children.len(), 3);
            assert_eq!(list.children[1], NodeChild::Text(","));
            [
                collect_comma_list(&list.children[0]),
                vec![list.children[2].as_node()],
            ]
            .concat()
        } else {
            vec![list]
        }
    } else {
        vec![]
    }
}

pub fn atomic_struct_member<'x, const M: AtomicStructMember>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 2);
    let this = env.push_placeholder(scope);
    let base = node.children[0].as_construct(pc, env, SPlain(this));
    env.define_construct(this, CAtomicStructMember(base, M));
    this
}

pub fn builtin_item<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 5);
    assert_eq!(node.children[1], Text(".AS_BUILTIN_ITEM"));
    assert_eq!(node.children[2], Text("["));
    assert_eq!(node.children[4], Text("]"));
    let base = node.children[0].as_construct_dyn_scope(pc, env, scope);
    let name = node.children[3].as_node().as_ident();
    env.define_builtin_item(name, base);
    base
}

pub fn equal<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 3);
    assert_eq!(node.children[1], Text("="));
    let this = env.push_placeholder(scope);
    let left = node.children[0].as_construct(pc, env, SPlain(this));
    let right = node.children[2].as_construct(pc, env, SPlain(this));
    env.define_construct(this, CEqual::new(left, right));
    this
}

pub fn identifier<'x>(
    _pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.phrase, "identifier");
    assert_eq!(node.children.len(), 1);
    env.push_unresolved(RIdentifier(node.children[0].as_text()), scope)
}

pub fn if_then_else<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 4);
    assert_eq!(node.children[0], Text("IF_THEN_ELSE"));
    assert_eq!(node.children[1], Text("["));
    assert_eq!(node.children[3], Text("]"));
    let args = collect_comma_list(&node.children[2]);
    assert_eq!(args.len(), 3);
    let this = env.push_placeholder(scope);

    let condition = args[0].as_construct(pc, env, SPlain(this));
    let then = args[1].as_construct(pc, env, SPlain(this));
    let elsee = args[2].as_construct(pc, env, SPlain(this));
    env.define_construct(this, CIfThenElse::new(condition, then, elsee));
    this
}

pub fn is_populated_struct<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 2);
    assert_eq!(node.children[1], Text(".IS_POPULATED_STRUCT"));
    let this = env.push_placeholder(scope);
    let base = node.children[0].as_construct(pc, env, SPlain(this));
    env.define_construct(this, CIsPopulatedStruct::new(base));
    this
}

pub fn parentheses<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 3);
    assert_eq!(node.children[0], Text("("));
    assert_eq!(node.children[2], Text(")"));
    node.children[1].as_construct_dyn_scope(pc, env, scope)
}

pub fn populated_struct<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert!(node.children.len() == 4);
    assert_eq!(node.children[0], Text("POPULATED_STRUCT"));
    assert_eq!(node.children[1], Text("["));
    assert_eq!(node.children[3], Text("]"));
    let args = collect_comma_list(&node.children[2]);
    assert_eq!(args.len(), 3);
    let this = env.push_placeholder(scope);

    let label = args[0].as_ident().to_owned();
    let value = args[1].as_construct(pc, env, SFieldAndRest(this));
    let rest = args[2].as_construct(pc, env, SField(this));
    env.define_construct(this, CPopulatedStruct::new(label, value, rest));
    this
}

pub fn shown<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 2);
    assert_eq!(node.children[1], Text(".SHOWN"));
    let this = env.push_placeholder(scope);
    let base = node.children[0].as_construct(pc, env, SPlain(this));
    env.define_construct(this, CShown::new(base));
    this
}

pub fn struct_from_fields<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    mut fields: Vec<(Option<&str>, &Node<'x>)>,
    scope: Box<dyn Scope>,
) -> ConstructId {
    if fields.is_empty() {
        env.get_builtin_item("void")
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

pub fn structt<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 3);
    assert_eq!(node.children[0], Text("{"));
    assert_eq!(node.children[2], Text("}"));
    let fields = collect_comma_list(&node.children[1]);
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

pub fn substitution<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children[1], Text("["));
    assert_eq!(node.children[3], Text("]"));
    assert!(node.children.len() == 4);
    let this = env.push_placeholder(scope);
    let base = node.children[0].as_construct(pc, env, SPlain(this));
    let mut named_subs = Vec::new();
    let mut anonymous_subs = Vec::new();
    for sub in collect_comma_list(&node.children[2]) {
        if sub.phrase == "is" {
            named_subs.push((
                sub.children[0].as_node().as_ident(),
                sub.children[2].as_construct(pc, env, SPlain(this)),
            ));
        } else {
            anonymous_subs.push(sub.as_construct(pc, env, SPlain(this)));
        }
    }
    env.define_unresolved(
        this,
        RSubstitution {
            base,
            named_subs,
            anonymous_subs,
        },
    );
    this
}

pub fn unique<'x>(
    _pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children, &[Text("UNIQUE")]);
    let id = env.push_unique();
    env.push_construct(CUnique::new(id), scope)
}

pub fn variable<'x>(
    pc: &ParseContext,
    env: &mut Environment<'x>,
    scope: Box<dyn Scope>,
    node: &Node<'x>,
) -> ConstructId {
    assert_eq!(node.children.len(), 4);
    assert_eq!(node.children[1], Text("["));
    assert_eq!(node.children[3], Text("]"));
    let mut invariants = Vec::new();
    let mut depends_on = Vec::new();
    let mut mode = 0;
    let this = env.push_placeholder(scope);
    for arg in collect_comma_list(&node.children[2]) {
        if arg.phrase == "identifier" && arg.children == &[Text("DEPENDS_ON")] {
            mode = 1;
        } else if mode == 0 {
            let con = arg.as_construct(pc, env, SVariableInvariants(this));
            invariants.push(con);
        } else {
            let con = arg.as_construct(pc, env, SPlain(this));
            depends_on.push(con);
        }
    }
    let id = env.push_variable();
    let def = RVariable {
        id,
        invariants,
        depends_on,
    };
    env.define_unresolved(this, def);
    this
}
