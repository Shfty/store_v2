use store_macros::impl_store_fields_iterator;

use std::collections::HashMap;
pub struct ConcreteImmutableField;
pub struct ConcreteMutableField;
pub struct OptionalImmutableField;
pub struct OptionalMutableField;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct StoreFieldsIterator<Key, Storage, FieldTypes>
where
    Key: Copy,
{
    keys: Vec<Key>,
    storage: Storage,
    _phantom: std::marker::PhantomData<FieldTypes>,
}

pub trait IterStoreFields<'a, Key, T>
where
    Key: Copy,
{
    type Storage;
    type FieldTypes;
    type Item;

    /// Iterates over fields that share common keys
    fn iter(&'a self) -> StoreFieldsIterator<Key, Self::Storage, Self::FieldTypes>;

    /// Iterates over fields based on a provided set of keys - more optimal than using filter(), as it avoids lookup costs
    fn iter_keys(
        &'a self,
        keys: Vec<Key>,
    ) -> StoreFieldsIterator<Key, Self::Storage, Self::FieldTypes>;

    /// Fetches a single set of fields by key
    fn get(&'a self, key: Key) -> Self::Item;
}

impl_store_fields_iterator!(HashMap, 1..6);
