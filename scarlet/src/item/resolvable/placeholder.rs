use super::{BoxedResolvable, Resolvable, ResolveResult};
use crate::{
    environment::Environment,
    impl_any_eq_from_regular_eq,
    item::{ContainmentType, ItemPtr},
    scope::Scope,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RPlaceholder;

impl_any_eq_from_regular_eq!(RPlaceholder);

impl Resolvable for RPlaceholder {
    fn is_placeholder(&self) -> bool {
        true
    }

    fn dyn_clone(&self) -> BoxedResolvable {
        Box::new(self.clone())
    }

    fn resolve(
        &self,
        env: &mut Environment,
        _this: ItemPtr,
        _scope: Box<dyn Scope>,
        _limit: u32,
    ) -> ResolveResult {
        eprintln!("{:#?}", env);
        unreachable!()
    }

    fn contents(&self) -> Vec<(ContainmentType, &ItemPtr)> {
        vec![]
    }
}
