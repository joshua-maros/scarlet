use super::context::LocalInfo;
use crate::{
    shared::{Definitions, ItemId},
    stage1::structure::{
        expression::Expression,
        statement::{Is, Statement},
    },
    stage2::ingest::{context::Context, expression::ingest_expression},
};

struct UnprocessedItem {
    id: ItemId,
    public: bool,
    name: Option<String>,
    def: Expression,
}

pub(super) fn process_definitions(
    ctx: &mut Context,
    statements: Vec<Statement>,
    other_defs: Vec<(String, ItemId)>,
) -> Result<Definitions, String> {
    process_definitions_with_info(ctx, statements, other_defs, LocalInfo::Plain)
}

fn is_statement_to_unprocessed_item(ctx: &mut Context, is: Is) -> Result<UnprocessedItem, String> {
    let name = is.name.expect_ident_owned()?;
    Ok(UnprocessedItem {
        id: ctx.environment.next_id(),
        public: is.public,
        name: Some(name),
        def: is.value,
    })
}

fn expr_statement_to_unprocessed_item(
    ctx: &mut Context,
    expr: Expression,
) -> Result<UnprocessedItem, String> {
    Ok(UnprocessedItem {
        id: ctx.environment.next_id(),
        public: false,
        name: None,
        def: expr,
    })
}

fn statements_to_unprocessed_items(
    ctx: &mut Context,
    statements: Vec<Statement>,
) -> Result<Vec<UnprocessedItem>, String> {
    let mut top_level_expressions = Vec::new();
    for statement in statements {
        let item = match statement {
            Statement::Is(is) => is_statement_to_unprocessed_item(ctx, is)?,
            Statement::Expression(expr) => expr_statement_to_unprocessed_item(ctx, expr)?,
            _ => todo!("Nice error"),
        };
        top_level_expressions.push(item);
    }
    Ok(top_level_expressions)
}

fn item_to_def(item: &UnprocessedItem) -> Option<(String, ItemId)> {
    item.name.as_ref().map(|name| (name.clone(), item.id))
}

fn definitions(other_defs: Vec<(String, ItemId)>, unprocessed: &[UnprocessedItem]) -> Definitions {
    let unprocessed_defs = unprocessed.iter().filter_map(item_to_def).collect();
    [other_defs, unprocessed_defs].concat()
}

pub(super) fn process_definitions_with_info(
    ctx: &mut Context,
    statements: Vec<Statement>,
    other_defs: Vec<(String, ItemId)>,
    info: LocalInfo,
) -> Result<Definitions, String> {
    let unprocessed = statements_to_unprocessed_items(ctx, statements)?;
    let definitions = definitions(other_defs, &unprocessed[..]);
    for item in unprocessed {
        let mut child_ctx = ctx
            .child()
            .with_id_scope_info(item.id, &definitions, info.clone());
        ingest_expression(&mut child_ctx, item.def)?;
    }
    Ok(definitions)
}