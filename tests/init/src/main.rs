use leimu::{
    DeviceAttributes, Features, Instance, SamplerFlags, vk_core,
};

fn main() -> Result<(), leimu::Error> {
    env_logger::init();
    let instance = Instance::new_standalone()?;
    let suitable_gpus = instance
        .find_suitable_gpus(DeviceAttributes {
            required_features: Features::from_iter([
                vk_core::push_descriptor::new(),
                vk_core::shader_zero_initialize_workgroup_memory::new(),
                vk_core::extended_dynamic_state2::builder()
                    .extended_dynamic_state2(true)
                    .build(),
                vk_core::sampler_anisotropy::new(true),
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
        queue_labels: vec!["graphics queue"],
    };
    let compute_queue_attr =    
        (graphics_family.index != compute_family.index).then(||
            leimu::DeviceQueueAttributes {
                queue_family_index: graphics_family.index,
                queue_priorities: vec![0.0],
                queue_labels: vec!["compute queue"],
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
        label: Some("my sampler"),
        flags: SamplerFlags::ANISOTROPY_ENABLE,
        mag_filter: leimu::SamplerFilter::Linear,
        min_filter: leimu::SamplerFilter::Linear,
        mipmap_mode: leimu::SamplerMipmapMode::Linear,
        max_anisotropy: 16.0,
        wrapping_mode: [leimu::SamplerWrappingMode::ClampToEdge; 3],
        ..Default::default()
    })?;
    let buffer = device.create_buffer(
        leimu::BufferAttributes {
            label: Some("my buffer"),
            size: 64,
            usage: leimu::BufferUsages::TRANSFER_SRC,
            memory_type: leimu::MemoryType::DeviceLocal
        },
        (),
    )?.clone();
    let _clone = buffer.clone();
    println!("{:?}", buffer.label());
    Ok(())
}
