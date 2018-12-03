use std::ops::Deref;

/// Pointer to specific element of a vector.
/// It is up to the user to ensure the vector is not moved
/// and that the index used to access the element stays valid.
#[derive(Debug)]
pub struct IndexPtr<T> {
    vec: *const Vec<T>,
    i: usize,
}

impl<T> IndexPtr<T> {
    #[allow(clippy::ptr_arg)]
    pub fn new(vec: &Vec<T>, i: usize) -> Self {
        Self { vec, i }
    }
}

impl<T> Deref for IndexPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let vec = unsafe { &*self.vec };
        &vec[self.i]
    }
}

// Needs to be implemented explicitly because
// using derive requires T: Clone
impl<T> Clone for IndexPtr<T> {
    fn clone(&self) -> Self {
        Self { vec: self.vec, i: self.i }
    }
}

// As long as the pointer is valid it is thread safe
// since it does not produce mutable references
unsafe impl<T> Sync for IndexPtr<T> where T: Sync {}
unsafe impl<T> Send for IndexPtr<T> where T: Send {}
