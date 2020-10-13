/// Trait for types that can be stored by StorageTrait
// (The T = Self parameter is a way to sidestep Self being unsized, need to research this)
pub trait Storable<T = Self>
where
    T: Storable,
{
    type Storage;
}
