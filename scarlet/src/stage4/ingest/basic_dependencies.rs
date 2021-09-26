use crate::{
    shared::{BuiltinOperation, ItemId},
    stage4::{ingest::var_list::VarList, structure::Environment},
    util::*,
};

impl Environment {
    pub fn from_type_dependencies(
        &mut self,
        base: ItemId,
        values: Vec<ItemId>,
        currently_computing: Vec<ItemId>,
    ) -> MaybeResult<VarList, String> {
        let mut res = VarList::new();
        for value in values {
            let value_deps = self.compute_dependencies(value, currently_computing.clone())?;
            res.append(value_deps.as_slice());
        }
        let base_deps = self.compute_dependencies(base, currently_computing.clone())?;
        res.append(base_deps.as_slice());
        MOk(res)
    }

    pub fn primitive_op_dependencies(
        &mut self,
        op: BuiltinOperation,
        currently_computing: Vec<ItemId>,
    ) -> MaybeResult<VarList, String> {
        let inputs = op.inputs();
        let mut res = VarList::new();
        for input in inputs {
            let input_deps = self.compute_dependencies(input, currently_computing.clone())?;
            res.append(input_deps.as_slice());
        }
        MOk(res)
    }

    pub fn variable_dependencies(
        &mut self,
        selff: ItemId,
        typee: ItemId,
        currently_computing: Vec<ItemId>,
    ) -> MaybeResult<VarList, String> {
        let mut res = self.compute_dependencies(typee, currently_computing)?;
        res.push(selff);
        MOk(res)
    }

    pub fn variant_instance_dependencies(
        &mut self,
        typee: ItemId,
        values: Vec<ItemId>,
        currently_computing: Vec<ItemId>,
    ) -> MaybeResult<VarList, String> {
        let mut res = self.compute_dependencies(typee, currently_computing.clone())?;
        for value in values {
            let value_deps = self.compute_dependencies(value, currently_computing.clone())?;
            res.append(value_deps.as_slice());
        }
        MOk(res)
    }
}