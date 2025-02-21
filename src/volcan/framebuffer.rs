use ash::vk;

use super::init::Volcan;

impl Volcan {
    pub fn create_framebuffers(&mut self) {
        let framebuffers: Vec<vk::Framebuffer> = self
            .swapchain_image_views
            .iter()
            .map(|&image_view| {
                let image_views = [image_view];

                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(*self.render_pass)
                    .attachments(&image_views)
                    .width(self.swapchain_extents.width)
                    .height(self.swapchain_extents.height)
                    .render_pass(*self.render_pass)
                    .layers(1);

                unsafe {
                    self.device
                        .create_framebuffer(&framebuffer_info, None)
                        .expect("Cannot create Framebuffer")
                }
            })
            .collect();

        self.framebuffers.set(framebuffers);
        println!("Framebuffers: {:?}", *self.framebuffers);
    }
}
