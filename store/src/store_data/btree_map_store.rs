use std::{cell::Ref, cell::RefCell, cell::RefMut, collections::BTreeMap, fmt::Debug};

use crate::{StoreTrait, TypedData};

use super::{StoreData, TypeKey};

pub struct BTreeMapKeys<'a, Key>(pub Option<std::collections::btree_map::Keys<'a, Key, TypedData>>);

impl<'a, Key> Iterator for BTreeMapKeys<'a, Key> {
    type Item = &'a Key;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.0 {
            iter.next()
        } else {
            None
        }
    }
}

pub struct BTreeMapStoreKeys<'a, Key>(Option<Ref<'a, BTreeMap<Key, TypedData>>>)
where
    Key: 'static;

impl<'a, Key> IntoIterator for &'a BTreeMapStoreKeys<'a, Key>
where
    Key: 'static,
{
    type Item = &'a Key;

    type IntoIter = BTreeMapKeys<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(storage) = &self.0 {
            BTreeMapKeys(Some(storage.keys()))
        } else {
            BTreeMapKeys(None)
        }
    }
}

pub type BTreeMapStore<Key> = StoreData<RefCell<BTreeMap<Key, TypedData>>>;

impl<Key> BTreeMapStore<Key>
where
    Key: Default + Copy + Ord,
{
    fn insert_impl<T>(&self, key: Key, value: T) -> Option<TypedData>
    where
        T: Debug + 'static,
    {
        self.get_storage::<T>()
            .unwrap()
            .borrow_mut()
            .insert(key, TypedData::new(value))
    }
}

/// BTreeMap-backed associative container for storing multiple types
impl<'a, Key> StoreTrait<'a> for BTreeMapStore<Key>
where
    Key: Default + Copy + Ord + 'static,
{
    type Key = Key;
    type KeyIterator = BTreeMapStoreKeys<'a, Key>;

    fn get<T>(&'a self, key: Self::Key) -> Option<Ref<'a, T>>
    where
        T: 'static,
    {
        let storage = self.get_storage::<T>()?.borrow();
        if storage.contains_key(&key) {
            Some(Ref::map(storage, |storage| {
                let value = storage.get(&key).unwrap();
                let value = value.downcast_ref::<T>().unwrap();
                value
            }))
        } else {
            None
        }
    }

    fn get_mut<T>(&'a self, key: Self::Key) -> Option<RefMut<'a, T>>
    where
        T: 'static,
    {
        let storage = self.get_storage::<T>()?.borrow_mut();
        if storage.contains_key(&key) {
            Some(RefMut::map(storage, |storage| {
                let value = storage.get_mut(&key).unwrap();
                let value = value.downcast_mut::<T>().unwrap();
                value
            }))
        } else {
            None
        }
    }

    fn contains_key(&self, key: &Self::Key) -> bool {
        self.values()
            .any(|storage| storage.borrow().contains_key(key))
    }

    fn contains_type_key<T>(&self, key: &Self::Key) -> bool
    where
        T: 'static,
    {
        if let Some(storage) = (**self).get(&TypeKey::of::<T>()) {
            storage.borrow().contains_key(key)
        }
        else {
            false
        }
    }

    fn insert<T>(&mut self, key: Self::Key, value: T) -> Option<TypedData>
    where
        T: Debug + 'static,
    {
        (**self).entry(TypeKey::of::<T>()).or_default();

        self.insert_impl(key, value)
    }

    fn remove<T>(&self, key: &Self::Key) -> Option<TypedData>
    where
        T: 'static,
    {
        self.get_storage::<T>()?.borrow_mut().remove(key)
    }

    fn remove_key(&self, key: &Self::Key) {
        for store in (**self).values() {
            store.borrow_mut().remove(key);
        }
    }

    fn clear(&self) {
        for store in (**self).values() {
            store.borrow_mut().clear();
        }
    }

    fn keys<T>(&'a self) -> BTreeMapStoreKeys<'a, Key>
    where
        T: 'static,
    {
        if let Some(storage) = self.get_storage::<T>() {
            BTreeMapStoreKeys(Some(storage.borrow()))
        } else {
            BTreeMapStoreKeys(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_keys() {
        let mut btree_map_store = BTreeMapStore::<usize>::default();
        btree_map_store.insert(0, false);
        btree_map_store.insert(2, true);
        btree_map_store.insert(4, true);
        btree_map_store.insert(8, false);

        let keys = btree_map_store.keys::<bool>();
        let dyn_iter: &mut dyn Iterator<Item = &usize> = &mut keys.into_iter();

        for key in dyn_iter {
            println!("Key: {:?}", key);
        }
    }
}
