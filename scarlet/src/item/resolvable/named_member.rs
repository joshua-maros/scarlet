use super::{BoxedResolvable, Resolvable, ResolveResult, UnresolvedItemError};
use crate::{
    environment::Environment,
    impl_any_eq_from_regular_eq,
    item::{
        definitions::structt::{AtomicStructMember, DAtomicStructMember, DPopulatedStruct},
        Item, ItemDefinition, ItemPtr,
    },
    scope::Scope,
};

#[derive(Clone, Debug)]
pub struct RNamedMember {
    pub base: ItemPtr,
    pub member_name: String,
}

impl PartialEq for RNamedMember {
    fn eq(&self, other: &Self) -> bool {
        self.base.is_same_instance_as(&other.base) && self.member_name == other.member_name
    }
}

impl_any_eq_from_regular_eq!(RNamedMember);

fn find_member(
    env: &mut Environment,
    inn: ItemPtr,
    name: &str,
) -> Result<Option<u32>, UnresolvedItemError> {
    if let Some(cstruct) = inn.downcast_definition::<DPopulatedStruct>() {
        if cstruct.get_label() == name {
            Ok(Some(0))
        } else {
            let rest = cstruct.get_rest();
            Ok(find_member(env, rest, name)?.map(|x| x + 1))
        }
    } else {
        Ok(None)
    }
}

impl Resolvable for RNamedMember {
    fn dyn_clone(&self) -> BoxedResolvable {
        Box::new(self.clone())
    }

    fn resolve(
        &self,
        env: &mut Environment,
        this: ItemPtr,
        scope: Box<dyn Scope>,
        _limit: u32,
    ) -> ResolveResult {
        let access_depth = find_member(env, self.base, &self.member_name)?;
        let access_depth = if let Some(ad) = access_depth {
            ad
        } else {
            todo!(
                "Nice error, failed to find a member named {}.",
                self.member_name
            );
        };
        let mut base = self.base;
        for _ in 0..access_depth {
            base = Item::new_boxed(
                Box::new(DAtomicStructMember::new(base, AtomicStructMember::Rest)),
                scope.dyn_clone(),
            );
        }
        let def = DAtomicStructMember::new(base, AtomicStructMember::Value);
        ResolveResult::Ok(def.clone_into_box())
    }
}