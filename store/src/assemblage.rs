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

pub trait MapAssembler<Key>: FnOnce(Assembler<Key>) -> Assembler<Key>
where
    Key: StoreKey + 'static,
{
}

impl<Key, T> MapAssembler<Key> for T
where
    Key: StoreKey + 'static,
    T: FnOnce(Assembler<Key>) -> Assembler<Key>,
{
}

type KeyClosures<Key> = Vec<Box<dyn FnOnce(&mut Store<Key>)>>;

pub struct Assembler<Key>
where
    Key: StoreKey + 'static,
{
    current_key: Option<Key>,
    closures: HashMap<Key, KeyClosures<Key>>,
}

impl<Key> Default for Assembler<Key>
where
    Key: StoreKey + 'static,
{
    fn default() -> Self {
        Assembler {
            current_key: None,
            closures: Default::default(),
        }
    }
}

impl<Key> Assembler<Key>
where
    Key: StoreKey,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn current_key(&self) -> Key {
        self.current_key.expect("current_key() called before key()")
    }

    pub fn key(mut self, key: Key) -> Self {
        self.current_key = Some(key);
        self.closures.entry(key).or_default();
        self
    }

    pub fn field<T>(mut self, field: T) -> Self
    where
        T: Debug + 'static,
    {
        let key = self.current_key.expect("field() called before key()");
        self.closures
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
        let key = self.current_key.expect("fields() called before key()");
        self.closures
            .get_mut(&key)
            .expect("Invalid key")
            .push(Box::new(move |db| {
                key.assemble(db, tuple);
            }));

        self
    }

    pub fn assemble<F>(self, f: F) -> Self
    where
        F: MapAssembler<Key>,
    {
        f(self)
    }

    pub fn finish(self, db: &mut Store<Key>) {
        let (_, closures): (Vec<Key>, Vec<KeyClosures<Key>>) = self.closures.into_iter().unzip();

        for closure in closures.into_iter().flatten() {
            closure(db);
        }
    }
}

impl_assemble!(12);
impl_disassemble!(12);
