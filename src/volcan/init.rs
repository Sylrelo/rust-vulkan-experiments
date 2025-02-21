use std::ffi::CString;

use ash::{
    khr,
    vk::{self, PhysicalDevice, SurfaceKHR, SwapchainKHR},
    Entry, Instance,
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::unwraped_option::{Lazy, UnwrappedOption};

pub struct Volcan {
    pub(super) entry: Entry,
    pub(crate) instance: Instance,
    pub(super) physical_device: PhysicalDevice,

    pub(super) surface: SurfaceKHR,
    pub(super) surface_loader: ash::khr::surface::Instance,

    pub(super) queue_index: u32,
    pub(super) primary_queue: vk::Queue,
    pub(crate) device: ash::Device,

    pub(super) swapchain: UnwrappedOption<SwapchainKHR>,
    pub(super) swapchain_extents: Lazy<vk::Extent2D>,
    pub(super) swapchain_loader: UnwrappedOption<khr::swapchain::Device>,
    pub(super) swapchain_format: UnwrappedOption<vk::Format>,
    pub(super) swapchain_images: UnwrappedOption<Vec<vk::Image>>,
    pub(super) swapchain_image_views: UnwrappedOption<Vec<vk::ImageView>>,

    pub(crate) render_pass: Lazy<vk::RenderPass>,
    pub(super) framebuffers: Lazy<Vec<vk::Framebuffer>>,

    pub(super) command_buffers: Lazy<Vec<vk::CommandBuffer>>,

    pub(super) img_available_sem: Lazy<vk::Semaphore>,
    pub(super) render_finished_sem: Lazy<vk::Semaphore>,
    pub(super) in_flight_fence: Lazy<vk::Fence>,
}

//TODO: Clean
impl Volcan {
    // pub fn init(window: &Window) -> Self {
    //     let mut volcan = Volcan::new(window);

    // }

    pub fn new(window: &Window) -> Self {
        let entry = unsafe { Entry::load().unwrap() };

        let app_name = CString::new("Hello Vulkan").unwrap();

        let mut required_instance_ext = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::get_physical_device_properties2::NAME.as_ptr(),
        ];

        let mut required_extensions =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();

        required_instance_ext.append(&mut required_extensions);

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            required_instance_ext.push(ash::khr::portability_enumeration::NAME.as_ptr());
            required_instance_ext.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(0)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_2);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_instance_ext)
            .flags(create_flags);

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        println!("Vulkan Instance Created!");

        let (surface, surface_loader) = Self::create_surface(&entry, &instance, window);

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to get physical devices")
        };

        if physical_devices.is_empty() {
            panic!("No Vulkan-compatible GPU found!");
        }

        let mut selected_device = None;
        let mut selected_queue_index: Option<_> = None;

        'outer: for (i, &physical_device) in physical_devices.iter().enumerate() {
            let device_properties =
                unsafe { instance.get_physical_device_properties(physical_device) };

            let device_name = unsafe {
                std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr()).to_string_lossy()
            };
            println!("====== {i}: {device_name} ======");
            //             if device_properties.device_type != PhysicalDeviceType::DISCRETE_GPU {
            // continue;;
            //             }

            let mut ray_tracing_pipeline_properties =
                vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
            let mut acceleration_structure_properties =
                vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();

            let mut device_properties2 = vk::PhysicalDeviceProperties2::default();

            device_properties2 = device_properties2
                .push_next(&mut ray_tracing_pipeline_properties)
                .push_next(&mut acceleration_structure_properties);

            unsafe {
                instance.get_physical_device_properties2(physical_device, &mut device_properties2)
            };

            println!(
                "  - Shader Group Handle Size: {}",
                ray_tracing_pipeline_properties.shader_group_handle_size
            );
            println!(
                "  - Max Ray Recursion Depth: {}",
                ray_tracing_pipeline_properties.max_ray_recursion_depth
            );
            println!(
                "  - Max Ray Dispatch Invocation Count: {}",
                ray_tracing_pipeline_properties.max_ray_dispatch_invocation_count
            );

            // Check Acceleration Structure support
            println!(
                "  - Max Instance Count: {}",
                acceleration_structure_properties.max_instance_count
            );
            println!(
                "  - Max Geometry Count: {}",
                acceleration_structure_properties.max_geometry_count
            );

            let device_queue_properties =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

            let available_extensions =
                Self::get_physical_device_extensions(&instance, physical_device);

            for (j, info) in device_queue_properties.iter().enumerate() {
                let supports_graphic_and_surface = unsafe {
                    info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && info.queue_flags.contains(vk::QueueFlags::COMPUTE)
                        && surface_loader
                            .get_physical_device_surface_support(physical_device, j as u32, surface)
                            .unwrap()
                };

                println!("{} - {:?}", info.queue_count, info.queue_flags);

                if supports_graphic_and_surface
                    && available_extensions.contains(&"VK_KHR_ray_tracing_pipeline".to_string())
                    && available_extensions.contains(
                        &ash::khr::acceleration_structure::NAME
                            .to_str()
                            .unwrap()
                            .to_string(),
                    )
                {
                    // vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::get
                    selected_device = Some(physical_device);
                    selected_queue_index = Some(j as u32);
                    println!(
                        "Selected device : {device_name}, queue {:?}",
                        selected_queue_index
                    );

                    break 'outer;
                }
            }
        }

        let selected_device = selected_device.expect("No suitable GPU found.");
        let selected_queue_index = selected_queue_index.expect("No suitable queue found.");

        /* ------------------------ CREATE DEVICE AND QUEUES ------------------------ */

        let device_extension_names_raw = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::ray_tracing_pipeline::NAME.as_ptr(),
            ash::khr::ray_tracing_maintenance1::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(selected_queue_index)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .create_device(selected_device, &device_create_info, None)
                .expect("Unable to create device.")
        };
        let present_queue = unsafe { device.get_device_queue(selected_queue_index, 0) };

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        Self {
            entry,
            instance,
            physical_device: selected_device,

            surface,
            surface_loader,

            primary_queue: present_queue,
            queue_index: selected_queue_index,
            device: device,

            swapchain: UnwrappedOption(None),
            swapchain_extents: Lazy::new(),
            swapchain_loader: UnwrappedOption(None),
            swapchain_format: UnwrappedOption(None),
            swapchain_images: UnwrappedOption(None),
            swapchain_image_views: UnwrappedOption(None),

            render_pass: Lazy::new(),
            framebuffers: Lazy::new(),
            command_buffers: Lazy::new(),

            img_available_sem: Lazy::new(),
            render_finished_sem: Lazy::new(),
            in_flight_fence: Lazy::new(),
        }
    }

    fn create_surface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> (SurfaceKHR, ash::khr::surface::Instance) {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("Failed to create surface")
        };

        let surface_loader = ash::khr::surface::Instance::new(entry, instance);

        (surface, surface_loader)
    }

    fn get_instance_extensions(entry: &Entry) {
        let available_extensions = unsafe {
            entry
                .enumerate_instance_extension_properties(None)
                .expect("Failed to list Vulkan extensions.")
        };

        println!("Available Vulkan Instance Extensions:");
        for ext in available_extensions {
            let ext_name =
                unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()).to_string_lossy() };
            println!("- {} (spec version: {})", ext_name, ext.spec_version);
        }
    }

    pub fn create_fences(&mut self) {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available_semaphore = unsafe {
            self.device
                .create_semaphore(&semaphore_info, None)
                .expect("Cannot create samphore.")
        };

        let render_finished_semaphore = unsafe {
            self.device
                .create_semaphore(&semaphore_info, None)
                .expect("Cannot create samphore.")
        };

        let in_flight_fence = unsafe {
            self.device
                .create_fence(&fence_info, None)
                .expect("Cannot create samphore.")
        };

        self.img_available_sem.set(image_available_semaphore);
        self.render_finished_sem.set(render_finished_semaphore);
        self.in_flight_fence.set(in_flight_fence);
    }

    pub fn test_draw(&self, test_raster_pipeline: vk::Pipeline) {
        /* --------- COMMAND BUFFER (TODO CALCULATE ONE TIME IF NO CHANGES) --------- */

        for (i, &command_buffer) in self.command_buffers.iter().enumerate() {
            let begin_info = vk::CommandBufferBeginInfo::default();
            unsafe {
                self.device
                    .begin_command_buffer(command_buffer, &begin_info)
                    .expect("Cannot begin command buffer");
            }
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(*self.render_pass)
                .framebuffer(self.framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: *self.swapchain_extents,
                })
                .clear_values(&clear_values);

            unsafe {
                self.device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    test_raster_pipeline,
                );
                self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
                self.device.cmd_end_render_pass(command_buffer);
                self.device
                    .end_command_buffer(command_buffer)
                    .expect("Cannot end command buffer.");
            }
        }

        /* --------------------------------- DRAWING -------------------------------- */

        unsafe {
            self.device
                .wait_for_fences(&[*self.in_flight_fence], true, std::u64::MAX)
                .expect("Failed to wait for fence");
            self.device
                .reset_fences(&[*self.in_flight_fence])
                .expect("Failed to reset fence");
        }

        let (image_index, _) = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    *self.swapchain,
                    std::u64::MAX,
                    *self.img_available_sem,
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image")
        };

        let wait_semaphores = [*self.img_available_sem];
        let signal_semaphores = [*self.render_finished_sem];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let command_buffers_binding = [self.command_buffers[image_index as usize]];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers_binding)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device
                .queue_submit(self.primary_queue, &[submit_info], *self.in_flight_fence)
                .expect("Failed to submit draw command buffer");
        }

        let swapchain_binding = [*self.swapchain];
        let image_index_binding: [u32; 1] = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchain_binding)
            .image_indices(&image_index_binding);

        unsafe {
            self.swapchain_loader
                .queue_present(self.primary_queue, &present_info)
                .expect("Failed to present swapchain image");
        }
    }

    fn get_physical_device_extensions(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> Vec<String> {
        let mut available_extensions: Vec<String> = Vec::new();

        let device_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to list device extensions")
        };

        println!("Available Vulkan Device Extensions:");
        for ext in device_extensions {
            let ext_name =
                unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()).to_string_lossy() };
            available_extensions.push(ext_name.to_string());
        }

        available_extensions
    }

    pub fn unload(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(*self.swapchain, None);
        }
        for image_view in self.swapchain_image_views.iter() {
            unsafe { self.device.destroy_image_view(*image_view, None) };
        }

        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
        unsafe {
            self.device.destroy_device(None);
        };
        unsafe { self.instance.destroy_instance(None) };
    }
}
