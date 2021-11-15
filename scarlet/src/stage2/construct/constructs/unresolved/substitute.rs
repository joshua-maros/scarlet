use super::CUnresolved;
use crate::stage2::{
    construct::constructs::Substitutions,
    structure::{ConstructId, Environment},
};

pub fn implementation<'x>(
    this: &CUnresolved<'x>,
    substitutions: &Substitutions<'x>,
    env: &mut Environment<'x>,
) -> ConstructId<'x> {
    todo!()
}
