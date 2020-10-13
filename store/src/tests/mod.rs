mod storable;

use super::*;
use storable::*;

#[test]
pub fn store() {
    println!();

    let mut store = Store::default();

    // Data insertion
    {
        store.add_storage_for::<SingletonStruct>();
        *store.get_storage::<SingletonStruct>().borrow_mut() = SingletonStruct {};

        store.add_storage_for::<ArrayStruct>();
        *store.get_storage::<ArrayStruct>().borrow_mut() = [ArrayStruct {}; 10];

        store.add_storage_for::<VecStruct>();
        store
            .get_storage::<VecStruct>()
            .borrow_mut()
            .push(VecStruct {});

        store.add_storage_for::<HashSetStruct>();
        store
            .get_storage::<HashSetStruct>()
            .borrow_mut()
            .insert(HashSetStruct {});

        store.add_storage_for::<HashMapStruct>();
        store
            .get_storage::<HashMapStruct>()
            .borrow_mut()
            .insert(HashMapKey::Foo, HashMapStruct {});

        store
            .get_storage::<HashMapStruct>()
            .borrow_mut()
            .insert(HashMapKey::Bar, HashMapStruct {});
    }

    // Immutable data retrieval
    {
        let singleton_struct = store.get_storage::<SingletonStruct>().borrow();
        println!("Singleton Struct: {:?}", singleton_struct);

        let array_struct = store.get_storage::<ArrayStruct>().borrow();
        let array_struct = array_struct.get(0);
        println!("Array Struct: {:?}", array_struct);

        let vec_struct = store.get_storage::<VecStruct>().borrow();
        let vec_struct = vec_struct.get(0);
        println!("Vec Struct: {:?}", vec_struct);

        let hash_set_struct = store.get_storage::<HashSetStruct>().borrow();
        let hash_set_struct = hash_set_struct.get(&HashSetStruct {});
        println!("Hash Set Struct: {:?}", hash_set_struct);

        let hash_map_struct_foo = store.get_storage::<HashMapStruct>().borrow();
        let hash_map_struct_foo = hash_map_struct_foo.get(&HashMapKey::Foo);
        println!("Hash Map Struct (Foo): {:?}", hash_map_struct_foo);

        let hash_map_struct_bar = store.get_storage::<HashMapStruct>().borrow();
        let hash_map_struct_bar = hash_map_struct_bar.get(&HashMapKey::Bar);
        println!("Hash Map Struct (Bar): {:?}", hash_map_struct_bar);

        let hash_map_struct_baz = store.get_storage::<HashMapStruct>().borrow();
        let hash_map_struct_baz = hash_map_struct_baz.get(&HashMapKey::Baz);
        println!("Hash Map Struct (Baz): {:?}", hash_map_struct_baz);
    }

    // Mutable data retrieval
    {
        let singleton_struct = store.get_storage::<SingletonStruct>();
        println!("mut Singleton Struct: {:?}", singleton_struct);

        let mut vec_struct = store.get_storage::<VecStruct>().borrow_mut();
        let vec_struct = vec_struct.get_mut(0);
        println!("mut Vec Struct: {:?}", vec_struct);

        let mut array_struct = store.get_storage::<ArrayStruct>().borrow_mut();
        let array_struct = array_struct.get_mut(0);
        println!("mut Array Struct: {:?}", array_struct);

        let mut hash_map_struct = store.get_storage::<HashMapStruct>().borrow_mut();
        println!("mut HashMapStruct Struct (Foo): {:?}", hash_map_struct.get_mut(&HashMapKey::Foo));
        println!("mut HashMapStruct Struct (Bar): {:?}", hash_map_struct.get_mut(&HashMapKey::Bar));
        println!("mut HashMapStruct Struct (Baz): {:?}", hash_map_struct.get_mut(&HashMapKey::Baz));
    }

    // Data removal
    {
        store.get_storage::<VecStruct>().borrow_mut().remove(0);
        store
            .get_storage::<HashSetStruct>()
            .borrow_mut()
            .remove(&HashSetStruct {});
        store
            .get_storage::<HashMapStruct>()
            .borrow_mut()
            .remove(&HashMapKey::Foo);
        store
            .get_storage::<HashMapStruct>()
            .borrow_mut()
            .remove(&HashMapKey::Bar);
        store
            .get_storage::<HashMapStruct>()
            .borrow_mut()
            .remove(&HashMapKey::Baz);
    }
}
