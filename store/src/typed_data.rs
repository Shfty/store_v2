use std::{
    any::Any,
    fmt::{self, Debug},
    ops::Deref,
    ops::DerefMut,
};

/// Introspective polymorphic storage for Any types
pub struct TypedData {
    data: Box<dyn Any>,
    fmt: Box<dyn Fn(&TypedData, &mut fmt::Formatter) -> Result<(), fmt::Error>>,
}

impl TypedData {
    pub fn new<T>(data: T) -> Self
    where
        T: Debug + Any,
    {
        TypedData {
            data: Box::new(data),
            fmt: Box::new(|type_data, f| type_data.data.downcast_ref::<T>().unwrap().fmt(f)),
        }
    }

    pub fn downcast<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.data.downcast_ref::<T>()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.data.downcast_mut::<T>()
    }
}

impl<'a> Debug for TypedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.fmt)(self, f)
    }
}

impl Deref for TypedData {
    type Target = Box<dyn Any>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for TypedData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
