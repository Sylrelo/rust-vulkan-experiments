use std::{ffi::CString, sync::Arc};

use ash::vk;

use super::{init::Volcan, shader_modules::VolcanShaderModule};

pub struct VolcanPipeline {
    // _volcan: Arc<Volcan>,
}

pub struct VolcanPipelineData {}

impl VolcanPipeline {
    // pub fn init(volcan: Arc<Volcan>) -> Self {
    //     Self {
    //         _volcan: volcan.clone(),
    //     }
    // }

    pub fn create_raster_pipeline(
        device: ash::Device,
        render_pass: vk::RenderPass,
    ) -> vk::Pipeline {
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Cannot create Pipeline.")
        };

        /* ------------------------------ SHADER STAGE ------------------------------ */

        let vert_shader_module =
            VolcanShaderModule::create_shader(&device, "./shaders/dist/basic_triangle.vert.spv");
        let frag_shader_module =
            VolcanShaderModule::create_shader(&device, "./shaders/dist/basic_triangle.frag.spv");

        let entry_point = CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        /* -------------------------------------------------------------------------- */

        let extent = vk::Extent2D {
            width: 1920,
            height: 1080,
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: extent,
        };

        let viewport_binding = [viewport];
        let scissor_binding = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewport_binding)
            .scissors(&scissor_binding);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blend_attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let graphics_pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        }
        .expect("Cannot create graphic pipeline")
        .remove(0);

        println!("graphics_pipeline: {:?}", graphics_pipeline);

        return graphics_pipeline;
    }

    pub fn create_raytracing_pipeline(instance: &ash::Instance, device: &ash::Device) {
        /* ------------------------------ SHADER STAGE ------------------------------ */

        let entry_point = CString::new("main").unwrap();

        let raygen_module =
            VolcanShaderModule::create_shader(&device, "./shaders/dist/raygen.rgen.spv");
        let raymiss_module =
            VolcanShaderModule::create_shader(&device, "./shaders/dist/raymiss.rmiss.spv");
        let rayhit_module =
            VolcanShaderModule::create_shader(&device, "./shaders/dist/rayhit.rchit.spv");
        // let intersection_module =
        // VolcanShaderModule::create_shader(&device, "./shaders/dist/intersection.rint.spv");

        let raygen_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::RAYGEN_KHR)
            .module(raygen_module)
            .name(&entry_point);

        let miss_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MISS_KHR)
            .module(raymiss_module)
            .name(&entry_point);

        let chit_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
            .module(rayhit_module)
            .name(&entry_point);

        // let intersection_stage = vk::PipelineShaderStageCreateInfo::default()
        //     .stage(vk::ShaderStageFlags::INTERSECTION_KHR)
        //     .module(intersection_stage)
        //     .name(&entry_point);

        let shader_stages = [
            raygen_stage,
            miss_stage,
            chit_stage,
            //intersection_stage
        ];

        // let procedural_hit_group = vk::RayTracingShaderGroupCreateInfoKHR::default()
        //     .ty(vk::RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP)
        //     .general_shader(vk::SHADER_UNUSED_KHR)
        //     .closest_hit_shader(2)
        //     .any_hit_shader(vk::SHADER_UNUSED_KHR)
        //     .intersection_shader(3);

        let raygen_group = vk::RayTracingShaderGroupCreateInfoKHR::default()
            .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
            .general_shader(0) // index in shader_stages array
            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR);

        let miss_group = vk::RayTracingShaderGroupCreateInfoKHR::default()
            .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
            .general_shader(1)
            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR);

        let hit_group = vk::RayTracingShaderGroupCreateInfoKHR::default()
            .ty(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
            .general_shader(vk::SHADER_UNUSED_KHR)
            .closest_hit_shader(2)
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR);

        let shader_groups = [
            raygen_group,
            miss_group,
            hit_group,
            //  procedural_hitgroup
        ];

        /* -------------------------------- PIPELINE -------------------------------- */

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let pipeline_info = vk::RayTracingPipelineCreateInfoKHR::default()
            .stages(&shader_stages)
            .groups(&shader_groups)
            .max_pipeline_ray_recursion_depth(1) // Adjust as needed for recursion
            .layout(pipeline_layout);

        let ray_tracing_pipeline_loader =
            ash::khr::ray_tracing_pipeline::Device::new(instance, device);

        let ray_tracing_pipeline = unsafe {
            ray_tracing_pipeline_loader
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    &[pipeline_info],
                    None,
                )
                .expect("Failed to create ray tracing pipeline")
                .remove(0)
        };

        println!("Raytracing pipeline: {:?}", ray_tracing_pipeline);
    }
}
