use super::{Ecc, Equal, OnlyCalledByEcc, PermissionToRefine};
use crate::item::resolvable::UnresolvedItemError;

pub type EqualResult = Result<Equal, UnresolvedItemError>;

pub trait EqualityFeature {
    #[allow(unused_variables)]
    fn get_equality_using_context(
        &self,
        ctx: &Ecc,
        can_refine: PermissionToRefine,
        _: OnlyCalledByEcc,
    ) -> EqualResult {
        Ok(Equal::Unknown)
    }
}