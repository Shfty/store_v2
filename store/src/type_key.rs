use std::{
    any::TypeId,
    fmt::Debug,
    hash::Hash,
    sync::RwLock,
};

use lazy_static::lazy_static;

use crate::HashMap;

lazy_static! {
    static ref SANITIZED_TYPE_NAMES: RwLock<HashMap<TypeId, String>> =
        RwLock::new(HashMap::default());
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
#[derive(Copy, Clone)]
pub struct TypeKey(TypeId, &'static str);

impl Debug for TypeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_name())
    }
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for TypeKey {}

impl PartialOrd for TypeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for TypeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for TypeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl TypeKey {
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        TypeKey(type_id, type_name)
    }

    pub fn get_name(&self) -> &str {
        let mut sanitized_type_names = SANITIZED_TYPE_NAMES.write().unwrap();

        let sanitized_type_name: &str = sanitized_type_names
            .entry(self.0)
            .or_insert_with(|| sanitize_type_name(self.1))
            .as_str();

        unsafe {
            let type_string = sanitized_type_name as *const str;
            &*type_string
        }
    }
}
