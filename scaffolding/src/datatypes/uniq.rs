use {
    super::ArenaVec,
    core::{
        cell::{Cell, UnsafeCell},
        mem, slice,
    },
};

/// A data structure for caching data and accessing that cache later.
///
/// A data structure for caching data with a [`UniqKey`]. This type has interior
/// mutability, so multiple cache entries from different [`UniqKey`]s can be
/// accessed at the same time.
///
///
/// # Examples
///
/// Functions can persist data across multiple function calls:
///
/// ```rs
/// use scaffolding::datatypes::{Uniq, uniq_key};
///
/// fn add_one(uniq: &Uniq) -> usize {
///     let cache: &mut usize = uniq.get_or_default(uniq_key!());
///     *cache += 1;
///
///     *cache
/// }
///
/// fn main() {
///     let uniq = Uniq::default();
///
///     assert_eq!(add_one(&uniq), 1);
///     assert_eq!(add_one(&uniq), 2);
///     assert_eq!(add_one(&uniq), 3);
/// }
/// ```
///
///
/// # Safety
///
/// This type works because [`UniqKey`]s are unique, opaque, and non-clone/copy.
/// It's therefore not possible to access the same entry from two different
/// places in code.
///
/// The only ways to break this guarantee are in unsafe Rust, by:
/// - Unsafely creating your own [`UniqKey`] that isn't unique or can be cloned
/// - Unsafely transmuting a [`UniqKey`]
/// - Using raw pointers to unsafely clone a [`UniqKey`]
///
/// See the [`uniq_key`] macro for an explanation on why it's safe.
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
    /// Create a [`Uniq`] that can store `cap` values.
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

    /// Get a cached value with a [`UniqKey`], or supply a default value if
    /// there isn't one cached.
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
    /// Get a cached value for a [`UniqKey`], or provide [`Default::default`]
    /// if there's no cached value.
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
unsafe impl Send for Uniq {}

/// A key for accessing a cached value from a [`Uniq`]. You can make a
/// [`UniqKey`] with the [`uniq_key`] macro.
///
/// [`UniqKey`]s **must** be unique, as that's the only reason [`Uniq`]s work.
/// To help accomplish this, [`UniqKey`] is an opaque, non-clone, non-copy
/// type.
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct UniqKey(usize);
impl UniqKey {
    /// Create a new [`UniqKey`]. It's recommended that you use the [`uniq_key`]
    /// macro to create a [`UniqKey`], instead of calling this function.
    ///
    /// # Safety
    /// `key` must be unique.
    pub unsafe fn new(key: usize) -> Self {
        Self(key)
    }
}

enum UniqIndex {
    Exact(usize),
    Collision(usize),
    None(usize),
}

#[derive(Copy, Clone)]
struct UniqEntry {
    key: usize,
    val: *mut u8,
    collision_slot: Option<usize>,
}

/// Generates a [`UniqKey`] based on the column, line, and file where the macro
/// was invoked.
///
/// Because of how the macro generates a key, `uniq_key!() != uniq_key!()`. The
/// first invocation of the macro will always be on a different line, column, or
/// file than the second invocation.
///
/// That being said, the macro is not flawless. This, for example, can lead to
/// unsafe behaviour:
/// ```rs
/// fn break_it() -> UniqKey {
///     uniq_key!()
/// }
/// ```
/// ...because it's possible to obtain the same key twice like this:
/// ```rs
/// let key1 = break_it();
/// let key2 = break_it();
/// ```
///
/// If you use this macro, *only* use it when creating variables, and don't
/// return it from a function.
#[macro_export]
macro_rules! uniq_key {
    ($($hashable:tt),*) => {{
        let mut hasher = scaffolding::_hash::Hasher::default();
        ::core::hash::Hash::hash(&column!(), &mut hasher);
        ::core::hash::Hash::hash(&line!(), &mut hasher);
        ::core::hash::Hash::hash(&file!(), &mut hasher);
        $(::core::hash::Hash::hash(&$hashable, &mut hasher);)*

        let result = ::core::hash::Hasher::finish(&hasher);

        unsafe {
            scaffolding::datatypes::uniq::UniqKey::new(
                (result % usize::MAX as u64) as usize
            )
        }
    }};
}
pub use crate::uniq_key;

#[cfg(test)]
mod tests {
    use {
        super::Uniq,
        crate::{self as scaffolding, datatypes::uniq::UniqKey},
        core::hash::Hash,
    };

    #[allow(dead_code)]
    fn is_sync<T: Sync>(_t: T) {}
    #[allow(dead_code)]
    fn is_send<T: Send>(_t: T) {}

    /// Adds 1 to a cached value in the given `Uniq`, then asserts the number is
    /// the same as the expected one.
    fn add_one_and_check(uniq: &mut Uniq, expected: usize) {
        let val: &mut usize = uniq.get_or_default(uniq_key!());
        *val += 1;

        assert_eq!(*val, expected);
    }

    #[test]
    fn types_test() {
        // Uniq should be Send but not Sync
        is_send(Uniq::default());
        // is_sync(Uniq::default());
    }

    #[test]
    fn test_in_loop() {
        let mut uniq = Uniq::default();

        for i in 1..10 {
            add_one_and_check(&mut uniq, i);
        }
        for i in (10..20).step_by(2) {
            add_one_and_check(&mut uniq, i);
            add_one_and_check(&mut uniq, i + 1);
        }
    }

    #[test]
    fn macro_tests() {
        assert_ne!(uniq_key!(), uniq_key!());

        fn with_hash<H: Hash>(h: H) -> UniqKey {
            uniq_key!(h)
        }
        assert_ne!(with_hash("h"), with_hash(1));

        // TODO: Try to find some way to prevent this...
        fn breaks() -> UniqKey {
            uniq_key!()
        }
        assert_eq!(breaks(), breaks());
    }
}
