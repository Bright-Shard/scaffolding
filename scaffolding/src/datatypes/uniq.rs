use {
    super::ArenaVec,
    core::{
        cell::{Cell, UnsafeCell},
        mem, slice,
    },
};

pub struct Uniq {
    data: ArenaVec<u8>,
    entries: UnsafeCell<ArenaVec<Option<UniqEntry>>>,
    used_entries: Cell<usize>,
}
impl Default for Uniq {
    fn default() -> Self {
        Self::with_capacity(4)
    }
}
impl Uniq {
    pub fn with_capacity(cap: usize) -> Self {
        let entries = ArenaVec::with_capacity(mem::size_of::<Option<UniqEntry>>() * cap);
        (0..cap).for_each(|_| {
            entries.push(None);
        });

        Self {
            data: ArenaVec::default(),
            entries: UnsafeCell::new(entries),
            used_entries: Cell::new(0),
        }
    }

    /// # Safety
    /// `key` must be unique.
    #[allow(clippy::mut_from_ref)]
    pub fn get<T>(&self, key: UniqKey, default: impl FnOnce() -> T) -> &mut T {
        let entries = unsafe { &mut *self.entries.get() };
        let raw_key = key.0;

        match self.idx_of(key) {
            UniqIndex::Exact(idx) => {
                let entry = entries[idx].as_ref().unwrap();
                unsafe { &mut *entry.val.cast() }
            }
            UniqIndex::Collision(last_idx) => {
                let entry_idx = self.next_idx();
                self.insert(entry_idx, raw_key, default());
                entries[last_idx].as_mut().unwrap().collision_slot = Some(entry_idx);

                let entry = entries[entry_idx].as_ref().unwrap();
                unsafe { &mut *entry.val.cast() }
            }
            UniqIndex::None(entry_idx) => {
                self.insert(entry_idx, raw_key, default());

                let entry = entries[entry_idx].as_ref().unwrap();
                unsafe { &mut *entry.val.cast() }
            }
        }
    }
    /// # Safety
    /// `key` must be unique.
    #[allow(clippy::mut_from_ref)]
    pub fn get_or_default<T: Default>(&self, key: UniqKey) -> &mut T {
        self.get(key, Default::default)
    }

    fn insert<T>(&self, idx: usize, key: usize, val: T) {
        let type_size = mem::size_of::<T>();

        let start_idx = self.data.len();
        let bytes = unsafe { slice::from_raw_parts(&val as *const T as *const u8, type_size) };
        self.data.extend_from_slice(bytes);

        let entries = unsafe { &mut *self.entries.get() };
        entries[idx] = Some(UniqEntry {
            key,
            val: &self.data[start_idx] as *const u8 as *mut u8,
            collision_slot: None,
        });
        self.used_entries.set(self.used_entries.get() + 1);
    }

    fn idx_of(&self, key: UniqKey) -> UniqIndex {
        let entries = unsafe { &mut *self.entries.get() };

        let mut idx = key.0 % entries.len();
        loop {
            let Some(entry) = entries[idx].as_mut() else {
                return UniqIndex::None(idx);
            };

            if entry.key == key.0 {
                return UniqIndex::Exact(idx);
            }

            let Some(new_idx) = entry.collision_slot else {
                return UniqIndex::Collision(idx);
            };
            idx = new_idx;
        }
    }
    fn next_idx(&self) -> usize {
        let entries = unsafe { &mut *self.entries.get() };

        if self.used_entries.get() == entries.len() {
            let cap = entries.len() * 2;
            let mut new_entries =
                ArenaVec::with_capacity(mem::size_of::<Option<UniqEntry>>() * cap);
            (0..cap).for_each(|_| {
                new_entries.push(None);
            });

            mem::swap(entries, &mut new_entries);

            new_entries.into_iter().flatten().for_each(|entry| {
                match self.idx_of(UniqKey(entry.key)) {
                    UniqIndex::None(idx) => {
                        entries[idx] = Some(entry);
                    }
                    UniqIndex::Collision(collision_idx) => {
                        let entry_idx = self.next_idx();
                        entries[entry_idx] = Some(entry);
                        entries[collision_idx].as_mut().unwrap().collision_slot = Some(entry_idx);
                    }
                    UniqIndex::Exact(_) => unreachable!(),
                }
            });
        }

        entries
            .iter()
            .enumerate()
            .find(|(_, val)| val.is_none())
            .unwrap()
            .0
    }
}

enum UniqIndex {
    Exact(usize),
    Collision(usize),
    None(usize),
}

#[repr(transparent)]
pub struct UniqKey(usize);
impl From<usize> for UniqKey {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl UniqKey {
    pub fn new(key: usize) -> Self {
        Self(key)
    }
}

#[derive(Copy, Clone)]
struct UniqEntry {
    key: usize,
    val: *mut u8,
    collision_slot: Option<usize>,
}

#[macro_export]
macro_rules! uniq_key {
    ($($hashable:tt),*) => {{
        let mut hasher = scaffolding::_ahash::AHasher::default();
        ::core::hash::Hash::hash(&column!(), &mut hasher);
        ::core::hash::Hash::hash(&line!(), &mut hasher);
        ::core::hash::Hash::hash(&file!(), &mut hasher);
        $(::core::hash::Hash::hash(&$hashable, &mut hasher);)*

        let result = ::core::hash::Hasher::finish(&hasher);
        scaffolding::datatypes::uniq::UniqKey::new((result % usize::MAX as u64) as usize)
    }};
}
pub use crate::uniq_key;
