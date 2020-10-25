use hibitset::BitSet;

use crate::{BTreeMap, HashMap, SparseVecMap, StorageTrait, TypedData};
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

/// Associative type-keyed storage
#[derive(Debug, Default)]
pub struct Store<Key>
where
    Key: Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    type_map: HashMap<TypeKey, StorageType>,
    btree_map: HashMap<TypeKey, StoreBacking<BTreeMap<Key, TypedData>>>,
    hash_map: HashMap<TypeKey, StoreBacking<HashMap<Key, TypedData>>>,
    sparse_vec_map: HashMap<TypeKey, StoreBacking<SparseVecMap<Key, TypedData>>>,
}

impl<Key> Store<Key>
where
    Key: Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    fn storage_type_of<T>(&self) -> &StorageType
    where
        T: 'static,
    {
        &StorageType::HashMap
        /*
        let type_key = TypeKey::of::<T>();

        if self.type_map.contains_key(&type_key) {
            self.type_map.get(&type_key).unwrap()
        } else if std::mem::size_of::<T>() < std::mem::size_of::<usize>() {
            &StorageType::SparseVecMap
        } else {
            &StorageType::HashMap
        }
        */
    }
}

impl<Key> Store<Key>
where
    Key: Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    pub fn register_storage_type_for<T>(&mut self, storage_type: StorageType)
    where
        T: 'static,
    {
        self.type_map.insert(TypeKey::of::<T>(), storage_type);
    }

    pub fn get<T>(&self, key: &Key) -> Option<Ref<T>>
    where
        T: 'static,
    {
        if self.contains_type_key::<T>(key) {
            let type_key = TypeKey::of::<T>();
            match self.storage_type_of::<T>() {
                StorageType::BTreeMap => Some(Ref::map(
                    self.btree_map.get(&type_key)?.values.borrow(),
                    move |storage| {
                        let value = storage.get(key).unwrap();
                        value.downcast_ref::<T>().unwrap()
                    },
                )),
                StorageType::HashMap => Some(Ref::map(
                    self.hash_map.get(&type_key)?.values.borrow(),
                    move |storage| {
                        let value = storage.get(key).unwrap();
                        value.downcast_ref::<T>().unwrap()
                    },
                )),
                StorageType::SparseVecMap => Some(Ref::map(
                    self.sparse_vec_map.get(&type_key)?.values.borrow(),
                    move |storage| {
                        let value = storage.get(key);
                        value.downcast_ref::<T>().unwrap()
                    },
                )),
            }
        } else {
            None
        }
    }

    pub fn get_mut<T>(&self, key: &Key) -> Option<RefMut<T>>
    where
        T: 'static,
    {
        if self.contains_type_key::<T>(key) {
            let type_key = TypeKey::of::<T>();
            match self.storage_type_of::<T>() {
                StorageType::BTreeMap => Some(RefMut::map(
                    self.btree_map.get(&type_key)?.values.borrow_mut(),
                    move |storage| {
                        let value = storage.get_mut(key).unwrap();
                        value.downcast_mut::<T>().unwrap()
                    },
                )),
                StorageType::HashMap => Some(RefMut::map(
                    self.hash_map.get(&type_key)?.values.borrow_mut(),
                    move |storage| {
                        let value = storage.get_mut(key).unwrap();
                        value.downcast_mut::<T>().unwrap()
                    },
                )),
                StorageType::SparseVecMap => Some(RefMut::map(
                    self.sparse_vec_map.get(&type_key)?.values.borrow_mut(),
                    move |storage| {
                        let value = storage.get_mut(key);
                        value.downcast_mut::<T>().unwrap()
                    },
                )),
            }
        } else {
            None
        }
    }

    pub fn insert<T>(&mut self, key: Key, value: T)
    where
        T: Debug + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let existing = self.contains_type_key::<T>(&key);

        match self.storage_type_of::<T>() {
            StorageType::BTreeMap => {
                let backing = self.btree_map.entry(type_key).or_default();

                backing.keys.borrow_mut().add(key.into());
                backing
                    .values
                    .borrow_mut()
                    .insert(key, TypedData::new(value));
            }
            StorageType::HashMap => {
                let backing = self.hash_map.entry(type_key).or_default();

                backing.keys.borrow_mut().add(key.into());
                backing
                    .values
                    .borrow_mut()
                    .insert(key, TypedData::new(value));
            }
            StorageType::SparseVecMap => {
                let backing = self.sparse_vec_map.entry(type_key).or_default();

                backing.keys.borrow_mut().add(key.into());
                backing
                    .values
                    .borrow_mut()
                    .insert(key, TypedData::new(value), existing);
            }
        }
    }

    pub fn remove<T>(&mut self, key: &Key)
    where
        T: Debug + 'static,
    {
        if self.contains_type_key::<T>(key) {
            let type_key = TypeKey::of::<T>();
            match self.storage_type_of::<T>() {
                StorageType::BTreeMap => {
                    let backing = self.btree_map.get(&type_key).unwrap();
                    backing.keys.borrow_mut().remove((*key).into());
                    backing.values.borrow_mut().remove(key);
                }
                StorageType::HashMap => {
                    let backing = self.hash_map.get(&type_key).unwrap();
                    backing.keys.borrow_mut().remove((*key).into());
                    backing.values.borrow_mut().remove(key);
                }
                StorageType::SparseVecMap => {
                    let backing = self.sparse_vec_map.get(&type_key).unwrap();
                    backing.keys.borrow_mut().remove((*key).into());
                    backing.values.borrow_mut().remove(key);
                }
            }
        }
    }

    pub fn remove_key(&mut self, key: &Key) {
        let u32_key: u32 = (*key).into();

        for backing in self.btree_map.values() {
            if backing.keys.borrow().contains(u32_key) {
                backing.keys.borrow_mut().remove(u32_key);
                backing.values.borrow_mut().remove(key);
            }
        }

        for backing in self.hash_map.values() {
            if backing.keys.borrow().contains(u32_key) {
                backing.keys.borrow_mut().remove(u32_key);
                backing.values.borrow_mut().remove(key);
            }
        }

        for backing in self.sparse_vec_map.values() {
            if backing.keys.borrow().contains(u32_key) {
                backing.keys.borrow_mut().remove(u32_key);
                backing.values.borrow_mut().remove(key);
            }
        }
    }

    pub fn clear<T>(&mut self)
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        match self.storage_type_of::<T>() {
            StorageType::BTreeMap => {
                let backing = self.btree_map.get(&type_key).unwrap();
                backing.keys.borrow_mut().clear();
                backing.values.borrow_mut().clear();
            }
            StorageType::HashMap => {
                let backing = self.hash_map.get(&type_key).unwrap();
                backing.keys.borrow_mut().clear();
                backing.values.borrow_mut().clear();
            }
            StorageType::SparseVecMap => {
                let backing = self.sparse_vec_map.get(&type_key).unwrap();
                backing.keys.borrow_mut().clear();
                backing.values.borrow_mut().clear();
            }
        }
    }

    pub fn contains_type<T>(&self) -> bool
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        match self.storage_type_of::<T>() {
            StorageType::BTreeMap => self.btree_map.contains_key(&type_key),
            StorageType::HashMap => self.hash_map.contains_key(&type_key),
            StorageType::SparseVecMap => self.sparse_vec_map.contains_key(&type_key),
        }
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.btree_map
            .values()
            .any(|backing| backing.keys.borrow().contains((*key).into()))
            || self
                .hash_map
                .values()
                .any(|backing| backing.keys.borrow().contains((*key).into()))
            || self
                .sparse_vec_map
                .values()
                .any(|backing| backing.keys.borrow().contains((*key).into()))
    }

    pub fn contains_type_key<T>(&self, key: &Key) -> bool
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        match self.storage_type_of::<T>() {
            StorageType::BTreeMap => {
                if let Some(backing) = self.btree_map.get(&type_key) {
                    backing.keys.borrow().contains((*key).into())
                } else {
                    false
                }
            }
            StorageType::HashMap => {
                if let Some(backing) = self.hash_map.get(&type_key) {
                    backing.keys.borrow().contains((*key).into())
                } else {
                    false
                }
            }
            StorageType::SparseVecMap => {
                if let Some(backing) = self.sparse_vec_map.get(&type_key) {
                    backing.keys.borrow().contains((*key).into())
                } else {
                    false
                }
            }
        }
    }

    pub fn keys<T>(&self) -> BitSet
    where
        T: 'static,
    {
        let type_key = TypeKey::of::<T>();
        match self.storage_type_of::<T>() {
            StorageType::BTreeMap => {
                if let Some(backing) = self.btree_map.get(&type_key) {
                    backing.keys.borrow().clone()
                } else {
                    BitSet::new()
                }
            }
            StorageType::HashMap => {
                if let Some(backing) = self.hash_map.get(&type_key) {
                    backing.keys.borrow().clone()
                } else {
                    BitSet::new()
                }
            }
            StorageType::SparseVecMap => {
                if let Some(backing) = self.sparse_vec_map.get(&type_key) {
                    backing.keys.borrow().clone()
                } else {
                    BitSet::new()
                }
            }
        }
    }

    pub fn keys_all(&self) -> BitSet {
        let mut bit_set = BitSet::new();

        for backing in self.btree_map.values() {
            let keys = backing.keys.borrow();
            let keys = &*keys;
            bit_set |= keys;
        }

        for backing in self.hash_map.values() {
            let keys = backing.keys.borrow();
            let keys = &*keys;
            bit_set |= keys;
        }

        for backing in self.sparse_vec_map.values() {
            let keys = backing.keys.borrow();
            let keys = &*keys;
            bit_set |= keys;
        }

        bit_set
    }
}
