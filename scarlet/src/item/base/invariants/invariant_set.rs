use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{item::ItemPtr, util::rcrc};

pub type SetJustification = Vec<StatementJustifications>;
pub type StatementJustifications = Vec<StatementJustification>;
pub type StatementJustification = Vec<InvariantSetPtr>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantSet {
    context: ItemPtr,
    statements: Vec<ItemPtr>,
    /// For the original statements to hold, all the statements in this list
    /// must also hold.
    pub(super) justification_requirements: Vec<ItemPtr>,
    pub(super) statement_justifications: Option<SetJustification>,
    pub(super) connected_to_root: bool,
    pub(super) required: bool,
    pub(super) dependencies: HashSet<ItemPtr>,
}

pub type InvariantSetPtr = Rc<RefCell<InvariantSet>>;

impl InvariantSet {
    pub fn new_empty(context: ItemPtr) -> InvariantSetPtr {
        Self::new(context, vec![], vec![], HashSet::new())
    }

    pub fn new(
        context: ItemPtr,
        statements: Vec<ItemPtr>,
        justification_requirements: Vec<ItemPtr>,
        dependencies: HashSet<ItemPtr>,
    ) -> InvariantSetPtr {
        rcrc(Self {
            context,
            statements,
            justification_requirements,
            statement_justifications: None,
            connected_to_root: false,
            required: true,
            dependencies,
        })
    }

    pub fn new_not_required(
        context: ItemPtr,
        statements: Vec<ItemPtr>,
        justification_requirements: Vec<ItemPtr>,
        dependencies: HashSet<ItemPtr>,
    ) -> InvariantSetPtr {
        rcrc(Self {
            context,
            statements,
            justification_requirements,
            statement_justifications: None,
            connected_to_root: false,
            required: false,
            dependencies,
        })
    }

    pub(crate) fn new_justified_by(
        context: ItemPtr,
        statements: Vec<ItemPtr>,
        justified_by: SetJustification,
    ) -> InvariantSetPtr {
        rcrc(Self {
            context,
            statements,
            justification_requirements: Vec::new(),
            statement_justifications: Some(justified_by),
            connected_to_root: false,
            required: false,
            dependencies: HashSet::new(),
        })
    }

    pub(super) fn new_recursive_justification(
        context: ItemPtr,
        dependencies: HashSet<ItemPtr>,
    ) -> InvariantSetPtr {
        rcrc(Self {
            context,
            statements: Vec::new(),
            justification_requirements: Vec::new(),
            statement_justifications: None,
            connected_to_root: true,
            required: false,
            dependencies,
        })
    }

    pub fn new_statements_depending_on(
        context: ItemPtr,
        statements: Vec<ItemPtr>,
        dependencies: HashSet<ItemPtr>,
    ) -> InvariantSetPtr {
        rcrc(Self {
            context,
            statements,
            justification_requirements: Vec::new(),
            statement_justifications: None,
            connected_to_root: false,
            required: true,
            dependencies,
        })
    }

    /// Get a reference to the invariant set's statements.
    #[must_use]
    pub fn statements(&self) -> &[ItemPtr] {
        self.statements.as_ref()
    }

    /// Get a reference to the invariant set's justification requirements.
    #[must_use]
    pub fn justification_requirements(&self) -> &[ItemPtr] {
        self.justification_requirements.as_ref()
    }

    /// Get a reference to the invariant set's justified by.
    #[must_use]
    pub fn justified_by(&self) -> Option<&SetJustification> {
        self.statement_justifications.as_ref()
    }

    /// Get a reference to the invariant set's dependencies.
    #[must_use]
    pub fn dependencies(&self) -> &HashSet<ItemPtr> {
        &self.dependencies
    }

    pub fn push(&mut self, statement: ItemPtr) {
        self.statements.push(statement);
    }
}
