//! Module for [`TypeMap`].

use {
    alloc::{
        alloc::{alloc, Layout},
        boxed::Box,
        vec::Vec,
    },
    core::{
        any::{Any, TypeId},
        mem,
        ptr::{self, NonNull},
        slice,
    },
};

/// Stores a single instance for some number of types. This acts like a
/// [`std::collections::HashMap`], except the keys are types and the values are
/// instances of those types. This type uses [`TypeId`]s, which are already
/// type hashes, so it doesn't perform any hashing itself.
///
/// Note that types can't be removed from a [`TypeMap`] after they're inserted.
/// This implementation allows the typemap to use an arena allocator internally,
/// which leads to more optimised code because the arena gives us memory
/// locality and a dead-simple allocator.
///
/// # Niche Behavior
/// - Creating a 0-capacity type map doesn't allocate anything.
/// - Inserting the same type twice will overwrite the old type.
/// - Typemaps will automatically reallocate with twice as many entries and
///   twice as much storage whenever [`TypeMap::insert`] is called and the
///   typemap is full.
pub struct TypeMap {
    /// A list of [`TypeMapEntry`]s, for every type that's been inserted into
    /// the [`TypeMap`].
    entries: Box<[Option<TypeMapEntry>]>,
    /// The buffer used to store all the objects in the [`TypeMap`].
    storage: Box<[u8]>,
    /// How much of the [`TypeMap`]'s storage has been used, in bytes.
    used_storage: usize,
    /// How many entries have been inserted into the [`TypeMap`].
    num_entries: usize,
}
impl Default for TypeMap {
    #[inline(always)]
    fn default() -> Self {
        Self::new(0, 0)
    }
}
impl TypeMap {
    pub fn new(num_entries: usize, storage_capacity: usize) -> Self {
        let entries = {
            let allocation: *mut Option<TypeMapEntry> = if num_entries > 0 {
                let allocation =
                    unsafe { alloc(Layout::array::<Option<TypeMapEntry>>(num_entries).unwrap()) };
                allocation.cast()
            } else {
                NonNull::dangling().as_ptr()
            };

            let entries = unsafe { slice::from_raw_parts_mut(allocation, num_entries) };
            for entry in entries.iter_mut() {
                *entry = None;
            }

            entries
        };

        let ptr = if storage_capacity > 0 {
            unsafe { alloc(Layout::array::<u8>(storage_capacity).unwrap()) }
        } else {
            NonNull::dangling().as_ptr()
        };
        let storage = unsafe { slice::from_raw_parts_mut(ptr, storage_capacity) };

        Self {
            entries: unsafe { Box::from_raw(entries as _) },
            storage: unsafe { Box::from_raw(storage) },
            used_storage: 0,
            num_entries: 0,
        }
    }

