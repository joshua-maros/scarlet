#![cfg(test)]

use std::assert_matches::assert_matches;

use crate::item::{
    definitions::substitution::{DSubstitution, Substitutions},
    equality::Equal,
    test_util::*,
    util::*,
};

#[test]
fn something_equals_itself() {
    let mut env = env();
    let thing = unique();
    assert_eq!(thing.get_equality(&thing, 0), Ok(Equal::yes()));
}

#[test]
fn something_equals_variable() {
    let mut env = env();
    let thing = unique();
    let (var_con, var_id) = variable_full();
    let expected = subs(vec![(var_id, thing)]);
    let left = Equal::Yes(expected.clone(), Default::default());
    assert_eq!(var_con.get_equality(&thing, 1), Ok(left));
    assert_eq!(var_con.get_equality(&thing, 0), Ok(Equal::NeedsHigherLimit));
    assert_eq!(thing.get_equality(&var_con, 0), Ok(Equal::NeedsHigherLimit));
}

#[test]
fn variable_equals_variable() {
    let mut env = env();
    let x = variable_full();
    let y = variable_full();
    let expected = subs(vec![(x.1, y.0)]);
    let left = Equal::Yes(expected.clone(), Default::default());
    assert_eq!(x.0.get_equality(&y.0, 1), Ok(left));
    assert_eq!(x.0.get_equality(&y.0, 0), Ok(Equal::NeedsHigherLimit));
    assert_eq!(y.0.get_equality(&x.0, 0), Ok(Equal::NeedsHigherLimit));
}

#[test]
fn var_sub_something_equals_something() {
    let mut env = env();
    let thing = unique();
    let another = unique();
    let (var_con, var_id) = variable_full();
    let var_sub_thing = unchecked_substitution(var_con, &subs(vec![(var_id, thing)]));
    assert_eq!(var_sub_thing.get_equality(&thing, 2), Ok(Equal::yes()));
    assert_eq!(thing.get_equality(&var_sub_thing, 2), Ok(Equal::yes()));
    assert_eq!(var_sub_thing.get_equality(&another, 2), Ok(Equal::No));
    assert_eq!(another.get_equality(&var_sub_thing, 2), Ok(Equal::No));
}

#[test]
fn decision_equals_identical_decision() {
    let mut env = env();
    let a = variable();
    let b = variable();
    let c = variable();
    let d = variable();
    let dec1 = decision(a, b, c, d);
    let dec2 = decision(a, b, c, d);
    assert_eq!(dec1.get_equality(&dec2, 2), Ok(Equal::yes()));
    assert_eq!(dec2.get_equality(&dec1, 2), Ok(Equal::yes()));
}

#[test]
fn aabc_is_ddef() {
    let mut env = env();
    let a = variable_full();
    let b = variable_full();
    let c = variable_full();
    let d = variable_full();
    let e = variable_full();
    let f = variable_full();
    let dec1 = decision(a.0, a.0, b.0, c.0);
    let dec2 = decision(d.0, d.0, e.0, f.0);
    let left_subs = subs(vec![(a.1, d.0), (b.1, e.0), (c.1, f.0)]);
    assert_eq!(
        dec1.get_equality(&dec2, 3),
        Ok(Equal::Yes(left_subs, Default::default()))
    );
    let right_subs = subs(vec![(d.1, a.0), (e.1, b.0), (f.1, c.0)]);
    assert_eq!(
        dec2.get_equality(&dec1, 3),
        Ok(Equal::Yes(right_subs, Default::default()))
    );
}

#[test]
fn xxbc_is_aabc() {
    let mut env = env();
    let a = unique();
    let b = unique();
    let c = unique();
    let x = variable_full();
    let dec1 = decision(x.0, x.0, b, c);
    let dec2 = decision(a, a, b, c);
    let left_subs = subs(vec![(x.1, a)]);
    assert_eq!(
        dec1.get_equality(&dec2, 3),
        Ok(Equal::Yes(left_subs, Default::default()))
    );
}

