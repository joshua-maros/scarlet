use crate::{
    stage1::structure::construct::Construct,
    stage2::{
        ingest::{
            context::Context, expression::ingest_expression, statements::process_definitions,
        },
        structure::{Definitions, Item, ItemId, PrimitiveValue},
    },
};

fn type_self(ctx: &mut Context) -> (ItemId, (String, ItemId)) {
    let self_id = ctx.get_or_create_current_id();
    let self_def = ("Self".to_string(), self_id);
    (self_id, self_def)
}

pub fn ingest_type_construct(ctx: &mut Context, root: Construct) -> Result<Item, String> {
    let statements = root.expect_statements("Type").unwrap().to_owned();
    let (self_id, self_def) = type_self(ctx);
    let inner_type = Item::InductiveType(self_id);
    let inner_type_id = ctx.environment.insert(inner_type);

    let definitions = process_definitions(ctx, statements, vec![self_def])?;
    Ok(Item::Defining {
        base: inner_type_id,
        definitions,
    })
}

fn resolve_ident_in_scope(scope: &Definitions, ident: &str) -> Option<ItemId> {
    for (name, val) in scope {
        if name == ident {
            return Some(*val);
        }
    }
    None
}

fn resolve_ident(ctx: &Context, ident: &str) -> Result<ItemId, String> {
    // Reverse to earch the closest parents first.
    for scope in ctx.parent_scopes.iter().rev() {
        if let Some(id) = resolve_ident_in_scope(scope, ident) {
            return Ok(id);
        }
    }
    Err(format!(
        "Could not find an item named {} in the current scope or its parents.",
        ident
    ))
}

pub fn ingest_identifier(ctx: &mut Context, root: Construct) -> Result<Item, String> {
    let ident = root.expect_ident()?;
    let resolved = resolve_ident(ctx, ident)?;
    Ok(Item::Item(resolved))
}

pub fn ingest_any_construct(ctx: &mut Context, root: Construct) -> Result<Item, String> {
    let typ_expr = root.expect_single_expression("any")?.clone();
    let typee = ingest_expression(&mut ctx.child(), typ_expr)?;
    let selff = ctx.get_or_create_current_id();
    Ok(Item::Variable { selff, typee })
}

pub fn ingest_i32_construct(root: Construct) -> Result<Item, String> {
    let val = root.expect_text("i32")?;
    let val: i32 = val.parse().unwrap();
    Ok(Item::PrimitiveValue(PrimitiveValue::I32(val)))
}
