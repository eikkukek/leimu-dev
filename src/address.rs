/// The size type of GPU buffers.s
///
/// May differ from [`usize`] on platforms that are not 64-bit.
pub type DeviceSize = u64;

/// A [`Device`][crate::Device] address type, which **can** be used in a shader to access a buffer's memory 
/// through a `PhysicalStorageBuffer` storage class pointer.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShaderDeviceAddress(pub(crate) u64);
