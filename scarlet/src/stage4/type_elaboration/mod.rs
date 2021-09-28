use super::structure::{Environment, ItemDefinition};
use crate::shared::{
    BuiltinOperation, BuiltinValue, IntegerMathOperation, Item, ItemId, Replacements, VarList,
};

impl Environment {
    fn get_or_insert(&mut self, item: Item, defined_in: Option<ItemId>) -> ItemId {
        for (index, this_item) in self.items.iter().enumerate() {
            if this_item.definition == item {
                return ItemId(index);
            }
        }
        let id = ItemId(self.items.len());
        self.items.push(ItemDefinition {
            info_requested: None,
            is_scope: false,
            definition: item,
            defined_in,
            cached_type: None,
        });
        id
    }

    fn get_or_insert_primary_type(&mut self) -> ItemId {
        self.get_or_insert(Item::BuiltinValue(BuiltinValue::PrimaryType), None)
    }

    fn get_or_insert_bool_type(&mut self) -> ItemId {
        self.get_or_insert(Item::BuiltinValue(BuiltinValue::BoolType), None)
    }

    fn get_or_insert_i32_type(&mut self) -> ItemId {
        self.get_or_insert(Item::BuiltinValue(BuiltinValue::I32Type), None)
    }

    fn get(&self, item: ItemId) -> &ItemDefinition {
        &self.items[item.0]
    }

    fn get_mut(&mut self, item: ItemId) -> &mut ItemDefinition {
        &mut self.items[item.0]
    }

    fn get_item(&self, item: ItemId) -> &Item {
        &self.get(item).definition
    }

    pub fn get_type(&mut self, of: ItemId) -> ItemId {
        if let Some(typee) = self.get(of).cached_type {
            return typee;
        }
        let typee = match self.get_item(of) {
            Item::Any { selff, typee } => {
                let (selff, typee) = (*selff, *typee);
                let typee = self.reduce(typee, &Default::default(), &[]);
                let type_type = self.get_type(typee);
                self.flatten_from_types(&[typee, type_type], &[selff])
            }
            Item::BuiltinOperation(op) => {
                let op = *op;
                let inputs = op.inputs();
                let bt = self.builtin_op_base_type(op);
                let mut types = vec![bt];
                for input in inputs {
                    types.push(self.get_type(input));
                }
                self.flatten_from_types(&types, &[])
            }
            Item::BuiltinValue(val) => {
                let val = *val;
                self.type_of_builtin_value(val)
            }
            Item::Defining { base, .. } => {
                let base = *base;
                self.get_type(base)
            }
            Item::FromType { base, .. } => {
                let base = *base;
                self.get_type(base)
            }
            Item::TypeIs { typee, .. } | Item::Variant { typee, .. } => {
                let typee = *typee;
                self.reduce(typee, &Default::default(), &[])
            }
            _ => todo!(),
        };
        self.get_mut(of).cached_type = Some(typee);
        typee
    }

    pub fn elaborate_all_types(&mut self) {
        let mut id = ItemId(0);
        while id.0 < self.items.len() {
            self.get_type(id);
            id.0 += 1;
        }
    }

    fn builtin_op_base_type(&mut self, op: BuiltinOperation) -> ItemId {
        match op {
            BuiltinOperation::I32Math(..) => self.get_or_insert_i32_type(),
            BuiltinOperation::AreSameVariant { .. } => self.get_or_insert_bool_type(),
            BuiltinOperation::Reinterpret { new_type, .. } => new_type,
        }
    }

    fn get_from_vars_and_base(&self, typee: ItemId) -> (VarList, ItemId) {
        match self.get_item(typee) {
            Item::Defining { base, .. } => self.get_from_vars_and_base(*base),
            Item::FromType { base, vars } => {
                let (mut list, base) = self.get_from_vars_and_base(*base);
                list.append(vars);
                (list, base)
            }
            _ => (VarList::new(), typee),
        }
    }

    /// Takes the given types and combines all the variables they specify (if
    /// they are From types) and combines them into a new From type.
    fn flatten_from_types(&mut self, types: &[ItemId], extra_vars: &[ItemId]) -> ItemId {
        assert!(types.len() > 0);
        let mut total_vars = VarList::new();
        let mut total_base = None;
        for typee in types {
            let (vars, base) = self.get_from_vars_and_base(*typee);
            total_vars.append(&vars);
            if total_base.is_none() {
                total_base = Some(base);
            }
        }
        for var in extra_vars {
            total_vars.push(*var);
        }
        if total_vars.len() > 0 {
            let item = Item::FromType {
                base: total_base.unwrap(),
                vars: total_vars,
            };
            let defined_in = self.get(types[0]).defined_in;
            self.get_or_insert(item, defined_in)
        } else {
            total_base.unwrap()
        }
    }

