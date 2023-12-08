mod ref_cell;

pub use ref_cell::*;

use hibitset::{BitIter, BitSet};

use crate::{BTreeMap, HashMap, SparseVecMap, StoreKey, TypedData};
use std::{cell::Ref, cell::RefMut, fmt::Debug};

#[derive(Debug)]
#[allow(clippy::clippy::enum_variant_names)]
pub enum StoreBacking<Key>
where
    Key: StoreKey + 'static,
{
    BTreeMap(StoreBackingRefCell<BTreeMap<Key, TypedData>>),
    HashMap(StoreBackingRefCell<HashMap<Key, TypedData>>),
    SparseVecMap(StoreBackingRefCell<SparseVecMap<Key, TypedData>>),
}

impl<Key> StoreBacking<Key>
where
    Key: StoreKey,
{
    pub fn get(&self, key: &Key) -> Option<Ref<TypedData>> {
        match self {
            StoreBacking::BTreeMap(backing) => backing.get(key),
            StoreBacking::HashMap(backing) => backing.get(key),
            StoreBacking::SparseVecMap(backing) => backing.get(key),
        }
    }

    pub fn get_mut(&self, key: &Key) -> Option<RefMut<TypedData>> {
        match self {
            StoreBacking::BTreeMap(backing) => backing.get_mut(key),
            StoreBacking::HashMap(backing) => backing.get_mut(key),
            StoreBacking::SparseVecMap(backing) => backing.get_mut(key),
        }
    }

    pub fn insert(&self, key: Key, value: TypedData) {
        match self {
            StoreBacking::BTreeMap(backing) => backing.insert(key, value),
            StoreBacking::HashMap(backing) => backing.insert(key, value),
            StoreBacking::SparseVecMap(backing) => {
                let u32_key: u32 = key.into();
                if !backing.keys.borrow().contains(u32_key) {
                    backing.keys.borrow_mut().add(u32_key);
                    backing.values.borrow_mut().insert(key, value, false);
                }
            }
        }
    }

    pub fn remove(&self, key: &Key) {
        match self {
            StoreBacking::BTreeMap(backing) => backing.remove(key),
            StoreBacking::HashMap(backing) => backing.remove(key),
            StoreBacking::SparseVecMap(backing) => {
                let u32_key: u32 = (*key).into();
                if backing.keys.borrow().contains(u32_key) {
                    backing.remove(key)
                }
            }
        }
    }

    pub fn clear(&self) {
        match self {
            StoreBacking::BTreeMap(backing) => backing.clear(),
            StoreBacking::HashMap(backing) => backing.clear(),
            StoreBacking::SparseVecMap(backing) => backing.clear(),
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        match self {
            StoreBacking::BTreeMap(backing) => backing.contains(key),
            StoreBacking::HashMap(backing) => backing.contains(key),
            StoreBacking::SparseVecMap(backing) => backing.contains(key),
        }
    }

    pub fn keys(&self) -> BitSet {
        match self {
            StoreBacking::BTreeMap(backing) => backing.keys(),
            StoreBacking::HashMap(backing) => backing.keys(),
            StoreBacking::SparseVecMap(backing) => backing.keys(),
        }
    }

    pub fn iter_keys(&self) -> BitIter<BitSet> {
        match self {
            StoreBacking::BTreeMap(backing) => backing.keys.borrow().clone().into_iter(),
            StoreBacking::HashMap(backing) => backing.keys.borrow().clone().into_iter(),
            StoreBacking::SparseVecMap(backing) => backing.keys.borrow().clone().into_iter(),
        }
    }
}
