use crate::tokens::structure::Token;

#[macro_export]
macro_rules! tfers {
    ($($transformer:expr),*) => {
        vec![$(Box::new($transformer) as Box<dyn crate::environment::resolve::transform::basics::Transformer>),*]
    }
}

pub fn expect_paren_group<'a, 't>(tt: &'a Token<'t>) -> &'a Vec<Token<'t>> {
    if let Token::Stream {
        label: "group()",
        contents: body,
    } = tt
    {
        body
    } else {
        todo!("nice error, expected parentheses")
    }
}