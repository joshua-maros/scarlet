#![cfg(test)]

use crate::{
    item::{definitions::variable::DVariable, test_util::*, util::unchecked_substitution},
    util::PtrExtension,
};

#[test]
fn basic_invariant() {
    let code = r"
    a IS UNIQUE
    y IS VAR(SELF = a)
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

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    statement IS fx = a

    t IS VAR(statement(x IS SELF))

    justify_this IS
    statement(x IS t)
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
    a IS VAR()

    x IS VAR()
    fx IS VAR(DEP x)

    statement IS fx(a)

    t IS VAR(statement)

    other IS VAR(SELF)(statement)
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "statement");
        let context = get_member(&root, "other");
        root.check_all();
        env.justify_all(&root).unwrap();
        env.justify(&root, &context, &justify_this, 2).unwrap();
    });
}

#[test]
fn moderate_invariant() {
    let code = r"
    a IS VAR()
    b IS VAR()

    x IS VAR()
    fx IS VAR(DEP x)

    statement IS 
    x(fx(b) = fx(a))

    invariant IS statement(t u)

    t IS VAR()
    u IS VAR()

    VAR(invariant)

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
    y IS VAR(SELF = a)
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
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

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
    x IS VAR()

    statement IS 
    x AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    a IS UNIQUE
    eq_ext_rev_t(a)
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
    x IS VAR()
    fx IS VAR(DEP x)

    statement IS 
    fx AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    identity IS VAR()

    a IS VAR()
    eq_ext_rev_t(a identity)
    justify_this IS a
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn indirect_function_invariant() {
    let code = r"
    x IS VAR()
    fx IS VAR(DEP x)
    y IS VAR()

    statement IS 
    fx(y) AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    identity IS VAR()

    a IS VAR()
    eq_ext_rev_t(identity a)
    justify_this IS a
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn equality_function_invariant() {
    let code = r"
    x IS VAR()
    fx IS VAR(DEP x)
    y IS VAR()

    statement IS 
    x(fx(x) = y) 
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    identity IS VAR()

    a IS VAR()
    b IS VAR()
    eq_ext_rev_t(a identity b)
    justify_this IS a = b
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn full_equality_function_invariant() {
    let code = r"
    x IS VAR()
    fx IS VAR(DEP x)
    y IS VAR()

    statement IS 
    x(fx(x) = fx(y)) 
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    identity IS VAR()

    a IS VAR()
    b IS VAR()
    eq_ext_rev_t(a identity b)
    justify_this IS a = b
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
    x IS VAR()
    y IS VAR()

    statement IS 
    x(x = y) 
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    a IS UNIQUE
    b IS UNIQUE
    eq_ext_rev_t(a b)
    justify_this IS statement(a b)
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn theorem_verbatim() {
    let code = r"
    a IS VAR()
    b IS VAR(a = SELF)

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    statement IS fx(b)
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    justify_this IS statement
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
    });
}

#[test]
fn simplified_real_theorem_invariant() {
    let code = r"
    y IS VAR()
    z IS VAR()

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    statement IS 
    x(fx(z) = fx(y))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    eq_ext_rev_t(fx IS x)

    justify_this IS z = y
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
    a IS VAR()
    b IS VAR(a = SELF)

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    statement IS 
    x(fx(b) = fx(a))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    c IS VAR()
    d IS VAR(c = SELF)
    identity IS VAR()

    eq_ext_rev_t(c d identity)

    justify_this IS d = c
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
        x IS VAR()
        fx IS VAR(DEP x)
    }

    eq_ext_rev_t(asdf.x)

    t_just IS VAR(SELF)
    t_just(b = a)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    asdf.x(asdf.fx(b) = asdf.fx(a))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    a IS VAR()
    b IS VAR(a = SELF)
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn real_theorem_rewritten_invariant() {
    let code = r"
    a IS VAR()
    b IS VAR(a = SELF)

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    statement IS 
    x(fx(b) = fx(a))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    c IS VAR()
    d IS VAR(c = SELF)

    eq_ext_rev_t(c d x)

    justify_this IS d = c

    t_just IS VAR(SELF)
    t_just(d = c)
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn subbed_statement() {
    let code = r"
    x IS VAR()
    y IS VAR()

    u IS VAR()
    v IS VAR(u = SELF)
    identity IS VAR()

    justify_this IS x(x = y)(u v)
    ";
    with_env_from_code(code, |mut env, root| {
        let justify_this = get_member(&root, "justify_this");
        env.justify(&root, &justify_this, &justify_this, 5).unwrap();
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn simpler_justified_substitution() {
    let code = r"
    a IS VAR(SELF)

    c IS VAR(SELF)

    a(c)
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn justify_unchecked_sub() {
    let code = r"
    a IS VAR(SELF)
    b IS VAR(SELF)
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
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn justified_substitution() {
    let code = r"
    a IS VAR()
    b IS VAR(a = SELF)

    c IS VAR()
    d IS VAR(c = SELF)

    {a b}(c d)
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn scope_separated_substitution() {
    let code = r"
    amod IS { a IS VAR(SELF) }
    bmod IS { 
        a(b)
        b IS VAR(SELF) 
    }
    a IS amod.a
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
#[should_panic]
fn unjustified_substitution() {
    let code = r"
    a IS VAR()
    b IS VAR(a = SELF)

    c IS VAR()
    d IS VAR()

    {a b}(c d)
    ";
    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn t_just_after_theorem() {
    let code = r"
    eq_ext_rev_t IS 
    {
        AXIOM(eq_ext_rev_t)

        a IS VAR()
        b IS VAR(a = SELF)

        statement IS 
        x(fx(b) = fx(a))
        AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)
    }
    .VALUE

    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    t_just IS VAR(SELF)

    justify_this IS b = a

    eq_ext_rev_t(a b x)

    a IS VAR()
    b IS VAR(a = SELF)
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
    x IS VAR() AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    x(fx(b) = fx(a))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t(fx a b)

    a IS VAR()
    b IS VAR(SELF = a)
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn fx_asserting_self_sub_a() {
    let code = r"
    a IS VAR()
    x IS VAR()
    fx IS VAR(SELF(a) DEP x)

    VAR(SELF)(fx(a))
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn eq_ext_simplified() {
    let code = r"
    y IS VAR()
    z IS VAR(y = SELF)

    x IS VAR(ORD 32) AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x ORD 32)

    x(fx(z) = fx(y))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    u IS VAR()
    v IS VAR(SELF = u)

    eq_ext_rev_t(fx v u)

    VAR(SELF)(fx(u) = fx(v))
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn eq_ext_full() {
    let code = r"
    y IS VAR()
    z IS VAR(y = SELF)

    x IS VAR(ORD 32) AS_LANGUAGE_ITEM(x)
    fx IS VAR(DEP x ORD 32)

    x(fx(z) = fx(y))
    AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

    eq_ext_rev_t IS AXIOM(eq_ext_rev_t)

    u IS VAR()
    v IS VAR(u = SELF)
    identity IS VAR()

    eq_symm_t IS eq_ext_rev_t(identity)

    eq_symm_t(u v)

    eq_ext_rev_t(fx v u)

    VAR(SELF)(fx(u) = fx(v))
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}

#[test]
fn eq_ext_full_separated() {
    let code = r"
    std IS {
        x IS VAR(ORD 32) AS_LANGUAGE_ITEM(x)
        fx IS VAR(DEP x ORD 32)
    }

    x IS std.x
    fx IS std.fx

    eq_ext_rev_t IS 
    {
        AXIOM(eq_ext_rev_t)

        x(fx(z) = fx(y))
        AS_LANGUAGE_ITEM(eq_ext_rev_t_statement)

        y IS VAR()
        z IS VAR(y = SELF)
    }
    .VALUE

    eq_symm_t IS 
    {
        eq_ext_rev_t(identity u v)

        u IS VAR()
        v IS VAR(u = SELF)
        identity IS VAR()
    }
    .VALUE


    {
        VAR(SELF)(fx(s) = fx(t))

        eq_ext_rev_t(fx t s)

        eq_symm_t(s t)

        s IS VAR()
        t IS VAR(s = SELF)
    }
    ";

    with_env_from_code(code, |mut env, root| {
        root.check_all();
        env.justify_all(&root).unwrap();
    });
}
