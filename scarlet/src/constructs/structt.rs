use super::{
    as_struct,
    base::{Construct, ConstructId},
    substitution::Substitutions,
    variable::CVariable,
    ConstructDefinition,
};
use crate::{
    environment::Environment,
    impl_any_eq_for_construct,
    scope::{SPlain, Scope},
    shared::TripleBool,
};

pub fn struct_from_unnamed_fields<'x>(
    env: &mut Environment<'x>,
    mut fields: Vec<ConstructId>,
) -> ConstructId {
    if fields.is_empty() {
        env.get_builtin_item("void")
    } else {
        let first_field = fields.remove(0);
        let rest = struct_from_unnamed_fields(env, fields);
        CPopulatedStruct::new(env, String::new(), first_field, rest)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CPopulatedStruct {
    label: String,
    value: ConstructId,
    rest: ConstructId,
}

impl CPopulatedStruct {
    pub fn new<'x>(
        env: &mut Environment<'x>,
        label: String,
        value: ConstructId,
        rest: ConstructId,
    ) -> ConstructId {
        let con = env.push_construct(Self { label, value, rest });
        env.set_scope(value, &SFieldAndRest(con));
        env.set_scope(rest, &SField(con));
        con
    }

    pub fn get_label(&self) -> &str {
        &self.label[..]
    }

    pub fn get_value(&self) -> ConstructId {
        self.value
    }

    pub fn get_rest(&self) -> ConstructId {
        self.rest
    }
}

impl_any_eq_for_construct!(CPopulatedStruct);

impl Construct for CPopulatedStruct {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn check<'x>(&self, _env: &mut Environment<'x>) {}

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> Vec<CVariable> {
        [
            env.get_dependencies(self.rest),
            env.get_dependencies(self.value),
        ]
        .concat()
    }

    fn is_def_equal<'x>(&self, _env: &mut Environment<'x>, _other: &dyn Construct) -> TripleBool {
        TripleBool::Unknown
    }

    fn substitute<'x>(
        &self,
        env: &mut Environment<'x>,
        substitutions: &Substitutions,
    ) -> ConstructId {
        let value = env.substitute(self.value, substitutions);
        let rest = env.substitute(self.rest, substitutions);
        Self::new(env, self.label.clone(), value, rest)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicStructMember {
    Label,
    Value,
    Rest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CAtomicStructMember(pub ConstructId, pub AtomicStructMember);

impl_any_eq_for_construct!(CAtomicStructMember);

impl Construct for CAtomicStructMember {
    fn dyn_clone(&self) -> Box<dyn Construct> {
        Box::new(self.clone())
    }

    fn check<'x>(&self, _env: &mut Environment<'x>) {}

    fn get_dependencies<'x>(&self, env: &mut Environment<'x>) -> Vec<CVariable> {
        env.get_dependencies(self.0)
    }

    fn is_def_equal<'x>(&self, _env: &mut Environment<'x>, _other: &dyn Construct) -> TripleBool {
        TripleBool::Unknown
    }

    fn reduce<'x>(&self, env: &mut Environment<'x>) -> ConstructDefinition<'x> {
        env.reduce(self.0);
        if let Some(structt) = as_struct(&**env.get_construct_definition(self.0)) {
            match self.1 {
                AtomicStructMember::Label => todo!(),
                AtomicStructMember::Value => structt.value.into(),
                AtomicStructMember::Rest => structt.rest.into(),
            }
        } else {
            self.dyn_clone().into()
        }
    }

    fn substitute<'x>(
        &self,
        env: &mut Environment<'x>,
        substitutions: &Substitutions,
    ) -> ConstructId {
        let subbed_base = env.substitute(self.0, substitutions);
        let subbed = Self(subbed_base, self.1);
        let con = env.push_construct(subbed);
        env.set_scope(subbed_base, &SPlain(con));
        con
    }
}

#[derive(Debug, Clone)]
pub struct SField(pub ConstructId);

impl Scope for SField {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn local_lookup_ident<'x>(
        &self,
        env: &mut Environment<'x>,
        ident: &str,
    ) -> Option<ConstructId> {
        if let Some(structt) = as_struct(&**env.get_construct_definition(self.0)) {
            let structt = structt.clone();
            if structt.label == ident {
                Some(structt.value)
            } else {
                None
            }
        } else {
            unreachable!()
        }
    }

    fn parent(&self) -> Option<ConstructId> {
        Some(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct SFieldAndRest(pub ConstructId);

fn lookup_ident_in<'x>(
    env: &mut Environment<'x>,
    ident: &str,
    inn: &CPopulatedStruct,
) -> Option<ConstructId> {
    if inn.label == ident {
        Some(inn.value)
    } else if let Some(rest) = as_struct(&**env.get_construct_definition(inn.rest)) {
        let rest = rest.clone();
        lookup_ident_in(env, ident, &rest)
    } else {
        None
    }
}

impl Scope for SFieldAndRest {
    fn dyn_clone(&self) -> Box<dyn Scope> {
        Box::new(self.clone())
    }

    fn local_lookup_ident<'x>(
        &self,
        env: &mut Environment<'x>,
        ident: &str,
    ) -> Option<ConstructId> {
        if let Some(structt) = as_struct(&**env.get_construct_definition(self.0)) {
            let structt = structt.clone();
            lookup_ident_in(env, ident, &structt)
        } else {
            unreachable!()
        }
    }

    fn parent(&self) -> Option<ConstructId> {
        Some(self.0)
    }
}
