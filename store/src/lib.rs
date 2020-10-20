mod store_data;
mod store_trait;
mod store_query;
mod type_key;
mod typed_data;
mod hash_map;

pub use store_data::*;
pub use store_trait::*;
pub use store_query::*;
pub use type_key::*;
pub use typed_data::*;
pub use hash_map::*;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SomeData<T>(pub T);

#[cfg(test)]
pub mod tests {
    use std::fmt::Debug;

    use super::*;

    fn test_insert<'a, T>(store: &mut T)
    where
        T: Debug + Default + StoreTrait<'a, Key = usize>,
    {
        store.insert(5, "Goodbye");
        store.insert(2, "World");
        store.insert(0, "Hello");

        store.insert(5, 128);
        store.insert(2, 64);
        store.insert(0, 32);

        store.insert(5, true);
        store.insert(2, false);
        store.insert(0, true);

        store.insert(5, SomeData("Foo".to_string()));
        store.insert(2, SomeData("Bar".to_string()));
        store.insert(0, SomeData("Baz".to_string()));

        println!("{:#?}\n", store);
    }

    fn test_remove<'a, T>(store: &mut T)
    where
        T: Debug + Default + StoreTrait<'a, Key = usize>,
    {
        store.remove::<bool>(&0usize);
        store.remove::<i32>(&0usize);
        store.remove::<&str>(&0usize);
        store.remove::<SomeData<String>>(&0usize);

        store.remove::<bool>(&1usize);
        store.remove::<i32>(&1usize);
        store.remove::<&str>(&1usize);
        store.remove::<SomeData<String>>(&1usize);

        println!("{:#?}\n", store);
    }

    fn test_remove_key<'a, T>(store: &mut T)
    where
        T: Debug + Default + StoreTrait<'a, Key = usize>,
    {
        store.remove_key(&2usize);
        println!("{:#?}\n", store);
    }

    fn test_clear<'a, T>(store: &mut T)
    where
        T: Debug + Default + StoreTrait<'a, Key = usize>,
    {
        store.clear();
        println!("{:#?}\n\n", store);
    }

    fn test_store<'a, T>(store: &mut T)
    where
        T: Debug + Default + StoreTrait<'a, Key = usize>,
    {
        println!("{:#?}\n", store);

        test_insert(store);
        test_remove(store);
        test_remove_key(store);
        test_clear(store);
    }

    #[test]
    pub fn hash_map_store() {
        let mut store = HashMapStore::<usize>::default();
        test_store(&mut store);
    }

    #[test]
    pub fn btree_map_store() {
        let mut store = BTreeMapStore::<usize>::default();
        test_store(&mut store);
    }

    #[test]
    pub fn sparse_vec_map_store() {
        let mut store = SparseVecMapStore::<usize>::default();
        test_store(&mut store);
    }

    #[test]
    pub fn dense_vec_map_store() {
        let mut store = DenseVecMapStore::<usize>::default();
        test_store(&mut store);
    }

    #[test]
    pub fn hybrid_store() {
        let mut store = HybridStore::<usize>::default();
        store.register_type_storage::<bool>(StoreType::SparseVecMap);
        store.register_type_storage::<i32>(StoreType::BTreeMap);
        store.register_type_storage::<&str>(StoreType::DenseVecMap);
        store.register_type_storage::<SomeData<String>>(StoreType::HashMap);
        test_store(&mut store);
    }
}
