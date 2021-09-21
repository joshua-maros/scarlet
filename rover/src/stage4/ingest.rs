use crate::{
    stage2::structure::{ItemId, PrimitiveValue, Replacements},
    stage3::structure::{self as stage3, Item},
    stage4::structure::Environment,
};
use std::collections::HashSet;

pub fn ingest(from: stage3::Environment) -> Result<Environment, String> {
    let mut env = Environment::new(from);
    let mut next_item = ItemId(0);
    while next_item.0 < env.items.len() {
        env.compute_type(next_item)?;
        next_item.0 += 1;
    }
    Ok(env)
}

impl Environment {
    fn resolve_variable(&self, reference: ItemId) -> Result<ItemId, String> {
        assert!(reference.0 < self.items.len());
        let item = &self.items[reference.0];
        match &item.base {
            Item::Defining { base, .. } => {
                let base = *base;
                self.resolve_variable(base)
            }
            Item::FromType { .. } => todo!("nice error"),
            Item::Replacing { base, .. } => {
                let base = *base;
                self.resolve_variable(base)
            }
            Item::GodType
            | Item::InductiveType(..)
            | Item::InductiveValue { .. }
            | Item::PrimitiveType(..)
            | Item::PrimitiveValue(..) => todo!("nice error, not a variable"),
            Item::Variable { selff, .. } => Ok(*selff),
        }
    }

    fn compute_type_after_replacing(
        &mut self,
        base: ItemId,
        replacements: Replacements,
    ) -> Result<ItemId, String> {
        let unreplaced_type = self.compute_type(base)?;
        let mut ids_to_replace = Vec::new();
        for (id, _) in replacements {
            ids_to_replace.push(self.resolve_variable(id)?)
        }
        let def = &self.items[unreplaced_type.0].base;
        let res = match def {
            Item::FromType { base, vars } => {
                let mut vars_after_reps = vars.clone();
                for index in (0..vars_after_reps.len()).rev() {
                    if ids_to_replace
                        .iter()
                        .any(|id| *id == vars_after_reps[index])
                    {
                        vars_after_reps.remove(index);
                    }
                }
                if vars_after_reps.len() == 0 {
                    *base
                } else if &vars_after_reps == vars {
                    unreplaced_type
                } else {
                    let base = *base;
                    self.insert(Item::FromType {
                        base,
                        vars: vars_after_reps,
                    })
                }
            }
            _ => unreplaced_type,
        };
        Ok(res)
    }

    // Collects all variables specified by From items pointed to by the provided ID.
    fn get_from_variables(&mut self, typee: ItemId) -> Result<HashSet<ItemId>, String> {
        Ok(match &self.items[typee.0].base {
            Item::Defining { base: id, .. } => {
                let id = *id;
                self.get_from_variables(id)?
            }
            Item::FromType { base, vars } => {
                let base = *base;
                let vars: HashSet<_> = vars.iter().copied().collect();
                let result = self.get_from_variables(base)?;
                result.union(&vars).copied().collect()
            }
            Item::Replacing { .. } => todo!(),
            _ => HashSet::new(),
        })
    }

    fn compute_type(&mut self, of: ItemId) -> Result<ItemId, String> {
        assert!(of.0 < self.items.len());
        let item = &self.items[of.0];
        let typee = match &item.base {
            Item::Defining { base, .. } => {
                let base = *base;
                self.compute_type(base)?
            }
            // TODO: This is not always correct.
            Item::FromType { .. } => self.god_type(),
            Item::GodType { .. } => self.god_type(),
            // TODO: This is not always correct. Need to finalize how inductive
            // types can depend on vars.
            Item::InductiveType(..) => self.god_type(),
            Item::InductiveValue { typee, records, .. } => {
                let mut from_vars = HashSet::new();
                let typee = *typee;
                for recorded in records.clone() {
                    let typee = self.compute_type(recorded)?;
                    for from_var in self.get_from_variables(typee)? {
                        from_vars.insert(from_var);
                    }
                }
                self.insert(Item::FromType {
                    base: typee,
                    vars: from_vars.into_iter().collect(),
                })
            }
            Item::PrimitiveType(..) => self.god_type(),
            Item::PrimitiveValue(pv) => match pv {
                PrimitiveValue::I32(..) => self.i32_type(),
            },
            Item::Replacing { base, replacements } => {
                let base = *base;
                let replacements = replacements.clone();
                self.compute_type_after_replacing(base, replacements)?
            }
            Item::Variable { typee, selff } => {
                let base = *typee;
                let vars = vec![*selff];
                self.insert(Item::FromType { base, vars })
            }
        };
        self.set_type(of, typee);
        Ok(typee)
    }
}
