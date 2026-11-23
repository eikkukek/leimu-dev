use leimu::{
    DescribeResult,
    mem::arena::Arena,
    raw_window_handle::HasDisplayHandle,
    include_spirv,
};

use cwinit::{run_app, Event, winit};

use winit::{window, event};

fn main() -> Result<(), leimu::Error> {
    env_logger::init();
    if let Some(res) = run_app::<_, leimu::Error>(async |mut event_loop| {
        let instance = leimu::Instance
            ::new_in_display(
                event_loop.display_handle().expect("failed to get display handle")
                    .as_raw()
            )?;
        let window = event_loop.create_window(window::WindowAttributes::default()
            .with_title("test")
        ).describe_err("failed to create window")?;
        let surface = instance.create_surface(Some("test surface"), &window)?;
        let suitable_gpus = instance
            .find_suitable_gpus(leimu::DeviceAttributes {
                required_features: leimu::Features::from_iter([
                    leimu::vk_khr::swapchain::require(),
                    leimu::vk_core::scalar_block_layout::require(),
                    leimu::vk_core::shader_buffer_int64_atomics::require(),
                    leimu::vk_core::shader_int64::require(),
                ]),
            })?;
        let gpu = suitable_gpus.gpus()
            .find(|gpu| gpu.gpu_type == leimu::GpuType::Discrete)
            .unwrap_or(suitable_gpus.get_gpu(0));
        let present_family = suitable_gpus
            .get_gpu_present_support(gpu.index, &surface)
            .next().unwrap();
        let device = suitable_gpus.create_device(
            gpu.index,
            [leimu::DeviceQueueAttributes {
                queue_family_index: present_family.index,
                queue_priorities: vec![1.0],
                queue_tags: vec!["present queue"],
            }],
            [],
        )?;
        let compute_module = device.create_shader_module(
            leimu::ShaderModuleAttributes {
                tag: Some("compute shader"),
                spirv: include_spirv!("../comp.spv"),
            },
            None,
        )?;
        let _ = device.create_pipeline_layout(
            leimu::PipelineLayoutAttributes {
                tag: Some("compute layout"),
                ty: leimu::PipelineLayoutType
                    ::compute((compute_module, "main"))?,
                set_attributes: &[],
                binding_attributes: &[],
            },
            None,
        ).describe_err("failed to create pipeline layout")?;
        let _ = device.create_pipeline_layout(
            leimu::PipelineLayoutAttributes {
                tag: Some("graphics layout"),
                ty: leimu::PipelineLayoutType::vertex(
                    (device.create_shader_module(
                        leimu::ShaderModuleAttributes {
                            tag: None,
                            spirv: include_spirv!("../vert.spv"),
                        },
                        None
                    )?, "main"),
                    Some((device.create_shader_module(
                        leimu::ShaderModuleAttributes {
                            tag: None,
                            spirv: include_spirv!("../frag.spv")
                        },
                        None
                    )?, "main"))
                )?,
                set_attributes: &[],
                binding_attributes: &[]
            },
            None
        ).describe_err("failed to create graphics layout")?;
        let present_queue = device.get_queue("present queue").unwrap();
        let swapchain = device.create_swapchain(
            Some("swapchain"),
            surface,
            present_queue.clone(),
        )?;
        let command_pool = device.create_command_pool(
            Some("main command pool"),
            present_queue.clone(),
        )?;
        let mut command_buffer = command_pool
            .allocate_command_buffer()?
            .with_tag("render pass command buffer");
        let tmp_arena = Arena
            ::with_fallback(1 << 16)
            .unwrap();
        while event_loop.poll().await {
            if let Event::WindowEvent {
                event: event::WindowEvent::CloseRequested, ..
            } = event_loop.event() {
                break;
            }
            let window_size = window.inner_size();
            swapchain.present(
                leimu::PresentParameters {
                    window_width: window_size.width,
                    window_height: window_size.height,
                    desired_image_count: 3,
                    clip: true,
                    force_recreate: false,
                    select_surface_format: &mut |formats| {
                        formats.iter()
                            .enumerate()
                            .find_map(|(i, format)|
                                if format.color_space == leimu::ColorSpace::SRGB_NONLINEAR_KHR
                                    && format.format == leimu::Format::R8g8b8a8_Srgb
                                {
                                    Some(i)
                                } else {
                                    None
                                }
                            ).unwrap_or(0)
                    },
                    select_composite_alpha: &mut |_| leimu::CompositeAlphaBit::Opaque
                },
                std::time::Duration::from_secs_f32(1.0),
                None,
                |image_view| {
                    let mut recorder = command_buffer.start_recording(
                        leimu::duration_from_secs(1.0),
                    )?;
                    let pass = recorder.render_pass(
                        leimu::RenderPassAttributes {
                            tag: Some("clear pass"),
                            render_area: leimu::RenderArea {
                                width: window_size.width,
                                height: window_size.height,
                                ..Default::default()
                            },
                            color_attachments: &[leimu::ColorAttachment::new(
                                &image_view,
                                None,
                                leimu::AttachmentLoadOp::Clear,
                                leimu::AttachmentStoreOp::Store,
                                Some(leimu::ClearColorValue::F32([
                                    1.0,
                                    35.0 / 255.0,
                                    82.0 / 255.0,
                                    1.0,
                                ]))
                            )?],
                            depth_stencil_attachment: leimu::DepthStencilAttachment::none(),
                            ..Default::default()
                        },
                        &tmp_arena,
                    )?;
                    pass.finish();
                    recorder.finish();
                    device.poll([&mut command_buffer], Some(&tmp_arena))?;
                    unsafe {
                        tmp_arena.clear();
                    }
                    Ok(())
                }
            )?;
            window.request_redraw();
        }
        Ok(())
    }) {
        res?
    }
    Ok(())
}
