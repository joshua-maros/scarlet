use crate::stage1::{
    ingest::{
        construct::{explicit, helpers, root_shorthands},
        nom_prelude::*,
    },
    structure::{
        construct::{Construct, Position},
        expression::Expression,
    },
};

pub fn target_parser<'i>() -> impl Parser<'i, Construct> {
    |input| {
        let (input, root) = alt((
            explicit::parser(Position::Root),
            root_shorthands::ident_parser(),
        ))(input)?;
        let (input, _) = nonempty_ws()(input)?;
        let (input, _) = tag("is")(input)?;
        let (input, _) = nonempty_ws()(input)?;

        let expression = Expression {
            pres: vec![],
            root,
            posts: vec![],
        };
        let construct = Construct::from_expression("target", expression);
        Ok((input, construct))
    }
}