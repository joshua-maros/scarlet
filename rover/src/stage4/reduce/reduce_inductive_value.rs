use super::ReduceOptions;
use crate::{
    shared::{Item, ItemId},
    stage4::structure::Environment,
};

impl Environment {
    pub fn reduce_inductive_type(
        &mut self,
        opts: ReduceOptions,
        params: Vec<ItemId>,
        selff: ItemId,
    ) -> ItemId {
        let mut new_params = Vec::new();
        for param in &params {
            let rparam = self.reduce(opts.with_item(*param));
            new_params.push(rparam);
        }
        if new_params == params {
            opts.item
        } else {
            let item = Item::InductiveType {
                selff,
                params: new_params,
            };
            let id = self.insert(item, opts.defined_in);
            self.compute_type(id, vec![]).unwrap();
            id
        }
    }

    pub fn reduce_inductive_value(
        &mut self,
        opts: ReduceOptions,
        typee: ItemId,
        records: Vec<ItemId>,
        variant_name: String,
    ) -> ItemId {
        let rtypee = self.reduce(opts.with_item(typee));
        let mut new_records = Vec::new();
        for record in &records {
            let rrecord = self.reduce(opts.with_item(*record));
            new_records.push(rrecord);
        }
        if new_records == records && rtypee == typee {
            opts.item
        } else {
            let item = Item::InductiveValue {
                typee: rtypee,
                records: new_records,
                variant_name,
            };
            let id = self.insert(item, opts.defined_in);
            self.compute_type(id, vec![]).unwrap();
            id
        }
    }
}