#[test]
fn aabc_eq_b_is_ddef_eq_e() {
    let mut env = env();
    let truee = unique();
    let falsee = unique();
    let a = variable_full();
    let b = variable_full();
    let c = variable_full();
    let d = variable_full();
    let e = variable_full();
    let f = variable_full();
    let dec1 = decision(a.0, a.0, b.0, c.0);
    let dec1 = decision(dec1, b.0, truee, falsee);
    let dec2 = decision(d.0, d.0, e.0, f.0);
    let dec2 = decision(dec2, e.0, truee, falsee);
    let left_subs = subs(vec![(a.1, d.0), (b.1, e.0), (c.1, f.0)]);
    assert_eq!(
        dec1.get_equality(&dec2, 3),
        Ok(Equal::Yes(left_subs, Default::default()))
    );
    let right_subs = subs(vec![(d.1, a.0), (e.1, b.0), (f.1, c.0)]);
    assert_eq!(
        dec2.get_equality(&dec1, 3),
        Ok(Equal::Yes(right_subs, Default::default()))
    );
}

#[test]
fn decision_equals_decision_with_subs() {
    let mut env = env();
    let a = variable_full();
    let b = variable_full();
    let c = variable_full();
    let d = variable_full();
    let w = unique();
    let x = unique();
    let y = unique();
    let z = unique();
    let dec1 = decision(a.0, b.0, c.0, d.0);
    let dec2 = decision(w, x, y, z);
    let subs = subs(vec![(a.1, w), (b.1, x), (c.1, y), (d.1, z)]);
    assert_eq!(
        dec1.get_equality(&dec2, 2),
        Ok(Equal::Yes(subs.clone(), Default::default()))
    );
}

