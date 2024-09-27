//! Miscellaneous tools and types used by Scaffolding.

use {
    crate::os::{Os, OsTrait},
    core::{
        alloc::Layout,
        hash::{BuildHasher, Hasher},
        ops::{Deref, DerefMut},
        ptr::{self, NonNull},
    },
};

/// Hashn'ts values. It implements hash but doesn't actually do any hashing.
/// This can be useful for hashmaps where the keys are numbers, especially if
/// you know those numbers will be unique.
///
/// Note: Rust has hashes and hash builders. This is technically the hash builder.
/// It's the type you want to use with hashmaps and related data structures; but
/// for the actual hash implementation, see [`HashntHash`].
#[derive(Default)]
pub struct Hashnt;
impl BuildHasher for Hashnt {
    type Hasher = HashntHash;

    fn build_hasher(&self) -> Self::Hasher {
        HashntHash::default()
    }
}

/// The [`Hasher`] implementation for [`Hashnt`]. It simply stores values written
/// to it while hashing and then returns the final value in `finish`, thus implementing
/// [`Hasher`] without doing any actual hashing.
#[derive(Default)]
pub struct HashntHash {
    pub result: u64,
}
impl Hasher for HashntHash {
    fn write(&mut self, i: &[u8]) {
        match i.len() {
            1 => self.write_u8(u8::from_ne_bytes(i.try_into().unwrap())),
            2 => self.write_u16(u16::from_ne_bytes(i.try_into().unwrap())),
            4 => self.write_u32(u32::from_ne_bytes(i.try_into().unwrap())),
            8 => self.write_u64(u64::from_ne_bytes(i.try_into().unwrap())),
            16 => self.write_u128(u128::from_ne_bytes(i.try_into().unwrap())),
            _ => unimplemented!(),
        }
    }

    fn write_u8(&mut self, i: u8) {
        self.result = i as u64;
    }
    fn write_i8(&mut self, i: i8) {
        self.result = i as u64;
    }
    fn write_u16(&mut self, i: u16) {
        self.result = i as u64;
    }
    fn write_i16(&mut self, i: i16) {
        self.result = i as u64;
    }
    fn write_u32(&mut self, i: u32) {
        self.result = i as u64;
    }
    fn write_i32(&mut self, i: i32) {
        self.result = i as u64;
    }
    fn write_u64(&mut self, i: u64) {
        self.result = i;
    }
    fn write_i64(&mut self, i: i64) {
        self.result = i as u64;
    }
    fn write_u128(&mut self, i: u128) {
        self.result = i as u64;
    }
    fn write_i128(&mut self, i: i128) {
        self.result = i as u64;
    }
    fn write_usize(&mut self, i: usize) {
        self.result = i as u64;
    }
    fn write_isize(&mut self, i: isize) {
        self.result = i as u64;
    }

    fn finish(&self) -> u64 {
        self.result
    }
}

/// An enum used to convert arbitrary memory amounts to that amount in bytes.
pub enum MemoryAmount {
    Bytes(usize),
    Kilobytes(usize),
    Kibibytes(usize),
    Megabytes(usize),
    Mebibytes(usize),
    Gigabytes(usize),
    Gibibytes(usize),
}
impl MemoryAmount {
    pub const fn into_bytes(self) -> usize {
        match self {
            Self::Bytes(amount) => amount,
            Self::Kilobytes(amount) => amount * 1000,
            Self::Kibibytes(amount) => amount * 1024,
            Self::Megabytes(amount) => amount * 1000 * 1000,
            Self::Mebibytes(amount) => amount * 1024 * 1024,
            Self::Gigabytes(amount) => amount * 1000 * 1000 * 1000,
            Self::Gibibytes(amount) => amount * 1024 * 1024 * 1024,
        }
    }
}

// thx spey https://github.com/Speykious/csussus/blob/cbbcfa4484e34a0d9d49e20019329261f4744e45/src/arena.rs#L314C1-L318C2

/// Align a value upwards to the given alignment.
#[inline(always)]
pub fn align(value: usize, to: usize) -> usize {
    (value as isize + (-(value as isize) & (to as isize - 1))) as usize
}

