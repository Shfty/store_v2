use hibitset::BitSet;

use crate::{BTreeMap, HashMap, SparseVecMap, StorageTrait, StoreKey, TypedData};
use std::{cell::Ref, cell::RefCell, cell::RefMut, fmt::Debug, hash::Hash};

use super::TypeKey;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StorageType {
    BTreeMap,
    HashMap,
    SparseVecMap,
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct StoreBacking<Storage>
where
    Storage: StorageTrait,
{
    pub keys: RefCell<BitSet>,
    pub values: RefCell<Storage>,
}

impl<Storage> Debug for StoreBacking<Storage>
where
    Storage: StorageTrait,
    <Storage as StorageTrait>::Key: From<u32>,
    <Storage as StorageTrait>::Value: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.keys.borrow().clone().into_iter().map(|key| {
                (
                    key,
                    Ref::map(self.values.borrow(), |values| {
                        values.get(&key.into()).unwrap()
                    }),
                )
            }))
            .finish()
    }
}

#[derive(Debug)]
#[allow(clippy::clippy::enum_variant_names)]
enum BackingType<Key>
where
    Key: StoreKey + 'static,
{
    BTreeMap(StoreBacking<BTreeMap<Key, TypedData>>),
    HashMap(StoreBacking<HashMap<Key, TypedData>>),
    SparseVecMap(StoreBacking<SparseVecMap<Key, TypedData>>),
}