    /// Returns the from vars of the item's type.
    fn get_dependencies(&mut self, item: ItemId) -> VarList {
        let item_type = self.get_type(item);
        self.get_from_vars_and_base(item_type).0
    }

    pub fn reduce_all_items(&mut self) {
        let mut id = ItemId(0);
        while id.0 < self.items.len() {
            let new = self.reduce(id, &Default::default(), &[]);
            if id != new {
                println!("{:?} becomes {:?}", id, new);
            }
            id.0 += 1;
        }
    }

    fn reduce(&mut self, item: ItemId, reps: &Replacements, visited: &[ItemId]) -> ItemId {
        if visited.contains(&item) {
            todo!()
        }
        let visited: Vec<_> = visited
            .iter()
            .copied()
            .chain(std::iter::once(item))
            .collect();
        let defined_in = self.get(item).defined_in;
        match self.get_item(item) {
            Item::Any { selff, .. } => reps.applied_to(*selff),
            Item::BuiltinOperation(op) => {
                let op = *op;
                let mut rinputs = Vec::new();
                for input in op.inputs() {
                    rinputs.push(self.reduce(input, reps, &visited));
                }
                if let Some(reduced) = self.reduce_builtin_operation(defined_in, op, &rinputs) {
                    reduced
                } else {
                    item
                }
            }
            Item::BuiltinValue(..) => item,
            Item::Defining { base, .. } => {
                let base = *base;
                self.reduce(base, reps, &visited)
            }
            Item::FromType { base, vars } => {
                let (base, values) = (*base, vars.clone());
                let mut vars_before_reps = VarList::new();
                for val in values {
                    vars_before_reps.append(&self.get_dependencies(val));
                }
                let mut vars_after_reps = VarList::new();
                for var in vars_before_reps {
                    let repped = reps.applied_to(var);
                    vars_after_reps.append(&self.get_dependencies(repped));
                }
                let rbase = self.reduce(base, reps, &visited);
                let (base, vars) = (rbase, vars_after_reps);
                let item = Item::FromType { base, vars };
                self.get_or_insert(item, defined_in)
            }
            Item::Variant { selff, typee } => {
                let (selff, typee) = (*selff, *typee);
                let item = Item::Variant {
                    selff,
                    typee: self.reduce(typee, reps, &visited),
                };
                self.get_or_insert(item, defined_in)
            }
            _ => todo!(),
        }
    }

    fn get_builtin_value(&self, from: ItemId) -> Option<&BuiltinValue> {
        match self.get_item(from) {
            Item::Defining { base, .. } => {
                let base = *base;
                self.get_builtin_value(base)
            }
            Item::BuiltinValue(val) => Some(val),
            _ => None,
        }
    }

    fn map_op_inputs<T>(
        &self,
        mapper: &impl Fn(&BuiltinValue) -> Option<T>,
        inputs: &[ItemId],
    ) -> Option<Vec<T>> {
        inputs
            .iter()
            .map(|i| self.get_builtin_value(*i).map(mapper).flatten())
            .collect()
    }

    fn reduce_builtin_operation(
        &mut self,
        defined_in: Option<ItemId>,
        op: BuiltinOperation,
        inputs: &[ItemId],
    ) -> Option<ItemId> {
        match op {
            BuiltinOperation::AreSameVariant { .. } => todo!(),
            BuiltinOperation::I32Math(op) => {
                let inputs = self.map_op_inputs(&BuiltinValue::as_i32, inputs)?;
                let val = match op {
                    IntegerMathOperation::Sum(..) => {
                        assert_eq!(inputs.len(), 2);
                        inputs[0] + inputs[1]
                    }
                    IntegerMathOperation::Difference(..) => {
                        assert_eq!(inputs.len(), 2);
                        inputs[0] - inputs[1]
                    }
                };
                let item = Item::BuiltinValue(BuiltinValue::I32(val));
                Some(self.get_or_insert(item, defined_in))
            }
            BuiltinOperation::Reinterpret { .. } => todo!(),
        }
    }

    fn type_of_builtin_value(&mut self, val: BuiltinValue) -> ItemId {
        match val {
            BuiltinValue::PrimaryType | BuiltinValue::BoolType | BuiltinValue::I32Type => {
                self.get_or_insert_primary_type()
            }
            BuiltinValue::Bool(..) => self.get_or_insert_bool_type(),
            BuiltinValue::I32(..) => self.get_or_insert_i32_type(),
        }
    }
}
