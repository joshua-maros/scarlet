use super::{Environment, TypedItem};
use crate::shared::{Item, ItemId};

impl Environment {
    pub fn _iter(&self) -> impl Iterator<Item = (ItemId, &Item, &Option<ItemId>)> {
        self.items
            .iter()
            .enumerate()
            .map(|(index, val)| (ItemId(index), &val.base, &val.typee))
    }

    pub fn _iter_mut(&mut self) -> impl Iterator<Item = (ItemId, &mut Item, &mut Option<ItemId>)> {
        self.items
            .iter_mut()
            .enumerate()
            .map(|(index, val)| (ItemId(index), &mut val.base, &mut val.typee))
    }

    pub fn insert(&mut self, def: Item) -> ItemId {
        if let Some(existing_id) = self.item_reverse_lookup.get(&def) {
            return *existing_id;
        }
        let id = ItemId(self.items.len());
        self.item_reverse_lookup.insert(def.clone(), id);
        self.items.push(TypedItem {
            base: def,
            typee: None,
        });
        id
    }

    pub fn insert_with_type(&mut self, def: Item, typee: ItemId) -> ItemId {
        if let Some(existing_id) = self.item_reverse_lookup.get(&def) {
            return *existing_id;
        }
        let id = ItemId(self.items.len());
        self.item_reverse_lookup.insert(def.clone(), id);
        self.items.push(TypedItem {
            base: def,
            typee: Some(typee),
        });
        id
    }

    pub fn set_type(&mut self, item: ItemId, typee: ItemId) {
        assert!(item.0 < self.items.len());
        self.items[item.0].typee = Some(typee)
    }
}