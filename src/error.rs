//! Contains all things [`Error`]-related.
//!
//! # New types
//! - [`Error`]: the primary error type returned by `leimu` functions.
//! - [`Description`]: a description of an error.
//!
//! # New traits
//! - [`DescribeResult`]: an extension trait for [`Result`], that provides the [`describe_err`][1]
//!   method.
//!
//! [1]: DescribeResult::describe_err

use core::{
    error,
    fmt::{self, Debug, Display},
};

/// Contains a description of an [`Error`].
///
/// This is just a wrapper around a string, that implements `From<&str>`, `From<FnOnce() -> String`
/// and `From<String>`.
///
/// This allows [`describe_err`][1] to take in many kinds of arguments.
///
/// [1]: DescribeResult::describe_err
pub struct Description {
    description: String,
}

impl Debug for Description {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.description, f)
    }
}

impl Display for Description {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.description, f)
    }
}

impl From<&str> for Description {

    #[inline]
    fn from(value: &str) -> Self {
        Self { description: value.to_string(), }
    }
}

impl<F> From<F> for Description
    where F: FnOnce() -> String
{

    #[inline]
    fn from(value: F) -> Self {
        Self { description: value(), }
    }
}

impl From<String> for Description
{

    #[inline]
    fn from(value: String) -> Self {
        Self { description: value, }
    }
}

pub(super) mod prelude {

    use super::*;

    /// A boxed `dyn` [`Error`][error::Error].
    pub type AnyError = Box<dyn error::Error + Send + Sync + 'static>;

    /// The error type returned by all `leimu` functions which may fail.
    #[derive(Debug)]
    pub enum Error {
        /// Contains the description and source of this error.
        Error {
            /// The source of this error.
            source: AnyError,
            /// A description of what went wrong.
            description: Description,
        },
        /// Contains only the description of this error, with no further source.
        String {
            /// A description of what went wrong.
            description: Description,
        },
    }

    impl Error {

        /// Constructs a new error from an optional `source` and a `description`.
        pub fn new(source: Option<AnyError>, description: impl Into<Description>) -> Self {
            if let Some(source) = source {
                Self::Error { source, description: description.into(), }
            } else {
                Self::String { description: description.into() }
            }
        }
    }

    impl Display for Error {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let disp: &dyn Display = match self {
                Self::Error { description, .. } => description,
                Self::String { description, } => description,
            };
            disp.fmt(f)
        }
    }

    impl error::Error for Error {

        #[inline]
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Error { source, .. } => Some(&**source),
                Self::String { .. } => None,
            }
        }
    }

    /// An extension trait for [`Result`] that contains a single method [`describe_err`][1].
    ///
    /// `describe_err` takes in any type that implements [`Into<Description>`] and maps the [`Err`] of
    /// the `Result` to [`Error`], which contains the original error and the provided description.
    ///
    /// [1]: Self::describe_err
    pub trait DescribeResult<T> {

        /// Describes the [`Err`] variant of the [`Result`].
        ///
        /// See the [`trait-level`][DescribeResult] documenation for details.
        fn describe_err(self, description: impl Into<Description>) -> Result<T, Error>;
    }

    impl<T, E> DescribeResult<T> for Result<T, E>
        where E: error::Error + Send + Sync + 'static
    {

        fn describe_err(self, description: impl Into<Description>) -> Result<T, Error> {
            self.map_err(|err| Error::Error { source: Box::new(err), description: description.into() })
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! err {
    ($($t:tt)*) => {
        Error::new(None, format!($($t)*))
    };
}

pub use prelude::*;
