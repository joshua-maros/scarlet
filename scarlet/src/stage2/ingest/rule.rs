use crate::{
    pattern,
    stage2::ingest::pattern::{rep, Pattern},
};

pub type Precedence = u8;

#[derive(Clone, Debug)]
pub struct Rule {
    pub name: String,
    pub pattern: Pattern,
    pub result_precedence: Precedence,
}

pub fn build_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: format!("+"),
            pattern: pattern!([80, "+", 79]),
            result_precedence: 80,
        },
        Rule {
            name: format!("*"),
            pattern: pattern!([70, "*", 69]),
            result_precedence: 70,
        },
        Rule {
            name: format!("^"),
            pattern: pattern!([59, "^", 60]),
            result_precedence: 60,
        },
        Rule {
            name: format!("paren"),
            pattern: pattern!(["(", rep(pattern!(255)), ")"]),
            result_precedence: 1,
        },
    ]
}
