use std::fmt::Debug;

use store_macros::{impl_assemble, impl_disassemble};

use crate::{HashMap, Store, StoreKey};

pub trait Assemble<Signature>
where
    Self: StoreKey,
{
    fn assemble(self, store: &mut Store<Self>, tuple: Signature) -> Self;
}

pub trait Disassemble<Key>
where
    Key: StoreKey,
{
    fn disassemble(component_store: &mut Store<Key>, key: &Key);
}

pub trait MapStoreBuilder<Key>: FnOnce(StoreBuilder<Key>) -> StoreBuilder<Key>
where
    Key: StoreKey + 'static,
{
}

impl<Key, T> MapStoreBuilder<Key> for T
where
    Key: StoreKey + 'static,
    T: FnOnce(StoreBuilder<Key>) -> StoreBuilder<Key>,
{
}

pub trait MapKeyBuilder<Key>: FnOnce(KeyBuilder<Key>) -> KeyBuilder<Key>
where
    Key: StoreKey + 'static,
{
}

impl<Key, T> MapKeyBuilder<Key> for T
where
    Key: StoreKey + 'static,
    T: FnOnce(KeyBuilder<Key>) -> KeyBuilder<Key>,
{
}

pub trait MapCurrentKey<Key>: FnOnce(StoreBuilder<Key>, Key) -> StoreBuilder<Key>
where
    Key: StoreKey + 'static,
{
}

impl<Key, T> MapCurrentKey<Key> for T
where
    Key: StoreKey + 'static,
    T: FnOnce(StoreBuilder<Key>, Key) -> StoreBuilder<Key>,
{
}

type KeyClosures<Key> = Vec<Box<dyn FnOnce(&mut Store<Key>)>>;

pub struct StoreBuilder<Key>
where
    Key: StoreKey + 'static,
{
    current_key: Option<Key>,
    closures: HashMap<Key, KeyClosures<Key>>,
}

impl<Key> Default for StoreBuilder<Key>
where
    Key: StoreKey + 'static,
{
    fn default() -> Self {
        StoreBuilder {
            current_key: None,
            closures: Default::default(),
        }
    }
}

impl<Key> StoreBuilder<Key>
where
    Key: StoreKey,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn current_key(&self) -> Key {
        self.current_key.expect("current_key() called before key()")
    }

    pub fn map<F>(self, f: F) -> Self
    where
        F: MapStoreBuilder<Key>,
    {
        f(self)
    }

    pub fn key(mut self, key: Key) -> KeyBuilder<Key> {
        self.current_key = Some(key);
        self.closures.entry(key).or_default();
        KeyBuilder::new(self)
    }

    pub fn map_key<F>(self, key: Key, f: F) -> Self
    where
        F: MapKeyBuilder<Key>,
    {
        f(self.key(key)).finish()
    }

    pub fn map_current_key<F>(self, f: F) -> Self
    where
        F: MapCurrentKey<Key>,
    {
        let current_key = self
            .current_key
            .expect("map_current_key() called before key()");
        f(self, current_key)
    }

    pub fn key_field<T>(self, key: Key, field: T) -> Self
    where
        T: Debug + 'static,
    {
        self.key(key).field(field).finish()
    }

    pub fn key_fields<T>(self, key: Key, tuple: T) -> Self
    where
        Key: Assemble<T>,
        T: Debug + 'static,
    {
        self.key(key).fields(tuple).finish()
    }

    pub fn finish(self, db: &mut Store<Key>) {
        let (_, closures): (Vec<Key>, Vec<KeyClosures<Key>>) = self.closures.into_iter().unzip();

        for closure in closures.into_iter().flatten() {
            closure(db);
        }
    }
}

pub struct KeyBuilder<Key>
where
    Key: StoreKey + 'static,
{
    key_assembler: StoreBuilder<Key>,
}

impl<Key> KeyBuilder<Key>
where
    Key: StoreKey,
{
    pub fn new(key_assembler: StoreBuilder<Key>) -> Self {
        KeyBuilder { key_assembler }
    }

    pub fn field<T>(mut self, field: T) -> Self
    where
        T: Debug + 'static,
    {
        let key = self
            .key_assembler
            .current_key
            .expect("field() called before key()");
        self.key_assembler
            .closures
            .get_mut(&key)
            .expect("Invalid key")
            .push(Box::new(move |db| db.insert(key, field)));

        self
    }

    pub fn fields<T>(mut self, tuple: T) -> Self
    where
        Key: Assemble<T>,
        T: Debug + 'static,
    {
        let key = self
            .key_assembler
            .current_key
            .expect("fields() called before key()");
        self.key_assembler
            .closures
            .get_mut(&key)
            .expect("Invalid key")
            .push(Box::new(move |db| {
                key.assemble(db, tuple);
            }));

        self
    }

    pub fn map<F>(self, f: F) -> Self
    where
        F: MapKeyBuilder<Key>,
    {
        f(self)
    }

    pub fn finish(self) -> StoreBuilder<Key> {
        self.key_assembler
    }
}

impl_assemble!(12);
impl_disassemble!(12);
