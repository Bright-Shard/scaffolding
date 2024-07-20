//! Module for [`Warehouse`].

use {
    crate::datatypes::{ArenaVec, StackVec, TypeMap},
    core::{
        cell::UnsafeCell,
        mem::ManuallyDrop,
        ops::{Deref, DerefMut},
    },
};

/// A storage system that allows taking temporary ownership of instances of
/// a type `T`. Instead of borrowing `T`, and instance of `T` is temporarily
/// moved from the [`Warehouse`], to be returned later. This is similar to [`Cell`]
/// in the standard library, but it can store several instances of `T`.
///
/// The warehouse stores instances of `T` in an [`ArenaVec`]. You can get an
/// instance of `T` from the warehouse with [`Warehouse::get_instance`]. If the warehouse
/// has an instance stored, it will return that; otherwise, it will make a new
/// instance and return that instead.
///
/// Instances are returned wrapped in a [`WarehouseValue`]. This type can deref
/// to `T`, and will automatically return itself to the [`Warehouse`] it was taken
/// from when dropped. If you absolutely need to take ownership of the instance,
/// you can use [`Warehouse::take_instance`] instead of [`Warehouse::get_instance`],
/// but you will be responsible for returning the instance to the [`Warehouse`] yourself.
///
/// Because [`Warehouse`]s move values instead of borrowing them, there's no need to
/// worry about pointers or memory safety. Thus, both [`Warehouse::get_instance`] and
/// [`Warehouse::return_instance`] take `&self`, not `&mut self`. This allows for a very
/// convenient way to store and reuse instances of type `T`, with the slight overhead
/// of having to frequently move those instances.
///
/// [`Cell`]: std::cell::Cell
#[derive(Default)]
pub struct Warehouse<T: Default + Reset> {
    storage: UnsafeCell<ArenaVec<T>>,
}
impl<T: Default + Reset> Warehouse<T> {
    /// Creates a new [`Warehouse`] backed by an [`ArenaVec`] that has the given reserved
    /// memory.
    pub fn with_reserved_memory(reserved_memory: usize) -> Self {
        Self {
            storage: UnsafeCell::new(ArenaVec::with_reserved_memory(reserved_memory)),
        }
    }

    /// Get an instance of `T` from the [`Warehouse`], or create a new instance of `T` if
    /// the [`Warehouse`] is empty. The instance will be wrapped in a [`WarehouseValue`].
    /// See the type-level docs for more info.
    pub fn get_instance(&self) -> WarehouseValue<'_, T> {
        WarehouseValue {
            val: ManuallyDrop::new(self.take_instance()),
            warehouse: self,
        }
    }

    /// Take an instance of `T` from the [`Warehouse`], or create a new instance of `T`
    /// if the [`Warehouse`] is empty. The type won't be wrapped in a [`WarehouseValue`],
    /// which makes you responsible for returning the type to the [`Warehouse`]. See
    /// the type-level docs for more info.
    pub fn take_instance(&self) -> T {
        let storage = unsafe { &mut *self.storage.get() };
        storage.remove(0).unwrap_or_default()
    }

    /// Return a taken instance of `T` to the [`Warehouse`]. [`WarehouseValue`]s call this
    /// method automatically when dropped.
    pub fn return_instance(&self, mut val: T) {
        let storage = unsafe { &*self.storage.get() };
        val.reset();
        storage.push(val);
    }
}

/// A wrapper type returned from [`Warehouse::get_instance`]. It derefs to `T`
/// and will automatically return itself to the [`Warehouse`] it was taken from.
///
/// If you need to get an actual instance of `T` and can't use this wrapper,
/// use [`Warehouse::take_instance`]. You will be responsible for returning the
/// instance to the [`Warehouse`] yourself, however.
pub struct WarehouseValue<'a, T: Default + Reset> {
    val: ManuallyDrop<T>,
    warehouse: &'a Warehouse<T>,
}
impl<'a, T: Default + Reset> Deref for WarehouseValue<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<'a, T: Default + Reset> DerefMut for WarehouseValue<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}
impl<'a, T: Default + Reset> AsRef<T> for WarehouseValue<'a, T> {
    fn as_ref(&self) -> &T {
        &self.val
    }
}
impl<'a, T: Default + Reset> AsMut<T> for WarehouseValue<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.val
    }
}
impl<'a, T: Default + Reset> Drop for WarehouseValue<'a, T> {
    fn drop(&mut self) {
        let instance = unsafe { ManuallyDrop::take(&mut self.val) };
        self.warehouse.return_instance(instance);
    }
}

/// A trait for "resetting" a type to its original state.
///
/// After calling [`Reset::reset`], an struct should more or less reset
/// to its initial state after creating it with some constructor, so that
/// it can be reused in the future.
///
/// This trait is implemented for all data structures in [`std::collections`] and
/// [`scaffolding::exrs`].
pub trait Reset {
    /// Reset the type - see the [`Reset`] docs.
    fn reset(&mut self);
}

impl Reset for TypeMap {
    fn reset(&mut self) {
        self.clear();
    }
}
impl<T> Reset for ArenaVec<T> {
    fn reset(&mut self) {
        self.clear();
    }
}
impl<T, const SIZE: usize> Reset for StackVec<T, SIZE> {
    fn reset(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use {super::Reset, std::collections::*};

    impl<T> Reset for Vec<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<T> Reset for VecDeque<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<T> Reset for LinkedList<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<T> Reset for HashSet<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<T> Reset for BTreeSet<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<T> Reset for BinaryHeap<T> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<K, V> Reset for HashMap<K, V> {
        fn reset(&mut self) {
            self.clear();
        }
    }
    impl<K, V> Reset for BTreeMap<K, V> {
        fn reset(&mut self) {
            self.clear();
        }
    }
}
