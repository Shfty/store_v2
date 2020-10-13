use fnv::FnvHashMap;

use crate::Storable;
use std::{any::Any, any::TypeId, cell::RefCell};

/// A struct for storing collections of data by type
#[derive(Debug)]
pub struct Store {
    storage: FnvHashMap<TypeId, Box<dyn Any>>,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            storage: FnvHashMap::default(),
        }
    }
}

impl<'a> Store {
    pub fn add_storage_for<T>(&'a mut self)
    where
        T: Storable + 'static,
        T::Storage: Default,
    {
        let type_id = TypeId::of::<T>();
        assert!(!self.has_storage_for::<T>());

        let storage = T::Storage::default();
        let ref_cell: RefCell<T::Storage> = RefCell::new(storage);
        let boxed: Box<dyn Any> = Box::new(ref_cell);
        self.storage.insert(type_id, boxed);
    }

    pub fn get_storage<T>(&'a self) -> &'a RefCell<T::Storage>
    where
        T: Storable + 'static,
    {
        let storage = self.get_storage_any::<T>();
        let storage = storage.downcast_ref::<RefCell<T::Storage>>();
        storage.unwrap_or_else(|| {
            panic!(
                "Failed to downcast storage for {}",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn has_storage_for<T>(&'a self) -> bool
    where
        T: Storable + 'static,
    {
        self.storage.contains_key(&TypeId::of::<T>())
    }

    fn get_storage_any<T>(&'a self) -> &'a Box<dyn Any>
    where
        T: 'static,
    {
        self.storage
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("Failed to get storage for {}", std::any::type_name::<T>()))
    }
}
