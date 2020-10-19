use std::{cell::Ref, cell::RefCell, cell::RefMut, fmt::Debug};

use crate::{StoreTrait, TypedData};

use super::{StoreData, TypeKey};

use vec_map::{DenseVecMap, DenseVecMapKeys};

pub struct DenseVecMapStoreKeys<'a, Key>
where
    Key: Into<usize> + 'static,
{
    storage: Option<Ref<'a, DenseVecMap<Key, TypedData>>>,
}

impl<'a, Key> IntoIterator for &'a DenseVecMapStoreKeys<'a, Key>
where
    Key: Copy + Into<usize> + 'static,
{
    type Item = &'a Key;

    type IntoIter = vec_map::DenseVecMapKeys<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(storage) = &self.storage {
            storage.keys()
        } else {
            DenseVecMapKeys(None)
        }
    }
}

pub type DenseVecMapStore<Key> = StoreData<RefCell<DenseVecMap<Key, TypedData>>>;

impl<Key> DenseVecMapStore<Key>
where
    Key: Copy + Into<usize>,
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

/// Vec-backed associative container for storing multiple types
impl<'a, Key> StoreTrait<'a> for DenseVecMapStore<Key>
where
    Key: Copy + Into<usize> + 'static,
{
    type Key = Key;
    type KeyIterator = DenseVecMapStoreKeys<'a, Key>;

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
        (**self)
            .values()
            .any(|storage| storage.borrow().contains_key(key))
    }

    fn contains_type_key<T>(&self, key: &Self::Key) -> bool
    where
        T: 'static,
    {
        let storage = (**self).get(&TypeKey::of::<T>());
        if let Some(storage) = storage {
            storage.borrow().contains_key(key)
        } else {
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

    fn keys<T>(&'a self) -> DenseVecMapStoreKeys<Key>
    where
        T: 'static,
    {
        if let Some(storage) = self.get_storage::<T>() {
            DenseVecMapStoreKeys {
                storage: Some(storage.borrow()),
            }
        } else {
            DenseVecMapStoreKeys { storage: None }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_keys() {
        let mut dense_vec_map_store = DenseVecMapStore::<usize>::default();
        dense_vec_map_store.insert(0, false);
        dense_vec_map_store.insert(2, true);
        dense_vec_map_store.insert(4, true);
        dense_vec_map_store.insert(8, false);

        let keys = dense_vec_map_store.keys::<bool>();
        let dyn_iter: &mut dyn Iterator<Item = &usize> = &mut keys.into_iter();

        for key in dyn_iter {
            println!("Key: {:?}", key);
        }
    }
}
