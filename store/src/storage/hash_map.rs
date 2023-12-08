use crate::{HashMap, StorageTrait, StoreKey};

/// HashMap-backed associative container for storing multiple types
impl<Key, Value> StorageTrait for HashMap<Key, Value>
where
    Key: StoreKey,
{
    type Key = Key;
    type Value = Value;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        HashMap::get_mut(self, key)
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        HashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        HashMap::remove(self, key)
    }

    fn clear(&mut self) {
        HashMap::clear(self)
    }
}
