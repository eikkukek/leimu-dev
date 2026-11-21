mod capability;

use core::{
    ptr::NonNull,
    alloc::Layout,
    slice,
};

use spirv_tools::val;

use val::Validator;

use leimu_spirv::{op, WordExt};

use smallvec::SmallVec;

use crate::*;

use sync::{
    atomic::{self, AtomicU64},
};

use mem::pack::{pack_alloc, pack_block};

use capability::*;

struct QueryArgs<'a> {
    module: leimu_spirv::Module<'a>,
    atomic_inst_ptr_types: SmallVec<[(op::Code, op::IdRef); 6]>,
}

type FpQuery = fn(&Device, &mut QueryArgs<'_>) -> Result<(), String>;

/// Specifies parameters for creating an [`ShaderModule`].
///
/// This **can** be used to create a [`ShaderModule`] with [`Device::create_shader_module`].
pub struct ShaderModuleAttributes<'a> {
    /// The debug label of this shader module.
    pub label: Option<&'a str>,
    /// The SPIR-V bytecode of this shader module.
    ///
    /// # Valid usage
    /// Valid usage of SPIR-V for Vulkan is described in detail in the Vulkan docs. The bytecode
    /// is validated through the [`spirv_tools`] crate with additional validation done in the
    /// reflection stage.
    pub spirv: &'a [u32],
}

impl ShaderModuleAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`ShaderModule`] will allocate.
    #[inline]
    pub fn calculate_allocation_size(&self) -> usize {
        size_of_val(self.spirv)
    }
}

struct Inner {
    device: Device,
    ref_count: AtomicU64,
    pack_layout: Layout,
}

/// A SPIR-V shader module.
pub struct ShaderModule {
    inner: NonNull<Inner>,
    spirv: NonNull<[u32]>,
    label: Option<NonNull<[u8]>>,
    allocator: AnyAllocator,
}

impl ShaderModule {

