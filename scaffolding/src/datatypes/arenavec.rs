//! Module for [`ArenaVec`].

use {
    crate::{
        os::{Os, OsMetadata, OsTrait},
        utils::MemoryAmount,
    },
    core::{
        borrow::{Borrow, BorrowMut},
        mem::{self, needs_drop, MaybeUninit},
        ops::{Bound, Deref, DerefMut, Range, RangeBounds},
        ptr::{self, addr_of, NonNull},
    },
};

/// Represents possible errors that vec functions can return
#[derive(Debug)]
pub enum Error {
    OutOfMemoryAddresses,
    IndexOutOfBounds,
}

pub type Result<T> = std::result::Result<T, Error>;

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
    /// A pointer to the base of the memory buffer storing all the arenavec's
    /// items.
    // TODO: Swap this pointer out for a `NonNull` once `nonnull_convenience`
    // is stabilised
    buffer: *mut T,
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
        let buffer = Os::reserve(reserved_memory).unwrap().as_ptr().cast::<T>();

        Self {
            reserved_memory,
            capacity,
            len: 0,
            buffer,
        }
    }

    pub fn push(&self, val: T) -> Result<()> {
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
                // panic!("ArenaVec needed to grow, but ran out of reserved memory");
                return Err(Error::OutOfMemoryAddresses);
            }

            let region_to_allocate =
                unsafe { NonNull::new_unchecked(self.buffer.byte_add(self.capacity)) };
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
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.as_mut_ptr().add(self.len() + 1).read() })
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            Some(unsafe { &*self.buffer.add(idx) })
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            Some(unsafe { &mut *self.buffer.add(idx) })
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
                ptr::copy(ptr.add(1), ptr, self.len - idx - 1);
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

    /// This function returns the count of Ts that can be pushed before the vector runs out of memory
    pub fn remaining_space(&self) -> usize {
        self.reserved_memory().div_ceil(std::mem::size_of::<T>()) - self.len()
    }

    pub fn as_ptr(&self) -> *const T {
        self.buffer as *const T
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buffer
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            arena_vec: self,
            idx: 0,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut {
            arena_vec: self,
            idx: 0,
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            arena_vec: self,
            idx: 0,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.buffer as *const T, self.len) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.buffer, self.len) }
    }
    // This isn't using the trait because it can fail
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<()> {
        let mut iter = iter.into_iter();
        // There's not enough space to fit the whole iterator in
        if iter.size_hint().0 > self.remaining_space() {
            Err(Error::OutOfMemoryAddresses)
        }
        // If this function errors out due to not enough memory addresses, it will have filled up its entire capacity
        // We may want to have it stay the same as it was before if it errors out
        // But we can change that later
        else {
            while let Some(val) = iter.next() {
                self.push(val)?;
            }
            Ok(())
        }
    }

    pub fn append(&mut self, other: &mut ArenaVec<T>) -> Result<()> {
        if self.remaining_space() < other.len() {
            Err(Error::OutOfMemoryAddresses)
        } else {
            unsafe { std::ptr::copy(other.as_ptr(), self.as_mut_ptr(), other.len()) };
            // Should we zero out the other vecs memory?
            other.clear();
            Ok(())
        }
    }

    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        // First, tag all duplicates
        // Arenavec?
        let mut duplicate_idxs = Vec::new();
        for idx in 0..(self.len() - 1) {
            if self.get(idx).unwrap().eq(&self.get(idx + 1).unwrap()) {
                duplicate_idxs.push(idx + 1);
            }
        }
        // memcpy contiguous memory regions down the vec
        let mut bgn = 0;
        let mut region_iter = duplicate_idxs.iter().map(|idx| {
            let region = bgn..*idx;
            bgn = idx + 1;
            region
        });
        // There is guaranteed to be at least one contiguous region
        let mut base_range = region_iter.next().unwrap();
        region_iter.for_each(|r| {
            unsafe {
                ptr::copy(
                    self.get(r.start).unwrap(),
                    self.get_mut(base_range.end).unwrap(),
                    r.len(),
                );
            }
            base_range = r;
        });
    }

    pub fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        T: PartialEq,
        F: FnMut(&mut T, &mut T) -> bool,
    {
        // First, tag all duplicates
        // Arenavec?
        let mut duplicate_idxs = Vec::new();
        // the iteration here has to be a little weird, because guarantees we make about eq types are no longer relavent here
        // if we have elements a, b, and c, and a ~= b, and a ~= c, this doesn't necessarily mean that b ~= c
        // Because the spec says that we compare two items with the func and remove the second one (although it's first in the function call),
        // We have to keep calling with the last element that flagged the function
        // e.g, we have to call f(b, a), and if that returns true, we then have to call f(c, a)
        // This means we have to keep track of indices
        // :(
        let mut base_idx = 0;
        for idx in 1..self.len() {
            // This is safe since idx is guaranteed to not equal base_idx
            let a = self.get_mut(idx).unwrap() as *mut T;
            let b = self.get_mut(base_idx).unwrap() as *mut T;
            if same_bucket(unsafe { a.as_mut().unwrap() }, unsafe {
                b.as_mut().unwrap()
            }) {
                duplicate_idxs.push(idx);
            } else {
                base_idx = idx;
            }
        }
        // memcpy contiguous memory regions down the vec
        let mut bgn = 0;
        let mut region_iter = duplicate_idxs.iter().map(|idx| {
            let region = bgn..*idx;
            bgn = idx + 1;
            region
        });
        // There is guaranteed to be at least one contiguous region
        let mut base_range = region_iter.next().unwrap();
        region_iter.for_each(|r| {
            unsafe {
                ptr::copy(
                    self.get(r.start).unwrap(),
                    self.get_mut(base_range.end).unwrap(),
                    r.len(),
                );
            }
            base_range = r;
        });
    }

    pub fn split_off(&mut self, at: usize) -> Result<ArenaVec<T>> {
        if at > self.len() {
            return Err(Error::IndexOutOfBounds);
        }
        let mut other = ArenaVec::with_capacity(self.len() - at);
        if at == self.len() {
            return Ok(other);
        }
        unsafe {
            ptr::copy(self.as_ptr().add(at), other.as_mut_ptr(), self.len() - at);
        }
        self.len = at;
        Ok(other)
    }

    pub fn truncate(&mut self, new_len: usize) -> Result<()> {
        if new_len > self.len() {
            return Err(Error::IndexOutOfBounds);
        }
        // There's definitely a better way to do this, but this is fine for now
        while self.len() > new_len {
            self.pop().unwrap();
        }
        Ok(())
    }

    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F) -> Result<()>
    where
        F: FnMut() -> T,
    {
        if new_len < self.len() {
            self.truncate(new_len)
        } else if new_len == self.len() {
            Ok(())
        } else {
            while self.len() < new_len {
                self.push(f())?;
            }
            Ok(())
        }
    }

    pub fn leak<'a>(mut self) -> &'a mut [T] {
        // This is just a guess at how this should work
        let slice: &mut [T] = self.as_mut_slice();
        // This for sure sucks
        unsafe { mem::transmute::<&mut [T], &'a mut [T]>(slice) }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len) as *mut MaybeUninit<T>,
                self.capacity - self.len,
            )
        }
    }

    pub fn resize(&mut self, new_len: usize, value: T) -> Result<()>
    where
        T: Clone,
    {
        // Inefficient, but it works
        let f = || value.clone();
        self.resize_with(new_len, f)
    }

    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<()>
    where
        T: Clone,
    {
        // Inefficient, but it works
        for val in other {
            self.push(val.clone())?;
        }
        Ok(())
    }

    pub fn extend_from_within<R>(&mut self, src: R) -> Result<()>
    where
        R: RangeBounds<usize>,
        T: Clone,
    {
        let range = (match src.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        })..(match src.end_bound() {
            Bound::Included(i) => *i + 1,
            Bound::Excluded(i) => *i,
            Bound::Unbounded => self.len(),
        });
        for idx in range {
            let val = unsafe { self.as_ptr().add(idx).read().clone() };
            self.push(val)?;
        }
        Ok(())
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
            let buffer = NonNull::new_unchecked(self.buffer);
            Os::decommit(buffer.cast(), self.capacity);
            Os::dereserve(buffer.cast(), self.reserved_memory);
        }
    }
}
impl<T: Clone> Clone for ArenaVec<T> {
    fn clone(&self) -> Self {
        let new_vec: ArenaVec<T> =
            ArenaVec::with_reserved_memory_and_capacity(self.reserved_memory, self.capacity());
        unsafe { new_vec.buffer.copy_from(self.buffer, self.len()) };

        new_vec
    }
}
impl<T> AsRef<[T]> for ArenaVec<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T> AsMut<[T]> for ArenaVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<T> Borrow<[T]> for ArenaVec<T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T> BorrowMut<[T]> for ArenaVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<T> Deref for ArenaVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T> DerefMut for ArenaVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
// I'm not implementing Extend right now, because I don't know if we want falliable APIs like that in the struct
impl<T: Clone> From<&[T]> for ArenaVec<T> {
    fn from(value: &[T]) -> Self {
        let mut v = Self::with_capacity(value.len());
        v.extend(value.iter().map(|t| t.clone())).unwrap();
        v
    }
}
impl<T: Clone, const N: usize> From<&[T; N]> for ArenaVec<T> {
    fn from(value: &[T; N]) -> Self {
        let mut v = Self::with_capacity(N);
        v.extend(value.iter().map(|t| t.clone())).unwrap();
        v
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

pub struct Iter<'a, T> {
    arena_vec: &'a ArenaVec<T>,
    idx: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        self.arena_vec.get(self.idx - 1)
    }
}

