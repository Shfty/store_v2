use fnv::FnvHasher;
use std::hash::BuildHasherDefault;

pub type HashSet<K> = std::collections::HashSet<K, BuildHasherDefault<FnvHasher>>;