    pub(crate) fn new(
        device: Device,
        attributes: ShaderModuleAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Self, ShaderValidationError> {
        if let Some(label) = attributes.label {
            log::info!("validating parameters for `ShaderModule` `{label}`");
        }
        let version =
            device
            .gpu_info()
            .api_version;
        let target_env =
            if version >= VULKAN_API_VERSION_1_4 {
                spirv_tools::TargetEnv::Vulkan_1_4
            } else if version >= VULKAN_API_VERSION_1_3 {
                spirv_tools::TargetEnv::Vulkan_1_3
            } else if version >= VULKAN_API_VERSION_1_2 {
                spirv_tools::TargetEnv::Vulkan_1_2
            } else if version >= VULKAN_API_VERSION_1_1 {
                spirv_tools::TargetEnv::Vulkan_1_1
            } else {
                spirv_tools::TargetEnv::Vulkan_1_0
            };
        let validator = val::create(Some(target_env));
        let opts = val::ValidatorOptions {
            relax_struct_store: false,
            relax_logical_pointer: false,
            before_legalization: false,
            relax_block_layout: None,
            uniform_buffer_standard_layout: device
                .get_feature(vk_12::uniform_buffer_standard_layout::NAME)
                .is_some(),
            scalar_block_layout: device
                .get_feature(vk_12::scalar_block_layout::NAME)
                .is_some(),
            skip_block_layout: false,
            max_limits: vec![],
        };
        validator
            .validate(attributes.spirv, Some(opts))?;
        let mut module = leimu_spirv::Module
            ::new(attributes.spirv)
            .expect("failed to create `leimu_spirv::Module`");
        let mut capabilities: SmallVec<[op::Capability; 7]> = SmallVec::new();
        let mut atomic_inst_ptr_types = SmallVec::new();
        let mut non_uniform_rotate = SmallVec::<[_; 13]>::new();
        module.parse_full_capture_inst(|module, mut inst| {
            if inst.code() == op::Code::CAPABILITY {
                capabilities.push(op::Capability::parse_one(&mut inst).unwrap());
                inst.reset();
            }
            let code = inst.code();
            if let Some(mut result) = if matches!(code,
                op::Code::ATOMIC_LOAD
                | op::Code::ATOMIC_EXCHANGE
                | op::Code::ATOMIC_COMPARE_EXCHANGE
                | op::Code::ATOMIC_IINCREMENT
                | op::Code::ATOMIC_IDECREMENT
                | op::Code::ATOMIC_IADD
                | op::Code::ATOMIC_ISUB
                | op::Code::ATOMIC_SMIN
                | op::Code::ATOMIC_UMIN
                | op::Code::ATOMIC_SMAX
                | op::Code::ATOMIC_UMAX
                | op::Code::ATOMIC_AND
                | op::Code::ATOMIC_OR
                | op::Code::ATOMIC_XOR
                | op::Code::ATOMIC_FMIN_EXT
                | op::Code::ATOMIC_FMAX_EXT
                | op::Code::ATOMIC_FADD_EXT
            ) {
                inst.advance(2).ok();
                let pointer = op::IdRef::parse_one(&mut inst).unwrap();
                inst.reset();
                Some(module.get_result(pointer).unwrap())
            } else if inst.code() == op::Code::ATOMIC_STORE {
                let pointer = op::IdRef::parse_one(&mut inst).unwrap();
                Some(module.get_result(pointer).unwrap())
            } else {
                None
            } {
                assert!(result.code().get_inst_info().has_id_result_type);
                atomic_inst_ptr_types.push((code, op::IdRef::parse_one(&mut result).unwrap()));
            }
            if code == op::Code::GROUP_NON_UNIFORM_ROTATE_KHR {
                inst.advance(1).ok();
                non_uniform_rotate.push(op::IdRef::parse_one(&mut inst).unwrap());
                inst.reset();
            }
        }).unwrap();
        let mut query_args = QueryArgs {
            module,
            atomic_inst_ptr_types,
        };
        for capability in capabilities {
            if let Some(query) = CAPABILITY_QUERIES.get(&capability) {
                (query)(&device, &mut query_args).map_err(|message| ShaderValidationError::CapabilityNotSupported {
                    capability,
                    message: Some(message)
                })?;
            } else {
                Err(ShaderValidationError::CapabilityNotSupported { capability, message: None })?
            }
        }
        for &(code, type_id) in query_args.atomic_inst_ptr_types
            .iter().filter(|&&(code, _)| !matches!(
                code,
                op::Code::ATOMIC_FADD_EXT | op::Code::ATOMIC_FMAX_EXT | op::Code::ATOMIC_FMIN_EXT
            ))
        {
            let mut result_type = query_args.module
                .get_result(type_id)
                .unwrap();
            result_type.advance(1).ok();
            let storage_class = op::StorageClass::parse_one(&mut result_type).unwrap();
            let mut ty = query_args.module
                .get_result(op::IdRef::parse_one(&mut result_type).unwrap())
                .unwrap();
            if ty.code() == op::Code::TYPE_FLOAT
            {
                if !matches!(code,
                    op::Code::ATOMIC_EXCHANGE
                    | op::Code::ATOMIC_LOAD
                    | op::Code::ATOMIC_STORE
                ) {
                    Err(format!(
                        "the atomic operation `{code}` is not supported for floating-point values",
                    ))?
                }
                ty.advance(1).ok();
                let width = ty.read().unwrap();
                if width == 16 {
                    Err("atomic operations on 16-bit floating-point memory are not supported"
                        .to_string()
                    )?
                } else if width == 32 {
                    match storage_class {
                        op::StorageClass::WORKGROUP => {
                            if !device
                                .is_feature_enabled(vk_ext::shader_shared_float32_atomics::NAME)
                            {
                                Err(format!(
                                    "`{code}` used on 32-bit floating-point `Workgroup` storage class memory, but the {}",
                                    "`vk_ext::shader_shared_float32_atomics` feature is not enabled"
                                ))?
                            }
                        },
                        op::StorageClass::IMAGE => {
                            if !device
                                .is_feature_enabled(vk_ext::shader_image_float32_atomics::NAME)
                            {
                                Err(format!(
                                    "`{code}` used on 32-bit floating-point `Image` storage class memory, but the {}",
                                    "`vk_ext::shader_image_float32_atomics` feature is not enabled"
                                ))?
                            }
                        },
                        _ => if !device
                            .is_feature_enabled(vk_ext::shader_buffer_float32_atomics::NAME)
                        {
                            Err(format!(
                                "`{code}` used on 32-bit floating-point buffer memory, but the {}",
                                "`vk_ext::shader_buffer_float32_atomics` feature is not enabled",
                            ))?
                        }
                    }
                } else if width == 64 {
                    match storage_class {
                        op::StorageClass::WORKGROUP => {
                            if !device
                                .is_feature_enabled(vk_ext::shader_shared_float64_atomics::NAME)
                            {
                                Err(format!(
                                    "`{code}` used on 64-bit floating-point `Workgroup` storage class memory, but the {}",
                                    "`vk_ext::shader_shared_float64_atomics` feature is not enabled"
                                ))?
                            }
                        },
                        op::StorageClass::IMAGE => {
                            Err(format!(
                                "`{code}` used on 64-bit floating-point `Image` storage class memory, which is not supported"
                            ))?
                        },
                        _ => if !device
                            .is_feature_enabled(vk_ext::shader_buffer_float64_atomics::NAME)
                        {
                            Err(format!(
                                "`{code}` used on 64-bit floating-point buffer memory, but the {}",
                                "`vk_ext::shader_buffer_float64_atomics` feature is not enabled",
                            ))?
                        }
                    }
                } else {
                    panic!("unsupported floating-point width `{width}")
                }
            }
        }
        if !device
            .is_feature_enabled(vk_14::shader_subgroup_rotate_clustered::NAME)
        {
            for id_ref in non_uniform_rotate {
                let mut inst = query_args.module
                    .get_result(id_ref)
                    .unwrap();
                // skip to `ClusterSize` operand
                inst.advance(5).ok();
                if inst.read().is_ok() {
                    Err(
                        "the `vk_14::shader_subgroup_rotate::clustered` feature is not enabled, but `OpGroupNonUniformRotateKHR` has the `ClusterSize` operand specified"
                        .to_string()
                    )?
                }
            }
        }
        let mut inner = NonNull::dangling();
        let mut label_ptr = None;
        let (pack_allocation, mut spirv) = 
            if let Some(label) = attributes.label {
                let mut no_slice_spirv = NonNull::dangling();
                let mut no_slice_label = NonNull::dangling();
                let allocation = pack_alloc(
                    &allocator,
                    [
                        pack_block(&mut inner, 1).unwrap(),
                        pack_block(&mut no_slice_spirv, attributes.spirv.len()).unwrap(),
                        pack_block(&mut no_slice_label, label.len()).unwrap(),
                    ],
                ).expect("allocation failed");
                let spirv = NonNull::slice_from_raw_parts(
                    no_slice_spirv,
                    attributes.spirv.len()
                );
                unsafe {
                    label.as_ptr()
                        .copy_to_nonoverlapping(
                            no_slice_label.as_ptr(),
                            label.len(),
                        );
                    label_ptr = Some(NonNull::slice_from_raw_parts(
                        no_slice_label,
                        label.len()
                    ))
                }
                (allocation, spirv)
            } else {
                let mut no_slice_spirv = NonNull::dangling();
                let allocation = pack_alloc(
                    &allocator,
                    [
                        pack_block(&mut inner, 1).unwrap(),
                        pack_block(&mut no_slice_spirv, attributes.spirv.len()).unwrap(),
                    ],
                ).expect("allocation failed");
                let spirv = NonNull::slice_from_raw_parts(
                    no_slice_spirv,
                    attributes.spirv.len()
                );
                (allocation, spirv)
            };
        assert!(pack_allocation.0.addr() == inner.addr());
        unsafe {
            inner.write(Inner {
                device,
                ref_count: AtomicU64::new(1),
                pack_layout: pack_allocation.1,
            });
            spirv.as_mut()
                .copy_from_slice(attributes.spirv);
        }
        if let Some(label) = attributes.label {
            log::info!("created `ShaderModule` `{label}`")
        }
        Ok(Self {
            inner,
            spirv,
            label: label_ptr, 
            allocator,
        })
    }
    
    #[inline]
    unsafe fn inner(&self) -> &Inner {
        unsafe {
            self.inner.as_ref()
        }
    }

    /// Gets the [`Device`] used to create this module.
    #[inline]
    pub fn device(&self) -> &Device {
        unsafe {
            &self.inner().device
        }
    }

    /// Gets the debug label of this module.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.map(|ptr| unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(
                    ptr.cast().as_ptr(),
                    ptr.len(),
                )
            )
        })
    }

    /// Gets the SPIR-V bytecode used to create this module.
    #[inline]
    pub fn spirv(&self) -> &[u32] {
        unsafe {
            slice::from_raw_parts(
                self.spirv.cast().as_ptr(),
                self.spirv.len(),
            )
        }
    }
}

impl Clone for ShaderModule {
    
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner.as_ref().ref_count
                .add(1, atomic::Ordering::Release);
        }
        Self {
            inner: self.inner,
            spirv: self.spirv,
            label: self.label,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for ShaderModule {

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.inner.as_ref()
                .ref_count
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}
