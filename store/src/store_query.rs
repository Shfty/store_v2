use std::{
    cell::{Ref, RefMut},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
};

use crate::{Store, TypeKey};

use hibitset::{BitIter, BitSet};
use store_macros::impl_store_fields_iterator;

// Core Types
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NoField<T>(TypeKey, PhantomData<T>);

impl<T> Default for NoField<T>
where
    T: 'static,
{
    fn default() -> Self {
        NoField(TypeKey::of::<T>(), PhantomData)
    }
}

impl<T> Debug for NoField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NoField").field(&self.0).finish()
    }
}

pub trait StoreQuery<'a, Signature>
where
    Self::Key: Debug + Copy + Ord + From<u32> + Into<u32> + Hash,
{
    type Key;

    fn get(&'a self, key: &Self::Key) -> Signature;
    fn iter(&'a self) -> StoreIterator<Self::Key, Signature>;
    fn iter_keys(&'a self, keys: &'a [Self::Key]) -> StoreIterator<Self::Key, Signature>;
}

pub struct StoreIterator<'a, Key, Signature>
where
    Key: Debug + Copy + Eq + Ord + Hash + From<u32> + Into<u32> + 'static,
{
    store: &'a Store<Key>,
    keys: BitIter<BitSet>,
    _phantom_data: PhantomData<Signature>,
}

impl_store_fields_iterator!(1..6);

// Tests
#[cfg(test)]
mod tests {
    use crate::SomeData;

    use super::*;

    #[test]
    fn debug() {
        let mut store = Store::<u32>::default();

        //store.register_storage_type_for::<SomeData<String>>(StorageType::BTreeMap);

        store.insert(0, false);
        store.insert(1, true);
        store.insert(2, true);
        store.insert(3, false);

        store.insert(0, "Hello");
        store.insert(1, "World");
        store.insert(2, "Goodbye");
        store.insert(3, "Farewell");

        //store.insert(0, 0);
        store.insert(1, 2);
        store.insert(2, 4);
        //store.insert(3, 6);

        //store.insert(0, SomeData("Ahoy".to_string()));
        store.insert(1, SomeData("Matey".to_string()));
        //store.insert(2, SomeData("Avast Ye".to_string()));
        store.insert(3, SomeData("Landlubbers".to_string()));

        println!("\nStore: {:#?}\n", store);
    }

    #[test]
    fn get() {
        let mut store = Store::<u32>::default();

        //store.register_storage_type_for::<SomeData<String>>(StorageType::BTreeMap);

        store.insert(0, false);
        store.insert(1, true);
        store.insert(2, true);
        store.insert(3, false);

        store.insert(0, "Hello");
        store.insert(1, "World");
        store.insert(2, "Goodbye");
        store.insert(3, "Farewell");

        //store.insert(0, 0);
        store.insert(1, 2);
        store.insert(2, 4);
        //store.insert(3, 6);

        //store.insert(0, SomeData("Ahoy".to_string()));
        store.insert(1, SomeData("Matey".to_string()));
        //store.insert(2, SomeData("Avast Ye".to_string()));
        store.insert(3, SomeData("Landlubbers".to_string()));

        {
            let query = StoreQuery::<(
                u32,
                NoField<i32>,
                Ref<bool>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, &0);
            println!("0: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                u32,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, &1);
            println!("1: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                u32,
                NoField<SomeData<String>>,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
            )>::get(&store, &2);
            println!("2: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                u32,
                NoField<i32>,
                Ref<bool>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, &3);
            println!("3: {:?}", query);
        }
    }

    #[test]
    fn iter() {
        let mut store = Store::<u32>::default();

        store.insert(0, false);
        store.insert(1, true);
        store.insert(2, true);
        store.insert(3, false);

        //store.insert(0, "Hello");
        store.insert(1, "World");
        //store.insert(2, "Goodbye");
        store.insert(3, "Farewell");

        //store.insert(0, 0);
        store.insert(1, 2);
        store.insert(2, 4);
        //store.insert(3, 6);

        store.insert(0, SomeData("Ahoy".to_string()));
        store.insert(1, SomeData("Matey".to_string()));
        store.insert(2, SomeData("Avast Ye".to_string()));
        store.insert(3, SomeData("Landlubbers".to_string()));

        println!();
        println!("Arity 4:");
        StoreQuery::<(
            u32,
            Ref<bool>,
            Option<Ref<i32>>,
            RefMut<&'static str>,
            Option<RefMut<SomeData<String>>>,
        )>::iter(&store)
        .for_each(|result| println!("Result: {:?}", result));

        println!();
        println!("Arity 2:");
        StoreQuery::<(u32, Ref<bool>, RefMut<&'static str>)>::iter(&store)
            .for_each(|result| println!("Result: {:?}", result));

        println!();
        println!("NoField<i32>:");
        StoreQuery::<(u32, NoField<i32>)>::iter(&store)
            .for_each(|result| println!("Result: {:?}", result));
    }
}