/// A type similar to [`alloc::boxed::Box`], but backed by Scaffolding's
/// [`OsTrait`] API.
///
/// When the allocator API is stabilised, we can implement `Allocator` for
/// [`Os`] and then use that with boxes. Right now, however, Scaffolding
/// will use its own box implementation to avoid needing nightly feature flags.
pub struct ScaffoldingBox<T: ?Sized>(NonNull<T>);
impl<T: Sized> ScaffoldingBox<T> {
    pub fn new(val: T) -> Self {
        let ptr: NonNull<T> = Os::allocate(Layout::new::<T>()).unwrap().cast();
        unsafe { ptr.as_ptr().write(val) };

        Self(ptr)
    }
}
impl<T: ?Sized> ScaffoldingBox<T> {
    /// Get the underlying pointer from this [`ScaffoldingBox`].
    pub fn as_raw(&self) -> NonNull<T> {
        self.0
    }

    /// Create a new [`ScaffoldingBox`] from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid, aligned pointer to an instance of `T`.
    pub unsafe fn from_raw(raw: NonNull<T>) -> Self {
        Self(raw)
    }
}
impl<T: ?Sized> Deref for ScaffoldingBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
impl<T: ?Sized> DerefMut for ScaffoldingBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
impl<T: ?Sized> Drop for ScaffoldingBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.0.as_ptr());
            let layout = Layout::for_value(self.0.as_ref());
            Os::deallocate(self.0.cast(), layout.size());
        }
    }
}

/// A wrapper type that forces the compiler to mark a type as `Sync`. This can
/// be useful when working with a type that isn't normally `Sync`, but has a
/// method where it can be assumed to be `Sync`.
#[repr(transparent)]
pub struct AssumeSync<T>(T);
#[allow(clippy::missing_safety_doc)]
impl<T> AssumeSync<T> {
    pub unsafe fn new(val: T) -> Self {
        Self(val)
    }
    pub unsafe fn take(self) -> T {
        self.0
    }
}
unsafe impl<T> Sync for AssumeSync<T> {}

/// A wrapper type that forces the compiler to mark a type as `Send`. This can
/// be useful when working with a type that isn't normally `Send`, but has a
/// method where it can be assumed to be `Send`.
#[repr(transparent)]
pub struct AssumeSend<T>(T);
#[allow(clippy::missing_safety_doc)]
impl<T> AssumeSend<T> {
    pub unsafe fn new(val: T) -> Self {
        Self(val)
    }
    pub unsafe fn take(self) -> T {
        self.0
    }
}
unsafe impl<T> Send for AssumeSend<T> {}

/// A wrapper type that forces the compiler to mark a type as `Sync` and
/// `Send`. This can be useful when working with a type that isn't normally
/// `Sync` or `Send`, but has a method where it can be assumed to be `Sync` and
/// `Send`.
#[repr(transparent)]
pub struct AssumeSyncSend<T>(T);
#[allow(clippy::missing_safety_doc)]
impl<T> AssumeSyncSend<T> {
    pub unsafe fn new(val: T) -> Self {
        Self(val)
    }
    pub unsafe fn take(self) -> T {
        self.0
    }
}
unsafe impl<T> Sync for AssumeSyncSend<T> {}
unsafe impl<T> Send for AssumeSyncSend<T> {}

