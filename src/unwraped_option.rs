use std::ops::Deref;

#[derive(Debug)]
pub struct UnwrappedOption<T>(pub(crate) Option<T>);

impl<T> Deref for UnwrappedOption<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.as_ref().expect("Attempted to deref a None value")
    }
}
