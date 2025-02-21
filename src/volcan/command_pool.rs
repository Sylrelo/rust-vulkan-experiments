use ash::vk;

use super::init::Volcan;

impl Volcan {
    pub fn create_command_pool(&mut self) {
        let command_pool_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(self.queue_index);

        let command_pool = unsafe {
            self.device
                .create_command_pool(&command_pool_info, None)
                .expect("Cannot create command pool.")
        };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.framebuffers.len() as u32);

        let command_buffers = unsafe {
            self.device
                .allocate_command_buffers(&alloc_info)
                .expect("Cannot allocate command buffer.")
        };

        self.command_buffers.set(command_buffers);
        println!("Command buffers: {:?}", *self.command_buffers)
    }
}
