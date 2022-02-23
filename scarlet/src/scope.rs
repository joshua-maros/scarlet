use std::fmt::Debug;

use maplit::hashset;

use crate::{
    constructs::ConstructId,
    environment::{
        discover_equality::Equal, invariants::Invariant, Environment, UnresolvedConstructError,
    },
};

pub type LookupIdentResult = Result<Option<ConstructId>, UnresolvedConstructError>;
pub type ReverseLookupIdentResult = Result<Option<String>, UnresolvedConstructError>;
pub type LookupInvariantResult = Result<Invariant, LookupInvariantError>;
/// The LHS of the Equal instance describes what needs to be substituted for the
/// invariant to match the original statement that was queried.
pub type LookupSimilarInvariantResult = Result<(Invariant, Equal), LookupInvariantError>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LookupInvariantError {
    Unresolved(UnresolvedConstructError),
    MightNotExist,
    DefinitelyDoesNotExist,
}

impl From<UnresolvedConstructError> for LookupInvariantError {
    fn from(v: UnresolvedConstructError) -> Self {
        Self::Unresolved(v)
    }
}

pub trait Scope: Debug {
    fn dyn_clone(&self) -> Box<dyn Scope>;

    fn is_placeholder(&self) -> bool {
        false
    }

    fn local_lookup_ident<'x>(&self, env: &mut Environment<'x>, ident: &str) -> LookupIdentResult;
    fn local_reverse_lookup_ident<'x>(
        &self,
        env: &mut Environment<'x>,
        value: ConstructId,
    ) -> ReverseLookupIdentResult;
    fn local_lookup_invariant<'x>(
        &self,
        env: &mut Environment<'x>,
        invariant: ConstructId,
        limit: u32,
    ) -> LookupSimilarInvariantResult;
    fn parent(&self) -> Option<ConstructId>;

    fn lookup_ident<'x>(&self, env: &mut Environment<'x>, ident: &str) -> LookupIdentResult {
        if let Some(result) = self.local_lookup_ident(env, ident)? {
            Ok(Some(result))
        } else if let Some(parent) = self.parent() {
            env.get_construct(parent)
                .scope
                .dyn_clone()
                .lookup_ident(env, ident)
        } else {
            Ok(None)
        }
    }

    fn reverse_lookup_ident<'x>(
        &self,
        env: &mut Environment<'x>,
        value: ConstructId,
    ) -> ReverseLookupIdentResult {
        if let Some(result) = self.local_reverse_lookup_ident(env, value)? {
            if result.len() > 0 {
                return Ok(Some(result.to_owned()));
            }
        }
        if let Some(parent) = self.parent() {
            env.get_construct(parent)
                .scope
                .dyn_clone()
                .reverse_lookup_ident(env, value)
        } else {
            Ok(None)
        }
    }

    fn lookup_invariant_limited<'x>(
        &self,
        env: &mut Environment<'x>,
        invariant: ConstructId,
        limit: u32,
    ) -> LookupSimilarInvariantResult {
        let result = self.local_lookup_invariant(env, invariant, limit);
        match result {
            Ok(inv) => Ok(inv),
            Err(LookupInvariantError::MightNotExist)
            | Err(LookupInvariantError::DefinitelyDoesNotExist) => {
                if let Some(parent) = self.parent() {
                    let parent_result = env
                        .get_construct(parent)
                        .scope
                        .dyn_clone()
                        .lookup_invariant_limited(env, invariant, limit);
                    if parent_result == Err(LookupInvariantError::DefinitelyDoesNotExist) {
                        result
                    } else {
                        parent_result
                    }
                } else {
                    result
                }
            }
            Err(other) => Err(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SPlain(pub ConstructId);

impl Scope for SPlain {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn local_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _ident: &str,
    ) -> LookupIdentResult {
        Ok(None)
    }

    fn local_reverse_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _value: ConstructId,
    ) -> ReverseLookupIdentResult {
        Ok(None)
    }

    fn local_lookup_invariant<'x>(
        &self,
        _env: &mut Environment<'x>,
        _invariant: ConstructId,
        _limit: u32,
    ) -> LookupSimilarInvariantResult {
        Err(LookupInvariantError::DefinitelyDoesNotExist)
    }

    fn parent(&self) -> Option<ConstructId> {
        Some(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct SRoot;

impl Scope for SRoot {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn local_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _ident: &str,
    ) -> LookupIdentResult {
        Ok(None)
    }

    fn local_reverse_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _value: ConstructId,
    ) -> ReverseLookupIdentResult {
        Ok(None)
    }

    fn local_lookup_invariant<'x>(
        &self,
        env: &mut Environment<'x>,
        invariant: ConstructId,
        limit: u32,
    ) -> LookupSimilarInvariantResult {
        let truee = env.get_language_item("true");
        match env.discover_equal(invariant, truee, limit)? {
            Equal::Yes(l, r) => {
                if l.len() == 0 && r.len() == 0 {
                    Ok((Invariant::new(truee, hashset![]), Equal::yes()))
                } else if l.len() > 0 {
                    Err(LookupInvariantError::DefinitelyDoesNotExist)
                } else {
                    unreachable!()
                }
            }
            Equal::NeedsHigherLimit => Err(LookupInvariantError::MightNotExist),
            Equal::Unknown | Equal::No => Err(LookupInvariantError::DefinitelyDoesNotExist),
        }
    }

    fn parent(&self) -> Option<ConstructId> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct SPlaceholder;

impl Scope for SPlaceholder {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn is_placeholder(&self) -> bool {
        true
    }

    fn local_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _ident: &str,
    ) -> LookupIdentResult {
        unreachable!()
    }

    fn local_reverse_lookup_ident<'x>(
        &self,
        _env: &mut Environment<'x>,
        _value: ConstructId,
    ) -> ReverseLookupIdentResult {
        unreachable!()
    }

    fn local_lookup_invariant<'x>(
        &self,
        _env: &mut Environment<'x>,
        _invariant: ConstructId,
        _limit: u32,
    ) -> LookupSimilarInvariantResult {
        unreachable!()
    }

    fn parent(&self) -> Option<ConstructId> {
        unreachable!()
    }
}

#[derive(Clone, Debug)]
pub struct SWithParent<Base: Scope + Clone>(pub Base, pub ConstructId);

impl<Base: Scope + Clone + 'static> Scope for SWithParent<Base> {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn is_placeholder(&self) -> bool {
        false
    }

    fn local_lookup_ident<'x>(&self, env: &mut Environment<'x>, ident: &str) -> LookupIdentResult {
        self.0.local_lookup_ident(env, ident)
    }

    fn local_reverse_lookup_ident<'x>(
        &self,
        env: &mut Environment<'x>,
        value: ConstructId,
    ) -> ReverseLookupIdentResult {
        self.0.local_reverse_lookup_ident(env, value)
    }

    fn local_lookup_invariant<'x>(
        &self,
        env: &mut Environment<'x>,
        invariant: ConstructId,
        limit: u32,
    ) -> LookupSimilarInvariantResult {
        self.0.local_lookup_invariant(env, invariant, limit)
    }

    fn parent(&self) -> Option<ConstructId> {
        Some(self.1)
    }
}
