use std::{any::Any, fmt::Debug};

use super::{structt::CPopulatedStruct, variable::CVariable};
use crate::{
    environment::{
        dependencies::DepResult,
        discover_equality::{DeqPriority, DeqResult, DeqSide, Equal},
        CheckResult, Environment,
    },
    resolvable::BoxedResolvable,
    scope::Scope,
    shared::{AnyEq, Id, Pool},
};

#[derive(Debug)]
pub enum ConstructDefinition<'x> {
    Other(ConstructId),
    Resolved(BoxedConstruct),
    Unresolved(BoxedResolvable<'x>),
}

impl<'x> ConstructDefinition<'x> {
    pub fn is_placeholder(&self) -> bool {
        match self {
            Self::Unresolved(resolvable) => resolvable.is_placeholder(),
            _ => false,
        }
    }

    pub fn as_other(&self) -> Option<ConstructId> {
        match self {
            &Self::Other(con) => Some(con),
            _ => None,
        }
    }

    pub fn as_resolved(&self) -> Option<&BoxedConstruct> {
        match self {
            Self::Resolved(con) => Some(con),
            _ => None,
        }
    }

    /// Returns `true` if the construct definition is [`Unresolved`].
    ///
    /// [`Unresolved`]: ConstructDefinition::Unresolved
    pub fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved(..))
    }
}

impl<'x> From<Box<dyn Construct>> for ConstructDefinition<'x> {
    fn from(input: Box<dyn Construct>) -> Self {
        Self::Resolved(input)
    }
}

impl<'a, 'x> From<&'a ConstructId> for ConstructDefinition<'x> {
    fn from(input: &'a ConstructId) -> Self {
        Self::Other(*input)
    }
}

impl<'x> From<ConstructId> for ConstructDefinition<'x> {
    fn from(input: ConstructId) -> Self {
        Self::Other(input)
    }
}

#[derive(Debug)]
pub struct AnnotatedConstruct<'x> {
    pub definition: ConstructDefinition<'x>,
    pub reduced: ConstructDefinition<'x>,
    pub invariants: Option<Vec<crate::environment::invariants::Invariant>>,
    pub scope: Box<dyn Scope>,
    /// A dex that, when a value is plugged in for its first dependency, will
    /// evaluate to true if and only if the plugged in value could have been
    /// generated by this construct.
    pub from_dex: Option<ConstructId>,
}

pub type ConstructPool<'x> = Pool<AnnotatedConstruct<'x>, 'C'>;
pub type ConstructId = Id<'C'>;

pub type GenInvResult = Vec<crate::environment::invariants::Invariant>;

pub type BoxedConstruct = Box<dyn Construct>;
pub trait Construct: Any + Debug + AnyEq {
    fn dyn_clone(&self) -> Box<dyn Construct>;

    #[allow(unused_variables)]
    fn check<'x>(
        &self,
        env: &mut Environment<'x>,
        this: ConstructId,
        scope: Box<dyn Scope>,
    ) -> CheckResult {
        Ok(())
    }

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> DepResult;

    #[allow(unused_variables)]
    fn generated_invariants<'x>(
        &self,
        this: ConstructId,
        env: &mut Environment<'x>,
    ) -> GenInvResult {
        vec![]
    }

    #[allow(unused_variables)]
    fn discover_equality<'x>(
        &self,
        env: &mut Environment<'x>,
        other_id: ConstructId,
        other: &dyn Construct,
        limit: u32,
        tiebreaker: DeqSide,
    ) -> DeqResult {
        Ok(Equal::Unknown)
    }

    #[allow(unused_variables)]
    fn deq_priority<'x>(&self) -> DeqPriority {
        0
    }

    fn as_def<'x>(&self) -> ConstructDefinition<'x> {
        ConstructDefinition::Resolved(self.dyn_clone())
    }
}

pub fn downcast_construct<T: Construct>(from: &dyn Construct) -> Option<&T> {
    (from as &dyn Any).downcast_ref()
}

pub fn downcast_boxed_construct<T: Construct>(from: Box<dyn Construct>) -> Option<T> {
    (from as Box<dyn Any>).downcast().ok().map(|b| *b)
}

pub fn as_struct(from: &dyn Construct) -> Option<&CPopulatedStruct> {
    downcast_construct(from)
}

pub fn as_variable(from: &dyn Construct) -> Option<&CVariable> {
    downcast_construct(from)
}

#[macro_export]
macro_rules! impl_any_eq_for_construct {
    ($ConstructName:ident) => {
        impl crate::shared::AnyEq for $ConstructName {
            fn eq(&self, other: &dyn crate::shared::AnyEq) -> bool {
                (other as &dyn std::any::Any)
                    .downcast_ref::<Self>()
                    .map(|x| self == x)
                    .unwrap_or(false)
            }
        }
    };
}
