use super::*;

use sync::{RwLock, RwLockReadGuard, MappedRwLockReadGuard, LazyLock};

use tuhka::{ext, khr};

#[doc(hidden)]
pub trait DeviceFunctions {
    type Get<'a>;
}

pub(crate) trait DeviceFunctionsGet<'a> {
    type Fn;
    fn get(self) -> MappedRwLockReadGuard<'a, Self::Fn>;
}

/// An opaque handle to a the functions of a feature's extension.
pub struct ExtensionFunctions<T: DeviceFunctions>(pub(super) for<'a> fn(&'a Device) -> T::Get<'a>);

impl<'a, T: DeviceFunctions> ExtensionFunctions<T>
{
    pub(crate) fn get(
        &'a self,
        device: &'a Device
    ) -> impl ::core::ops::Deref<Target = <T::Get<'a> as DeviceFunctionsGet<'a>>::Fn>
        where T::Get<'a>: DeviceFunctionsGet<'a>
    {
        (self.0)(device).get()
    }
}

macro_rules! impl_fns {
    ($([
        static $c:ident: $fn:ty;
        fn $get_fn:ident;
        struct $feature_fn:ident;
    ])*) => {
        $(
        static $c: LazyLock<RwLock<Option<$fn>>> = LazyLock::new(||
            RwLock::new(None)
        );

        pub(super) fn $get_fn(device: &Device) -> $feature_fn<'_> {
            let mut guard = Some($c.read());
            if (unsafe { guard.as_ref().unwrap_unchecked() }).as_ref().is_none() {
                guard.take();
                let mut write_guard = $c.write();
                *write_guard = Some(<$fn>::new(device.functions()));
                drop(write_guard);
                guard = Some($c.read());
            }
            unsafe {
                $feature_fn {
                    lock: RwLockReadGuard::map(guard
                        .unwrap_unchecked(),
                        |f| {
                            f.as_ref().unwrap_unchecked()
                        }
                    )
                }
            }
        }
        pub struct $feature_fn<'a> {
            lock: MappedRwLockReadGuard<'a, $fn>,
        }
        impl DeviceFunctions for $feature_fn<'_> {
            type Get<'a> = $feature_fn<'a>;
        }
        impl<'a> DeviceFunctionsGet<'a> for $feature_fn<'a> {

            type Fn = $fn;
            #[inline]
            fn get(self) -> MappedRwLockReadGuard<'a, Self::Fn> {
                self.lock
            }
        }
        )*
        pub(crate) fn clear_devices() {
            $(
                *$c.write() = None;
            )*
        }
    };
}

impl_fns! {
    [
        static SWAPCHAIN_FN: khr::swapchain::Device;
        fn get_swapchain_fn;
        struct SwapchainFn;
    ]
    [
        static MESH_SHADER_FN: ext::mesh_shader::Device;
        fn get_mesh_shader_fn;
        struct MeshShaderFn;
    ]
    [
        static EXTENDED_DYNAMIC_STATE2_FN: ext::extended_dynamic_state2::Device;
        fn get_extended_dynamic_state2_fn;
        struct ExtendedDynamicState2Fn;
    ]
    [
        static EXTENDED_DYNAMIC_STATE3_FN: ext::extended_dynamic_state3::Device;
        fn get_extended_dynamic_state3_fn;
        struct ExtendedDynamicState3Fn;
    ]
    [
        static VERTEX_INPUT_DYNAMIC_STATE_FN: ext::vertex_input_dynamic_state::Device;
        fn get_vertex_input_dynamic_state_fn;
        struct VertexInputDynamicStateFn;
    ]
}
