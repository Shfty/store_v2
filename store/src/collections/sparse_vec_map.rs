use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit};

/// Vec-backed associative map. Keys directly index into the underlying Vec, empty indices are uninitialized memory.
pub struct SparseVecMap<K, V>
where
    K: Into<u32>,
{
    values: Vec<MaybeUninit<V>>,
    _phantom_data: PhantomData<K>,
}

impl<K, V> Debug for SparseVecMap<K, V>
where
    K: Debug + Into<u32>,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.values.iter().enumerate())
            .finish()
    }
}

impl<K, V> Default for SparseVecMap<K, V>
where
    K: Copy + Into<u32>,
{
    fn default() -> Self {
        SparseVecMap::new()
    }
}

// Public interface
impl<K, V> SparseVecMap<K, V>
where
    K: Copy + Into<u32>,
{
    pub fn new() -> Self {
        SparseVecMap {
            values: Vec::new(),
            _phantom_data: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SparseVecMap {
            values: Vec::with_capacity(capacity),
            _phantom_data: PhantomData,
        }
    }

    pub fn get(&self, key: &K) -> &V {
        let key: u32 = (*key).into();
        let value = &self.values[key as usize];
        let out: &V;
        unsafe {
            let value: *const V = value.as_ptr();
            out = &*value;
        }
        out
    }

    pub fn get_mut(&mut self, key: &K) -> &mut V {
        let key: u32 = (*key).into();
        let value = &mut self.values[key as usize];
        let out: &mut V;
        unsafe {
            let value: *mut V = value.as_mut_ptr();
            out = &mut *value;
        }
        out
    }

    pub fn insert(&mut self, key: K, value: V, existing: bool) {
        let key: u32 = key.into();
        let key: usize = key as usize;

        // If the new key is outside the array's length, resize to the correct length - 1
        if key > self.values.len() {
            self.values
                .resize_with(key, || unsafe { MaybeUninit::uninit().assume_init() });
        }

        // Push the new value onto the end of the array
        self.values.push(MaybeUninit::new(value));

        if key < self.values.len() - 1 {
            let value = self.values.swap_remove(key);
            if existing {
                unsafe {
                    value.assume_init();
                }
            }
        }
    }

    pub fn remove(&mut self, key: &K) {
        self.values
            .push(unsafe { MaybeUninit::uninit().assume_init() });

        let key: u32 = (*key).into();
        unsafe {
            self.values.swap_remove(key as usize).assume_init();
        }
    }

    pub fn clear(&mut self) {
        for value in self.values.drain(..) {
            unsafe {
                value.assume_init();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        println!();

        let mut vec_map: SparseVecMap<u32, i32> = SparseVecMap::new();

        vec_map.insert(0, 0, false);
        vec_map.insert(2, 1, false);
        vec_map.insert(4, 2, false);
        vec_map.insert(6, 3, false);
        vec_map.insert(8, 4, false);

        println!("{:#?}\n", vec_map);
    }

    #[test]
    fn remove() {
        println!();

        let mut vec_map: SparseVecMap<u32, i32> = SparseVecMap::new();

        vec_map.insert(0, 0, false);
        vec_map.insert(2, 1, false);
        println!("{:#?}\n", vec_map);

        vec_map.remove(&0);
        println!("{:#?}\n", vec_map);

        vec_map.remove(&1);
        println!("{:#?}\n", vec_map);

        vec_map.remove(&2);
        println!("{:#?}\n", vec_map);

        vec_map.remove(&3);
        println!("{:#?}\n", vec_map);
    }

    #[test]
    fn clear() {
        println!();

        let mut vec_map: SparseVecMap<u32, i32> = SparseVecMap::new();

        vec_map.insert(0, 0, false);
        vec_map.insert(1, 1, false);
        vec_map.insert(2, 2, false);
        vec_map.insert(3, 3, false);

        println!("{:#?}\n", vec_map);

        vec_map.clear();

        println!("{:#?}\n", vec_map);
    }

    #[test]
    fn get() {
        println!();

        let mut vec_map: SparseVecMap<u32, i32> = SparseVecMap::new();

        println!("{:#?}\n", vec_map);

        vec_map.insert(0, 1, false);
        vec_map.insert(1, 1, false);
        vec_map.insert(2, 1, false);
        vec_map.insert(3, 3, false);

        println!(
            "0: {:?}\n1: {:?}\n2: {:?}\n3: {:?}",
            vec_map.get(&0),
            vec_map.get(&1),
            vec_map.get(&2),
            vec_map.get(&3)
        );
    }

    #[test]
    fn get_mut() {
        println!();

        let mut vec_map: SparseVecMap<u32, i32> = SparseVecMap::new();

        println!("{:#?}\n", vec_map);

        vec_map.insert(0, 0, false);
        vec_map.insert(1, 1, false);
        vec_map.insert(2, 2, false);
        vec_map.insert(3, 3, false);

        println!("0: {:?}", vec_map.get_mut(&0));
        println!("1: {:?}", vec_map.get_mut(&1));
        println!("2: {:?}", vec_map.get_mut(&2));
        println!("3: {:?}", vec_map.get_mut(&3));
        println!();
    }
}