    pub fn resize(&mut self, new_entry_capacity: usize, new_storage_capacity: usize) {
        if new_entry_capacity < self.num_entries || new_storage_capacity < self.used_storage {
            panic!("TypeMap error: Called resize with new sizes that are too small to hold the current typemap data");
        }

        let entries =
            unsafe { alloc(Layout::array::<Option<TypeMapEntry>>(new_entry_capacity).unwrap()) };
        let storage = unsafe { alloc(Layout::array::<u8>(new_storage_capacity).unwrap()) };

        unsafe { storage.copy_from(self.storage.as_ptr(), self.used_storage) };

        let entries = unsafe { slice::from_raw_parts_mut(entries.cast(), new_entry_capacity) };
        for entry in entries.iter_mut() {
            *entry = None;
        }

        let mut entries = unsafe { Box::from_raw(entries as *mut [Option<TypeMapEntry>]) };
        let mut storage =
            unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(storage, new_storage_capacity)) };

        mem::swap(&mut self.entries, &mut entries);
        mem::swap(&mut self.storage, &mut storage);

        // `entries` and `storage` now contain the old entries because of the
        // swap above
        let old_storage_address = storage.as_ptr() as isize;
        let new_storage_address = self.storage.as_ptr() as isize;
        // Technically could overflow if the two addresses were more than half
        // of the virtual address space apart, but this is impossible since
        // pointers don't use all 64 bits
        // TODO: Use `Box::into_iter` when it's added to stable... currently
        // it's only in nightly
        for mut entry in Vec::from(entries).into_iter().flatten() {
            entry.ptr =
                (new_storage_address + (entry.ptr as isize - old_storage_address)) as *mut u8;
            self.copy_entry(entry);
        }
    }

    pub fn contains<T: Any>(&self) -> bool {
        let type_id = PubTypeId::of::<T>();
        let idx = type_id.val.0 as usize % self.entries.len();

        unsafe { self.entries.get_unchecked(idx).is_some() }
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        match self._get(PubTypeId::of::<T>()) {
            Some(ptr) => unsafe { Some(&*ptr.cast()) },
            None => None,
        }
    }
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        match self._get(PubTypeId::of::<T>()) {
            Some(ptr) => unsafe { Some(&mut *ptr.cast()) },
            None => None,
        }
    }
    pub fn get_raw(&self, type_id: PubTypeId) -> Option<NonNull<()>> {
        self._get(type_id)
            .map(|ptr| unsafe { NonNull::new_unchecked(ptr.cast()) })
    }

    #[inline(always)]
    pub const fn num_entries(&self) -> usize {
        self.num_entries
    }
    #[inline(always)]
    pub const fn available_entries(&self) -> usize {
        self.entries.len()
    }
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.num_entries == 0
    }
    #[inline(always)]
    pub const fn used_storage(&self) -> usize {
        self.used_storage
    }
    #[inline(always)]
    pub const fn storage_capacity(&self) -> usize {
        self.storage.len()
    }
    #[inline(always)]
    pub const fn unused_storage(&self) -> usize {
        self.storage.len() - self.used_storage
    }
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.num_entries == self.entries.len() || self.used_storage == self.storage.len()
    }

    pub fn insert<T: Any>(&mut self, val: T) {
        let type_size = mem::size_of::<T>();

        if self.is_full() || self.unused_storage() < type_size {
            let new_entry_capacity = if self.num_entries() == 0 {
                1
            } else {
                self.num_entries() * 2
            };
            let new_storage_capacity = if self.storage_capacity() == 0 {
                1
            } else {
                self.storage_capacity() * 2
            };

            self.resize(new_entry_capacity, new_storage_capacity);
            self.insert(val);
            return;
        }

        let type_id = PubTypeId::of::<T>();
        let idx = type_id.val.0 as usize % self.entries.len();

        // SAFETY: The idx is the typeid % self.entries.len(), so we know it's in-bounds
        let existing_entry = unsafe { self.entries.get_unchecked_mut(idx) };

        match existing_entry {
            Some(ref mut entry) => {
                if entry.type_id == type_id {
                    // Type was inserted twice - overwrite the old value
                    let ptr: *mut T = entry.ptr.cast();
                    unsafe { ptr::drop_in_place(ptr) };
                    unsafe { ptr.write(val) };
                } else {
                    // Collision - put the value in a different slot and set the `collision_slot` field
                    // Find an empty slot to use
                    let mut collision_idx = usize::MAX;
                    for (idx, entry) in self.entries.iter().enumerate() {
                        if entry.is_none() {
                            collision_idx = idx;
                            break;
                        }
                    }
                    debug_assert_ne!(collision_idx, usize::MAX);

                    // Set the `collision_slot` field
                    let mut last_linked_list_node =
                        unsafe { self.entries.get_unchecked_mut(idx).as_mut().unwrap() };
                    while let Some(idx) = last_linked_list_node.collision_slot {
                        last_linked_list_node =
                            unsafe { self.entries.get_unchecked_mut(idx).as_mut().unwrap() };
                    }
                    last_linked_list_node.collision_slot = Some(collision_idx);

                    // Insert our new entry
                    self.align::<T>();
                    let ptr = unsafe {
                        self.storage.get_unchecked_mut(self.used_storage) as *mut u8 as *mut T
                    };
                    unsafe { ptr.write(val) };
                    let entry = unsafe { self.entries.get_unchecked_mut(collision_idx) };
                    *entry = Some(TypeMapEntry {
                        type_id,
                        ptr: ptr.cast(),
                        drop: |val| {
                            let ptr: *mut T = val.cast();
                            drop(unsafe { ptr.read() });
                        },
                        collision_slot: None,
                    });

                    self.num_entries += 1;
                    self.used_storage += type_size;
                }
            }
            None => {
                // No collision - we can just insert the value
                self.align::<T>();
                let ptr = unsafe {
                    self.storage.get_unchecked_mut(self.used_storage) as *mut u8 as *mut T
                };
                unsafe { ptr.write(val) };
                let entry = unsafe { self.entries.get_unchecked_mut(idx) };
                *entry = Some(TypeMapEntry {
                    type_id,
                    ptr: ptr.cast(),
                    drop: |val| {
                        let ptr: *mut T = val.cast();
                        drop(unsafe { ptr.read() });
                    },
                    collision_slot: None,
                });

                self.num_entries += 1;
                self.used_storage += type_size;
            }
        }
    }
    /// Removes all entries from the typemap. This doesn't remove its allocation.
    pub fn clear(&mut self) {
        self.num_entries = 0;
        self.used_storage = 0;
    }
    /// Changes `self.used_storage` to be aligned to `T`.
    #[inline(always)]
    fn align<T>(&mut self) {
        let align = mem::align_of::<T>();
        self.used_storage = self.used_storage + align - 1;
        self.used_storage -= self.used_storage % align;
    }

    /// Copies an entry from another typemap. This doesn't add the entry's value to `storage`, or increment
    /// `num_entries`/`used_storage` - that must be done separately.
    fn copy_entry(&mut self, mut entry: TypeMapEntry) {
        entry.collision_slot = None;

        let idx = entry.type_id.val.0 as usize % self.entries.len();
        // SAFETY: The idx is the typeid % self.entries.len(), so we know it's in-bounds
        let existing_entry = unsafe { self.entries.get_unchecked_mut(idx) };

        match existing_entry {
            Some(_) => {
                // Collision - put the value in a different slot and set the `collision_slot` field
                // Find an empty slot to use
                let mut collision_idx = usize::MAX;
                for (idx, entry) in self.entries.iter().enumerate() {
                    if entry.is_none() {
                        collision_idx = idx;
                        break;
                    }
                }
                debug_assert_ne!(collision_idx, usize::MAX);

                // Set the `collision_slot` field
                let mut last_linked_list_node =
                    unsafe { self.entries.get_unchecked_mut(idx).as_mut().unwrap() };
                while let Some(idx) = last_linked_list_node.collision_slot {
                    last_linked_list_node =
                        unsafe { self.entries.get_unchecked_mut(idx).as_mut().unwrap() };
                }
                last_linked_list_node.collision_slot = Some(collision_idx);

                // Insert our new entry
                let new_entry = unsafe { self.entries.get_unchecked_mut(collision_idx) };
                *new_entry = Some(entry);
            }
            None => {
                // No collision - we can insert normally
                *existing_entry = Some(entry);
            }
        }
    }

    fn _get(&self, type_id: PubTypeId) -> Option<*mut u8> {
        let idx = type_id.val.0 as usize % self.entries.len();
        let entry = unsafe { self.entries.get_unchecked(idx).as_ref() };

        match entry {
            Some(entry) => {
                // Check for collision
                if entry.type_id == type_id {
                    Some(entry.ptr)
                } else {
                    let mut last_linked_list_node = entry;
                    while let Some(idx) = last_linked_list_node.collision_slot {
                        last_linked_list_node =
                            unsafe { self.entries.get_unchecked(idx).as_ref().unwrap() };
                        if last_linked_list_node.type_id == type_id {
                            break;
                        }
                    }
                    if last_linked_list_node.type_id != type_id {
                        return None;
                    }

                    Some(last_linked_list_node.ptr)
                }
            }
            None => None,
        }
    }
}
impl Drop for TypeMap {
    fn drop(&mut self) {
        for entry in self.entries.iter_mut().filter_map(|val| val.as_mut()) {
            (entry.drop)(entry.ptr.cast())
        }
    }
}

