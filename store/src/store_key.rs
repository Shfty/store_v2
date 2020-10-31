use std::{fmt::Debug, hash::Hash};

pub trait StoreKey: Debug + Copy + Ord + Hash + From<u32> + Into<u32> {}

impl<T> StoreKey for T where T: Debug + Copy + Ord + Hash + From<u32> + Into<u32> {}
