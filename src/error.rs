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
            __variant_some!(Self::InternalError = self)
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

    /// A reflection error returned by [`ShaderModule::in_entry_point`].
    #[derive(Debug)]
    pub enum ShaderReflectError {
        /// Indicates that the entry point was not found.
        EntryPointNotFound(ShaderStage, String),
        /// Indicates a general reflection error.
        ReflectError(spirv::reflect::ReflectError),
        /// Indicates that a descriptor set was not found for a resource.
        NoDescriptorSet(CompactStr),
        /// Indicates that a descriptor binding was not found for a resource.
        NoDescriptorBinding(CompactStr),
        /// Indicates that the shader has more than one push constant.
        TooManyPushConstants,
    }

    impl Display for ShaderReflectError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::EntryPointNotFound(stage, name) => {
                    write!(f, "entry point with shader stage {stage:?} and name {name} not found")
                },
                Self::ReflectError(_) => {
                    write!(f, "reflection error")
                },
                Self::NoDescriptorSet(name) => {
                    write!(f, "descriptor set not found for resource `{name}`")
                },
                Self::NoDescriptorBinding(name) => {
                    write!(f, "descriptor binding not found for resource `{name}`")
                },
                Self::TooManyPushConstants => {
                    write!(f, "more than one push constant range specified for shader")
                },
            }
        }
    }
    impl error::Error for ShaderReflectError {
        #[inline]
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            __variant_some!(Self::ReflectError = self)
        }
    }

    #[derive(Debug)]
    pub enum PipelineLayoutError {
        Internal(Error),
        MismatchedShaderStage {
            tag: Option<String>,
            expected: ShaderStage,
            found: ShaderStage,
        },
        DuplicateSetBinding {
            set: u32,
            binding: u32,
        },
        InlineUniformBlockNotEnabled,
        InvalidInlineUniformBlock {
            name: CompactStr,
            found_type: DescriptorType,
        },
        UnresolvedStructSize {
            resource_name: CompactStr,
        },
        StructSizeTooBig {
            resource_name: CompactStr,
            resolved_size: usize,
        },
        InlineUniformBlockTooBig {
            resource_name: CompactStr,
            resolved_size: u32,
            max_size: u32,
        },
        TooManyDescriptors {
            count: u32,
            max_count: u32,
            descriptor_types: &'static [DescriptorType],
        },
        TooManyDescriptorsInStage {
            count: u32,
            max_count: u32,
            stage: ShaderStage,
            descriptor_types: &'static [DescriptorType],
        },
        MaxInlineUniformTotalSizeExceeded {
            max_total_size: u32,
            total_size: u32,
        },
        TooManyResourcesInStage {
            count: u32,
            max_count: u32,
            stage: ShaderStage,
        },
        InvalidPushConstantOffset {
            offset: u32,
            stage: ShaderStage,
        },
        InvalidPushConstantSize {
            size: u32,
            stage: ShaderStage,
        },
        PushConstantOutOfRange {
            offset: u32,
            size: u32,
            max_size: u32,
            stage: ShaderStage,
        },
        PushDescriptorNotEnabled,
        TooManyPushDescriptors {
            count: u32,
            max_count: u32,
        },
        InvalidPushDescriptorType {
            descriptor_type: DescriptorType,
        },
    }

    impl Display for PipelineLayoutError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Internal(_) => write!(f, "internal error occurred"),
                Self::MismatchedShaderStage { tag, expected, found } => {
                    if let Some(tag) = tag {
                        write!(f, "expected `Shader` `{tag}` to have the {expected:?} shader stage, but it has {found:?}")
                    } else {
                        write!(f, "expected `Shader` to have the {expected:?} shader stage, but it has {found:?}")
                    }
                },
                Self::DuplicateSetBinding { set, binding } => {
                    write!(f, "duplicate `set/binding` pair `({set}, {binding})`")
                },
                Self::InlineUniformBlockNotEnabled => {
                    write!(f, "inline uniform blocks used, but the `vk_14::inline_uniform_block` feature is not enbaled")
                },
                Self::InvalidInlineUniformBlock { name, found_type } => {
                    write!(f, "cannot convert descriptor `{found_type:?}` to `InlineUniformBlock` for descriptor binding `{name}`")
                },
                Self::UnresolvedStructSize { resource_name } => {
                    write!(f, "couldn't resolve struct size for resource `{resource_name}`")
                },
                Self::StructSizeTooBig { resource_name, resolved_size } => {
                    write!(f, "the size `{resolved_size}` of the block of resource `{resource_name}` is more than `u32::MAX`")
                },
                Self::InlineUniformBlockTooBig { resource_name, resolved_size, max_size } => {
                    write!(f,
                        "the size `{resolved_size}` of `InlineUniformBlock` binding `{resource_name}` is grater than {}",
                        format_args!("the maximum {max_size}"),
                    )
                },
                Self::TooManyDescriptors { count, max_count, descriptor_types } => {
                    write!(f, "too many descriptors of type `{descriptor_types:?}` in pipeline layout: {}",
                        format_args!("maximum is `{max_count}`, but count is `{count}`")
                    )
                },
                Self::TooManyDescriptorsInStage { count, max_count, stage, descriptor_types } => {
                    write!(f, "too many descriptors of type `{descriptor_types:?}` in shader stage `{stage:?}: {}",
                        format_args!("maximum is `{max_count}`, but count is `{count}`")
                    )
                },
                Self::MaxInlineUniformTotalSizeExceeded { max_total_size, total_size } => {
                    write!(f,
                        "maximum total size `{max_total_size}` of inline uniform blocks accross all stages exceeded: {}",
                        format_args!("caluclated size is `{total_size}`"),
                    )
                },
                Self::TooManyResourcesInStage { count, max_count, stage } => {
                    write!(f,
                        "too many descriptors in shader stage `{stage:?}`: the maximum is `{max_count}`, but the count is `{count}`",
                    )
                },
                Self::InvalidPushConstantOffset { offset, stage } => {
                    write!(f, "the offset of the push constant in stage `{stage:?}` (`{offset}`) is not a multiple of 4")
                },
                &Self::InvalidPushConstantSize { size, stage } => {
                    if size == 0 {
                        write!(f, "the size of the push constant in stage `{stage:?}` is zero")
                    } else {
                        write!(f, "the size of the push constant in stage `{stage:?}` (`{size}`) is not a multiple of `4`")
                    }
                },
                &Self::PushConstantOutOfRange { offset, size, max_size, stage } => {
                    write!(f,
                        "the push constant in stage `{stage:?}` is out of range of the maximum push constant size `{max_size}` {}",
                        format_args!("with offset `{offset}` and size `{size}`")
                    )
                },
                Self::PushDescriptorNotEnabled => {
                    write!(f, "push descriptor used, but the `vk_14::push_descriptor` feature is not enabled")
                },
                Self::TooManyPushDescriptors { count, max_count } => {
                    write!(f, "too many push descriptor declared: the maximum is `{max_count}`, but the count is `{count}`")
                },
                Self::InvalidPushDescriptorType { descriptor_type } => {
                    write!(f, "push descriptor binding has an invalid descriptor type `{descriptor_type:?}`")
                },
            }
        }
    }

    impl error::Error for PipelineLayoutError {

        #[inline]
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            __variant_some!(Self::Internal = self)
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! err {
    ($($t:tt)*) => {
        $crate::Error::new::<$crate::Error>(None, format!($($t)*))
    };
}

pub use prelude::*;
