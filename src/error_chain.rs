use std::{error, fmt, iter};

/// An iterator over an [`error::Error`] and its sources.
///
/// If you want to omit the initial error and only process
/// its sources, use `skip(1)`.
#[derive(Clone, Debug)]
pub struct ErrorChain<'a>(Option<&'a (dyn error::Error + 'static)>);

impl<'a> ErrorChain<'a> {
    /// Creates new [`ErrorChain`] instance over given [`error::Error`] reference.
    pub const fn new(error: &'a (dyn error::Error + 'static)) -> Self {
        Self(Some(error))
    }
}

impl<'a> Iterator for ErrorChain<'a> {
    type Item = &'a (dyn error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0;
        self.0 = self.0.and_then(error::Error::source);
        current
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.0.is_some() {
            (1, None)
        } else {
            (0, Some(0))
        }
    }
}

impl<'a> iter::FusedIterator for ErrorChain<'a> {}

pub trait SourceKind {}

pub mod source {

    pub struct Anyhow;

    impl super::SourceKind for Anyhow {}

    pub struct StdError;

    impl super::SourceKind for StdError {}

    pub struct Displayable;

    impl super::SourceKind for Displayable {}
}
use self::source::*;

pub trait IntoErrorChain<S: SourceKind> {
    type Iter: Iterator<Item: fmt::Display> + Clone;

    fn into_error_chain(self) -> Self::Iter;
}

impl<'a> IntoErrorChain<Anyhow> for &'a anyhow::Error {
    type Iter = anyhow::Chain<'a>;

    fn into_error_chain(self) -> Self::Iter {
        self.chain()
    }
}

impl<'a, E> IntoErrorChain<StdError> for &'a E
where
    E: error::Error + 'static,
{
    type Iter = ErrorChain<'a>;

    fn into_error_chain(self) -> Self::Iter {
        ErrorChain::new(self)
    }
}

impl<'a, D> IntoErrorChain<Displayable> for &'a D
where
    D: fmt::Display,
{
    type Iter = iter::Once<&'a D>;

    fn into_error_chain(self) -> Self::Iter {
        iter::once(self)
    }
}
