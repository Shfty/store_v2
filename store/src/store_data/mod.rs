mod hash_map_store;
mod btree_map_store;
mod sparse_vec_map_store;
mod dense_vec_map_store;
mod hybrid_store;

pub use hash_map_store::*;
pub use btree_map_store::*;
pub use sparse_vec_map_store::*;
pub use dense_vec_map_store::*;
pub use hybrid_store::*;

use std::{fmt::Debug, ops::Deref, ops::DerefMut};
use fnv::FnvHashMap;
use super::TypeKey;

/// Associative type-keyed storage
#[derive(Default)]
pub struct StoreData<Storage> {
    data: FnvHashMap<TypeKey, Storage>,
}

impl<Storage> StoreData<Storage> {
    pub fn get_storage<T>(&self) -> Option<&Storage>
    where
        T: 'static,
    {
        self.get_storage_by_id(&TypeKey::of::<T>())
    }

    pub fn get_storage_by_id(&self, type_id: &TypeKey) -> Option<&Storage> {
        self.data.get(&type_id)
    }
}

impl<Key> Debug for StoreData<Key>
where
    Key: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.data.iter()).finish()
    }
}

impl<Storage> Deref for StoreData<Storage> {
    type Target = FnvHashMap<TypeKey, Storage>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<Storage> DerefMut for StoreData<Storage> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
