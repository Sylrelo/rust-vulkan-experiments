use ash::{khr, vk};

use crate::unwraped_option::UnwrappedOption;

use super::init::Volcan;

struct VolcanSwapchain {
    //TODO: Move all swapchain_* variables from Volcan here
}

impl Volcan {
    pub fn create_swapchain(&mut self, window_width: u32, window_height: u32) {
        let capabilities = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .expect("Failed to get surface capabilities")
        };

        /* --------------------------------- FORMAT --------------------------------- */

        let formats = unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(self.physical_device, self.surface)
                .expect("Failed to get surface formats")
        };

        let surface_format = formats
            .iter()
            .cloned()
            .find(|sfmt| sfmt.format == vk::Format::B8G8R8A8_UNORM)
            .unwrap_or(formats[0]);

        /* ------------------------------ PRESENT MODE ------------------------------ */

        let present_modes = unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(self.physical_device, self.surface)
                .expect("Failed to get present modes")
        };

        let present_mode = present_modes
            .into_iter()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        /* ---------------------------------- SIZE ---------------------------------- */

        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: window_width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        /* -------------------------------- SWAPCHAIN ------------------------------- */

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain_loader = khr::swapchain::Device::new(&self.instance, &self.device);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swapchain.")
        };

        self.swapchain_loader = UnwrappedOption(Some(swapchain_loader));
        self.swapchain = UnwrappedOption(Some(swapchain));
        self.swapchain_format = UnwrappedOption(Some(surface_format.format));
        self.swapchain_extents.set(extent);
    }

    pub fn create_swapchain_images(&mut self) {
        let swapchain_images = unsafe {
            self.swapchain_loader
                .get_swapchain_images(*self.swapchain)
                .expect("Failed to get swapchain images")
        };

        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(*self.swapchain_format) // store this when creating the swapchain
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe {
                    self.device
                        .create_image_view(&create_info, None)
                        .expect("Failed to create image view")
                }
            })
            .collect();

        println!("{:?}", swapchain_image_views);
        self.swapchain_images = UnwrappedOption(Some(swapchain_images));
        self.swapchain_image_views = UnwrappedOption(Some(swapchain_image_views));
    }
}
