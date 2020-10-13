use crate::Storable;

/// Struct stored directly
#[derive(Debug, Default)]
pub struct SingletonStruct {}

impl Storable for SingletonStruct {
    type Storage = SingletonStruct;
}
