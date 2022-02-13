use super::{BoxedResolvable, Resolvable, ResolveResult};
use crate::{environment::Environment, scope::Scope, constructs::{ConstructId, ConstructDefinition, substitution::CSubstitution, variable::CVariable}};

#[derive(Clone, Debug)]
pub struct RFrom {
    pub left: ConstructId,
    pub right: ConstructId,
}

impl<'x> Resolvable<'x> for RFrom {
    fn dyn_clone(&self) -> BoxedResolvable<'x> {
        Box::new(self.clone())
    }

    fn resolve(
        &self,
        env: &mut Environment<'x>,
        _scope: Box<dyn Scope>,
        _limit: u32,
    ) -> ResolveResult<'x> {
        let base = env.create_from_dex(self.right)?;
        let x = env.get_language_item("x");
        let x = env.get_and_downcast_construct_definition::<CVariable>(x)?;
        let x_id = x.unwrap().get_id();
        let subs = vec![(x_id, self.left)].into_iter().collect();
        let subbed = CSubstitution::new_unchecked(base, subs);
        Ok(ConstructDefinition::Resolved(Box::new(subbed)))
    }
}