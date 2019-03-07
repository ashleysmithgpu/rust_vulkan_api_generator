
fn main() {

	let instance = vk::InstanceBuilder {
		application_name: "hello triangle".to_string(),
		.. Default::default()
	}.create_instance().expect("Couldn't create instance");

	let wsi_info = instance.create_wsi(800, 600);

	let physical_device;
	let graphics_queue;
	let compute_queue;
	let transfer_queue;
	let device;
	let heaps;
	{
		let mut db = vk::DeviceBuilder::new(&instance);
		device = db.default_queues_physical_device(&wsi_info.0).create_device().expect("Couldn't create logical device");

		let (physical_device2, physical_device_index) = db.physical_device.unwrap();
		physical_device = physical_device2;

		println!("Using device index {:?}, graphics, compute, transfer queue family inices: {}, {}, {}",
			physical_device_index, db.queue_create_infos[0].0, db.queue_create_infos[1].0, db.queue_create_infos[2].0);

		graphics_queue = device.get_queue(db.queue_create_infos[0].0, 0);
		compute_queue = device.get_queue(db.queue_create_infos[1].0, 0);
		transfer_queue = device.get_queue(db.queue_create_infos[2].0, 0);
		heaps = physical_device.memory_properties();
	}

	let formats = physical_device.supported_surface_formats(&wsi_info.0).unwrap();
	let caps = physical_device.surface_capabilities(&wsi_info.0).unwrap();
	let modes = physical_device.present_modes(&wsi_info.0).unwrap();

	println!("formats {:?}", formats);
	println!("caps {:?}", caps);
	println!("modes {:?}", modes);

	let mem = vk::MemoryAllocator::new(&device);
	let mut vertex_buffer = device.create_buffer(std::mem::size_of::<[f32; 18]>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT).unwrap();
	let mut vertex_memory = mem.allocate_buffer_memory(&vertex_buffer, vk::staging_memory(&heaps)).unwrap();
	//let mut gpu_only_data = mem.allocate_buffer_memory(std::mem::size_of::<GPUData>(), vk::gpu_only_memory(&heaps));
	{
		let mut mapped = vertex_memory.map::<[f32; 18]>();
		(*mapped) = [
			1.0, 1.0, 0.0,	1.0, 0.0, 0.0,
			-1.0, 1.0, 0.0,	0.0, 1.0, 0.0,
			0.0, -1.0, 0.0,	0.0, 0.0, 1.0
		];
	}
	let mut index_buffer = device.create_buffer(std::mem::size_of::<[u32; 3]>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_INDEX_BUFFER_BIT).unwrap();
	let mut index_memory = mem.allocate_buffer_memory(&index_buffer, vk::staging_memory(&heaps)).unwrap();
	{
		let mut mapped = index_memory.map::<[u32; 3]>();
		(*mapped) = [0, 1, 2];
	}
	
	#[repr(C)]
	struct UniformBufferData {
		projection_from_view: glm::Mat4,
		view_from_model: glm::Mat4,
		world_from_model: glm::Mat4
	};
	
	let mut uniform_buffers = vec![
		device.create_buffer(std::mem::size_of::<UniformBufferData>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT).unwrap(),
		device.create_buffer(std::mem::size_of::<UniformBufferData>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT).unwrap(),
		device.create_buffer(std::mem::size_of::<UniformBufferData>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT).unwrap(),
	];
	let mut uniform_memory = vec![
		mem.allocate_buffer_memory(&uniform_buffers[0], vk::staging_memory(&heaps)).unwrap(),
		mem.allocate_buffer_memory(&uniform_buffers[1], vk::staging_memory(&heaps)).unwrap(),
		mem.allocate_buffer_memory(&uniform_buffers[2], vk::staging_memory(&heaps)).unwrap()
	];
}
