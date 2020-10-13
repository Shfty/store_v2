use std::{collections::HashSet};

use crate::Storable;

/// Struct stored in a HashSet
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct HashSetStruct {}

impl Storable for HashSetStruct {
    type Storage = HashSet<Self>;
}
