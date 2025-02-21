use std::{cell::OnceCell, fmt::Debug, ops::Deref};

#[derive(Debug)]
pub struct UnwrappedOption<T>(pub(crate) Option<T>);

impl<T> Deref for UnwrappedOption<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.as_ref().expect("Attempted to deref a None value")
    }
}

#[derive(Debug)]
pub struct Lazy<T>(OnceCell<T>);

impl<T> Lazy<T> {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }

    pub fn set(&self, data: T) {
        _ = self.0.set(data)
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get().expect("Lazy value not initialized")
    }
}
