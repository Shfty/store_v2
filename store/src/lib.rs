mod assemblage;
mod collections;
mod storage;
mod store_backing;
mod store_key;
mod store_query;
mod type_key;
mod typed_data;

pub use assemblage::*;
pub use collections::*;
pub use storage::*;
pub use store_backing::*;
pub use store_key::*;
pub use store_query::*;
pub use type_key::*;
pub use typed_data::*;

use hibitset::BitSet;
use std::{cell::Ref, cell::RefMut, fmt::Debug, hash::Hash};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StorageType {
    BTreeMap,
    HashMap,
    SparseVecMap,
}

/// Associative type-keyed storage
#[derive(Debug, Default)]
pub struct Store<Key>
where
    Key: Debug + Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    type_map: HashMap<TypeKey, StoreBacking<Key>>,
}

impl<Key> Store<Key>
where
    Key: Debug + Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    fn storage_type_of<T>() -> &'static StorageType
    where
        T: 'static,
    {
        if std::mem::size_of::<T>() < std::mem::size_of::<usize>() {
            &StorageType::SparseVecMap
        } else {
            &StorageType::HashMap
        }
    }
}

impl<Key> Store<Key>
where
    Key: Debug + Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    pub fn get<T>(&self, key: &Key) -> Option<Ref<T>>
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        let type_backing = self.type_map.get(&type_key)?;
        let data = type_backing.get(key)?;
        Some(Ref::map(data, |data| data.downcast::<T>().unwrap()))
    }

    pub fn get_mut<T>(&self, key: &Key) -> Option<RefMut<T>>
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        let type_backing = self.type_map.get(&type_key)?;
        let data = type_backing.get_mut(key)?;
        Some(RefMut::map(data, |data| data.downcast_mut::<T>().unwrap()))
    }

    fn create_storage_for<T>() -> StoreBacking<Key>
    where
        T: 'static,
    {
        match Self::storage_type_of::<T>() {
            StorageType::BTreeMap => StoreBacking::BTreeMap(StoreBackingRefCell::default()),
            StorageType::HashMap => StoreBacking::HashMap(StoreBackingRefCell::default()),
            StorageType::SparseVecMap => StoreBacking::SparseVecMap(StoreBackingRefCell::default()),
        }
    }

    pub fn insert<T>(&mut self, key: Key, value: T)
    where
        T: Debug + 'static,
    {
        let type_backing = self
            .type_map
            .entry(TypeKey::of::<T>())
            .or_insert_with(Self::create_storage_for::<T>);

        type_backing.insert(key, TypedData::new(value));
    }

    pub fn remove<T>(&mut self, key: &Key)
    where
        T: Debug + 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            type_backing.remove(key);
        }
    }

    pub fn remove_key(&mut self, key: &Key) {
        for type_backing in self.type_map.values() {
            type_backing.remove(key);
        }
    }

    pub fn clear<T>(&mut self)
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            type_backing.clear();
        }
    }

    pub fn contains_type<T>(&self) -> bool
    where
        T: 'static,
    {
        self.type_map.get(&TypeKey::of::<T>()).is_some()
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.type_map.values().any(|backing| backing.contains(key))
    }

    pub fn contains_type_key<T>(&self, key: &Key) -> bool
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            type_backing.contains(key)
        } else {
            false
        }
    }

    pub fn keys<T>(&self) -> BitSet
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            type_backing.keys()
        } else {
            BitSet::new()
        }
    }

    pub fn keys_all(&self) -> BitSet {
        let mut bit_set = BitSet::new();

        for type_backing in self.type_map.values() {
            bit_set |= &type_backing.keys();
        }

        bit_set
    }

    pub fn iter_untyped(&self) -> impl Iterator<Item = (TypeKey, Ref<TypedData>)> {
        let mut typed_data: Vec<(TypeKey, Ref<TypedData>)> = vec![];

        for (type_key, store_backing) in &self.type_map {
            for u32_key in store_backing.iter_keys() {
                let key: &Key = &u32_key.into();
                let data_ref = store_backing.get(key).unwrap();
                typed_data.push((*type_key, data_ref));
            }
        }

        typed_data.into_iter()
    }

    pub fn iter_key_untyped(&self, key: &Key) -> impl Iterator<Item = (TypeKey, Ref<TypedData>)> {
        let mut typed_data: Vec<(TypeKey, Ref<TypedData>)> = vec![];

        for (type_key, store_backing) in self.type_map.iter() {
            if store_backing.contains(key) {
                let data_ref = store_backing.get(key).unwrap();
                typed_data.push((*type_key, data_ref));
            }
        }

        typed_data.into_iter()
    }

    pub fn iter_types(&self) -> impl Iterator<Item = &TypeKey> {
        self.type_map.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_types() {
        println!();

        let mut store = Store::<u32>::default();
        store.insert(0, "Hello");
        store.insert(1, "Foo");
        store.insert(2, "Bar");

        store.insert(0, 1);
        store.insert(1, 2);

        store.insert(0, 0..10);

        for key in store.iter_types() {
            println!("Type: {:?}", key);
        }
    }

    #[test]
    fn iter_untyped() {
        println!();

        let mut store = Store::<u32>::default();
        store.insert(0, "Hello");
        store.insert(1, "Foo");
        store.insert(2, "Bar");

        store.insert(0, 1);
        store.insert(1, 2);

        store.insert(0, 0..10);

        for (key, value) in store.iter_untyped() {
            println!("Type: {:?}, Data: {:?}", key, value);
        }
    }

    #[test]
    fn iter_key_untyped() {
        println!();

        let mut store = Store::<u32>::default();
        store.insert(0, "Hello");
        store.insert(1, "Foo");
        store.insert(2, "Bar");

        store.insert(0, 1);
        store.insert(1, 2);

        store.insert(0, 0..10);

        for i in 0..3 {
            println!("Key {}:", i);
            for (key, value) in store.iter_key_untyped(&i) {
                println!("Type: {:?}, Data: {:?}", key, value);
            }
        }
    }
}
