use crate::StorageTrait;

use crate::SparseVecMap;

/// Vec-backed associative container for storing multiple types
impl<Key, Value> StorageTrait for SparseVecMap<Key, Value>
where
    Key: Copy + Into<u32> + 'static,
{
    type Key = Key;
    type Value = Value;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        Some(SparseVecMap::get(self, key))
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        Some(SparseVecMap::get_mut(self, key))
    }

    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
        SparseVecMap::insert(self, key, value, false);
        None
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value> {
        SparseVecMap::remove(self, key);
        None
    }

    fn clear(&mut self) {
        SparseVecMap::clear(self)
    }
}
