use super::structure::{Environment, Item, Namespace, Value, ValueId};
use crate::{
    stage1::structure::{
        construct::{Construct, ConstructBody},
        expression::Expression,
        statement::{Is, Statement},
    },
    stage2::structure::{BuiltinOperation, BuiltinValue},
};

fn single_expr_construct(label: &str, expr: Expression) -> Construct {
    Construct {
        body: ConstructBody::Statements(vec![Statement::Expression(expr)]),
        label: label.to_owned(),
    }
}

fn expressions_construct(label: &str, expressions: Vec<Expression>) -> Construct {
    statements_construct(
        label,
        expressions.into_iter().map(Statement::Expression).collect(),
    )
}

fn statements_construct(label: &str, statements: Vec<Statement>) -> Construct {
    Construct {
        body: ConstructBody::Statements(statements),
        label: label.to_owned(),
    }
}

fn text_construct(label: &str, text: String) -> Construct {
    Construct {
        body: ConstructBody::PlainText(text),
        label: label.to_owned(),
    }
}

fn identifier(name: &str) -> Construct {
    text_construct("identifier", name.to_owned())
}

fn simple_builtin_item(name: &str) -> Construct {
    single_expr_construct("builtin_item", just_root_expression(identifier(name)))
}

fn just_root_expression(root: Construct) -> Expression {
    Expression {
        root,
        others: vec![],
    }
}

pub fn vomit(env: &Environment, item: Item) -> Expression {
    let namespace = env[item.namespace].as_ref().expect("TODO: Nice error");
    let value = env[item.value].as_ref().expect("TODO: Nice error");
    if let Value::From { .. } = value {
        return todo!();
    }
    match namespace {
        Namespace::Defining {
            base, definitions, ..
        } => {
            let mut statements = Vec::new();
            for (name, value) in definitions {
                let name = just_root_expression(identifier(name));
                let value = vomit(env, *value);
                let statement = Is { name, value };
                statements.push(Statement::Is(statement));
            }
            let construct = statements_construct("defining", statements);
            let base_item = Item {
                namespace: *base,
                value: item.value,
            };
            let mut expr = vomit(env, base_item);
            expr.others.push(construct);
            expr
        }
        Namespace::Empty => vomit_value_impl(env, value),
        Namespace::Identifier { name, .. } => {
            if let Value::Identifier { name: vname, .. } = value {
                debug_assert_eq!(name, vname);
                just_root_expression(identifier(name))
            } else {
                unreachable!("ICE")
            }
        }
        Namespace::Member { base, name } => {
            if let Value::Member {
                base: vbase,
                name: vname,
                previous_value,
            } = value
            {
                debug_assert_eq!(base, vbase);
                debug_assert_eq!(name, vname);
                let base = Item {
                    value: *previous_value,
                    namespace: *base,
                };
                let mut base = vomit(env, base);
                let ident = just_root_expression(identifier(name));
                let member = single_expr_construct("member", ident);
                base.others.push(member);
                base
            } else {
                unreachable!("ICE")
            }
        }
        Namespace::Root(item) => vomit(env, *item),
        Namespace::Replacing { .. } => todo!(),
        _ => todo!(),
    }
}
fn vomit_value(env: &Environment, value: ValueId) -> Expression {
    let value = env[value].as_ref().expect("TODO: Nice error");
    vomit_value_impl(env, value)
}

fn vomit_value_impl(env: &Environment, value: &Value) -> Expression {
    let construct = match value {
        Value::Any { variable } => {
            let typee = env[*variable].original_type;
            let typee = vomit_value(env, typee);
            single_expr_construct("any", typee)
        }
        Value::BuiltinOperation(op) => {
            let name = match op {
                BuiltinOperation::Cast { .. } => "cast",
            };
            let mut exprs = vec![just_root_expression(identifier(name))];
            for arg in op.inputs() {
                exprs.push(vomit_value(env, arg));
            }
            expressions_construct("builtin_item", exprs)
        }
        Value::BuiltinValue(val) => match val {
            BuiltinValue::OriginType => simple_builtin_item("TYPE"),
            BuiltinValue::U8(value) => text_construct("u8", format!("{}", value)),
            BuiltinValue::U8Type => simple_builtin_item("UnsignedInteger8"),
        },
        Value::From { .. } => unimplemented!(),
        Value::Identifier { .. } => unreachable!(),
        Value::Member { .. } => unreachable!(),
        Value::Replacing { .. } => todo!(),
        Value::Variant { .. } => todo!(),
    };
    just_root_expression(construct)
}
