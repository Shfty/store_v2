use crate::{BTreeMap, StorageTrait, StoreKey};

/// BTreeMap-backed associative container for storing multiple types
impl<Key, Value> StorageTrait for BTreeMap<Key, Value>
where
    Key: StoreKey,
{
    type Key = Key;
    type Value = Value;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        BTreeMap::get_mut(self, key)
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        BTreeMap::insert(self, key, value)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        BTreeMap::remove(self, key)
    }

    fn clear(&mut self) {
        BTreeMap::clear(self)
    }
}
