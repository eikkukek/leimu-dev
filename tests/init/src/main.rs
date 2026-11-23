use std::alloc::Global;

use core::error::Error as StdError;

use leimu::{
    bytemuck::cast_slice,
    vk_core,
};

fn main() -> Result<(), leimu::Error> {
    env_logger::init();
    let instance = leimu::Instance::new_standalone()?;
    let suitable_gpus = instance
        .find_suitable_gpus(leimu::DeviceAttributes {
            required_features: leimu::Features::from_iter([
                vk_core::push_descriptor::require(),
                vk_core::shader_zero_initialize_workgroup_memory::require(),
                vk_core::extended_dynamic_state2::require(),
                vk_core::sampler_anisotropy::require(),
                vk_core::image_cube_array::require(),
            ]),
        })?;
    let gpu = suitable_gpus.gpus()
        .find(|gpu| gpu.gpu_type == leimu::GpuType::Discrete)
        .unwrap_or(suitable_gpus.get_gpu(0));
    let graphics_family = gpu.queue_families
        .iter()
        .find(|q| q.flags.contains(leimu::QueueFlags::GRAPHICS))
        .expect("failed to find graphics queue");
    let compute_family = gpu.queue_families
        .iter()
        .find(|q| q.flags.contains(leimu::QueueFlags::COMPUTE))
        .expect("failed to find compute queue");
    let graphics_queue_attr = leimu::DeviceQueueAttributes {
        queue_family_index: graphics_family.index,
        queue_priorities: vec![0.0],
        queue_tags: vec!["graphics queue"],
    };
    let compute_queue_attr =    
        (graphics_family.index != compute_family.index).then(||
            leimu::DeviceQueueAttributes {
                queue_family_index: graphics_family.index,
                queue_priorities: vec![0.0],
                queue_tags: vec!["compute queue"],
            }
        );
    let queue_aliases = if compute_queue_attr.is_none() {
        [("graphics queue", "compute queue")].as_slice()
    } else { [].as_slice() };
    let device = suitable_gpus.create_device(
        gpu.index,
        if let Some(q) = compute_queue_attr {
            vec![graphics_queue_attr, q]
        } else {
            vec![graphics_queue_attr]
        },
        queue_aliases.iter().copied(),
    )?;
    assert!(device
        .get_queue("graphics queue")
        .is_some()
    );
    assert!(device
        .get_queue("compute queue")
        .is_some()
    );
    let _sampler = device.create_sampler(leimu::SamplerAttributes {
        tag: Some("my sampler"),
        flags: leimu::SamplerFlags::ANISOTROPY_ENABLE,
        mag_filter: leimu::SamplerFilter::Linear,
        min_filter: leimu::SamplerFilter::Linear,
        mipmap_mode: leimu::SamplerMipmapMode::Linear,
        max_anisotropy: 16.0,
        wrapping_mode: [leimu::SamplerWrappingMode::ClampToEdge; 3],
        ..Default::default()
    })?;
    let buffer = device.create_buffer(
        leimu::BufferAttributes {
            tag: Some("my buffer"),
            size: 64,
            usage: leimu::BufferUsages::TRANSFER_SRC,
            memory_type: leimu::MemoryType::HostVisible,
        },
        None,
    )?.clone();
    let _clone = buffer.clone();
    buffer.map(leimu::MapMode::WriteOnly, |buf| {
        if let Ok(mut buf) = buf {
            assert!(buf.read_only(..).is_err());
            let slice = buf.write_only_slice(..)?;
            let (mut slice, _) = slice
                .try_cast_slice::<f32>()?
                .subslice(0..8)
                .into_chunks::<4>();
            slice.copy_from_slice(&[
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
            ]);
        }
        Ok(())
    });
    _clone.map(leimu::MapMode::ReadOnly, |buf| {
        if let Ok(mut buf) = buf {
            assert!(buf.write_only::<f32>(0).is_err());
            assert_eq!(
                cast_slice::<u8, f32>(buf.read_only(0..4*4*2).unwrap())
                    .as_chunks::<4>().0,
                [[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]]
            );
        }
        Ok(())
    });
    assert_eq!(_clone.tag(), Some("my buffer"));
    let view_formats = [
        leimu::Format::Bc1_Rgb_Unorm_Block,
    ];
    match device.create_image(
        leimu::ImageAttributes {
            tag: Some("my image"),
            format: leimu::Format::Bc1_Rgb_Srgb_Block,
            usage: leimu::ImageUsages::SAMPLED,
            array_layers: 6,
            extent: leimu::ImageExtent::default(),
            samples: leimu::SampleCountBit::X1,
            view_formats: &view_formats,
            ..Default::default()
        },
        None,
    ) {
        Ok(img) => {
            let img = img.clone().clone().clone();
            assert_eq!(Some("my image"), img.tag());
        }
        Err(err) => {
            if let Some(src) = err.source() {
                println!("{}", src);
            } else {
                println!("{}", err);
            }
        },
    }
    let arena = leimu::mem::arena::Arena::new(1 << 16).unwrap();
    let cube_image = device.create_image(
        leimu::ImageAttributes  {
            tag: Some("cube image"),
            format: leimu::Format::R8g8b8a8_Srgb,
            usage: leimu::ImageUsages::SAMPLED,
            array_layers: 12,
            extent: leimu::ImageExtent::default()
                .width(256)
                .height(256),
            view_compatibility: leimu::ImageViewCompatibility::CUBE,
            ..Default::default()
        },
        Some(leimu::any_allocator(arena)),
    )?;
    let _cube_view = cube_image.create_view(
        leimu::ImageViewAttributes {
            tag: Some("cube view"),
            format: None,
            dimensions: leimu::ImageViewDim::Cube,
            range: leimu::ImageSubresourceRange::default()
                .layers(0..6),
            component_mapping: Default::default(),
        },
        None,
    )?;
    let image_3d = device.create_image(
        leimu::ImageAttributes {
            tag: Some("3d image"),
            format: leimu::Format::R8g8b8a8_Srgb,
            usage: leimu::ImageUsages::SAMPLED,
            dimensions: leimu::ImageDim::Dim3,
            extent: leimu::ImageExtent::default()
                .map(|_| 256),
            view_compatibility: leimu::ImageViewCompatibility::DIM3_SLICE_AS_ARRAY,
            mip_levels: 2,
            ..Default::default()
        },
        None,
    )?;
    let view_3d_slice = image_3d.create_view(
        leimu::ImageViewAttributes {
            tag: Some("2d array of 3d"),
            format: Some(leimu::Format::R8g8b8a8_Srgb),
            dimensions: leimu::ImageViewDim::Dim2Array,
            range: leimu::ImageSubresourceRange::default()
                .levels(0..1)
                .layers(0..64),
            component_mapping: Default::default(),
        },
        None,
    )?;
    let view_3d_value = image_3d.create_view(
        leimu::ImageViewAttributes {
            tag: Some("2d slice of 3d"),
            format: Some(leimu::Format::R8g8b8a8_Srgb),
            dimensions: leimu::ImageViewDim::Dim2,
            range: leimu::ImageSubresourceRange::default()
                .levels(0..1)
                .layers(0..1),
            component_mapping: Default::default(),
        },
        None,
    )?;
    let view = view_3d_value;
    println!("{}", view.tag().unwrap());
    Ok(())
}
