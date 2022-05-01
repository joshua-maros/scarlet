#![cfg(test)]

use crate::{
    item::{
        definitions::{structt::DPopulatedStruct, variable::DVariable},
        test_util::*,
        util::unchecked_substitution,
        ItemPtr,
    },
    util::PtrExtension,
};

fn get_member(root: &ItemPtr, name: &str) -> ItemPtr {
    root.downcast_definition::<DPopulatedStruct>()
        .unwrap()
        .get_value()
        .lookup_ident(name)
        .unwrap()
        .unwrap()
}

#[test]
fn basic_invariant() {
    let code = r"
    a IS UNIQUE
    y IS VAR[SELF = a]
    y_statement IS y = a
    ";
    with_env_from_code(code, |mut env, root| {
        let y_statement = get_member(&root, "y_statement");
        env.justify(&root, &y_statement, &y_statement, 2).unwrap();
        root.check_all();
    });
}

#[test]
fn sub_invariant() {
    let code = r"
    a IS UNIQUE

    x IS VAR[].AS_LANGUAGE_ITEM[x]
    fx IS VAR[DEP x]

    statement IS 
    (fx = a)
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t IS VAR[statement[x IS SELF]]

    justify_this IS
    statement[x IS t]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 10)
            .unwrap();
        root.check_all();
    });
}

#[test]
fn sub_fx_invariant() {
    let code = r"
    a IS VAR[]

    x IS VAR[]
    fx IS VAR[DEP x]

    statement IS fx[a]

    t IS VAR[statement]

    justify_this IS statement
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 10)
            .unwrap();
        root.check_all();
    });
}

#[test]
fn moderate_invariant() {
    let code = r"
    a IS VAR[]
    b IS VAR[]

    x IS VAR[]
    fx IS VAR[DEP x]

    statement IS 
    (fx[b] = fx[a])

    invariant IS statement[t u]

    t IS VAR[]
    u IS VAR[invariant]

    justify_this IS invariant
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 10)
            .unwrap();
        root.check_all();
    });
}

#[test]
fn nonexistant_invariant() {
    let code = r"
    a IS UNIQUE
    b IS UNIQUE
    y IS VAR[SELF = a]
    z_statement IS y = b
    ";
    with_env_from_code(code, |mut env, root| {
        let z_statement = get_member(&root, "z_statement");
        env.justify(&root, &z_statement, &z_statement, 1)
            .unwrap_err();
    });
}

#[test]
fn basic_theorem_invariant() {
    let code = r"
    statement IS 
    UNIQUE
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    justify_this IS statement
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn subbed_theorem_invariant() {
    let code = r"
    x IS VAR[]

    statement IS 
    x.AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    a IS UNIQUE
    t_eq_ext_rev[a]
    justify_this IS a
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn function_invariant() {
    let code = r"
    x IS VAR[]
    fx IS VAR[DEP x]

    statement IS 
    fx.AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    identity IS VAR[]

    a IS VAR[]
    t_eq_ext_rev[identity a]
    justify_this IS statement[identity a]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn equality_theorem_invariant() {
    let code = r"
    x IS VAR[]
    y IS VAR[]

    statement IS 
    (x = y).AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    a IS UNIQUE
    b IS UNIQUE
    t_eq_ext_rev[a b]
    justify_this IS statement[a b]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn real_theorem_invariant() {
    let code = r"
    a IS VAR[]
    b IS VAR[a = SELF]

    x IS VAR[].AS_LANGUAGE_ITEM[x]
    fx IS VAR[DEP x]

    statement IS 
    (fx[b] = fx[a])
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    c IS VAR[]
    d IS VAR[c = SELF]
    identity IS VAR[]

    t_eq_ext_rev[c d identity]

    justify_this IS 
    statement[c d identity]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn real_theorem_separated_invariant() {
    let code = r"
    asdf IS {
        x IS VAR[]
        fx IS VAR[DEP x]
    }

    t_eq_ext_rev[asdf.x]

    t_just IS VAR[SELF]
    t_just[b = a]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    (asdf.fx[b] = asdf.fx[a])
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    a IS VAR[]
    b IS VAR[a = SELF]
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn real_theorem_rewritten_invariant() {
    let code = r"
    a IS VAR[]
    b IS VAR[a = SELF]

    x IS VAR[].AS_LANGUAGE_ITEM[x]
    fx IS VAR[DEP x]

    statement IS 
    (fx[b] = fx[a])
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    c IS VAR[]
    d IS VAR[c = SELF]

    t_eq_ext_rev[c d x]

    justify_this IS d = c

    t_just IS VAR[SELF]
    t_just[d = c]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn subbed_statement() {
    let code = r"
    a IS VAR[]
    b IS VAR[]

    c IS VAR[]
    d IS VAR[c = SELF]
    identity IS VAR[]

    justify_this IS (a = b)[c d]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn simpler_justified_substitution() {
    let code = r"
    a IS VAR[SELF]

    c IS VAR[SELF]

    a[c]
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn justify_unchecked_sub() {
    let code = r"
    a IS VAR[SELF]
    b IS VAR[SELF]
    ";
    with_env_from_code(code, |mut env, root| {
        let a = get_member(&root, "a");
        let a_var = a
            .dereference()
            .downcast_definition::<DVariable>()
            .unwrap()
            .get_variable()
            .ptr_clone();
        let b = get_member(&root, "b");
        let a_sub_b = unchecked_substitution(a, &subs(vec![(a_var, b.ptr_clone())]));
        env.justify(&root, &b, &a_sub_b, 10).unwrap();
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn justified_substitution() {
    let code = r"
    a IS VAR[]
    b IS VAR[a = SELF]

    c IS VAR[]
    d IS VAR[c = SELF]

    {a b}[c d]
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn scope_separated_substitution() {
    let code = r"
    amod IS { a IS VAR[SELF] }
    bmod IS { 
        a[b]
        b IS VAR[SELF] 
    }
    a IS amod.a
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
#[should_panic]
fn unjustified_substitution() {
    let code = r"
    a IS VAR[]
    b IS VAR[a = SELF]

    c IS VAR[]
    d IS VAR[]

    {a b}[c d]
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}

#[test]
fn t_just_after_theorem() {
    let code = r"
    t_eq_ext_rev IS 
    {
        AXIOM[t_eq_ext_rev]

        a IS VAR[]
        b IS VAR[a = SELF]

        statement IS 
        (fx[b] = fx[a])
        .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]
    }
    .VALUE

    x IS VAR[].AS_LANGUAGE_ITEM[x]
    fx IS VAR[DEP x]

    t_just IS VAR[SELF]

    justify_this IS b = a

    t_eq_ext_rev[a b x]

    a IS VAR[]
    b IS VAR[a = SELF]
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn mysterious_hang() {
    let code = r"
    x IS VAR[].AS_LANGUAGE_ITEM[x]
    fx IS VAR[DEP x]

    t_eq_ext_rev IS AXIOM[t_eq_ext_rev]

    (fx[b] = fx[a])
    .AS_LANGUAGE_ITEM[t_eq_ext_rev_statement]

    t_eq_ext_rev[fx b a]

    a IS VAR[]
    b IS VAR[SELF = a]
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root);
    });
}
