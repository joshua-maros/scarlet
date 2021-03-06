use super::{BoxedResolvable, Resolvable, ResolveResult, UnresolvedItemError};
use crate::{
    diagnostic::{Diagnostic, Position},
    environment::Environment,
    impl_any_eq_from_regular_eq,
    item::{
        definitions::structt::{AtomicStructMember, DAtomicStructMember, DPopulatedStruct},
        ContainmentType, Item, ItemDefinition, ItemPtr,
    },
    scope::Scope,
};

#[derive(Clone, Debug)]
pub struct RNamedMember {
    pub base: ItemPtr,
    pub member_name: String,
    pub position: Position,
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
    if let Some(cstruct) = inn
        .dereference_resolved()?
        .downcast_resolved_definition::<DPopulatedStruct>()?
    {
        if cstruct.get_label() == name {
            Ok(Some(0))
        } else {
            let rest = cstruct.get_rest().ptr_clone();
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
        let access_depth = find_member(env, self.base.ptr_clone(), &self.member_name)?;
        let access_depth = if let Some(ad) = access_depth {
            ad
        } else {
            return ResolveResult::Err(
                Diagnostic::new()
                    .with_text_error(format!(
                        "Failed to find a member named {}:",
                        self.member_name
                    ))
                    .with_source_code_block_error(self.position)
                    .with_text_info(format!("The base is defined as follows:"))
                    .with_item_info(
                        self.base.dereference_once().as_ref().unwrap_or(&self.base),
                        &this,
                        env,
                    )
                    .into(),
            );
        };
        let mut base = self.base.ptr_clone();
        for _ in 0..access_depth {
            base = Item::new_boxed(
                Box::new(DAtomicStructMember::new(base, AtomicStructMember::Rest)),
                scope.dyn_clone(),
            );
        }
        let def = DAtomicStructMember::new(base, AtomicStructMember::Value);
        ResolveResult::Ok(def.clone_into_box())
    }

    fn contents(&self) -> Vec<(ContainmentType, &ItemPtr)> {
        vec![(ContainmentType::Computational, &self.base)]
    }
}
