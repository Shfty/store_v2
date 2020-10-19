use std::{any::TypeId, fmt::Debug, sync::RwLock};

use fnv::FnvHashMap;
use std::ops::Deref;

use lazy_static::lazy_static;

lazy_static! {
    static ref SANITIZED_TYPE_NAMES: RwLock<FnvHashMap<TypeId, String>> =
        RwLock::new(FnvHashMap::default());
}

fn sanitize_type_name(string: &str) -> String {
    let before: &str;
    let after: Option<&str>;

    if let Some(open_bracket) = string.find('<') {
        let (split_before, split_after) = string.split_at(open_bracket);
        before = split_before;
        after = Some(split_after);
    } else {
        before = string;
        after = None;
    }

    let before = before.split("::").last().unwrap();
    if let Some(after) = after {
        before.to_string() + "<" + &sanitize_type_name(&after[1..after.len() - 1]) + ">"
    } else {
        before.into()
    }
}

/// Introspective type-backed key
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TypeKey(TypeId);

impl TypeKey {
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();

        let sanitized_type_name = sanitize_type_name(std::any::type_name::<T>());
        SANITIZED_TYPE_NAMES
            .write()
            .unwrap()
            .insert(type_id, sanitized_type_name);

        TypeKey(type_id)
    }

    pub fn get_name(&self) -> &str {
        let sanitized_type_names = SANITIZED_TYPE_NAMES.read().unwrap();
        let sanitized_type_name = sanitized_type_names.get(&self).unwrap().as_str();

        let name: &str;
        unsafe {
            let type_string = sanitized_type_name as *const str;
            name = &*type_string;
        }
        name
    }
}

impl Debug for TypeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_name())
    }
}

impl Deref for TypeKey {
    type Target = TypeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
