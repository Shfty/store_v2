use std::{
    cell::{Ref, RefMut},
    fmt::Debug,
};

use super::TypedData;

/// Trait for an associative container that can store multiple types
pub trait StoreTrait<'a> {
    type Key: Copy;
    type KeyIterator;

    fn get<T>(&'a self, key: Self::Key) -> Option<Ref<'a, T>>
    where
        T: 'static;

    fn get_mut<T>(&'a self, key: Self::Key) -> Option<RefMut<'a, T>>
    where
        T: 'static;

    fn contains_key(&self, key: &Self::Key) -> bool;
    fn contains_type_key<T>(&self, key: &Self::Key) -> bool
    where
        T: 'static;

    fn insert<T>(&mut self, key: Self::Key, value: T) -> Option<TypedData>
    where
        T: Debug + 'static;

    fn remove<T>(&self, key: &Self::Key) -> Option<TypedData>
    where
        T: 'static;

    fn remove_key(&self, key: &Self::Key);

    fn clear(&self);

    fn keys<T>(&'a self) -> Self::KeyIterator
    where
        T: 'static;
}
