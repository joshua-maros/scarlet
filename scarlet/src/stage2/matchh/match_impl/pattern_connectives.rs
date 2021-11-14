use crate::stage2::{
    matchh::result::MatchResult,
    structure::{Environment, ConstructId, VariableId},
};

impl<'x> Environment<'x> {
    pub(super) fn on_right_and(
        &mut self,
        original_value: ConstructId<'x>,
        value: ConstructId<'x>,
        left: ConstructId<'x>,
        eager_vars: &[VariableId<'x>],
        right: ConstructId<'x>,
        var: VariableId<'x>,
    ) -> MatchResult<'x> {
        let left = self
            .matches_impl(original_value, value, left, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        let right = self
            .matches_impl(original_value, value, right, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        MatchResult::and(vec![left, right]).with_sub_if_match(var, original_value)
    }

    pub(super) fn on_left_or(
        &mut self,
        original_value: ConstructId<'x>,
        left: ConstructId<'x>,
        pattern: ConstructId<'x>,
        eager_vars: &[VariableId<'x>],
        right: ConstructId<'x>,
        var: VariableId<'x>,
    ) -> MatchResult<'x> {
        let left = self
            .matches_impl(original_value, left, pattern, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        let right = self
            .matches_impl(original_value, right, pattern, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        MatchResult::and(vec![left, right]).with_sub_if_match(var, original_value)
    }

    pub(super) fn on_right_or(
        &mut self,
        original_value: ConstructId<'x>,
        value: ConstructId<'x>,
        left: ConstructId<'x>,
        eager_vars: &[VariableId<'x>],
        right: ConstructId<'x>,
        var: VariableId<'x>,
    ) -> MatchResult<'x> {
        let left = self
            .matches_impl(original_value, value, left, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        let right = self
            .matches_impl(original_value, value, right, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        MatchResult::or(vec![left, right]).with_sub_if_match(var, original_value)
    }

    pub(super) fn on_left_and(
        &mut self,
        original_value: ConstructId<'x>,
        left: ConstructId<'x>,
        pattern: ConstructId<'x>,
        eager_vars: &[VariableId<'x>],
        right: ConstructId<'x>,
        var: VariableId<'x>,
    ) -> MatchResult<'x> {
        let left = self
            .matches_impl(original_value, left, pattern, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        let right = self
            .matches_impl(original_value, right, pattern, eager_vars)
            .keeping_only_eager_subs(eager_vars);
        MatchResult::or(vec![left, right]).with_sub_if_match(var, original_value)
    }
}
