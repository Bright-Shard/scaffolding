//! Module for [`StackVec`].

use {
    alloc::vec::Vec,
    core::ops::{Index, IndexMut},
};

/// A vector whose first few items are in a stack-based array.
///
/// The stackvec stores `SIZE` items in an array on the stack, and stores any additional items
/// that are pushed to it in a regular [`Vec`]. Unless [`StackVec::with_capacity`] is used,
/// the stackvec will not allocate on the heap until its array has been filled.
pub struct StackVec<T, const SIZE: usize> {
    stack: [Option<T>; SIZE],
    vec: Vec<T>,
    len: usize,
}
impl<T, const SIZE: usize> Default for StackVec<T, SIZE> {
    fn default() -> Self {
        Self {
            stack: [const { None }; SIZE],
            vec: Vec::default(),
            len: 0,
        }
    }
}
impl<T, const SIZE: usize> StackVec<T, SIZE> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`StackVec`] with the given capacity. This capacity includes `SIZE`, and thus
    /// the stackvec won't allocate unless the given capacity is larger than `SIZE`.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            stack: [const { None }; SIZE],
            vec: Vec::with_capacity(cap - SIZE),
            len: 0,
        }
    }

    /// Push a value to the [`StackVec`].
    pub fn push(&mut self, val: T) {
        if self.len < SIZE {
            self.stack[self.len] = Some(val);
        } else {
            self.vec.push(val);
        }

        self.len += 1;
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < SIZE {
            self.stack[idx].as_ref()
        } else {
            self.vec.get(idx)
        }
    }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < SIZE {
            self.stack[idx].as_mut()
        } else {
            self.vec.get_mut(idx)
        }
    }

    /// The total capacity of the [`StackVec`]. This is a sum of the capacity of its
    /// heap-based vector and `SIZE` (the size of its stack-based array).
    pub fn capacity(&self) -> usize {
        self.vec.capacity() + SIZE
    }
    pub fn vec_capacity(&self) -> usize {
        self.vec.capacity()
    }
    /// The number of elements currently in the [`StackVec`].
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn iter(&self) -> Iter<'_, T, SIZE> {
        Iter {
            stackvec: self,
            progress: 0,
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, T, SIZE> {
        IterMut {
            stackvec: self,
            progress: 0,
        }
    }
}
impl<T, const SIZE: usize> Index<usize> for StackVec<T, SIZE> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        if idx < SIZE {
            self.stack[idx].as_ref().unwrap()
        } else {
            self.vec.index(idx)
        }
    }
}
impl<T, const SIZE: usize> IndexMut<usize> for StackVec<T, SIZE> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx < SIZE {
            self.stack[idx].as_mut().unwrap()
        } else {
            self.vec.index_mut(idx)
        }
    }
}
impl<T, const SIZE: usize> IntoIterator for StackVec<T, SIZE> {
    type Item = T;
    type IntoIter = IntoIter<T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            stackvec: self,
            progress: 0,
        }
    }
}
impl<T, const SIZE: usize> FromIterator<T> for StackVec<T, SIZE> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut this = Self::with_capacity(iter.size_hint().1.unwrap_or(0));

        for val in iter {
            this.push(val);
        }

        this
    }
}
impl<T: Clone, const SIZE: usize> Clone for StackVec<T, SIZE> {
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            vec: self.vec.clone(),
            len: self.len,
        }
    }
}

pub struct IntoIter<T, const SIZE: usize> {
    stackvec: StackVec<T, SIZE>,
    progress: usize,
}
impl<T, const SIZE: usize> Iterator for IntoIter<T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.progress == self.stackvec.len {
            return None;
        }

        let ptr = if self.progress < SIZE {
            self.stackvec.stack[self.progress].as_mut().unwrap() as *mut T
        } else {
            (&mut self.stackvec.vec[self.progress]) as *mut T
        };

        self.progress += 1;

        Some(unsafe { ptr.read() })
    }
}

pub struct Iter<'a, T, const SIZE: usize> {
    stackvec: &'a StackVec<T, SIZE>,
    progress: usize,
}
impl<'a, T, const SIZE: usize> Iterator for Iter<'a, T, SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.progress == self.stackvec.len {
            return None;
        }

        let val = if self.progress < SIZE {
            self.stackvec.stack[self.progress].as_ref().unwrap()
        } else {
            &self.stackvec.vec[self.progress]
        };

        self.progress += 1;

        Some(val)
    }
}

pub struct IterMut<'a, T, const SIZE: usize> {
    stackvec: &'a mut StackVec<T, SIZE>,
    progress: usize,
}
impl<'a, T, const SIZE: usize> Iterator for IterMut<'a, T, SIZE> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.progress == self.stackvec.len {
            return None;
        }

        let ptr = if self.progress < SIZE {
            self.stackvec.stack[self.progress].as_mut().unwrap() as *mut T
        } else {
            &mut self.stackvec.vec[self.progress] as *mut T
        };

        self.progress += 1;

        Some(unsafe { &mut *ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::StackVec;

    #[test]
    fn push() {
        let mut sv: StackVec<u32, 3> = StackVec::default();
        assert_eq!(sv.len(), 0);
        sv.push(0);
        assert_eq!(sv.len(), 1);
        sv.push(0);
        assert_eq!(sv.len(), 2);
        sv.push(0);
        assert_eq!(sv.len(), 3);
    }

    #[test]
    fn into_iter() {
        let mut sv: StackVec<u32, 4> = StackVec::default();
        sv.push(0);
        sv.push(1);
        sv.push(2);
        sv.push(3);

        let mut iter = sv.into_iter();
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn iter() {
        let mut sv: StackVec<u32, 4> = StackVec::default();
        sv.push(0);
        sv.push(1);
        sv.push(2);
        sv.push(3);

        let mut iter = sv.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn iter_mut() {
        let mut sv: StackVec<u32, 4> = StackVec::default();
        sv.push(0);
        sv.push(1);
        sv.push(2);
        sv.push(3);

        let mut iter = sv.iter_mut();
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn with_capacity() {
        let sv: StackVec<u32, 3> = StackVec::with_capacity(4);
        assert_eq!(sv.capacity(), 4);
    }
}
