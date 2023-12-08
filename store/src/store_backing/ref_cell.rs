use hibitset::BitSet;

use crate::StorageTrait;
use std::{cell::Ref, cell::RefCell, cell::RefMut, fmt::Debug};

#[derive(Default, Clone, Eq, PartialEq)]
pub struct StoreBackingRefCell<Storage>
where
    Storage: StorageTrait,
{
    pub keys: RefCell<BitSet>,
    pub values: RefCell<Storage>,
}

impl<Storage> StoreBackingRefCell<Storage>
where
    Storage: StorageTrait,
{
    pub fn get(
        &self,
        key: &<Storage as StorageTrait>::Key,
    ) -> Option<Ref<<Storage as StorageTrait>::Value>> {
        let u32_key: u32 = (*key).into();
        if self.keys.borrow().contains(u32_key) {
            Some(Ref::map(self.values.borrow(), |values| {
                values.get(key).unwrap()
            }))
        } else {
            None
        }
    }

    pub fn get_mut(
        &self,
        key: &<Storage as StorageTrait>::Key,
    ) -> Option<RefMut<<Storage as StorageTrait>::Value>> {
        let u32_key: u32 = (*key).into();
        if self.keys.borrow_mut().contains(u32_key) {
            Some(RefMut::map(self.values.borrow_mut(), |values| {
                values.get_mut(key).unwrap()
            }))
        } else {
            None
        }
    }

    pub fn insert(
        &self,
        key: <Storage as StorageTrait>::Key,
        value: <Storage as StorageTrait>::Value,
    ) {
        let u32_key: u32 = key.into();
        self.keys.borrow_mut().add(u32_key);
        self.values.borrow_mut().insert(key, value);
    }

    pub fn remove(&self, key: &<Storage as StorageTrait>::Key) {
        let u32_key: u32 = (*key).into();
        self.keys.borrow_mut().remove(u32_key);
        self.values.borrow_mut().remove(&key);
    }

    pub fn clear(&self) {
        self.keys.borrow_mut().clear();
        self.values.borrow_mut().clear();
    }

    pub fn contains(&self, key: &<Storage as StorageTrait>::Key) -> bool {
        let u32_key: u32 = (*key).into();
        self.keys.borrow().contains(u32_key)
    }

    pub fn keys(&self) -> BitSet {
        self.keys.borrow().clone()
    }
}

impl<Storage> Debug for StoreBackingRefCell<Storage>
where
    Storage: StorageTrait,
    <Storage as StorageTrait>::Value: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.keys.borrow().clone().into_iter().map(|key| {
                (
                    key,
                    Ref::map(self.values.borrow(), |values| {
                        values.get(&key.into()).unwrap()
                    }),
                )
            }))
            .finish()
    }
}
