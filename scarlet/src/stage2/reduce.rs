use crate::stage2::{
    matchh::MatchResult,
    structure::{
        BuiltinOperation, BuiltinValue, Definition, Environment, ItemId, StructField, VarType,
    },
};

impl<'x> Environment<'x> {
    pub fn reduce(&mut self, item: ItemId<'x>) -> ItemId<'x> {
        if let Some(reduction) = &self.items[item].cached_reduction {
            *reduction
        } else if self.query_stack_contains(item) {
            println!("{:#?}", self);
            todo!("Handle recursive reduction on {:?}", item)
        } else {
            let result = self.with_query_stack_frame(item, |this| this.reduce_from_scratch(item));
            self.items[item].cached_reduction = Some(result);
            self.get_deps(item);
            self.get_deps(result);
            // println!("{:#?}", self);
            // println!("{:?} becomes {:?}", item, result);
            assert!(self.get_deps(result).len() <= self.get_deps(item).len());
            // println!("{:#?}", self);
            assert_eq!(self.reduce(result), result);
            result
        }
    }

    fn reduce_from_scratch(&mut self, item: ItemId<'x>) -> ItemId<'x> {
        let definition = self.items[item].definition.clone().unwrap();
        match definition {
            Definition::Match {
                base,
                conditions,
                else_value,
            } => {
                let base = self.reduce(base);
                let mut new_conditions = Vec::new();
                let mut else_value = else_value;

                for condition in conditions {
                    match self.matches(base, condition.pattern) {
                        MatchResult::Match(subs) => {
                            else_value = self.substitute(condition.value, &subs).unwrap();
                            break;
                        }
                        MatchResult::NoMatch => (),
                        MatchResult::Unknown => new_conditions.push(condition),
                    }
                }

                let conditions = new_conditions;
                if conditions.len() == 0 {
                    else_value
                } else {
                    let def = Definition::Match {
                        base,
                        conditions,
                        else_value,
                    };
                    self.item_with_new_definition(item, def, false)
                }
            }
            Definition::Member(base, name) => {
                let base = self.reduce(base);
                if let Definition::Struct(fields) = self.get_definition(base) {
                    for (index, field) in fields.iter().enumerate() {
                        if field.name == Some(&name) || format!("{}", index) == name {
                            return field.value;
                        }
                    }
                    todo!("Nice error, no field named {}", name)
                } else {
                    let def = Definition::Member(base, name);
                    self.item_with_new_definition(item, def, false)
                }
            }
            Definition::Other(other) => self.reduce(other),
            Definition::ResolvedSubstitute(base, subs) => {
                let subbed = self.substitute(base, &subs).unwrap();
                if subbed == item {
                    subbed
                } else {
                    self.reduce(subbed)
                }
            }
            Definition::UnresolvedSubstitute(_, _) => {
                self.resolve_substitution(item);
                self.reduce_from_scratch(item)
            }
            _ => {
                let reduced_definition = self.reduce_definition(definition);
                self.item_with_new_definition(item, reduced_definition, false)
            }
        }
    }

    fn reduce_op(
        &mut self,
        original_def: Definition<'x>,
        args: Vec<ItemId<'x>>,
        compute: impl Fn(Vec<BuiltinValue>) -> BuiltinValue,
    ) -> Definition<'x> {
        if let Some(args) = self.args_as_builtin_values(&args[..]) {
            Definition::BuiltinValue(compute(args))
        } else {
            original_def
        }
    }

    fn reduce_32u_32u_x_op(
        &mut self,
        original_def: Definition<'x>,
        args: Vec<ItemId<'x>>,
        compute: impl Fn(u32, u32) -> BuiltinValue,
    ) -> Definition<'x> {
        self.reduce_op(original_def, args, |args| {
            compute(args[0].unwrap_32u(), args[1].unwrap_32u())
        })
    }

    fn reduce_32u_32u_32u_op(
        &mut self,
        original_def: Definition<'x>,
        args: Vec<ItemId<'x>>,
        compute: impl Fn(u32, u32) -> u32,
    ) -> Definition<'x> {
        self.reduce_32u_32u_x_op(original_def, args, |a, b| BuiltinValue::_32U(compute(a, b)))
    }

    fn reduce_32u_32u_bool_op(
        &mut self,
        original_def: Definition<'x>,
        args: Vec<ItemId<'x>>,
        compute: impl Fn(u32, u32) -> bool,
    ) -> Definition<'x> {
        self.reduce_32u_32u_x_op(original_def, args, |a, b| BuiltinValue::Bool(compute(a, b)))
    }

    fn reduce_builtin_op(
        &mut self,
        def: Definition<'x>,
        op: BuiltinOperation,
        args: Vec<ItemId<'x>>,
    ) -> Definition<'x> {
        match op {
            BuiltinOperation::Sum32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a + b),
            BuiltinOperation::Difference32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a - b),
            BuiltinOperation::Product32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a * b),
            BuiltinOperation::Quotient32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a / b),
            BuiltinOperation::Modulo32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a % b),
            BuiltinOperation::Power32U => self.reduce_32u_32u_32u_op(def, args, |a, b| a.pow(b)),

            BuiltinOperation::GreaterThan32U => {
                self.reduce_32u_32u_bool_op(def, args, |a, b| a > b)
            }
            BuiltinOperation::GreaterThanOrEqual32U => {
                self.reduce_32u_32u_bool_op(def, args, |a, b| a >= b)
            }
            BuiltinOperation::LessThan32U => self.reduce_32u_32u_bool_op(def, args, |a, b| a < b),
            BuiltinOperation::LessThanOrEqual32U => {
                self.reduce_32u_32u_bool_op(def, args, |a, b| a <= b)
            }
        }
    }

    fn reduce_definition(&mut self, def: Definition<'x>) -> Definition<'x> {
        match def.clone() {
            Definition::Other(_) => unreachable!(),
            Definition::ResolvedSubstitute(..) => unreachable!(),
            Definition::UnresolvedSubstitute(..) => unreachable!(),

            Definition::BuiltinOperation(op, args) => self.reduce_builtin_op(def, op, args),
            Definition::BuiltinValue(..) => def,
            Definition::Match { .. } => unreachable!(),
            Definition::Member(..) => unreachable!(),
            Definition::SetEager { base, vals, eager } => {
                let base = self.reduce(base);
                let vals = vals.into_iter().map(|x| self.reduce(x)).collect();
                Definition::SetEager { base, vals, eager }
            }
            Definition::Struct(fields) => {
                let mut reduced_fields = Vec::new();
                for field in fields {
                    let name = field.name;
                    let value = self.reduce(field.value);
                    reduced_fields.push(StructField { name, value })
                }
                Definition::Struct(reduced_fields)
            }
            Definition::Variable { var, typee } => {
                let typee = match typee {
                    VarType::Bool | VarType::God | VarType::_32U => typee,
                    VarType::Just(other) => VarType::Just(self.reduce(other)),
                    VarType::And(l, r) => VarType::And(self.reduce(l), self.reduce(r)),
                    VarType::Or(l, r) => VarType::Or(self.reduce(l), self.reduce(r)),
                };
                Definition::Variable { var, typee }
            }
        }
    }
}
