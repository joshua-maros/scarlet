use std::collections::HashSet;

use super::{BoxedResolvable, Resolvable, ResolveError, ResolveResult};
use crate::{
    constructs::{
        substitution::{CSubstitution, Substitutions},
        variable::{CVariable, Variable},
        ConstructDefinition, ConstructId,
    },
    environment::Environment,
    scope::Scope,
    shared::OrderedMap,
};

#[derive(Clone, Debug)]
pub struct RVariable {
    pub invariants: Vec<ConstructId>,
    pub dependencies: Vec<ConstructId>,
}

impl<'x> Resolvable<'x> for RVariable {
    fn dyn_clone(&self) -> BoxedResolvable<'x> {
        Box::new(self.clone())
    }

    fn resolve(
        &self,
        env: &mut Environment<'x>,
        _scope: Box<dyn Scope>,
        _limit: u32,
    ) -> ResolveResult<'x> {
        let id = env.push_variable(Variable {
            id: None,
            invariants: self.invariants.clone(),
            dependencies: self.dependencies.clone(),
        });
        let con = CVariable::new(id);
        Ok(ConstructDefinition::Resolved(Box::new(con)))
    }
}