#[test]
fn fx_is_gy() {
    let mut env = env();
    let x = variable_full();
    let y = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let g = variable_full_with_deps(vec![y.0]);
    assert_matches!(f.0.get_equality(&g.0, 2), Ok(Equal::Yes(..)));
    assert_matches!(g.0.get_equality(&f.0, 2), Ok(Equal::Yes(..)));
    assert_matches!(g.0.get_equality(&f.0, 1), Ok(Equal::NeedsHigherLimit));
    if let Ok(Equal::Yes(lsubs, _)) = f.0.get_equality(&g.0, 2) {
        assert_eq!(lsubs.len(), 2);
        let mut entries = lsubs.iter();
        let next = entries.next().unwrap();
        assert_eq!(next, &(x.1, y.0));
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected second substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_a_is_gy_sub_a() {
    let mut env = env();
    let a = unique();
    let x = variable_full();
    let y = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let g = variable_full_with_deps(vec![y.0]);
    let f_sub_a = unchecked_substitution(f.0, &subs(vec![(x.1, a)]));
    let g_sub_a = unchecked_substitution(g.0, &subs(vec![(y.1, a)]));
    assert_matches!(f_sub_a.get_equality(&g_sub_a, 2), Ok(Equal::Yes(..)));
    assert_matches!(g_sub_a.get_equality(&f_sub_a, 2), Ok(Equal::Yes(..)));
    assert_matches!(
        g_sub_a.get_equality(&f_sub_a, 1),
        Ok(Equal::NeedsHigherLimit)
    );
    if let Ok(Equal::Yes(lsubs, _)) = f_sub_a.get_equality(&g_sub_a, 2) {
        assert_eq!(lsubs.len(), 1);
        let mut entries = lsubs.iter();
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_gy_is_gy_sub_x() {
    let mut env = env();
    let x = variable_full(); // 13/0
    let y = variable_full(); // 14/1
    let f = variable_full_with_deps(vec![x.0]); // 15/2
    let g = variable_full_with_deps(vec![y.0]); // 16/3

    let gx = unchecked_substitution(g.0, &subs(vec![(y.1, x.0)])); // 17
    let fx_sub_gy = unchecked_substitution(f.0, &subs(vec![(f.1, gx)])); // 18

    assert_eq!(fx_sub_gy.get_equality(&gx, 6), Ok(Equal::yes()));
    assert_eq!(gx.get_equality(&fx_sub_gy, 6), Ok(Equal::yes()));
    assert_eq!(gx.get_equality(&fx_sub_gy, 1), Ok(Equal::NeedsHigherLimit));
}

#[test]
fn fx_sub_nothing_is_gy_sub_nothing() {
    let mut env = env();
    let x = variable_full();
    let y = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let f_sub = unchecked_substitution(f.0, &Default::default());
    let g = variable_full_with_deps(vec![y.0]);
    let g_sub = unchecked_substitution(g.0, &Default::default());
    assert_matches!(f_sub.get_equality(&g_sub, 3), Ok(Equal::Yes(..)));
    assert_matches!(g_sub.get_equality(&f_sub, 3), Ok(Equal::Yes(..)));
    assert_matches!(g_sub.get_equality(&f_sub, 0), Ok(Equal::NeedsHigherLimit));
    if let Ok(Equal::Yes(lsubs, _)) = f_sub.get_equality(&g_sub, 3) {
        assert_eq!(lsubs.len(), 2);
        let mut entries = lsubs.iter();
        assert_eq!(entries.next(), Some(&(x.1, y.0)));
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected second substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_z_is_gy_sub_nothing() {
    let mut env = env();
    let x = variable_full();
    let y = variable_full();
    let z = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let f_sub = unchecked_substitution(f.0, &subs(vec![(x.1, z.0)]));
    let g = variable_full_with_deps(vec![y.0]);
    let g_sub = unchecked_substitution(g.0, &Default::default());
    assert_matches!(f_sub.get_equality(&g_sub, 3), Ok(Equal::Yes(..)));
    assert_matches!(g_sub.get_equality(&f_sub, 3), Ok(Equal::Yes(..)));
    assert_matches!(g_sub.get_equality(&f_sub, 0), Ok(Equal::NeedsHigherLimit));
    if let Ok(Equal::Yes(lsubs, _)) = f_sub.get_equality(&g_sub, 3) {
        assert_eq!(lsubs.len(), 2);
        let mut entries = lsubs.iter();
        assert_eq!(entries.next(), Some(&(z.1, y.0)));
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected second substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_decision_is_gy_sub_decision() {
    let mut env = env();
    let a = unique();
    let b = unique();
    let c = unique();
    let d = unique();

    let dec = decision(a, b, c, d);
    let x = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let f_dec = unchecked_substitution(f.0, &subs(vec![(x.1, dec)]));

    let dec = decision(a, b, c, d);
    let y = variable_full();
    let g = variable_full_with_deps(vec![y.0]);
    let g_dec = unchecked_substitution(g.0, &subs(vec![(y.1, dec)]));

    assert_matches!(f_dec.get_equality(&g_dec, 3), Ok(Equal::Yes(..)));
    assert_matches!(g_dec.get_equality(&f_dec, 3), Ok(Equal::Yes(..)));
    if let Ok(Equal::Yes(lsubs, _)) = f_dec.get_equality(&g_dec, 3) {
        assert_eq!(lsubs.len(), 1);
        let mut entries = lsubs.iter();
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected second substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn dex_sub_decision_is_gy_sub_decision() {
    let mut env = env();
    let a = unique();
    let b = unique();
    let c = unique();
    let d = unique();

    let s = variable_full();
    let t = variable_full();
    let u = variable_full();
    let v = variable_full();

    let dec_for_dex = decision(a, b, c, d);
    let x = variable_full();
    let dex = decision(x.0, d, c, b);
    let dex_dec = unchecked_substitution(dex, &subs(vec![(x.1, dec_for_dex)]));

    let dec_for_g = decision(s.0, t.0, u.0, v.0);
    let y = variable_full();
    let g = variable_full_with_deps(vec![y.0]);
    let g_dec = unchecked_substitution(g.0, &subs(vec![(y.1, dec_for_g)]));

    assert_matches!(g_dec.get_equality(&dex_dec, 3), Ok(Equal::Yes(..)));
    if let Ok(Equal::Yes(lsubs, _)) = g_dec.get_equality(&dex_dec, 3) {
        assert_eq!(lsubs.len(), 5);
        let mut entries = lsubs.iter();
        assert_eq!(entries.next().unwrap(), &(s.1, a));
        assert_eq!(entries.next().unwrap(), &(t.1, b));
        assert_eq!(entries.next().unwrap(), &(u.1, c));
        assert_eq!(entries.next().unwrap(), &(v.1, d));
        let first = entries.next().unwrap();
        assert_eq!(first.0, g.1);
        if let Some(sub) = first.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), dex);
            assert_eq!(sub.substitutions(), &subs(vec![(x.1, y.0)]))
        } else {
            panic!("Expected last substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_decision_with_var_is_gy_sub_decision() {
    let mut env = env();

    let aa = variable_full();

    let a = unique();
    let b = unique();
    let c = unique();
    let d = unique();

    let dec = decision(aa.0, b, c, d);
    let x = variable_full();
    let f = variable_full_with_deps(vec![x.0]);
    let f_dec = unchecked_substitution(f.0, &subs(vec![(x.1, dec)]));

    let dec = decision(a, b, c, d);
    let y = variable_full();
    let g = variable_full_with_deps(vec![y.0]);
    let g_dec = unchecked_substitution(g.0, &subs(vec![(y.1, dec)]));

    assert_matches!(f_dec.get_equality(&g_dec, 4), Ok(Equal::Yes(..)));
    if let Ok(Equal::Yes(lsubs, _)) = f_dec.get_equality(&g_dec, 4) {
        assert_eq!(lsubs.len(), 2);
        let mut entries = lsubs.iter();
        assert_eq!(Some(&(aa.1, a)), entries.next());
        let last = entries.next().unwrap();
        assert_eq!(last.0, f.1);
        if let Some(sub) = last.1.downcast_definition::<DSubstitution>() {
            assert_eq!(sub.base(), g.0);
            assert_eq!(sub.substitutions(), &subs(vec![(y.1, x.0)]))
        } else {
            panic!("Expected second substitution to be itself another substitution");
        }
    } else {
        unreachable!()
    }
}

#[test]
fn fx_sub_gy_sub_a_is_gy_sub_a() {
    let mut env = env();

    // 13
    let a = unique();
    // 14
    let x = variable_full();
    // 15
    let y = variable_full();
    // 16
    let g = variable_full_with_deps(vec![y.0]);
    // 17
    let gx = unchecked_substitution(g.0, &subs(vec![(y.1, x.0)]));

    // 18
    let f = variable_full_with_deps(vec![x.0]);
    // 19
    let f_sub_gx = unchecked_substitution(f.0, &subs(vec![(f.1, gx)]));
    // 20
    let f_sub_gx_sub_a = unchecked_substitution(f_sub_gx, &subs(vec![(x.1, a)]));

    // 21
    let gy_sub_a = unchecked_substitution(g.0, &subs(vec![(y.1, a)]));

    assert_eq!(f_sub_gx_sub_a.get_equality(&gy_sub_a, 5), Ok(Equal::yes()));
    assert_eq!(gy_sub_a.get_equality(&f_sub_gx_sub_a, 5), Ok(Equal::yes()));
}

#[test]
fn fx_sub_a_sub_gy_is_gy_sub_a() {
    let mut env = env();

    let a = unique();

    let x = variable_full();
    let y = variable_full();
    let g = variable_full_with_deps(vec![y.0]);
    let gx = unchecked_substitution(g.0, &subs(vec![(y.1, x.0)]));

    let f = variable_full_with_deps(vec![x.0]);
    let f_sub_a = unchecked_substitution(f.0, &subs(vec![(x.1, a)]));
    let f_sub_a_sub_gy = unchecked_substitution(f_sub_a, &subs(vec![(f.1, gx)]));

    let gy_sub_a = unchecked_substitution(g.0, &subs(vec![(y.1, a)]));

    assert_eq!(f_sub_a_sub_gy.get_equality(&gy_sub_a, 4), Ok(Equal::yes()));
    assert_eq!(gy_sub_a.get_equality(&f_sub_a_sub_gy, 4), Ok(Equal::yes()));
}

#[test]
fn x_eq_y_sub_true_true_is_a_equal_a() {
    let mut env = env();
    let truee = unique();
    let falsee = unique();

    let a = variable_full();
    let x = variable_full();
    let y = variable_full();

    let x_eq_y = decision(x.0, y.0, truee, falsee);
    let true_eq_true = unchecked_substitution(x_eq_y, &subs(vec![(x.1, truee), (y.1, truee)]));
    let a_eq_a = decision(a.0, a.0, truee, falsee);

    assert_matches!(a_eq_a.get_equality(&true_eq_true, 3), Ok(Equal::Yes(..)));
    if let Ok(Equal::Yes(lsubs, _)) = a_eq_a.get_equality(&true_eq_true, 3) {
        assert_eq!(lsubs.len(), 1);
        let mut entries = lsubs.iter();
        let last = entries.next().unwrap();
        assert_eq!(last, &(a.1, truee));
    } else {
        unreachable!()
    }
}

#[test]
fn is_bool_sub_y_is_y_is_bool() {
    let mut env = env();

    let x = variable_full();
    let y = variable_full();
    let t = unique();
    let f = unique();

    let x_is_false = decision(x.0, f, t, f);
    let x_is_bool = decision(x.0, t, t, x_is_false);

    let y_is_false = decision(y.0, f, t, f);
    let y_is_bool = decision(y.0, t, t, y_is_false);

    let x_sub_y_is_bool = unchecked_substitution(x_is_bool, &subs(vec![(x.1, y.0)]));

    assert_eq!(
        y_is_bool.get_equality(&x_sub_y_is_bool, 4),
        Ok(Equal::yes())
    );
}

/// f[z] <=> DECISION[x y a b]
#[test]
fn multi_variable_dex_is_single_variable_dex() {
    let mut env = env();

    let a = unique();
    let b = unique();
    let x = variable_full();
    x.0.set_name("x".to_owned());
    let y = variable_full();
    y.0.set_name("y".to_owned());
    let z = variable_full();
    z.0.set_name("z".to_owned());

    let fz = variable_full_with_deps(vec![z.0]);
    fz.0.set_name("fz".to_owned());

    let multi_variable_dex = decision(x.0, y.0, a, b);

    if let Equal::Yes(subs, _) = fz.0.get_equality(&multi_variable_dex, 15).unwrap() {
        assert_eq!(subs.len(), 2);
        let sub = *subs.get(&z.1).unwrap();
        assert_eq!(sub, x.0);
        let sub = *subs.get(&fz.1).unwrap();
        if let Some(def) = sub.downcast_definition::<DSubstitution>() {
            let mut expected = Substitutions::new();
            expected.insert_no_replace(x.1, z.0);
            assert_eq!(def.substitutions(), &expected);
            assert_eq!(def.base(), multi_variable_dex);
        } else {
            panic!("Substituted value is not itself a substitution!");
        }
    } else {
        panic!("Not equal!");
    }
}

/// f[z] <=> DECISION[x y a b][a]
#[test]
fn multi_variable_dex_sub_something_is_single_variable_dex() {
    let mut env = env();

    let a = unique();
    let b = unique();
    let x = variable_full();
    x.0.set_name("x".to_owned());
    let y = variable_full();
    y.0.set_name("y".to_owned());
    let z = variable_full();
    z.0.set_name("z".to_owned());

    let fz = variable_full_with_deps(vec![z.0]);
    fz.0.set_name("fz".to_owned());

    let multi_variable_dex = decision(x.0, y.0, a, b);
    let subbed_multi_variable_dex =
        unchecked_substitution(multi_variable_dex, &subs(vec![(x.1, a)]));

    if let Equal::Yes(subs, _) = fz.0.get_equality(&subbed_multi_variable_dex, 15).unwrap() {
        assert_eq!(subs.len(), 2);
        let sub = *subs.get(&z.1).unwrap();
        assert_eq!(sub, y.0);
        let sub = *subs.get(&fz.1).unwrap();
        if let Some(def) = sub.downcast_definition::<DSubstitution>() {
            let def = def.clone();
            let mut expected = Substitutions::new();
            expected.insert_no_replace(y.1, z.0);
            assert_eq!(def.substitutions(), &expected);
            assert_eq!(
                def.base().get_equality(&subbed_multi_variable_dex, 4),
                Ok(Equal::yes())
            );
        } else {
            panic!("Substituted value is not itself a substitution!");
        }
    } else {
        panic!("Not equal!");
    }
}

/// f[z] <=> DECISION[x y a b][x2 y2]
#[test]
fn multi_variable_dex_sub_two_vars_is_single_variable_dex() {
    let mut env = env();

    let a = unique();
    let b = unique();
    let x = variable_full();
    x.0.set_name("x".to_owned());
    let x2 = variable_full();
    x2.0.set_name("x2".to_owned());
    let y = variable_full();
    y.0.set_name("y".to_owned());
    let y2 = variable_full();
    y2.0.set_name("y2".to_owned());
    let z = variable_full();
    z.0.set_name("z".to_owned());

    let fz = variable_full_with_deps(vec![z.0]);
    fz.0.set_name("fz".to_owned());

    let multi_variable_dex = decision(x.0, y.0, a, b);
    let subbed_multi_variable_dex =
        unchecked_substitution(multi_variable_dex, &subs(vec![(x.1, x2.0), (y.1, y2.0)]));

    if let Equal::Yes(subs, _) = fz.0.get_equality(&subbed_multi_variable_dex, 15).unwrap() {
        assert_eq!(subs.len(), 2);
        let sub = *subs.get(&z.1).unwrap();
        assert_eq!(sub, x2.0);
        let sub = *subs.get(&fz.1).unwrap();
        if let Some(def) = sub.downcast_definition::<DSubstitution>() {
            let def = def.clone();
            let mut expected = Substitutions::new();
            expected.insert_no_replace(x2.1, z.0);
            assert_eq!(def.substitutions(), &expected);
            assert_eq!(
                def.base().get_equality(&subbed_multi_variable_dex, 4),
                Ok(Equal::yes())
            );
        } else {
            panic!("Substituted value is not itself a substitution!");
        }
    } else {
        panic!("Not equal!");
    }
}

/// f[z] <=> DECISION[x y a b][a b]
#[test]
fn multi_variable_dex_sub_two_uniques_is_single_variable_dex() {
    let mut env = env();

    let a = unique();
    let b = unique();
    let x = variable_full();
    x.0.set_name("x".to_owned());
    let y = variable_full();
    y.0.set_name("y".to_owned());
    let z = variable_full();
    z.0.set_name("z".to_owned());

    let fz = variable_full_with_deps(vec![z.0]);
    fz.0.set_name("fz".to_owned());

    let multi_variable_dex = decision(x.0, y.0, a, b);
    let subbed_multi_variable_dex =
        unchecked_substitution(multi_variable_dex, &subs(vec![(x.1, a), (y.1, b)]));

    if let Equal::Yes(subs, _) = fz.0.get_equality(&subbed_multi_variable_dex, 15).unwrap() {
        assert_eq!(subs.len(), 2);
        let sub = *subs.get(&z.1).unwrap();
        assert_eq!(sub, a);
        let sub = *subs.get(&fz.1).unwrap();
        if let Some(def) = sub.downcast_definition::<DSubstitution>() {
            let def = def.clone();
            let mut expected = Substitutions::new();
            expected.insert_no_replace(x.1, z.0);
            assert_eq!(def.substitutions(), &expected);
            if let Some(def) = def.base().downcast_definition::<DSubstitution>() {
                let def = def.clone();
                assert_eq!(def.substitutions().len(), 1);
                assert_eq!(
                    (*def.substitutions().get(&y.1).unwrap()).get_equality(&b, 4),
                    Ok(Equal::yes())
                );
                assert_eq!(
                    def.base().get_equality(&multi_variable_dex, 4),
                    Ok(Equal::yes())
                );
            } else {
                panic!("Expected another substitution!");
            }
        } else {
            panic!("Substituted value is not itself a substitution!");
        }
    } else {
        panic!("Not equal!");
    }
}

/// fx[fx IS x = y   x IS a   y IS b] <=/=> a = b
#[test]
fn sneaky_substitution() {
    let mut env = env();

    // I13
    let a = unique();
    // I14
    let b = unique();
    // I15
    let t = unique();
    // I16
    let f = unique();
    // I17 V0
    let x = variable_full();
    x.0.set_name("x".to_owned());
    // I18 V1
    let y = variable_full();
    y.0.set_name("y".to_owned());

    // I19 V2
    let fx = variable_full_with_deps(vec![x.0]);
    fx.0.set_name("fx".to_owned());
    let x_eq_y = decision(x.0, y.0, t, f);
    let a_eq_b = decision(a, b, t, f);

    let this_subs = subs(vec![(fx.1, x_eq_y), (x.1, a), (y.1, b)]);
    let tricky_sub = unchecked_substitution(fx.0, &this_subs);

    assert_eq!(
        tricky_sub.get_equality(&a_eq_b, 5),
        Ok(Equal::Yes(subs(vec![(y.1, b)]), Default::default()))
    );
}

// DECISION[a b SELF c] = SELF (recursion)
// u should be
#[test]
fn recursion_is_tracked_in_decision() {
    let mut env = env();

    let a = unique();
    let b = unique();
    let c = unique();
    let dec = decision(a, b, c, c);
    let dec_rec = todo!();

    assert_eq!(
        dec.get_equality(&dec_rec, 3),
        Ok(Equal::Yes(subs(vec![]), vec![dec]))
    );
}