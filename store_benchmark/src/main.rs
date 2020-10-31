use std::{
    cell::{Ref, RefMut},
    time::Duration,
};

use store::{Store, StoreQuery};

fn main() {
    let mut store = Store::<u32>::default();

    //store.register_storage_type_for::<String>(StorageType::BTreeMap);

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
    store.insert(1, "Matey".to_string());
    //store.insert(2, SomeData("Avast Ye".to_string()));
    store.insert(3, "Landlubbers".to_string());

    println!("\nStore: {:#?}\n", store);

    loop {
        main_inner(&store);
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main_inner(store: &Store<u32>) {
    StoreQuery::<(
        u32,
        Ref<bool>,
        Option<Ref<i32>>,
        RefMut<&str>,
        Option<RefMut<String>>,
    )>::iter(store)
    .for_each(|(key, bool, str, i32, some_data)| {
        println!(
            "Key: {:?}, bool: {:?}, str: {:?}, i32:{:?}, String: {:?}",
            key, bool, str, i32, some_data
        )
    });
}
