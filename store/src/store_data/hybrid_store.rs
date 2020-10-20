#![allow(clippy::enum_variant_names)]

use std::hash::Hash;

use crate::{BTreeMapKeys, HashMap, HashMapKeys, StoreTrait, TypeKey};

use vec_map::{DenseVecMapKeys, SparseVecMapKeys};

use super::{
    BTreeMapStore, BTreeMapStoreKeys, DenseVecMapStore, DenseVecMapStoreKeys, HashMapStore,
    HashMapStoreKeys, SparseVecMapStore, SparseVecMapStoreKeys,
};

pub enum StoreKeys<'a, Key>
where
    Key: Into<usize> + 'static,
{
    BTreeMap(BTreeMapStoreKeys<'a, Key>),
    HashMap(HashMapStoreKeys<'a, Key>),
    SparseVecMap(SparseVecMapStoreKeys<'a, Key>),
    DenseVecMap(DenseVecMapStoreKeys<'a, Key>),
}

impl<'a, Key> IntoIterator for &'a StoreKeys<'a, Key>
where
    Key: Copy + PartialEq + Into<usize> + 'static,
{
    type Item = &'a Key;

    type IntoIter = StoreKeyIterator<'a, Key>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            StoreKeys::BTreeMap(keys) => StoreKeyIterator::BTreeMap(keys.into_iter()),
            StoreKeys::HashMap(keys) => StoreKeyIterator::HashMap(keys.into_iter()),
            StoreKeys::SparseVecMap(keys) => StoreKeyIterator::SparseVecMap(keys.into_iter()),
            StoreKeys::DenseVecMap(keys) => StoreKeyIterator::DenseVecMap(keys.into_iter()),
        }
    }
}

pub enum StoreKeyIterator<'a, Key>
where
    Key: Into<usize>,
{
    BTreeMap(BTreeMapKeys<'a, Key>),
    HashMap(HashMapKeys<'a, Key>),
    SparseVecMap(SparseVecMapKeys<'a, Key>),
    DenseVecMap(DenseVecMapKeys<'a, Key>),
}

impl<'a, Key> Iterator for StoreKeyIterator<'a, Key>
where
    Key: Into<usize>,
{
    type Item = &'a Key;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StoreKeyIterator::BTreeMap(iter) => iter.next(),
            StoreKeyIterator::HashMap(iter) => iter.next(),
            StoreKeyIterator::SparseVecMap(iter) => iter.next(),
            StoreKeyIterator::DenseVecMap(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StoreType {
    BTreeMap,
    HashMap,
    SparseVecMap,
    DenseVecMap,
}

#[derive(Debug, Default)]
pub struct HybridStore<Key>
where
    Key: Copy + Ord + Into<usize>,
{
    type_map: HashMap<TypeKey, StoreType>,
    btree_map_store: BTreeMapStore<Key>,
    hash_map_store: HashMapStore<Key>,
    sparse_vec_map_store: SparseVecMapStore<Key>,
    dense_vec_map_store: DenseVecMapStore<Key>,
}

// Public interface
impl<Key> HybridStore<Key>
where
    Key: Copy + Ord + Into<usize>,
{
    pub fn register_type_storage<T>(&mut self, store_type: StoreType)
    where
        T: 'static,
    {
        self.type_map.insert(TypeKey::of::<T>(), store_type);
    }
}

// Private interface
impl<Key> HybridStore<Key>
where
    Key: Default + Copy + Ord + Hash + Into<usize> + 'static,
{
    fn store_type_of<T>(&self) -> StoreType
    where
        T: 'static,
    {
        *self
            .type_map
            .get(&TypeKey::of::<T>())
            .unwrap_or(&StoreType::HashMap)
    }
}

impl<'a, Key> StoreTrait<'a> for HybridStore<Key>
where
    Key: Copy + Ord + Default + Hash + Into<usize> + 'static,
{
    type Key = Key;
    type KeyIterator = StoreKeys<'a, Key>;

    fn get<T>(&'a self, key: Self::Key) -> Option<std::cell::Ref<'a, T>>
    where
        T: 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreTrait::get(&self.btree_map_store, key),
            StoreType::HashMap => StoreTrait::get(&self.hash_map_store, key),
            StoreType::SparseVecMap => StoreTrait::get(&self.sparse_vec_map_store, key),
            StoreType::DenseVecMap => StoreTrait::get(&self.dense_vec_map_store, key),
        }
    }

    fn get_mut<T>(&'a self, key: Self::Key) -> Option<std::cell::RefMut<'a, T>>
    where
        T: 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreTrait::get_mut(&self.btree_map_store, key),
            StoreType::HashMap => StoreTrait::get_mut(&self.hash_map_store, key),
            StoreType::SparseVecMap => StoreTrait::get_mut(&self.sparse_vec_map_store, key),
            StoreType::DenseVecMap => StoreTrait::get_mut(&self.dense_vec_map_store, key),
        }
    }

    fn contains_key(&self, key: &Self::Key) -> bool {
        self.btree_map_store.contains_key(key)
            || self.hash_map_store.contains_key(key)
            || self.sparse_vec_map_store.contains_key(key)
            || self.dense_vec_map_store.contains_key(key)
    }

    fn contains_type_key<T>(&self, key: &Self::Key) -> bool
    where
        T: 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreTrait::contains_type_key::<T>(&self.btree_map_store, key),
            StoreType::HashMap => StoreTrait::contains_type_key::<T>(&self.hash_map_store, key),
            StoreType::SparseVecMap => {
                StoreTrait::contains_type_key::<T>(&self.sparse_vec_map_store, key)
            }
            StoreType::DenseVecMap => {
                StoreTrait::contains_type_key::<T>(&self.dense_vec_map_store, key)
            }
        }
    }

    fn insert<T>(&mut self, key: Self::Key, value: T) -> Option<crate::TypedData>
    where
        T: std::fmt::Debug + 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreTrait::insert(&mut self.btree_map_store, key, value),
            StoreType::HashMap => StoreTrait::insert(&mut self.hash_map_store, key, value),
            StoreType::SparseVecMap => {
                StoreTrait::insert(&mut self.sparse_vec_map_store, key, value)
            }
            StoreType::DenseVecMap => StoreTrait::insert(&mut self.dense_vec_map_store, key, value),
        }
    }

    fn remove<T>(&self, key: &Self::Key) -> Option<crate::TypedData>
    where
        T: 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreTrait::remove::<T>(&self.btree_map_store, key),
            StoreType::HashMap => StoreTrait::remove::<T>(&self.hash_map_store, key),
            StoreType::SparseVecMap => StoreTrait::remove::<T>(&self.sparse_vec_map_store, key),
            StoreType::DenseVecMap => StoreTrait::remove::<T>(&self.dense_vec_map_store, key),
        }
    }

    fn remove_key(&self, key: &Self::Key) {
        self.btree_map_store.remove_key(key);
        self.hash_map_store.remove_key(key);
        self.sparse_vec_map_store.remove_key(key);
        self.dense_vec_map_store.remove_key(key);
    }

    fn clear(&self) {
        self.btree_map_store.clear();
        self.hash_map_store.clear();
        self.sparse_vec_map_store.clear();
        self.dense_vec_map_store.clear();
    }

    fn keys<T>(&'a self) -> StoreKeys<Key>
    where
        T: 'static,
    {
        match self.store_type_of::<T>() {
            StoreType::BTreeMap => StoreKeys::BTreeMap(self.btree_map_store.keys::<T>()),
            StoreType::HashMap => StoreKeys::HashMap(self.hash_map_store.keys::<T>()),
            StoreType::SparseVecMap => {
                StoreKeys::SparseVecMap(self.sparse_vec_map_store.keys::<T>())
            }
            StoreType::DenseVecMap => StoreKeys::DenseVecMap(self.dense_vec_map_store.keys::<T>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_keys() {
        let mut hybrid_store = HybridStore::<usize>::default();
        hybrid_store.insert(0, false);
        hybrid_store.insert(2, true);
        hybrid_store.insert(4, true);
        hybrid_store.insert(8, false);

        let keys = hybrid_store.keys::<bool>();
        for key in keys.into_iter() {
            println!("Key: {:?}", key);
        }
    }
}
