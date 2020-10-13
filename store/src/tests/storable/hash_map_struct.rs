use std::collections::HashMap;

use crate::Storable;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum HashMapKey {
    Foo,
    Bar,
    Baz,
}

/// Struct stored in a HashSet
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct HashMapStruct {}

impl Storable for HashMapStruct {
    type Storage = HashMap<HashMapKey, Self>;
}
