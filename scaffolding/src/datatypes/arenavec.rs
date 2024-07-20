//! Module for [`ArenaVec`].

use {
    crate::{
        os::{Os, OsMetadata, OsTrait},
        utils::{self, MemoryAmount},
    },
    core::{
        mem,
        ptr::{self, addr_of, NonNull},
    },
};

/// A vector backed by an arena allocator. Arenavecs never reallocate, meaning pushing to an
/// arenavec is guaranteed to never move its items in memory. This unique property allows an
/// arenavec to safely be pushed to from an immutable reference - that is, [`ArenaVec::push`]
/// takes `&self`, not `&mut self`.
///
/// # How it Works
/// Disclaimer: The following information is just implementation details. It's not needed unless
/// you want to make your own arenavec or are just curious about how it works.
///
/// Arenavecs rely on the virtual address space to work correctly (if you aren't familiar with
/// virtual addresses, you will need to read up on that before understanding arenavecs). When
/// an arenavec is created, it will reserve a large portion of the virtual address space to be
/// allocated later. If you create a default arenavec, it will reserve [`ArenaVec::DEFAULT_RESERVED_MEMORY`]
/// bytes, or you can reserve a specific amount of memory with [`ArenaVec::with_reserved_memory`].
/// This memory is not allocated - that is, it's not backed by actual RAM. It's just a bunch
/// of virtual addresses that the arenavec has reserved for its own use, and will be able to
/// allocate later.
///
/// As vectors grow, they will need more and more memory to store their items. Normal vectors
/// solve this by reallocating after they fill up with items. Arenavecs, however, can make use
/// of their reserved addresses; instead of reallocating the entire vector, they can just allocate
/// memory at one of their reserved addresses. The effect is that the arenavec can grow in-place
/// until it runs out of reserved memory addresses.
///
/// This is the difference between a [`Vec`] and an [`ArenaVec`]: When a vec is pushed to, it might
/// overflow and have to reallocate, which would change its location in memory. Reallocation would
/// also involve copying all of the vec's old contents into the new allocation. Reallocation would
/// also invalidate all pointers to the vec, because its memory location has changed. To stay
/// memory-safe, vecs have to take an `&mut self` when being pushed to, just in case they reallocate.
///
/// Arenavecs will never reallocate. Their contents never have to be copied to a new buffer, and
/// pointers to arenavecs are never invalidated (until they're dropped). Thus, arenavecs don't need
/// `&mut self` when being pushed to - they can just continue to grow in-place.
///
/// The obvious tradeoff to this system is that if the arenavec runs out of reserved addresses, and
/// needs to grow, it won't be able to and will have to panic. However, reserving a large amount
/// of the virtual address space has little (if any) overhead, so it's fairly easy to just reserve
/// an unreasonable amount of addresses and then leave it alone.
pub struct ArenaVec<T> {
    /// The total amount of memory the arenavec reserved when it was created.
    reserved_memory: usize,
    /// The amount of memory the arenavec has allocated and can be used to store data.
    capacity: usize,
    /// The number of entries in the arenavec.
    len: usize,
    /// A pointer to the base of the memory buffer storing all the arenavec's items.
    buffer: NonNull<T>,
}
impl<T> ArenaVec<T> {
    /// This is the default amount of memory an arenavec will reserve when it's
    /// created. I'm not sure what a reasonable number for this is.
    ///
    /// Modern x86-64 systems have a 48-bit address space, which can address up
    /// to `281,474,976,710,656` bytes (or `262,144` GiB). Most computers also
    /// probably have between 8 and 32 GiB of RAM. Reserving 10GiB of memory
    /// would use just over .003% of the virtual address space, and will
    /// realistically probably never be filled (if *one* buffer is using
    /// 10GiB of memory, your program probably has other issues).
    ///
    /// Thus 10GiB seems like a buffer size that will probably never be filled
    /// and also doesn't take a large portion of the virtual address space
    /// (thousands of arenavecs could still be created without filling it).
    pub const DEFAULT_RESERVED_MEMORY: usize = MemoryAmount::Gibibytes(10).into_bytes();

    /// Creates a default [`ArenaVec`]. This will reserve virtual addresses, but does not allocate.
    pub fn new() -> Self {
        Self::with_reserved_memory(Self::DEFAULT_RESERVED_MEMORY)
    }

    /// Create an [`ArenaVec`] with the specified amount of reserved virtual addresses. This does not allocate.
    pub fn with_reserved_memory(reserved_memory: usize) -> Self {
        Self::with_reserved_memory_and_capacity(reserved_memory, 0)
    }

