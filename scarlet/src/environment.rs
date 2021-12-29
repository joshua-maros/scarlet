pub mod dependencies;
pub mod overlay;
pub mod path;
mod reduce;
pub mod resolve;
pub mod substitute;
pub mod util;
mod vomit;

use std::collections::HashMap;

use crate::{
    constructs::{
        base::{AnnotatedConstruct, ConstructDefinition, ConstructId, ConstructPool},
        unique::{Unique, UniqueId, UniquePool},
        variable::{Variable, VariableId, VariablePool},
        Construct,
    },
    resolvable::{BoxedResolvable, RPlaceholder, Resolvable},
    scope::{SRoot, Scope},
    shared::{Pool, TripleBool},
};

pub const LANGUAGE_ITEM_NAMES: &[&str] = &["true", "false", "void"];

#[derive(Debug)]
pub struct Environment<'x> {
    language_items: HashMap<&'static str, ConstructId>,
    pub(crate) constructs: ConstructPool<'x>,
    pub(crate) uniques: UniquePool,
    pub(crate) variables: VariablePool,
}

impl<'x> Environment<'x> {
    pub fn new() -> Self {
        let mut this = Self {
            language_items: HashMap::new(),
            constructs: Pool::new(),
            uniques: Pool::new(),
            variables: Pool::new(),
        };
        for &name in LANGUAGE_ITEM_NAMES {
            let id = this.push_placeholder(Box::new(SRoot));
            this.language_items.insert(name, id);
        }
        this
    }

    pub fn define_language_item(&mut self, name: &str, definition: ConstructId) {
        let id = self.get_language_item(name);
        self.constructs[id].definition = definition.into();
    }

    pub fn define_construct(&mut self, construct: ConstructId, definition: impl Construct) {
        self.define_dyn_construct(construct, Box::new(definition))
    }

    pub fn define_dyn_construct(&mut self, construct: ConstructId, definition: Box<dyn Construct>) {
        self.constructs[construct].definition = definition.into();
    }

    pub fn define_unresolved(
        &mut self,
        construct: ConstructId,
        definition: impl Resolvable<'x> + 'x,
    ) {
        self.constructs[construct].definition =
            ConstructDefinition::Unresolved(Box::new(definition));
    }

    pub fn get_language_item(&self, name: &str) -> ConstructId {
        *self
            .language_items
            .get(name)
            .unwrap_or_else(|| todo!("nice error, no language item named {}", name))
    }

    pub fn push_placeholder(&mut self, scope: Box<dyn Scope>) -> ConstructId {
        let con = AnnotatedConstruct {
            definition: ConstructDefinition::Unresolved(Box::new(RPlaceholder)),
            scope,
        };
        self.constructs.push(con)
    }

    pub fn push_construct(
        &mut self,
        construct: impl Construct,
        scope: Box<dyn Scope>,
    ) -> ConstructId {
        self.push_dyn_construct(Box::new(construct), scope)
    }

    pub fn push_dyn_construct(
        &mut self,
        construct: Box<dyn Construct>,
        scope: Box<dyn Scope>,
    ) -> ConstructId {
        let con = AnnotatedConstruct {
            definition: ConstructDefinition::Resolved(construct),
            scope,
        };
        self.constructs.push(con)
    }

    pub fn push_unique(&mut self) -> UniqueId {
        self.uniques.push(Unique)
    }

    pub fn push_variable(&mut self) -> VariableId {
        self.variables.push(Variable)
    }

    pub fn push_unresolved(
        &mut self,
        definition: impl Resolvable<'x> + 'x,
        scope: Box<dyn Scope>,
    ) -> ConstructId {
        self.push_dyn_unresolved(Box::new(definition), scope)
    }

    pub fn push_dyn_unresolved(
        &mut self,
        definition: BoxedResolvable<'x>,
        scope: Box<dyn Scope>,
    ) -> ConstructId {
        self.constructs.push(AnnotatedConstruct {
            definition: ConstructDefinition::Unresolved(definition),
            scope,
        })
    }

    pub(crate) fn check(&mut self, con_id: ConstructId) {
        let con = self.get_construct_definition(con_id).dyn_clone();
        con.check(self);
    }

    pub(crate) fn check_all(&mut self) {
        let mut next_id = self.constructs.first();
        while let Some(id) = next_id {
            self.check(id);
            next_id = self.constructs.next(id);
        }
    }

    pub(crate) fn is_def_equal(&mut self, left: ConstructId, right: ConstructId) -> TripleBool {
        let other = self.get_construct_definition(right).dyn_clone();
        self.get_construct_definition(left)
            .dyn_clone()
            .is_def_equal(self, &*other)
    }
}
