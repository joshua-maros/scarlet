use itertools::Itertools;

use super::{
    base::{Construct, ItemId},
    substitution::Substitutions,
    variable::{CVariable, Dependency, VariableId},
    GenInvResult,
};
use crate::{
    environment::{
        dependencies::{DepResult, Dependencies},
        discover_equality::{DeqResult, DeqSide},
        Environment,
    },
    impl_any_eq_for_construct,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CWithDependencies {
    base: ItemId,
    dependencies: Vec<ItemId>,
}

impl CWithDependencies {
    pub fn new<'x>(base: ItemId, dependencies: Vec<ItemId>) -> Self {
        Self { base, dependencies }
    }

    pub fn base(&self) -> ItemId {
        self.base
    }

    pub(crate) fn dependencies(&self) -> &[ItemId] {
        &self.dependencies
    }
}

impl_any_eq_for_construct!(CWithDependencies);

impl Construct for CWithDependencies {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn generated_invariants<'x>(&self, _this: ItemId, env: &mut Environment<'x>) -> GenInvResult {
        env.generated_invariants(self.base)
    }

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> DepResult {
        let mut deps = Dependencies::new();
        let base_deps = env.get_dependencies(self.base);
        for &priority_dep in &self.dependencies {
            let dep = env
                .get_and_downcast_construct_definition::<CVariable>(priority_dep)
                .unwrap()
                .unwrap();
            let dep = dep.get_id();
            if let Some(dep) = base_deps.get_var(dep) {
                deps.push_eager(dep.clone());
            // } else if let Some(err) = priority_dep_error {
            //     deps.append(Dependencies::new_error(err));
            } else if let Some(err) = base_deps.error() {
                // If the base had an error, we might not be accounting for
                // all the dependencies it has. We might be throwing out a
                // priority dep that it actually has, so we need to
                // terminate the dependency list now before anything else
                // gets added out of order.
                deps.append(Dependencies::new_error(err));
            }
            // if let Some(err) = priority_dep_error {
            //     deps.append(Dependencies::new_error(err));
            // }
        }
        deps.append(base_deps);
        deps
    }

    fn dereference(
        &self,
        env: &mut Environment,
    ) -> Option<(ItemId, Option<&Substitutions>, Option<Vec<VariableId>>)> {
        let mut reordering = Vec::new();
        for &dep in &self.dependencies {
            let dep = env
                .get_and_downcast_construct_definition::<CVariable>(dep)
                .unwrap()
                .unwrap();
            let dep = dep.get_id();
            reordering.push(dep);
        }
        Some((self.base, None, Some(reordering)))
    }
}
