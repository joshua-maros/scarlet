use maplit::hashset;

use super::{BoxedResolvable, Resolvable, ResolveError, ResolveResult};
use crate::{
    diagnostic::{Diagnostic, Position},
    environment::Environment,
    impl_any_eq_from_regular_eq,
    item::{
        definitions::{
            substitution::{DSubstitution, Substitutions},
            variable::{DVariable, Variable},
        },
        dependencies::{Dcc, Dependencies},
        invariants::{InvariantSet, InvariantsResult},
        util::unchecked_substitution,
        ContainmentType, ItemDefinition, ItemPtr,
    },
    scope::Scope,
    shared::OrderedMap,
    util::PtrExtension,
};

#[derive(Clone, Debug)]
pub struct RSubstitution {
    pub base: ItemPtr,
    pub position: Position,
    pub named_subs: Vec<(Position, String, ItemPtr)>,
    pub anonymous_subs: Vec<ItemPtr>,
}

impl PartialEq for RSubstitution {
    fn eq(&self, other: &Self) -> bool {
        if !self.base.is_same_instance_as(&other.base)
            || self.named_subs.len() != other.named_subs.len()
            || self.anonymous_subs.len() != other.anonymous_subs.len()
        {
            return false;
        }
        'next_sub: for (_, key, value) in &self.named_subs {
            for (_, other_key, other_value) in &other.named_subs {
                if key == other_key {
                    if value.is_same_instance_as(other_value) {
                        continue 'next_sub;
                    } else {
                        // The target is replaced with something else.
                        return false;
                    }
                }
            }
            // There is no matching substitution.
            return false;
        }
        for (sub, other_sub) in self.anonymous_subs.iter().zip(other.anonymous_subs.iter()) {
            if !sub.is_same_instance_as(other_sub) {
                return false;
            }
        }
        true
    }
}

impl_any_eq_from_regular_eq!(RSubstitution);

impl Resolvable for RSubstitution {
    fn dyn_clone(&self) -> BoxedResolvable {
        Box::new(self.clone())
    }

    fn resolve(
        &self,
        env: &mut Environment,
        this: ItemPtr,
        _scope: Box<dyn Scope>,
        limit: u32,
    ) -> ResolveResult {
        let base = self.base.dereference();
        let base_scope = base.clone_scope();
        let mut subs = OrderedMap::new();
        let mut remaining_deps = self.base.get_dependencies();
        let total_dep_count = remaining_deps.num_variables();

        self.resolve_named_subs(base_scope, env, &mut subs, &mut remaining_deps)?;
        self.resolve_anonymous_subs(total_dep_count, remaining_deps, env, &mut subs)?;
        resolve_dep_subs(&mut subs)?;

        let justifications = make_justification_statements(&subs, limit)?;
        let invs = create_invariants(env, this, base, &subs, justifications)?;
        let csub = DSubstitution::new(self.base.ptr_clone(), subs, invs);
        ResolveResult::Ok(csub.clone_into_box())
    }

    fn estimate_dependencies(&self, ctx: &mut Dcc, affects_return_value: bool) -> Dependencies {
        let mut result = Dependencies::new();
        for (_, _, arg) in &self.named_subs {
            result.append(ctx.get_dependencies(arg, affects_return_value));
        }
        for arg in &self.anonymous_subs {
            result.append(ctx.get_dependencies(arg, affects_return_value));
        }
        result
    }

    fn contents(&self) -> Vec<(ContainmentType, &ItemPtr)> {
        let mut result = vec![(ContainmentType::Computational, &self.base)];
        for (_, _, value) in &self.named_subs {
            result.push((ContainmentType::Computational, value));
        }
        for value in &self.anonymous_subs {
            result.push((ContainmentType::Computational, value));
        }
        result
    }
}

fn create_invariants(
    _env: &mut Environment,
    this: ItemPtr,
    base: ItemPtr,
    subs: &Substitutions,
    justifications: Vec<ItemPtr>,
) -> InvariantsResult {
    let mut invs = Vec::new();
    let base_set = base.get_invariants()?;
    for inv in base_set.borrow().statements() {
        // Apply the substitutions to the statement the invariant is making.
        let new_inv = unchecked_substitution(inv.ptr_clone(), subs);
        invs.push(new_inv);
    }
    Ok(InvariantSet::new(this, invs, justifications, hashset![]))
}

