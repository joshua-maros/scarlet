use indexmap::IndexMap;
use regex::Regex;

use super::{Node, ParseContext};
use crate::{
    environment::{vomit::VomitContext, Environment},
    item::{resolvable::UnresolvedItemError, ItemPtr},
    scope::Scope, diagnostic::Diagnostic,
};

pub type Precedence = u8;
pub type Priority = u8;
pub type PhraseName = String;

#[derive(Debug)]
pub struct NodePhraseComponent {
    pub prec: Precedence,
    pub additional_allowed_phraes: Vec<PhraseName>,
}

#[derive(Debug)]
pub enum PhraseComponent {
    Node(NodePhraseComponent),
    Text(Regex),
}

impl From<Precedence> for PhraseComponent {
    fn from(prec: Precedence) -> Self {
        Self::Node(NodePhraseComponent {
            prec,
            additional_allowed_phraes: vec![],
        })
    }
}

impl From<&str> for PhraseComponent {
    fn from(regex: &str) -> Self {
        Self::Text(Regex::new(regex).unwrap())
    }
}

impl PhraseComponent {
    /// Returns `true` if the phrase component is [`Node`].
    ///
    /// [`Node`]: PhraseComponent::Node
    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node { .. })
    }

    /// Returns `true` if the phrase component is [`Text`].
    ///
    /// [`Text`]: PhraseComponent::Text
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(..))
    }
}

pub type CreateFn = fn(&ParseContext, &mut Environment, Box<dyn Scope>, &Node) -> Result<ItemPtr, Diagnostic>;

pub type UncreateFn =
    for<'a> fn(&mut Environment, &mut VomitContext<'a, '_>, ItemPtr) -> UncreateResult<'a>;
pub type UncreateResult<'a> = Result<Option<Node<'a>>, UnresolvedItemError>;

pub type VomitFn = fn(&ParseContext, &Node) -> String;

pub struct Phrase {
    pub name: &'static str,
    pub components: Vec<PhraseComponent>,
    pub create_and_uncreate: Option<(CreateFn, UncreateFn)>,
    pub vomit: VomitFn,
    pub precedence: Precedence,
    pub priority: Priority,
    pub vomit_priority: Priority,
}

impl Phrase {
    pub fn upcoming(
        &self,
        starting_at_component: usize,
    ) -> (Option<&NodePhraseComponent>, Option<&Regex>) {
        let mut preceding_node_precedence = None;
        for component in &self.components[starting_at_component..] {
            if let PhraseComponent::Text(regex) = component {
                return (preceding_node_precedence, Some(regex));
            } else if let PhraseComponent::Node(node) = component {
                preceding_node_precedence = Some(node)
            }
        }
        (preceding_node_precedence, None)
    }
}

pub type PhraseTable = IndexMap<PhraseName, Phrase>;
