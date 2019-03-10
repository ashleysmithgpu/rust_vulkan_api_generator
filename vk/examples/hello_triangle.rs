
use std::ptr;
fn main() {

	let args: Vec<String> = std::env::args().collect();

	let instance = vk::InstanceBuilder {
		application_name: "hello triangle".to_string(),
		args: args,
		.. Default::default()
	}.create_instance().expect("Couldn't create instance");

	let wsi_info = instance.create_wsi(800, 600);

	let physical_device;
	let graphics_queue;
	let compute_queue;
	let transfer_queue;
	let device = {
		let mut db = vk::DeviceBuilder::new(&instance);
		let device = db.default_queues_physical_device(&wsi_info.0).create_device().expect("Couldn't create logical device");

		let (physical_device2, physical_device_index) = db.physical_device.unwrap();
		physical_device = physical_device2;

		println!("Using device index {:?}, graphics, compute, transfer queue family inices: {}, {}, {}",
			physical_device_index, db.queue_create_infos[0].0, db.queue_create_infos[0].0, db.queue_create_infos[0].0);

		graphics_queue = device.get_queue(db.queue_create_infos[0].0, 0);
		compute_queue = device.get_queue(db.queue_create_infos[0].0, 0);
		transfer_queue = device.get_queue(db.queue_create_infos[0].0, 0);
		device
	};

	let heaps = physical_device.memory_properties();
	let formats = physical_device.supported_surface_formats(&wsi_info.0).unwrap();
	let caps = physical_device.surface_capabilities(&wsi_info.0).unwrap();
	let modes = physical_device.present_modes(&wsi_info.0).unwrap();

	let swapchain = {
		let mut sb = vk::SwapchainBuilder::new(&device, &wsi_info.0);
		sb.width = 800;
		sb.height = 600;
		sb.num_swapchain_images = 2;
		sb.colour_format = formats[0].format;
		sb.colour_space = formats[0].colorSpace;
		sb.present_mode = modes[0];
		sb.create().unwrap()
	};

	let swapchain_images = swapchain.get_swapchain_images();
	let swapchain_image_views: Vec<vk::ImageView> = swapchain_images.iter().map(|x| vk::ImageViewBuilder::new(x, formats[0].format).create().unwrap()).collect();

	let command_pool = device.create_command_pool().unwrap();
	let cmdbs = command_pool.create_command_buffers(swapchain_images.len()).unwrap();

	let ds_image = {
		let mut ib = vk::ImageBuilder::new(&device);
		ib.extent.width = 800;
		ib.extent.height = 600;
		ib.format = vkraw::VkFormat::VK_FORMAT_D32_SFLOAT;
		ib.usage = vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
		ib.create().unwrap()
	};
	let mem = vk::MemoryAllocator::new(&device);
	let ds_image_mem = mem.allocate_image_memory(&ds_image, vk::staging_memory(&heaps));
	let ds_image_view = vk::ImageViewBuilder::new(&ds_image, vkraw::VkFormat::VK_FORMAT_D32_SFLOAT).depth_view().create().unwrap();

	let rp = vk::RenderPassBuilder::new(&device).default_single_colour_depth(formats[0].format, vkraw::VkFormat::VK_FORMAT_D32_SFLOAT).create().unwrap();

	let mut fbb = vk::FramebufferBuilder::new(&device);
	fbb.width = 800;
	fbb.height = 600;
	fbb.image_view_attachments = vec![&swapchain_image_views[0], &ds_image_view];
	fbb.render_pass = Some(&rp);
	let framebuffers = vec![
		fbb.create().unwrap(),
		fbb.set_attachments(vec![&swapchain_image_views[1], &ds_image_view]).create().unwrap()];

	let present_complete_sem = device.create_semaphore();
	let render_complete_sem = device.create_semaphore();
	let fences = vec![
		device.create_fence(),
		device.create_fence(),
		device.create_fence()
	];

	let dsl = {
		let mut dslb = vk::DescriptorSetLayoutBuilder::new(&device);
		dslb.bindings = vec![vkraw::VkDescriptorSetLayoutBinding {
			binding: 0,
			descriptorType: vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
			descriptorCount: 1,
			stageFlags: vkraw::VkShaderStageFlags::VK_SHADER_STAGE_VERTEX_BIT,
			pImmutableSamplers: 0 as *const vkraw::VkSampler
		}];
		dslb.create().unwrap()
	};

	let pl = {
		let mut plb = vk::PipelineLayoutBuilder::new(&device);
		plb.dsls = vec![&dsl];
		plb.create().unwrap()
	};

	let pipeline = {
		let mut pb = vk::PipelineBuilder::new(&device, &pl, &rp);
		let vsh = device.load_spirv_shader_from_disk("triangle.vert.spv").unwrap();
		let fsh = device.load_spirv_shader_from_disk("triangle.frag.spv").unwrap();
		pb.shader_stages = vec![
			vk::ShaderStage {
				module: vsh,
				entry_point: "main".to_string(),
				stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_VERTEX_BIT
			},
			vk::ShaderStage {
				module: fsh,
				entry_point: "main".to_string(),
				stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_FRAGMENT_BIT
			},
		];
		let vertex_size = std::mem::size_of::<f32>() * 6;
		let vertex_input_bindings = vkraw::VkVertexInputBindingDescription {
			binding: 0,
			stride: vertex_size as u32,
			inputRate: vkraw::VkVertexInputRate::VK_VERTEX_INPUT_RATE_VERTEX
		};
		let vertex_input_attributes = [vkraw::VkVertexInputAttributeDescription {
			location: 0,
			binding: 0,
			format: vkraw::VkFormat::VK_FORMAT_R32G32B32_SFLOAT,
			offset: 0
		},
		vkraw::VkVertexInputAttributeDescription {
			location: 1,
			binding: 0,
			format: vkraw::VkFormat::VK_FORMAT_R32G32B32_SFLOAT,
			offset: 12 // TODO get this from somewhere
		}];
		let vertex_input = vkraw::VkPipelineVertexInputStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			vertexBindingDescriptionCount: 1,
			pVertexBindingDescriptions: &vertex_input_bindings,
			vertexAttributeDescriptionCount: 2,
			pVertexAttributeDescriptions: vertex_input_attributes.as_ptr()
		};
		let input_assembly = vkraw::VkPipelineInputAssemblyStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			topology: vkraw::VkPrimitiveTopology::VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
			primitiveRestartEnable: vkraw::VK_FALSE
		};
		let viewports = vkraw::VkViewport {
			x: 0.0,
			y: 0.0,
			width: 800 as f32,
			height: 600 as f32,
			minDepth: 0.0,
			maxDepth: 1.0
		};
		let scissors = vkraw::VkRect2D {
			offset: vkraw::VkOffset2D {
				x: 0,
				y: 0
			},
			extent: vkraw::VkExtent2D {
				width: 800,
				height: 600
			}
		};
		let viewport = vkraw::VkPipelineViewportStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			viewportCount: 1,
			pViewports: &viewports,
			scissorCount: 1,
			pScissors: &scissors
		};

		let rasterisation = vkraw::VkPipelineRasterizationStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			depthClampEnable: vkraw::VK_FALSE,
			rasterizerDiscardEnable: vkraw::VK_FALSE,
			polygonMode: vkraw::VkPolygonMode::VK_POLYGON_MODE_FILL,
			cullMode: vkraw::VkCullModeFlags::VK_CULL_MODE_NONE,
			frontFace: vkraw::VkFrontFace::VK_FRONT_FACE_COUNTER_CLOCKWISE,
			depthBiasEnable: vkraw::VK_FALSE,
			depthBiasConstantFactor: 0.0,
			depthBiasClamp: 0.0,
			depthBiasSlopeFactor: 0.0,
			lineWidth: 1.0
		};
		let multisample = vkraw::VkPipelineMultisampleStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			rasterizationSamples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
			sampleShadingEnable: vkraw::VK_FALSE,
			minSampleShading: 0.0,
			pSampleMask: ptr::null(),
			alphaToCoverageEnable: vkraw::VK_FALSE,
			alphaToOneEnable: vkraw::VK_FALSE
		};
		let depth_stencil = vkraw::VkPipelineDepthStencilStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			depthTestEnable: vkraw::VK_TRUE,
			depthWriteEnable: vkraw::VK_TRUE,
			depthCompareOp: vkraw::VkCompareOp::VK_COMPARE_OP_LESS_OR_EQUAL,
			depthBoundsTestEnable: vkraw::VK_FALSE,
			stencilTestEnable: vkraw::VK_FALSE,
			front: vkraw::VkStencilOpState {
				failOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				passOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				depthFailOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				compareOp: vkraw::VkCompareOp::VK_COMPARE_OP_ALWAYS,
				compareMask: 0,
				writeMask: 0,
				reference: 0
			},
			back: vkraw::VkStencilOpState {
				failOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				passOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				depthFailOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				compareOp: vkraw::VkCompareOp::VK_COMPARE_OP_ALWAYS,
				compareMask: 0,
				writeMask: 0,
				reference: 0
			},
			minDepthBounds: 0.0,
			maxDepthBounds: 1.0
		};
		let blend_attachments = vkraw::VkPipelineColorBlendAttachmentState {
			blendEnable: vkraw::VK_FALSE,
			srcColorBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			dstColorBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			colorBlendOp: vkraw::VkBlendOp::VK_BLEND_OP_ADD ,
			srcAlphaBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			dstAlphaBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			alphaBlendOp: vkraw::VkBlendOp::VK_BLEND_OP_ADD ,
			colorWriteMask: vkraw::VkColorComponentFlags::all()
		};

		let colour_blend = vkraw::VkPipelineColorBlendStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			logicOpEnable: vkraw::VK_FALSE,
			logicOp: vkraw::VkLogicOp::VK_LOGIC_OP_CLEAR,
			attachmentCount: 1,
			pAttachments: &blend_attachments,
			blendConstants: [0.0, 0.0, 0.0, 0.0]
		};
		let dynamic_states = [
			vkraw::VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT
		];
		let dynamic = vkraw::VkPipelineDynamicStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			dynamicStateCount: 1,
			pDynamicStates: dynamic_states.as_ptr()
		};

		pb.vertex_input = Some(vertex_input);
		pb.input_assembly = Some(input_assembly);
		pb.viewport = Some(viewport);
		pb.rasterisation = Some(rasterisation);
		pb.multisample = Some(multisample);
		pb.depth_stencil = Some(depth_stencil);
		pb.colour_blend = Some(colour_blend);
		pb.dynamic = Some(dynamic);
		pb.create().unwrap()
	};

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
