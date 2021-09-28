use self::other::ingest_non_defining_postfix_construct;
use super::{context::Context, expression::ingest_expression};
use crate::{
    shared::{Definitions, Item},
    stage1::structure::{construct::Construct, expression::Expression},
    stage2::{ingest::definitions::process_definitions, structure::UnresolvedItem},
};

mod from;
mod other;

pub fn ingest_postfix_construct(
    ctx: &mut Context,
    post: Construct,
    remainder: Expression,
    extra_defines: Definitions,
) -> Result<UnresolvedItem, String> {
    if post.label == "defining" {
        ingest_defining_construct(ctx, post, remainder, extra_defines)
    } else {
        let base_id = ingest_expression(&mut ctx.child(), remainder, extra_defines)?;
        ingest_non_defining_postfix_construct(ctx, base_id, post)
    }
}

pub fn ingest_defining_construct(
    ctx: &mut Context,
    post: Construct,
    remainder: Expression,
    extra_defines: Definitions,
) -> Result<UnresolvedItem, String> {
    let statements = post.expect_statements("defining")?.to_owned();
    let self_id = ctx.get_or_create_current_id();
    let body = process_definitions(&mut ctx.child(), statements, extra_defines, self_id)?;
    let mut child_ctx = ctx.child().with_additional_scope(&body);
    let base_id = ingest_expression(&mut child_ctx, remainder, Default::default())?;
    ctx.environment.set_defined_in(base_id, self_id);
    Ok(Item::Defining {
        base: base_id,
        definitions: body,
    }
    .into())
}