/// Associative type-keyed storage
#[derive(Debug, Default)]
pub struct Store<Key>
where
    Key: Debug + Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    type_map: HashMap<TypeKey, BackingType<Key>>,
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
        let u32_key: u32 = (*key).into();
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            match type_backing {
                BackingType::BTreeMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(Ref::map(backing.values.borrow(), |values| {
                            values.get(key).unwrap().downcast_ref::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
                BackingType::HashMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(Ref::map(backing.values.borrow(), |values| {
                            values.get(key).unwrap().downcast_ref::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
                BackingType::SparseVecMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(Ref::map(backing.values.borrow(), |values| {
                            values.get(key).downcast_ref::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn get_mut<T>(&self, key: &Key) -> Option<RefMut<T>>
    where
        T: 'static,
    {
        let u32_key: u32 = (*key).into();
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            match type_backing {
                BackingType::BTreeMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(RefMut::map(backing.values.borrow_mut(), |values| {
                            values.get_mut(key).unwrap().downcast_mut::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
                BackingType::HashMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(RefMut::map(backing.values.borrow_mut(), |values| {
                            values.get_mut(key).unwrap().downcast_mut::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
                BackingType::SparseVecMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        Some(RefMut::map(backing.values.borrow_mut(), |values| {
                            values.get_mut(key).downcast_mut::<T>().unwrap()
                        }))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn insert<T>(&mut self, key: Key, value: T)
    where
        T: Debug + 'static,
    {
        let existing = self.contains_type_key::<T>(&key);

        let type_backing =
            self.type_map.entry(TypeKey::of::<T>()).or_insert_with(
                || match Self::storage_type_of::<T>() {
                    StorageType::BTreeMap => BackingType::BTreeMap(StoreBacking::default()),
                    StorageType::HashMap => BackingType::HashMap(StoreBacking::default()),
                    StorageType::SparseVecMap => BackingType::SparseVecMap(StoreBacking::default()),
                },
            );

        let u32_key: u32 = key.into();
        match type_backing {
            BackingType::BTreeMap(backing) => {
                backing.keys.borrow_mut().add(u32_key);

                backing
                    .values
                    .borrow_mut()
                    .insert(key, TypedData::new(value));
            }
            BackingType::HashMap(backing) => {
                backing.keys.borrow_mut().add(u32_key);

                backing
                    .values
                    .borrow_mut()
                    .insert(key, TypedData::new(value));
            }
            BackingType::SparseVecMap(backing) => {
                if !backing.keys.borrow().contains(u32_key) {
                    backing.keys.borrow_mut().add(u32_key);

                    backing
                        .values
                        .borrow_mut()
                        .insert(key, TypedData::new(value), existing);
                }
            }
        }
    }

    pub fn remove<T>(&mut self, key: &Key)
    where
        T: Debug + 'static,
    {
        let u32_key: u32 = (*key).into();
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            match type_backing {
                BackingType::BTreeMap(backing) => {
                    backing.keys.borrow_mut().remove(u32_key);
                    backing.values.borrow_mut().remove(key);
                }
                BackingType::HashMap(backing) => {
                    backing.keys.borrow_mut().remove(u32_key);
                    backing.values.borrow_mut().remove(key);
                }
                BackingType::SparseVecMap(backing) => {
                    backing.keys.borrow_mut().remove(u32_key);
                    backing.values.borrow_mut().remove(key);
                }
            }
        }
    }

    pub fn remove_key(&mut self, key: &Key) {
        let u32_key: u32 = (*key).into();
        for backing in self.type_map.values() {
            match backing {
                BackingType::BTreeMap(backing) => {
                    backing.keys.borrow_mut().remove(u32_key);
                    backing.values.borrow_mut().remove(key);
                }
                BackingType::HashMap(backing) => {
                    backing.keys.borrow_mut().remove(u32_key);
                    backing.values.borrow_mut().remove(key);
                }
                BackingType::SparseVecMap(backing) => {
                    if backing.keys.borrow().contains(u32_key) {
                        backing.keys.borrow_mut().remove(u32_key);
                        backing.values.borrow_mut().remove(key);
                    }
                }
            }
        }
    }

    pub fn clear<T>(&mut self)
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            match type_backing {
                BackingType::BTreeMap(backing) => {
                    backing.keys.borrow_mut().clear();
                    backing.values.borrow_mut().clear();
                }
                BackingType::HashMap(backing) => {
                    backing.keys.borrow_mut().clear();
                    backing.values.borrow_mut().clear();
                }
                BackingType::SparseVecMap(backing) => {
                    backing.keys.borrow_mut().clear();
                    backing.values.borrow_mut().clear();
                }
            }
        }
    }

    pub fn contains_type<T>(&self) -> bool
    where
        T: 'static,
    {
        self.type_map.get(&TypeKey::of::<T>()).is_some()
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        let u32_key: u32 = (*key).into();

        self.type_map.values().any(|backing| match backing {
            BackingType::BTreeMap(backing) => backing.keys.borrow().contains(u32_key),
            BackingType::HashMap(backing) => backing.keys.borrow().contains(u32_key),
            BackingType::SparseVecMap(backing) => backing.keys.borrow().contains(u32_key),
        })
    }

    pub fn contains_type_key<T>(&self, key: &Key) -> bool
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            let u32_key: u32 = (*key).into();
            match type_backing {
                BackingType::BTreeMap(backing) => backing.keys.borrow().contains(u32_key),
                BackingType::HashMap(backing) => backing.keys.borrow().contains(u32_key),
                BackingType::SparseVecMap(backing) => backing.keys.borrow().contains(u32_key),
            }
        } else {
            false
        }
    }

    pub fn keys<T>(&self) -> BitSet
    where
        T: 'static,
    {
        if let Some(type_backing) = self.type_map.get(&TypeKey::of::<T>()) {
            match type_backing {
                BackingType::BTreeMap(backing) => backing.keys.borrow().clone(),
                BackingType::HashMap(backing) => backing.keys.borrow().clone(),
                BackingType::SparseVecMap(backing) => backing.keys.borrow().clone(),
            }
        } else {
            BitSet::new()
        }
    }

    pub fn keys_all(&self) -> BitSet {
        let mut bit_set = BitSet::new();

        for backing in self.type_map.values() {
            let keys = &*match backing {
                BackingType::BTreeMap(backing) => backing.keys.borrow(),
                BackingType::HashMap(backing) => backing.keys.borrow(),
                BackingType::SparseVecMap(backing) => backing.keys.borrow(),
            };
            bit_set |= keys;
        }

        bit_set
    }

    pub fn iter_untyped(&self) -> impl Iterator<Item = (TypeKey, Ref<TypedData>)> {
        let mut typed_data: Vec<(TypeKey, Ref<TypedData>)> = vec![];

        for (type_key, store_backing) in &self.type_map {
            for u32_key in match store_backing {
                BackingType::BTreeMap(backing) => backing.keys.borrow().clone().into_iter(),
                BackingType::HashMap(backing) => backing.keys.borrow().clone().into_iter(),
                BackingType::SparseVecMap(backing) => backing.keys.borrow().clone().into_iter(),
            } {
                let key: &Key = &u32_key.into();
                let data_ref = match store_backing {
                    BackingType::BTreeMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key).unwrap())
                    }
                    BackingType::HashMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key).unwrap())
                    }
                    BackingType::SparseVecMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key))
                    }
                };

                typed_data.push((*type_key, data_ref));
            }
        }

        typed_data.into_iter()
    }

    pub fn iter_key_untyped(&self, key: &Key) -> impl Iterator<Item = (TypeKey, Ref<TypedData>)> {
        let mut typed_data: Vec<(TypeKey, Ref<TypedData>)> = vec![];

        for (type_key, store_backing) in self.type_map.iter() {
            let u32_key: u32 = (*key).into();
            if match store_backing {
                BackingType::BTreeMap(backing) => backing.keys.borrow().contains(u32_key),
                BackingType::HashMap(backing) => backing.keys.borrow().contains(u32_key),
                BackingType::SparseVecMap(backing) => backing.keys.borrow().contains(u32_key),
            } {
                let data_ref = match store_backing {
                    BackingType::BTreeMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key).unwrap())
                    }
                    BackingType::HashMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key).unwrap())
                    }
                    BackingType::SparseVecMap(backing) => {
                        Ref::map(backing.values.borrow(), |values| values.get(key))
                    }
                };

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
