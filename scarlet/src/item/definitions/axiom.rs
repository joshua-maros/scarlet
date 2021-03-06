use maplit::hashset;

use crate::{
    environment::Environment,
    impl_any_eq_from_regular_eq,
    item::{
        check::CheckFeature,
        dependencies::{Dcc, DepResult, DependenciesFeature, OnlyCalledByDcc},
        equality::{Ecc, Equal, EqualResult, EqualityFeature, OnlyCalledByEcc},
        invariants::{Icc, InvariantSet, InvariantsFeature, InvariantsResult, OnlyCalledByIcc},
        ContainmentType, ItemDefinition, ItemPtr,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DAxiom {
    statement: ItemPtr,
}

impl DAxiom {
    fn new(env: &mut Environment, statement: &str) -> Option<Self> {
        Some(Self {
            statement: env.get_language_item(statement)?.ptr_clone(),
        })
    }

    pub fn from_name(env: &mut Environment, name: &str) -> Option<Self> {
        Self::new(env, &format!("{}_statement", name))
    }

    pub fn get_statement(&self, env: &mut Environment) -> &'static str {
        for lang_item_name in env.language_item_names() {
            let lang_item = env.get_language_item(lang_item_name).unwrap();
            if self
                .statement
                .get_trimmed_equality(&lang_item)
                .as_ref()
                .map(Equal::is_trivial_yes)
                == Ok(true)
            {
                return lang_item_name;
            }
        }
        panic!("{:?} is not an axiom statement", self.statement)
    }
}

impl_any_eq_from_regular_eq!(DAxiom);

impl ItemDefinition for DAxiom {
    fn clone_into_box(&self) -> Box<dyn ItemDefinition> {
        Box::new(self.clone())
    }

    fn contents(&self) -> Vec<(ContainmentType, ItemPtr)> {
        vec![(ContainmentType::Definitional, self.statement.ptr_clone())]
    }
}

impl CheckFeature for DAxiom {}

impl DependenciesFeature for DAxiom {
    fn get_dependencies_using_context(
        &self,
        _this: &ItemPtr,
        ctx: &mut Dcc,
        affects_return_value: bool,
        _: OnlyCalledByDcc,
    ) -> DepResult {
        ctx.get_dependencies(&self.statement, affects_return_value)
    }
}

impl EqualityFeature for DAxiom {
    fn get_equality_using_context(&self, ctx: &mut Ecc, _: OnlyCalledByEcc) -> EqualResult {
        let statements = if let Some(other) = ctx.other().downcast_definition::<Self>() {
            let self_statement = self.statement.ptr_clone();
            let other_statement = other.statement.ptr_clone();
            Some((self_statement, other_statement))
        } else {
            None
        };
        if let Some((self_statement, other_statement)) = statements {
            Ok(ctx
                .with_primary_and_other(self_statement, other_statement)
                .get_equality_left()?)
        } else {
            Ok(Equal::Unknown)
        }
    }
}

impl InvariantsFeature for DAxiom {
    fn get_invariants_using_context(
        &self,
        this: &ItemPtr,
        _ctx: &mut Icc,
        _: OnlyCalledByIcc,
    ) -> InvariantsResult {
        Ok(InvariantSet::new(
            this.ptr_clone(),
            vec![self.statement.ptr_clone()],
            vec![],
            hashset![],
        ))
    }
}
