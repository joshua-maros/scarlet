use super::{
    base::{Construct, ConstructId},
    substitution::Substitutions,
};
use crate::{
    environment::Environment,
    impl_any_eq_for_construct,
    shared::{Id, Pool, TripleBool},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable;
pub type VariablePool = Pool<Variable, 'V'>;
pub type VariableId = Id<'V'>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CVariable {
    pub id: VariableId,
    pub invariants: ConstructId,
    pub capturing: bool,
}

impl CVariable {
    pub fn is_same_variable_as(&self, other: &Self) -> bool {
        self.id == other.id && self.capturing == other.capturing
    }
}

impl_any_eq_for_construct!(CVariable);

impl Construct for CVariable {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn check<'x>(&self, _env: &mut Environment<'x>) {}

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> Vec<CVariable> {
        let mut base = env.get_dependencies(self.invariants);
        base.push(self.clone());
        base
    }

    fn is_def_equal<'x>(&self, _env: &mut Environment<'x>, _other: &dyn Construct) -> TripleBool {
        TripleBool::Unknown
    }

    fn substitute<'x>(
        &self,
        env: &mut Environment<'x>,
        substitutions: &Substitutions,
    ) -> ConstructId {
        for (target, value) in substitutions {
            if target.id == self.id && target.capturing == self.capturing {
                return *value;
            }
        }
        let invariants = env.substitute(self.invariants, substitutions);
        let new = Self {
            id: self.id,
            invariants,
            capturing: self.capturing,
        };
        env.push_construct(Box::new(new), vec![invariants])
    }
}
