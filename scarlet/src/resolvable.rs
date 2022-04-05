pub mod from;
mod identifier;
mod named_member;
mod placeholder;
mod substitution;
mod variable;

use std::fmt::Debug;

pub use identifier::RIdentifier;
pub use named_member::RNamedMember;
pub use placeholder::RPlaceholder;
pub use substitution::RSubstitution;
pub use variable::RVariable;

use crate::{
    constructs::ItemDefinition,
    environment::{dependencies::Dependencies, Environment, UnresolvedItemError},
    scope::{LookupInvariantError, Scope},
};

#[derive(Clone, Debug)]
pub enum ResolveError {
    Unresolved(UnresolvedItemError),
    InvariantDeadEnd(String),
    MaybeInvariantDoesNotExist,
    Placeholder,
}

impl From<UnresolvedItemError> for ResolveError {
    fn from(v: UnresolvedItemError) -> Self {
        Self::Unresolved(v)
    }
}

impl From<LookupInvariantError> for ResolveError {
    fn from(err: LookupInvariantError) -> Self {
        match err {
            LookupInvariantError::Unresolved(err) => Self::Unresolved(err),
            LookupInvariantError::MightNotExist => Self::MaybeInvariantDoesNotExist,
            LookupInvariantError::DefinitelyDoesNotExist => {
                Self::InvariantDeadEnd(format!("No additional info"))
            }
        }
    }
}

pub type ResolveResult<'x> = Result<ItemDefinition<'x>, ResolveError>;

pub trait Resolvable<'x>: Debug {
    fn is_placeholder(&self) -> bool {
        false
    }
    fn dyn_clone(&self) -> BoxedResolvable<'x>;
    fn resolve(
        &self,
        env: &mut Environment<'x>,
        scope: Box<dyn Scope>,
        limit: u32,
    ) -> ResolveResult<'x>;
    #[allow(unused_variables)]
    fn estimate_dependencies(&self, env: &mut Environment) -> Dependencies {
        Dependencies::new()
    }
}

pub type BoxedResolvable<'x> = Box<dyn Resolvable<'x> + 'x>;