/// Create a struct that stores bitflags. Inspired by the bitflags crate but
/// done my way. Example usage:
///
/// ```rs
/// bitflags! {
///     // The struct type that stores the flags, and how big it is.
///     struct Flags: u8;
///     // The enum containing the flags that will be stored in [`Flags`].
///     bitflags FlagTypes {
///         // One of the bitflags, and the bit it occupies
///         Flag1 = 0b0000_0001,
///         Flag2 = 0b0000_0010,
///     }
/// }
///
/// fn some_fn() {
///     // Can create the flag storage type by or-ing flags together
///     let mut flags = FlagTypes::Flag1 | FlagTypes::Flag2;
///
///     // Can add flags to a flag storage type with |=, +=, or add_flag
///     // (these three lines all do the same thing)
///     flags |= FlagTypes::Flag3;
///     flags += FlagTypes::Flag3;
///     flags.add_flag(FlagTypes::Flag3);
///
///     // Can remove flags from a flag storage type with ^=, -=, or remove_flag
///     flags ^= FlagTypes::Flag3;
///     flags -= FlagType::Flag3;
///     flags.remove_flag(FlagType::Flag3);
///
///     // Can check for flags with & or contains
///     assert!(flags & FlagType::Flag2);
///     assert!(flags & FlagType::Flag2);
/// }
/// ```
#[macro_export]
macro_rules! bitflags {
    (struct $struct:ident : $repr:ty; bitflags $bitflags:ident {$($variant:ident = $val:literal$(,)?)*}) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $struct($repr);
        #[allow(dead_code)]
        impl $struct {
            pub fn contains(&self, flag: $bitflags) -> bool {
                (self.0 & flag as $repr) != 0
            }
            pub fn add_flag(&mut self, flag: $bitflags) {
                self.0 |= flag as $repr;
            }
            pub fn remove_flag(&mut self, flag: $bitflags) {
                self.0 &= !(flag as $repr);
            }
            pub fn merge(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }
        impl Default for $struct {
            fn default() -> Self {
                Self(0)
            }
        }
        impl From<$bitflags> for $struct {
            fn from(val: $bitflags) -> Self {
                Self(val as $repr)
            }
        }
        impl ::core::ops::BitOrAssign<$bitflags> for $struct {
            fn bitor_assign(&mut self, rhs: $bitflags) {
                self.add_flag(rhs);
            }
        }
        impl ::core::ops::AddAssign<$bitflags> for $struct {
            fn add_assign(&mut self, rhs: $bitflags) {
                self.add_flag(rhs);
            }
        }
        impl ::core::ops::BitXorAssign<$bitflags> for $struct {
            fn bitxor_assign(&mut self, rhs: $bitflags) {
                self.remove_flag(rhs);
            }
        }
        impl ::core::ops::SubAssign<$bitflags> for $struct {
            fn sub_assign(&mut self, rhs: $bitflags) {
                self.remove_flag(rhs);
            }
        }
        impl ::core::ops::BitAnd<$bitflags> for $struct {
            type Output = bool;

            fn bitand(self, rhs: $bitflags) -> Self::Output {
                self.contains(rhs)
            }
        }
        impl ::core::ops::BitOr<$bitflags> for $struct {
            type Output = Self;

            fn bitor(self, rhs: $bitflags) -> Self::Output {
                Self(self.0 | rhs as $repr)
            }
        }
        impl ::core::ops::BitOr<$struct> for $struct {
            type Output = Self;

            fn bitor(self, rhs: $struct) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        #[repr($repr)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $bitflags {
            $($variant = $val),*
        }
        impl ::core::ops::BitOr<$bitflags> for $bitflags {
            type Output = $struct;

            fn bitor(self, rhs: $bitflags) -> $struct {
                $struct(self as $repr | rhs as $repr)
            }
        }
    };
}
pub use crate::bitflags;

#[cfg(test)]
mod tests {
    #[test]
    fn bitflags() {
        bitflags! {
            struct FlagStore: u8;
            bitflags Flags {
                Flag1 = 0b0000_0001,
                Flag2 = 0b0000_0010,
            }
        };

        let mut flags = Flags::Flag1 | Flags::Flag2;
        assert!(flags.contains(Flags::Flag1));
        assert!(flags.contains(Flags::Flag2));
        assert!(flags & Flags::Flag1);
        assert!(flags & Flags::Flag2);

        flags ^= Flags::Flag1;
        assert!(!flags.contains(Flags::Flag1));
        assert!(flags.contains(Flags::Flag2));
        assert!(!(flags & Flags::Flag1));
        assert!(flags & Flags::Flag2);

        flags |= Flags::Flag1;
        assert!(flags.contains(Flags::Flag1));
        assert!(flags.contains(Flags::Flag2));
        assert!(flags & Flags::Flag1);
        assert!(flags & Flags::Flag2);

        flags.remove_flag(Flags::Flag1);
        assert!(!flags.contains(Flags::Flag1));
        assert!(flags.contains(Flags::Flag2));
        assert!(!(flags & Flags::Flag1));
        assert!(flags & Flags::Flag2);

        flags.add_flag(Flags::Flag1);
        assert!(flags.contains(Flags::Flag1));
        assert!(flags.contains(Flags::Flag2));
        assert!(flags & Flags::Flag1);
        assert!(flags & Flags::Flag2);
    }
}