/// Identical to [`TypeId`], except its value is public. Because it stores the same data, this
/// type can be safely transmuted to/from a regular [`TypeId`], allowing access to its raw value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PubTypeId {
    pub val: (u64, u64),
}
impl PubTypeId {
    pub fn of<T: Any>() -> Self {
        unsafe { mem::transmute(TypeId::of::<T>()) }
    }
}
impl From<TypeId> for PubTypeId {
    fn from(value: TypeId) -> Self {
        unsafe { mem::transmute(value) }
    }
}

/// Represents one entry in a [`TypeMap`].
pub struct TypeMapEntry {
    /// The raw type ID of the type this entry stores. Used to check for
    /// collisions.
    type_id: PubTypeId,
    /// A pointer to the type's instance in memory.
    ptr: *mut u8,
    /// The destructor for this type.
    drop: fn(*mut ()),
    /// If there was a collision, this stores the index of the colliding typemap entry.
    collision_slot: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct SomeType {
        text: String,
        num: u32,
    }
    #[derive(Debug)]
    #[allow(dead_code)]
    struct SomeOtherType {
        val: i32,
    }
    #[derive(Debug, PartialEq, Eq)]
    #[allow(dead_code)]
    enum SomeEnum {
        Variant,
        Idk,
    }

    #[test]
    fn good_typemap() {
        let mut store = TypeMap::new(3, 100);

        store.insert(SomeType {
            text: "Hello!".to_string(),
            num: 42,
        });
        store.insert(SomeOtherType { val: 69 });
        store.insert(SomeEnum::Variant);

        let some_enum_val = store.get::<SomeEnum>().unwrap();
        let some_other_type_val = store.get::<SomeOtherType>().unwrap();
        let some_type_val = store.get::<SomeType>().unwrap();

        assert_eq!(some_type_val.text.as_str(), "Hello!");
        assert_eq!(some_type_val.num, 42);
        assert_eq!(some_other_type_val.val, 69);
        assert_eq!(*some_enum_val, SomeEnum::Variant);

        println!("SomeType: {some_type_val:?} // SomeOtherType: {some_other_type_val:?} // SomeEnum: {some_enum_val:?}");
        println!(
            "Final num entries: {} // Final used storage: {}",
            store.num_entries(),
            store.used_storage()
        );
    }

    #[test]
    #[should_panic]
    fn bad_typemap() {
        let mut store = TypeMap::new(2, 100);
        store.insert(SomeType {
            text: "Hello!".to_string(),
            num: 42,
        });
        store.insert(SomeOtherType { val: 69 });

        let some_type_val = store.get::<SomeType>().unwrap();
        let some_other_type_val = store.get::<SomeOtherType>().unwrap();
        let some_enum_val = store.get::<SomeEnum>().unwrap();
        println!("SomeType: {some_type_val:?} // SomeOtherType: {some_other_type_val:?} // SomeEnum: {some_enum_val:?}");
        println!(
            "Final num entries: {} // Final used storage: {}",
            store.num_entries(),
            store.used_storage()
        );
    }

    #[test]
    fn realloc() {
        let mut store = TypeMap::new(2, 100);
        store.insert(SomeType {
            text: "Hello!".to_string(),
            num: 42,
        });
        store.insert(SomeOtherType { val: 69 });

        assert_eq!(store.available_entries(), 2);
        assert_eq!(store.storage_capacity(), 100);

        store.insert(SomeEnum::Variant);

        assert_eq!(store.available_entries(), 4);
        assert_eq!(store.storage_capacity(), 200);
    }
}
