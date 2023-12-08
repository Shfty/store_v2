mod btree_map;
mod hash_map;
mod sparse_vec_map;

pub use btree_map::*;
pub use hash_map::*;
pub use sparse_vec_map::*;

use crate::StoreKey;

/// Trait for an associative container that can store multiple types
pub trait StorageTrait {
    type Key: StoreKey;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value>;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;
    fn clear(&mut self);
}
