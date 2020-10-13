use crate::Storable;

/// Struct stored in a Vec
#[derive(Debug)]
pub struct VecStruct {}

impl Storable for VecStruct {
    type Storage = Vec<Self>;
}