pub struct IterMut<'a, T> {
    arena_vec: &'a mut ArenaVec<T>,
    idx: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        if idx >= self.arena_vec.len() {
            return None;
        }
        self.idx += 1;
        let ptr = self.arena_vec.as_mut_ptr();
        Some(unsafe { ptr.add(idx).as_mut().unwrap() })
    }
}

pub struct IntoIter<T> {
    arena_vec: ArenaVec<T>,
    idx: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        if idx >= self.arena_vec.len() {
            return None;
        }
        self.idx += 1;
        let ptr = self.arena_vec.as_mut_ptr();
        Some(unsafe { ptr.add(idx).read() })
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

    #[test]
    fn basic_iter() {
        let vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);

        drop(iter);
        assert_eq!(*vec.get(0).unwrap(), 0);
        assert_eq!(*vec.get(1).unwrap(), 1);
        assert_eq!(*vec.get(2).unwrap(), 2);
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn basic_iter_mut() {
        let mut vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        let mut iter = vec.iter_mut();
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), None);

        drop(iter);
        assert_eq!(*vec.get(0).unwrap(), 0);
        assert_eq!(*vec.get(1).unwrap(), 1);
        assert_eq!(*vec.get(2).unwrap(), 2);
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn iter_mut_modify() {
        let mut vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        let iter = vec.iter_mut();
        let mut iter = iter.map(|num| {
            *num += 1;
            num
        });
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);

        drop(iter);
        assert_eq!(*vec.get(0).unwrap(), 1);
        assert_eq!(*vec.get(1).unwrap(), 2);
        assert_eq!(*vec.get(2).unwrap(), 3);
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn basic_into_iter() {
        let vec = ArenaVec::default();
        vec.push(0);
        vec.push(1);
        vec.push(2);

        let mut iter = vec.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
    }
}
