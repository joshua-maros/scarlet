use super::{
    downcast_construct, variable::CVariable, BoxedConstruct, Construct, ConstructDefinition,
    ConstructId,
};
use crate::{
    environment::{dependencies::Dependencies, Environment},
    impl_any_eq_for_construct,
    shared::{OrderedMap, TripleBool},
};

pub type Substitutions = OrderedMap<CVariable, ConstructId>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSubstitution(ConstructId, Substitutions);

impl CSubstitution {
    pub fn new<'x>(base: ConstructId, subs: Substitutions) -> Self {
        Self(base, subs.clone())
    }

    pub fn base(&self) -> ConstructId {
        self.0
    }

    pub fn substitutions(&self) -> &Substitutions {
        &self.1
    }
}

impl_any_eq_for_construct!(CSubstitution);

impl Construct for CSubstitution {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn check<'x>(&self, env: &mut Environment<'x>) {
        for (target, value) in &self.1 {
            env.reduce(*value);
            if !target.can_be_assigned(*value, env) {
                println!("{:#?}", env);
                println!("THIS EXPRESSION:");
                env.show(*value, *value);
                println!("DOES NOT SATISFY ALL OF THE FOLLOWING REQUIREMENTS:");
                for &inv in target.get_invariants() {
                    println!("Must satisfy invariant:");
                    env.show(inv, *value);
                }
                for &sub in target.get_substitutions() {
                    println!("Must have a dependency that can be assigned:");
                    env.show(sub, *value);
                }
                todo!("nice error.");
            }
        }
    }

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> Dependencies {
        let mut deps = Dependencies::new();
        let base = env.get_dependencies(self.0);
        for dep in base.as_variables() {
            if let Some(rep) = self.1.get(&dep) {
                let replaced_deps = env.get_dependencies(*rep);
                for rdep in replaced_deps
                    .into_variables()
                    .skip(dep.get_substitutions().len())
                {
                    deps.push_eager(rdep);
                }
            } else {
                deps.push_eager(dep.inline_substitute(env, &self.1).unwrap());
            }
        }
        deps
    }

    fn generated_invariants<'x>(
        &self,
        _this: ConstructId,
        env: &mut Environment<'x>,
    ) -> Vec<ConstructId> {
        let mut invs = Vec::new();
        for inv in env.generated_invariants(self.0) {
            let inv = env.substitute(inv, &self.1);
            env.reduce(inv);
            invs.push(inv);
        }
        invs
    }

    fn is_def_equal<'x>(&self, env: &mut Environment<'x>, other: &dyn Construct) -> TripleBool {
        if let Some(other) = downcast_construct::<Self>(other) {
            let mut result = env.is_def_equal(self.0, other.0);
            if self.1.len() != other.1.len() {
                result = TripleBool::False;
            }
            for (target, value) in &self.1 {
                if let Some(other_value) = other.1.get(target) {
                    result = TripleBool::and(vec![result, env.is_def_equal(*value, *other_value)]);
                } else {
                    result = TripleBool::False
                }
            }
            result
        } else {
            TripleBool::Unknown
        }
    }

    fn reduce<'x>(&self, env: &mut Environment<'x>) -> ConstructDefinition<'x> {
        self.check(env);
        env.reduce(self.0);
        let subbed = env.substitute(self.0, &self.1);
        env.reduce(subbed);
        subbed.into()
    }

    fn substitute<'x>(
        &self,
        env: &mut Environment<'x>,
        substitutions: &Substitutions,
    ) -> BoxedConstruct {
        let base = self.0;
        let mut new_subs = self.1.clone();
        for (_, value) in &mut new_subs {
            let subbed = env.substitute(*value, substitutions);
            *value = subbed;
        }
        for (target, value) in substitutions {
            let mut already_present = false;
            for (existing_target, _) in &new_subs {
                if existing_target.is_same_variable_as(target) {
                    already_present = true;
                    break;
                }
            }
            if !already_present {
                new_subs.insert_no_replace(target.clone(), *value);
            }
        }
        Self::new(base, new_subs).dyn_clone()
    }
}
