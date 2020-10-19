use std::{
    cell::{Ref, RefMut},
    hash::Hash,
    marker::PhantomData,
};

use super::{HybridStore, StoreTrait};

// Core Types
pub trait StoreQuery<'a, Signature> {
    type Key: Copy + Ord + Into<usize>;

    fn get(&'a self, key: Self::Key) -> Signature;
    fn iter(&'a self) -> StoreIterator<Self::Key, Signature>;
    fn iter_keys(&'a self, keys: &[Self::Key]) -> StoreIterator<Self::Key, Signature>;
}

pub struct StoreIterator<'a, Key, Signature>
where
    Key: Copy + Ord + Into<usize>,
{
    store: &'a HybridStore<Key>,
    keys: Vec<Key>,
    _phantom_data: PhantomData<Signature>,
}

use store_macros::impl_store_fields_iterator;

impl_store_fields_iterator!(1..6);

// Tests
#[cfg(test)]
mod tests {
    use crate::{StoreType, SomeData};

    use super::*;

    #[test]
    fn get() {
        let mut store = HybridStore::<usize>::default();

        store.register_type_storage::<bool>(StoreType::SparseVecMap);
        store.register_type_storage::<i32>(StoreType::DenseVecMap);
        store.register_type_storage::<&'static str>(StoreType::BTreeMap);
        store.register_type_storage::<SomeData<String>>(StoreType::HashMap);

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
                usize,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, 0);
            println!("0: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                usize,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, 1);
            println!("1: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                usize,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, 2);
            println!("2: {:?}", query);
        }

        {
            let query = StoreQuery::<(
                usize,
                Ref<bool>,
                Option<Ref<i32>>,
                RefMut<&'static str>,
                Option<RefMut<SomeData<String>>>,
            )>::get(&store, 3);
            println!("3: {:?}", query);
        }
    }

    #[test]
    fn iter() {
        let mut store = HybridStore::<usize>::default();

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
            usize,
            Ref<bool>,
            Option<Ref<i32>>,
            RefMut<&'static str>,
            Option<RefMut<SomeData<String>>>,
        )>::iter(&store)
        .for_each(|result| println!("Result: {:?}", result));

        println!();
        println!("Arity 2:");
        StoreQuery::<(usize, Ref<bool>, RefMut<&'static str>)>::iter(&store)
            .for_each(|result| println!("Result: {:?}", result));
    }
}
