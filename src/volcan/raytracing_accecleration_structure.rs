use ash::vk;

struct VolcanAccelKhr {}

struct VolcanBLASKhr {}

impl VolcanBLASKhr {
    pub fn create_buffer(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), vk::Result> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .expect("Cannot create buffer")
        };

        // Get memory requirements for the buffer.
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        // Find a suitable memory type.
        let mem_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            properties,
        )
        .expect("Cannot find memory index");

        // Allocate memory for the buffer.
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let buffer_memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .expect("Cannot allocate memory")
        };

        // Bind the buffer with the allocated memory.
        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Cannot bind buffer memory")
        };

        Ok((buffer, buffer_memory))
    }

    fn find_memory_type(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, vk::Result> {
        let mem_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) != 0
                && (memory_type.property_flags & properties) == properties
            {
                return Ok(i as u32);
            }
        }
        Err(vk::Result::ERROR_FEATURE_NOT_PRESENT)
    }

    pub fn create_blas(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
    ) {
        let aabb_buffer_address = 0u64;
        let aabb_count = 0u32;

        let acceleration_structure_loader =
            ash::khr::acceleration_structure::Device::new(&instance, &device);

        let aabb_data = vk::AccelerationStructureGeometryAabbsDataKHR::default()
            .data(vk::DeviceOrHostAddressConstKHR {
                device_address: aabb_buffer_address,
            })
            .stride(std::mem::size_of::<vk::AabbPositionsKHR>() as u64);

        let blas_geometry = vk::AccelerationStructureGeometryKHR::default()
            .geometry_type(vk::GeometryTypeKHR::AABBS)
            .geometry(vk::AccelerationStructureGeometryDataKHR { aabbs: aabb_data })
            .flags(vk::GeometryFlagsKHR::OPAQUE);

        let blas_build_range_info = vk::AccelerationStructureBuildRangeInfoKHR {
            primitive_count: aabb_count,
            primitive_offset: 0,
            first_vertex: 0,
            transform_offset: 0,
        };

        let blas_geometry_binding = [blas_geometry];
        let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(&blas_geometry_binding);

        let mut accecleration_build_size_info =
            vk::AccelerationStructureBuildSizesInfoKHR::default();

        let size_info = unsafe {
            acceleration_structure_loader.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &build_geometry_info,
                &[blas_build_range_info.primitive_count],
                &mut accecleration_build_size_info,
            )
        };

        let blas_usage = vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS;
        let blas_properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        let blas_buffer = Self::create_buffer(
            instance,
            device,
            physical_device,
            accecleration_build_size_info.acceleration_structure_size,
            blas_usage,
            blas_properties,
        )
        .unwrap();

        let blas_create_info = vk::AccelerationStructureCreateInfoKHR::default()
            .buffer(blas_buffer.0)
            .offset(0)
            .size(accecleration_build_size_info.acceleration_structure_size)
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL);

        let blas = unsafe {
            acceleration_structure_loader
                .create_acceleration_structure(&blas_create_info, None)
                .expect("Failed to create BLAS")
        };

        // let build_cmd = unsafe {
        //     device.cmd_build_acceleration_structures_khr(
        //         command_buffer,
        //         &[build_geometry_info],
        //         &[&[blas_build_range_info]],
        //     )
        // };
    }
}
