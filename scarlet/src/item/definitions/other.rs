use std::fmt::Debug;

use crate::{
    impl_any_eq_from_regular_eq,
    item::{
        check::CheckFeature,
        dependencies::{Dcc, DepResult, Dependencies, DependenciesFeature, OnlyCalledByDcc},
        equality::{Ecc, Equal, EqualResult, EqualityFeature, EqualityTestSide, OnlyCalledByEcc},
        invariants::{
            Icc, InvariantSet, InvariantSetPtr, InvariantsFeature, InvariantsResult,
            OnlyCalledByIcc,
        },
        ContainmentType, ItemDefinition, ItemPtr,
    },
    shared::{Id, Pool},
};

#[derive(Clone, PartialEq, Eq)]
pub struct DOther {
    other: ItemPtr,
    computationally_recursive: bool,
    definitionally_recursive: bool,
}

impl Debug for DOther {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.computationally_recursive {
            write!(
                f,
                "(computationally recursive) {}",
                self.other.debug_label()
            )
        } else if self.definitionally_recursive {
            write!(f, "(definitionally recursive) {}", self.other.debug_label())
        } else {
            write!(f, "(other) ")?;
            self.other.fmt(f)
        }
    }
}

impl DOther {
    pub fn new(
        other: ItemPtr,
        definitionally_recursive: bool,
        computationally_recursive: bool,
    ) -> Self {
        Self {
            other,
            definitionally_recursive,
            computationally_recursive,
        }
    }

    pub fn new_plain(other: ItemPtr) -> Self {
        Self::new(other, false, false)
    }

    pub fn new_computationally_recursive(other: ItemPtr) -> Self {
        Self::new(other, false, true)
    }

    pub fn other(&self) -> &ItemPtr {
        &self.other
    }

    pub fn is_recursive(&self) -> bool {
        self.computationally_recursive || self.definitionally_recursive
    }

    pub fn is_computationally_recursive(&self) -> bool {
        self.computationally_recursive
    }

    pub fn mark_recursive(&mut self, t: ContainmentType) {
        if let ContainmentType::Computational = t {
            self.computationally_recursive = true;
        } else {
            self.definitionally_recursive = true;
        }
    }
}

impl_any_eq_from_regular_eq!(DOther);

impl ItemDefinition for DOther {
    fn clone_into_box(&self) -> Box<dyn ItemDefinition> {
        Box::new(self.clone())
    }

    fn contents(&self) -> Vec<(ContainmentType, &ItemPtr)> {
        if self.computationally_recursive || self.definitionally_recursive {
            vec![]
        } else {
            vec![(ContainmentType::Computational, &self.other)]
        }
    }
}

impl CheckFeature for DOther {}

impl DependenciesFeature for DOther {
    fn get_dependencies_using_context(
        &self,
        this: &ItemPtr,
        ctx: &mut Dcc,
        affects_return_value: bool,
        _: OnlyCalledByDcc,
    ) -> DepResult {
        ctx.get_dependencies(&self.other, affects_return_value)
    }
}

impl EqualityFeature for DOther {
    fn get_equality_using_context(&self, ctx: &mut Ecc, _: OnlyCalledByEcc) -> EqualResult {
        if self.computationally_recursive {
            todo!()
        } else {
            ctx.with_primary(self.other.ptr_clone()).get_equality_left()
        }
    }
}

impl InvariantsFeature for DOther {
    fn get_invariants_using_context(
        &self,
        _this: &ItemPtr,
        ctx: &mut Icc,
        _: OnlyCalledByIcc,
    ) -> InvariantsResult {
        ctx.get_invariants(&self.other)
    }
}
