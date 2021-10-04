use crate::{
    shared::{Id, OrderedMap, OrderedSet},
    stage2::structure::{BuiltinOperation, BuiltinValue},
};

pub type Substitution = (VariableId, ValueId);
pub type Variables = OrderedSet<VariableId>;

pub type ValueId = Id<Value, 'L'>;
pub type VariableId = Id<Variable, 'V'>;
pub type VariantId = Id<Variant, 'T'>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Any { id: VariableId, typee: ValueId },
    BuiltinOperation(BuiltinOperation<ValueId>),
    BuiltinValue(BuiltinValue),
    From {
        base: ValueId,
        variable: VariableId,
    },
    Substituting {
        base: ValueId,
        target: VariableId,
        value: ValueId,
    },
    Variant(VariantId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Variable;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Variant {
    pub typee: ValueId,
}
