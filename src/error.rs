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

    use super::{error, *};

    use crate::*;

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
        pub fn new<E>(source: Option<E>, description: impl Into<Description>) -> Self
            where E: error::Error + Send + Sync + 'static
        {
            if let Some(source) = source {
                Self::Error { source: Box::new(source), description: description.into(), }
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

    /// Indicates an error that's the result of [`Device::query_image_creation_limits`].
    #[derive(Debug)]
    pub enum FormatQueryError {
        /// Indicates that the query was called with invalid input.
        InvalidInput(String),
        /// Indicates the format is not supported on this device.
        FormatNotSupported(Format),
        /// Indicates the specified combination of image usage is not supported on this device.
        UsageNotSupported(ImageUsages, Option<ImageUsages>),
        /// Indicates that an internal error has occurred.
        ///
        /// If this happens, the program **must** be terminated.
        InternalError(Error),
    }

    impl Display for FormatQueryError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidInput(message) => write!(f, "{message}"),
                Self::FormatNotSupported(format) => write!(f, "format `{format}` is not supported on this device for these parameters"),
                Self::UsageNotSupported(usage, stencil) => {
                    if let Some(stencil) = stencil {
                        write!(f, "this combination of usages are not supported here: `usages`: `{usage}`, `stencil_usages`: `{stencil}`")
                    } else {
                        write!(f, "this combination of usages are not supported here: `usages`: `{usage}`")
                    }
                },
                Self::InternalError(_) => write!(f, "an internal error has occurred"),
            }
        }
    }

    impl error::Error for FormatQueryError {

        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            if let Self::InternalError(err) = self {
                Some(err)
            } else {
                None
            }
        }
    }

    /// A validation error returned by [`Device::create_shader_module`].
    #[derive(Debug)]
    pub enum ShaderValidationError {
        /// A general validation error.
        ValidationError(String),
        /// Specifies that a SPIR-V capability is not supported in the context.
        CapabilityNotSupported {
            /// The [`spirv::op::Capability`] that didn't pass validation.
            capability: spirv::op::Capability,
            /// The error message.
            message: Option<String>,
        },
    }
    impl From<spirv_tools::Error> for ShaderValidationError {
        #[inline]
        fn from(value: spirv_tools::Error) -> Self {
            Self::ValidationError(value.to_string())
        }
    }
    impl From<String> for ShaderValidationError {
        #[inline]
        fn from(value: String) -> Self {
            Self::ValidationError(value)
        }
    }
    impl Display for ShaderValidationError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ValidationError(message) => {
                    write!(f, "validation of SPIR-V failed because: {message}")
                },
                Self::CapabilityNotSupported { capability, message } => {
                    if let Some(msg) = message {
                        write!(f, "SPIR-V capability {capability:?} not supported because: {msg}")
                    } else {
                        write!(f, "SPIR-V capability {capability:?} not supported")
                    }
                },
            }
        }
    }
    impl error::Error for ShaderValidationError {}
}

#[macro_export]
#[doc(hidden)]
macro_rules! err {
    ($($t:tt)*) => {
        $crate::Error::new::<$crate::Error>(None, format!($($t)*))
    };
}

pub use prelude::*;
