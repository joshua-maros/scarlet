use crate::{
    shared::OrderedMap,
    stage2::{
        matchh::MatchResult,
        structure::{
            After, Condition, Definition, Environment, Item, ItemId, Substitution, Target,
        },
    },
};

impl<'x> Environment<'x> {
    pub(super) fn reduce_after(
        &mut self,
        original: ItemId<'x>,
        base: ItemId<'x>,
        vals: Vec<ItemId<'x>>,
    ) -> ItemId<'x> {
        let base = self.reduce(base);
        let vals: Vec<_> = vals.into_iter().map(|i| self.reduce(i)).collect();
        let mut vals_with_deps = Vec::new();
        for val in vals {
            if self.get_deps(val).len() > 0 {
                vals_with_deps.push(val);
            }
        }
        if vals_with_deps.len() == 0 {
            base
        } else {
            let def = Definition::After {
                base,
                vals: vals_with_deps,
            };
            self.item_with_new_definition(original, def, false)
        }
    }

    pub(super) fn reduce_match(
        &mut self,
        base: ItemId<'x>,
        else_value: ItemId<'x>,
        conditions: Vec<Condition<'x>>,
        original: ItemId<'x>,
    ) -> ItemId<'x> {
        let base = self.reduce(base);
        let mut new_conditions = Vec::new();
        let mut else_value = else_value;
        for condition in conditions.clone() {
            let pattern = self.reduce(condition.pattern);
            // Don't reduce yet as that might lead to needless infinite recursion.
            let value = condition.value;
            match self.matches(base, pattern) {
                MatchResult::Match(subs) => {
                    // If the match is always true, no need to evaluate further conditions.
                    // This should always be used when the previous conditions fail.
                    if subs.len() > 0 {
                        else_value = self.substitute(condition.value, &subs).unwrap();
                    } else {
                        else_value = condition.value;
                    }
                    break;
                }
                // If the match will never occur, skip it.
                MatchResult::NoMatch => (),
                // If the match might occur, save it for later.
                MatchResult::Unknown => new_conditions.push(Condition { pattern, value }),
            }
        }
        let is_fundamentally_different = conditions != new_conditions;
        if new_conditions.len() == 0 {
            self.reduce(else_value)
        } else {
            let def = Definition::Match {
                base,
                conditions: new_conditions,
                else_value,
            };
            self.item_with_new_definition(original, def, is_fundamentally_different)
        }
    }

    pub(super) fn reduce_member(&mut self, base: ItemId<'x>, member: String) -> ItemId<'x> {
        let rbase = self.reduce(base);
        if let Definition::Struct(fields) = self.definition_of(rbase) {
            for field in fields {
                if let Some(name) = &field.name {
                    if name == &member {
                        return field.value;
                    }
                }
            }
            if let Ok(index) = member.parse::<usize>() {
                return fields[index].value;
            }
            todo!("Nice error, no member named {:?}", member)
        } else {
            todo!()
        }
    }

    pub(super) fn reduce_other(&mut self, original: ItemId<'x>, base: ItemId<'x>) -> ItemId<'x> {
        self.reduce(base)
    }

    pub(super) fn reduce_substitution(
        &mut self,
        subs: Vec<Substitution<'x>>,
        base: ItemId<'x>,
        original: ItemId<'x>,
    ) -> ItemId<'x> {
        let mut final_subs = OrderedMap::new();
        for sub in subs {
            if let Target::ResolvedItem(target) = sub.target {
                let target = self.reduce(target);
                match self.matches(sub.value, target) {
                    MatchResult::Match(subs) => final_subs = final_subs.union(subs),
                    MatchResult::NoMatch => {
                        todo!(
                            "Nice error, argument {:?} will definitely not match {:?}",
                            sub.value,
                            target
                        )
                    }
                    MatchResult::Unknown => {
                        todo!("Nice error, argument might not match what it is assigned to.")
                    }
                }
            } else {
                unreachable!()
            }
        }
        let base = self.reduce(base);
        let subbed = self.with_fresh_query_stack(|this| this.substitute(base, &final_subs));
        if let Some(subbed) = subbed {
            let shown_from = self.items[original].shown_from.clone();
            self.items[subbed].shown_from = shown_from;
            self.reduce(subbed)
        } else {
            original
        }
    }
}
