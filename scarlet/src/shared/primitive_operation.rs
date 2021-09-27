use std::fmt::{self, Debug, Formatter};

use super::ItemId;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IntegerMathOperation {
    Sum(ItemId, ItemId),
    Difference(ItemId, ItemId),
    /* Multiply(ItemId, ItemId),
     * IntegerDivide(ItemId, ItemId),
     * Modulo(ItemId, ItemId),
     * Negate(ItemId), */
}

impl IntegerMathOperation {
    pub fn inputs(&self) -> Vec<ItemId> {
        match self {
            Self::Sum(a, b) | Self::Difference(a, b) => vec![*a, *b],
        }
    }

    pub fn with_inputs(&self, new_inputs: Vec<ItemId>) -> Self {
        match self {
            Self::Sum(..) => {
                assert_eq!(new_inputs.len(), 2);
                Self::Sum(new_inputs[0], new_inputs[1])
            }
            Self::Difference(..) => {
                assert_eq!(new_inputs.len(), 2);
                Self::Difference(new_inputs[0], new_inputs[1])
            }
        }
    }
}

impl Debug for IntegerMathOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sum(l, r) => write!(f, "sum {:?} {:?}}}", l, r),
            Self::Difference(l, r) => write!(f, "difference {:?} {:?}}}", l, r),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BuiltinOperation {
    I32Math(IntegerMathOperation),
    AreSameVariant {
        base: ItemId,
        other: ItemId,
    },
    Reinterpret {
        this: ItemId,
        parameterized_type: ItemId,
        original: ItemId,
    },
}

impl BuiltinOperation {
    pub fn inputs(&self) -> Vec<ItemId> {
        match self {
            Self::I32Math(op) => op.inputs(),
            Self::AreSameVariant { base, other } => vec![*base, *other],
            Self::Reinterpret {
                this,
                parameterized_type,
                original,
            } => vec![*this, *parameterized_type, *original],
        }
    }

    pub fn with_inputs(&self, new_inputs: Vec<ItemId>) -> Self {
        match self {
            Self::I32Math(op) => Self::I32Math(op.with_inputs(new_inputs)),
            Self::AreSameVariant { .. } => {
                assert_eq!(new_inputs.len(), 2);
                let base = new_inputs[0];
                let other = new_inputs[1];
                Self::AreSameVariant { base, other }
            }
            Self::Reinterpret { .. } => {
                assert_eq!(new_inputs.len(), 3);
                Self::Reinterpret {
                    this: new_inputs[0],
                    parameterized_type: new_inputs[1],
                    original: new_inputs[2],
                }
            }
        }
    }
}

impl Debug for BuiltinOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32Math(op) => write!(f, "builtin_item{{i32_{:?}", op),
            Self::AreSameVariant { base, other } => {
                write!(f, "builtin_item{{are_same_variant {:?} {:?}}}", base, other)
            }
            Self::Reinterpret {
                this,
                parameterized_type,
                original,
            } => {
                write!(
                    f,
                    "builtin_item{{reinterpret {:?} {:?} {:?}}}",
                    this, parameterized_type, original
                )
            }
        }
    }
}