/// Finds invariants that confirm the substitutions we're performing are legal.
/// For example, an_int[an_int IS something] would need an invariant of the form
/// `(an_int FROM I32)[an_int IS something]`.
fn make_justification_statements(
    subs: &Substitutions,
    _limit: u32,
) -> Result<Vec<ItemPtr>, ResolveError> {
    let mut justifications = Vec::new();
    let mut previous_subs = Substitutions::new();
    for (target, value) in subs {
        justifications.append(&mut Variable::assignment_justifications(
            target,
            value.ptr_clone(),
            &previous_subs,
        ));
        previous_subs.insert_no_replace(target.ptr_clone(), value.ptr_clone());
    }
    Ok(justifications)
}

/// Turns things like fx[fx IS gy] to fx[fx IS gy[y IS x]] so that the
/// dependencies match.
fn resolve_dep_subs(subs: &mut Substitutions) -> Result<(), ResolveError> {
    for (target, value) in subs {
        let mut dep_subs = Substitutions::new();
        let value_deps = value.get_dependencies();
        let mut value_deps_iter = value_deps.as_variables();
        for dep in target.borrow().get_dependencies() {
            let dep_args = dep.get_dependencies();
            for desired_arg in dep_args.as_complete_variables()? {
                // We want to convert a dependency in the value to the
                // dependency required by the variable it is assigned to.
                if let Some(existing_dep) = value_deps_iter.next() {
                    if !existing_dep.is_same_variable_as(&desired_arg) {
                        let desired_dep = desired_arg.var.borrow().item().ptr_clone();
                        dep_subs.insert_no_replace(existing_dep.var.ptr_clone(), desired_dep);
                    }
                } else if let Some(err) = value_deps.error() {
                    return Err(err.clone().into());
                }
            }
        }
        if dep_subs.len() > 0 {
            *value = unchecked_substitution(value.ptr_clone(), &dep_subs);
        }
    }
    Ok(())
}

impl RSubstitution {
    fn resolve_anonymous_subs(
        &self,
        total_dep_count: usize,
        mut remaining_deps: Dependencies,
        env: &mut Environment,
        subs: &mut Substitutions,
    ) -> Result<(), ResolveError> {
        for value in &self.anonymous_subs {
            if remaining_deps.num_variables() == 0 {
                if let Some(partial_dep_error) = remaining_deps.error() {
                    return Err(partial_dep_error.clone().into());
                } else {
                    return Err(Diagnostic::new()
                        .with_text_error(format!("This substitution has too many arguments:"))
                        .with_source_code_block_error(self.position)
                        .with_text_info(format!(
                            "The base only has {} dependencies:",
                            total_dep_count
                        ))
                        .with_item_info(&self.base, &self.base, env)
                        .into());
                }
            }
            let dep = remaining_deps.pop_front().var;
            subs.insert_no_replace(dep, value.ptr_clone());
        }
        Ok(())
    }

    fn resolve_named_subs(
        &self,
        base_scope: Box<dyn Scope>,
        env: &mut Environment,
        subs: &mut Substitutions,
        remaining_deps: &mut Dependencies,
    ) -> Result<(), ResolveError> {
        for (position, name, value) in &self.named_subs {
            let target = base_scope.lookup_ident(&name)?.unwrap();
            if let Some(var) = target
                .dereference()
                .downcast_resolved_definition::<DVariable>()?
            {
                subs.insert_no_replace(var.get_variable().ptr_clone(), value.ptr_clone());
                remaining_deps.remove(var.get_variable());
            } else {
                return Err(Diagnostic::new()
                    .with_text_error(format!(
                        "{} is used as a variable here but it is actually something else:",
                        name
                    ))
                    .with_source_code_block_error(*position)
                    .with_text_info(format!("{} is actually defined as follows:", name))
                    .with_item_info(&target, value, env)
                    .into());
            }
            drop(target);
        }
        Ok(())
    }
}
