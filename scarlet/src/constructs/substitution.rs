use std::{cell::RefCell, collections::HashSet};

use super::{
    downcast_construct,
    variable::{CVariable, VariableId},
    Construct, ConstructDefinition, ConstructId, GenInvResult, Invariant,
};
use crate::{
    environment::{
        dependencies::{DepResult, Dependencies, DependencyError},
        CheckResult, DefEqualResult, Environment, UnresolvedConstructError,
    },
    impl_any_eq_for_construct,
    scope::Scope,
    shared::{OrderedMap, TripleBool},
};

pub type Substitutions = OrderedMap<VariableId, ConstructId>;
pub type NestedSubstitutions<'a> = OrderedMap<VariableId, SubExpr<'a>>;
type Justifications = Result<Vec<Invariant>, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubExpr<'a>(pub ConstructId, pub &'a NestedSubstitutions<'a>);

impl<'a> SubExpr<'a> {
    pub fn deps(&self, env: &mut Environment) -> DepResult {
        let mut result = Dependencies::new();
        let base = env.get_dependencies(self.0);
        for (target, value) in self.1.iter() {
            if base.contains_var(*target) {
                result.append(value.deps(env));
            }
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSubstitution {
    base: ConstructId,
    subs: Substitutions,
    invs: Vec<Invariant>,
}

impl CSubstitution {
    pub fn new<'x>(base: ConstructId, subs: Substitutions, invs: Vec<Invariant>) -> Self {
        Self { base, subs, invs }
    }

    pub fn new_unchecked(base: ConstructId, subs: Substitutions) -> Self {
        Self::new(base, subs, Vec::new())
    }

    pub fn base(&self) -> ConstructId {
        self.base
    }

    pub fn substitutions(&self) -> &Substitutions {
        &self.subs
    }
}

impl_any_eq_for_construct!(CSubstitution);

impl Construct for CSubstitution {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn check<'x>(
        &self,
        env: &mut Environment<'x>,
        _this: ConstructId,
        _scope: Box<dyn Scope>,
    ) -> CheckResult {
        Ok(())
    }

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> DepResult {
        let mut deps = Dependencies::new();
        let base = env.get_dependencies(self.base);
        let base_error = base.error();
        for dep in base.as_variables() {
            if let Some((_, rep)) = self.subs.iter().find(|(var, _)| *var == dep.id) {
                let replaced_deps = env.get_dependencies(*rep);
                let replaced_err = replaced_deps.error();
                for rdep in replaced_deps.into_variables() {
                    if !dep.swallow.contains(&rdep.id) {
                        deps.push_eager(rdep);
                    }
                }
                if let Some(err) = replaced_err {
                    deps.append(Dependencies::new_error(err));
                }
            } else {
                deps.push_eager(dep.clone());
            }
        }
        if let Some(err) = base_error {
            deps.append(Dependencies::new_error(err));
        }
        for inv in self.invs.iter() {
            for &dep in &inv.dependencies {
                if let Ok(Some(var)) = env.get_and_downcast_construct_definition::<CVariable>(dep) {
                    let id = var.get_id();
                    deps.push_eager(env.get_variable(id).clone().as_dependency(env));
                } else {
                    deps.append(env.get_dependencies(dep));
                }
            }
        }
        deps
    }

    fn generated_invariants<'x>(
        &self,
        this: ConstructId,
        env: &mut Environment<'x>,
    ) -> GenInvResult {
        self.invs.clone()
    }

    fn is_def_equal<'x>(
        &self,
        env: &mut Environment<'x>,
        subs: &NestedSubstitutions,
        other: SubExpr,
        recursion_limit: u32,
    ) -> DefEqualResult {
        assert_ne!(recursion_limit, 0);
        let mut new_subs = subs.clone();
        for (target, value) in &self.subs {
            new_subs.insert_or_replace(target.clone(), SubExpr(*value, subs));
        }
        env.is_def_equal(SubExpr(self.base, &new_subs), other, recursion_limit - 1)
    }
}