    /// Create an [`ArenaVec`] with the specified allocation size. This will either reserve the default amount of reserved
    /// memory (see [`ArenaVec::DEFAULT_RESERVED_MEMORY`]) or the allocation size, whichever is larger.
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > Self::DEFAULT_RESERVED_MEMORY {
            Self::with_reserved_memory_and_capacity(capacity, capacity)
        } else {
            Self::with_reserved_memory_and_capacity(Self::DEFAULT_RESERVED_MEMORY, capacity)
        }
    }

    /// Create an [`ArenaVec`] with the specified amount of reserved virtual addresses and allocate enough memory to store
    /// `capacity` elements.
    pub fn with_reserved_memory_and_capacity(reserved_memory: usize, capacity: usize) -> Self {
        if reserved_memory < capacity {
            panic!("Attempted to create an ArenaVec with less reserved memory than allocated capacity.");
        }

        crate::init();

        let reserved_memory = unsafe { OsMetadata::global_unchecked().page_align(reserved_memory) };
        let buffer = Os::reserve(reserved_memory).unwrap();
        let buffer_aligned = utils::align(buffer.as_ptr() as usize, mem::align_of::<T>());
        let buffer = unsafe { NonNull::new_unchecked(buffer_aligned as *mut T) };

        Self {
            reserved_memory,
            capacity,
            len: 0,
            buffer,
        }
    }

    pub fn push(&self, val: T) {
        if self.len + 1 > self.capacity {
            let used_memory = mem::size_of::<T>() * self.len;

            // Double in size if possible, else reserve all memory
            let growth_amount = if used_memory == 0 {
                mem::size_of::<T>()
            } else if used_memory * 2 < self.reserved_memory {
                used_memory
            } else {
                self.reserved_memory - used_memory
            };

            // global state was loaded when the arenavec was created
            let growth_amount = unsafe { OsMetadata::global_unchecked().page_align(growth_amount) };

            if used_memory + growth_amount > self.reserved_memory {
                // rip bozo
                panic!("ArenaVec needed to grow, but ran out of reserved memory");
            }

            let region_to_allocate = unsafe { self.buffer.byte_add(self.capacity) };
            unsafe { Os::commit(region_to_allocate.cast(), growth_amount) };

            unsafe {
                *addr_of!(self.capacity).cast_mut() += growth_amount;
            }
        }

        unsafe {
            let ptr = self.buffer.add(self.len);
            ptr.write(val);
            *addr_of!(self.len).cast_mut() += 1;
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            Some(unsafe { self.buffer.add(idx).as_ref() })
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            Some(unsafe { self.buffer.add(idx).as_mut() })
        } else {
            None
        }
    }

    /// Removes an item from the vector, moving all items after it down a slot.
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx < self.len {
            let ptr = unsafe { self.buffer.add(idx) };
            let val = unsafe { ptr.read() };

            unsafe {
                ptr::copy(ptr.add(1).as_ptr(), ptr.as_ptr(), self.len - idx - 1);
            }

            self.len -= 1;

            Some(val)
        } else {
            None
        }
    }

    /// Returns an iterator over all the items in this arenavec. This iterator will set the arenavec's
    /// length to 0, regardless of how much you progress through it.
    pub fn drain(&mut self) -> Drain<'_, T> {
        let len = self.len;

        Drain {
            arena_vec: self,
            progress: 0,
            len,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserved_memory(&self) -> usize {
        self.reserved_memory
    }
}
impl<T> Default for ArenaVec<T> {
    fn default() -> Self {
        Self::with_reserved_memory(Self::DEFAULT_RESERVED_MEMORY)
    }
}
impl<T> Drop for ArenaVec<T> {
    fn drop(&mut self) {
        unsafe {
            Os::decommit(self.buffer.cast(), self.capacity);
            Os::dereserve(self.buffer.cast(), self.reserved_memory);
        }
    }
}
impl<T: Clone> Clone for ArenaVec<T> {
    fn clone(&self) -> Self {
        let new_vec =
            ArenaVec::with_reserved_memory_and_capacity(self.reserved_memory, self.capacity());
        unsafe { new_vec.buffer.copy_from(self.buffer, self.len()) };

        new_vec
    }
}

/// The iterator returned by [`ArenaVec::drain`].
pub struct Drain<'a, T> {
    arena_vec: &'a mut ArenaVec<T>,
    progress: usize,
    len: usize,
}
impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.progress < self.len {
            let ptr = unsafe { self.arena_vec.buffer.add(self.progress) };
            self.progress += 1;

            Some(unsafe { ptr.read() })
        } else {
            None
        }
    }
}
impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        self.arena_vec.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::ArenaVec;

    #[test]
    fn do_it_work_tho() {
        let vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        assert_eq!(*vec.get(0).unwrap(), 0);
        assert_eq!(*vec.get(1).unwrap(), 1);
        assert_eq!(*vec.get(2).unwrap(), 2);
    }

    #[test]
    fn remove() {
        let mut vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        assert_eq!(vec.remove(1).unwrap(), 1);
        assert_eq!(*vec.get(0).unwrap(), 0);
        assert_eq!(*vec.get(1).unwrap(), 2);
        assert_eq!(vec.get(2), None);
    }
}
