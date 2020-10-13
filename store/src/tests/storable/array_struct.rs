use crate::Storable;

/// Struct stored in a Vec
#[derive(Debug, Default, Copy, Clone)]
pub struct ArrayStruct {}

impl Storable for ArrayStruct {
    type Storage = [Self; 10];
}
