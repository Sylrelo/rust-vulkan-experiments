use std::{io::Cursor, sync::Arc};

use ash::{
    util::read_spv,
    vk::{self},
};
use log::error;

pub struct VolcanShaderModule {}

impl VolcanShaderModule {
    pub fn create_shader(device: &ash::Device, file_path: &str) -> vk::ShaderModule {
        let shader_source = match file_path {
            // "shaders/compute_old.wgsl" => {
            //     include_bytes!("../../shaders/dist/basic_triangle.vert.spv")
            // }
            // "shaders/compute.wgsl" => include_str!("../shaders/compute.wgsl"),
            // "shaders/render.wgsl" => include_str!("../shaders/render.wgsl"),
            _ => &{
                error!("{} will NOT be integrated into the binary.", file_path);
                std::fs::read(file_path).expect("Failed to read shader file")
            },
        };

        let shader_source_cursor = &mut Cursor::new(shader_source);
        let code = read_spv(shader_source_cursor).expect("Error during SPV parsing");

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Cannot create shader module")
        }
    }
}
