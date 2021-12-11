use super::{ConstructDefinition, ConstructId, Environment};
use crate::{
    constructs::{base::Construct, downcast_construct, shown::CShown},
    tokens::structure::Token,
    transform::{self, ApplyContext},
};

impl<'x> Environment<'x> {
    pub fn show_all_requested(&mut self) {
        let mut to_vomit = Vec::new();
        for (_, acon) in &self.constructs {
            if let ConstructDefinition::Resolved(con) = &acon.definition {
                if let Some(shown) = downcast_construct::<CShown>(&**con) {
                    to_vomit.push(shown.0);
                }
            }
        }
        for con_id in to_vomit {
            let con_id = self.resolve(con_id);
            let vomited = self.vomit(con_id);
            println!("{:?} is\n{}", con_id, vomited);
            println!("depends on:");
            for dep in self.get_dependencies(con_id) {
                let kind = match dep.capturing {
                    true => "capturing",
                    false => "without capturing",
                };
                println!("    {} (", kind);
                for inv in &dep.invariants {
                    println!("        {}", self.vomit(*inv));
                }
                println!("    )");
            }
            println!();
        }
        // println!("{:#?}", self);
    }

    fn expand_token(&mut self, input: Token<'x>) -> Token<'x> {
        let extras = Default::default();
        let tfers = transform::all_transformers(&extras);
        let _scope = self.root_scope();
        for tfer in &tfers {
            let mut context = ApplyContext { env: self };
            if let Some(replace_with) = tfer.as_ref().vomit(&mut context, &input) {
                return self.expand_token(replace_with);
            }
        }
        if let Token::Construct(_con_id) = input {
            input
        } else if let Token::Stream { label, contents } = input {
            let contents = contents.into_iter().map(|t| self.expand_token(t)).collect();
            Token::Stream { label, contents }
        } else {
            input
        }
    }

    fn vomit(&mut self, con_id: ConstructId) -> String {
        format!("{:?}", self.expand_token(con_id.into()))
    }
}
