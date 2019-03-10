
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
	let command_buffers = command_pool.create_command_buffers(swapchain_images.len()).unwrap();

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

	let render_pass = vk::RenderPassBuilder::new(&device).default_single_colour_depth(formats[0].format, vkraw::VkFormat::VK_FORMAT_D32_SFLOAT).create().unwrap();

	let mut fbb = vk::FramebufferBuilder::new(&device);
	fbb.width = 800;
	fbb.height = 600;
	fbb.image_view_attachments = vec![&swapchain_image_views[0], &ds_image_view];
	fbb.render_pass = Some(&render_pass);
	let framebuffers = vec![
		fbb.create().unwrap(),
		fbb.set_attachments(vec![&swapchain_image_views[1], &ds_image_view]).create().unwrap()];

	let present_complete_sem = device.create_semaphore().unwrap();
	let render_complete_sem = device.create_semaphore().unwrap();
	let fences = vec![
		device.create_fence().unwrap(),
		device.create_fence().unwrap(),
		device.create_fence().unwrap()
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

	let pipeline_layout = {
		let mut plb = vk::PipelineLayoutBuilder::new(&device);
		plb.dsls = vec![&dsl];
		plb.create().unwrap()
	};

	let pipeline = {
		let mut pb = vk::PipelineBuilder::new(&device, &pipeline_layout, &render_pass);
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

	let descriptor_pool = device.create_descriptor_pool(2, vec![(2, vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER)]).unwrap();
	let descriptor_sets = descriptor_pool.create_descriptor_sets(vec![&dsl; 2]).unwrap();

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
	];
	let mut uniform_memory = vec![
		mem.allocate_buffer_memory(&uniform_buffers[0], vk::staging_memory(&heaps)).unwrap(),
		mem.allocate_buffer_memory(&uniform_buffers[1], vk::staging_memory(&heaps)).unwrap(),
	];

	descriptor_sets[0].update_as_buffer(vkraw::VkDescriptorBufferInfo {
			buffer: uniform_buffers[0 as usize].buffer,
			offset: 0,
			range: std::mem::size_of::<UniformBufferData>() as u64,
		}, 0, 0, vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER);
	descriptor_sets[1].update_as_buffer(vkraw::VkDescriptorBufferInfo {
			buffer: uniform_buffers[1 as usize].buffer,
			offset: 0,
			range: std::mem::size_of::<UniformBufferData>() as u64,
		}, 0, 0, vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER);



	let begin_info = vkraw::VkCommandBufferBeginInfo {
		sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
		pNext: ptr::null(),
		flags: vkraw::VkCommandBufferUsageFlags::_EMPTY,
		pInheritanceInfo: ptr::null()
	};
	let clear_values = [
		vkraw::VkClearValue { colour: vkraw::VkClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] } },
		vkraw::VkClearValue { depthStencil: vkraw::VkClearDepthStencilValue { depth: 1.0, stencil: 0 } },
	];
	let mut rp_begin_info = vkraw::VkRenderPassBeginInfo {
		sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
		pNext: ptr::null(),
		renderPass: render_pass.render_pass,
		framebuffer: vkraw::VK_NULL_HANDLE,
		renderArea: vkraw::VkRect2D {
			offset: vkraw::VkOffset2D {
				x: 0,
				y: 0
			},
			extent: vkraw::VkExtent2D {
				width: 800,
				height: 600
			}
		},
		clearValueCount: 2,
		pClearValues: clear_values.as_ptr()
	};
	let mut res;
	for i in 0..2 {
		rp_begin_info.framebuffer = framebuffers[i as usize].framebuffer;

		unsafe {
			res = vkraw::vkBeginCommandBuffer(command_buffers[i as usize].command_buffer, &begin_info);
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);
		unsafe {
			vkraw::vkCmdBeginRenderPass(command_buffers[i as usize].command_buffer, &rp_begin_info, vkraw::VkSubpassContents::VK_SUBPASS_CONTENTS_INLINE);
		}
		let vp = vkraw::VkViewport {
			x: 0.0,
			y: 0.0,
			width: 800 as f32,
			height: 600 as f32,
			minDepth: 0.0,
			maxDepth: 1.0,
		};
		unsafe {
			vkraw::vkCmdSetViewport(command_buffers[i as usize].command_buffer, 0, 1, &vp);
		}
		let sc = vkraw::VkRect2D {
			offset: vkraw::VkOffset2D {
				x: 0,
				y: 0
			},
			extent: vkraw::VkExtent2D {
				width: 800,
				height: 600
			}
		};
		let offset = 0;
		unsafe {
			vkraw::vkCmdSetScissor(command_buffers[i as usize].command_buffer, 0, 1, &sc);
			vkraw::vkCmdBindDescriptorSets(command_buffers[i as usize].command_buffer, vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_layout.pipeline_layout, 0, 1, &descriptor_sets[(i % 2) as usize].descriptor_set, 0, ptr::null());
			vkraw::vkCmdBindPipeline(command_buffers[i as usize].command_buffer, vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.pipeline);
			vkraw::vkCmdBindVertexBuffers(command_buffers[i as usize].command_buffer, 0, 1, &vertex_buffer.buffer, &offset);
			vkraw::vkCmdBindIndexBuffer(command_buffers[i as usize].command_buffer, index_buffer.buffer, 0, vkraw::VkIndexType::VK_INDEX_TYPE_UINT32);
			vkraw::vkCmdDrawIndexed(command_buffers[i as usize].command_buffer, 3, 1, 0, 0, 1);
			vkraw::vkCmdEndRenderPass(command_buffers[i as usize].command_buffer);
			vkraw::vkEndCommandBuffer(command_buffers[i as usize].command_buffer);
		}
	}




	let mut current_buffer = 0;
	let mut frame_index = 0;
	let mut rotation = 0.0;
	let mut quit = false;

	// Render loop
	while !quit {

		println!("Frame {}", frame_index);

		#[cfg(feature = "xcb")]
		{
			let event = wsi_info.0.poll_for_event();
			match event {
				None => {}
				Some(event) => {
					let r = event.response_type() & !0x80;
					println!("xcb event {:?}", r);
					match r {
						xcb::EXPOSE => {
							println!("Expose");
						},
						xcb::KEY_PRESS => {
							let key_press : &xcb::KeyPressEvent = unsafe {
								xcb::cast_event(&event)
							};
							println!("Key {} pressed", key_press.detail());
							break;
						},
						xcb::CLIENT_MESSAGE => {
							let cmev = unsafe {
								xcb::cast_event::<xcb::ClientMessageEvent>(&event)
							};
							if cmev.type_() == wm_protocols && cmev.format() == 32 {
								let protocol = cmev.data().data32()[0];
								if protocol == wm_delete_window {
									println!("wm_delete_window");
									quit = true;
									break;
								}
							}
						},
						_ => {}
					}
				}
			}
		}
		#[cfg(feature = "winapi")]
		unsafe {
			let mut message: winapi::um::winuser::MSG = std::mem::uninitialized();

			while winapi::um::winuser::PeekMessageW(&mut message as *mut winapi::um::winuser::MSG, ptr::null_mut(), 0, 0, winapi::um::winuser::PM_REMOVE) > 0 {
				winapi::um::winuser::TranslateMessage(&message as *const winapi::um::winuser::MSG);
				winapi::um::winuser::DispatchMessageW(&message as *const winapi::um::winuser::MSG);
				if message.message == winapi::um::winuser::WM_QUIT {
					println!("WM_QUIT");
					quit = true;
					break;
				}
			}

			if winapi::um::winuser::IsIconic(wsi_info.0) > 0 {
				continue;
			}
		}

		if quit {
			println!("WSI requested quit");
			break;
		}




		assert!(instance.vk.AcquireNextImageKHR.is_some());
		res = instance.vk.AcquireNextImageKHR.unwrap()(device.device, swapchain.swapchain, std::u64::MAX, present_complete_sem.semaphore, vkraw::VK_NULL_HANDLE, &mut current_buffer);
		if res != vkraw::VkResult::VK_SUCCESS {
			println!("Acquire returned {:?}, breaking", res);
			break;
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);

		if frame_index > 1 {
			unsafe {
				res = vkraw::vkWaitForFences(device.device, 1, &fences[current_buffer as usize].fence, vkraw::VK_TRUE, std::u64::MAX);
			}
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			unsafe {
				res = vkraw::vkResetFences(device.device, 1, &fences[current_buffer as usize].fence);
			}
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}


		// Per frame logic
		// TODO: index returned from vkAcquireNextImageKHR may not be sequential
		// need to allocate uniform buffers[num swapchain images][num frames in flight at once]
		{
			let projection = glm::ext::perspective(glm::radians(60.0), 800 as f32 / 600 as f32, 0.01, 100.0);
			let view = glm::ext::translate(&num::one(), glm::vec3(0.0, 0.0, -2.5));
			let model: glm::Mat4 = glm::ext::rotate(&num::one(), rotation, glm::vec3(0.0, 0.0, 1.0));
			let mut ub_data = UniformBufferData {
				projection_from_view: projection,
				view_from_model: view,
				world_from_model: model
			};
			rotation += 0.01;

			{
				let mut mapped = uniform_memory[current_buffer as usize].map::<UniformBufferData>();
				unsafe {
					libc::memcpy(mapped.get_ptr() as *mut core::ffi::c_void, (&mut ub_data as *mut UniformBufferData) as *mut libc::c_void, std::mem::size_of::<UniformBufferData>() as libc::size_t);
				}
			}
		}

		let wait_stage_mask = vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
		let submit_info = vkraw::VkSubmitInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SUBMIT_INFO,
			pNext: ptr::null(),
			waitSemaphoreCount: 1,
			pWaitSemaphores: &present_complete_sem.semaphore,
			pWaitDstStageMask: &wait_stage_mask,
			commandBufferCount: 1,
			pCommandBuffers: &command_buffers[current_buffer as usize].command_buffer,
			signalSemaphoreCount: 1,
			pSignalSemaphores: &render_complete_sem.semaphore
		};

		unsafe {
			vkraw::vkQueueSubmit(graphics_queue, 1, &submit_info, fences[current_buffer as usize].fence);
		}

		let mut result = vkraw::VkResult::VK_SUCCESS;
		let mut image_indices = current_buffer;
		let present_info = vkraw::VkPresentInfoKHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
			pNext: ptr::null(),
			waitSemaphoreCount: 1,
			pWaitSemaphores: &render_complete_sem.semaphore,
			swapchainCount: 1,
			pSwapchains: &swapchain.swapchain,
			pImageIndices: &mut image_indices,
			pResults: &mut result
		};
		assert!(instance.vk.QueuePresentKHR.is_some());
		instance.vk.QueuePresentKHR.unwrap()(graphics_queue, &present_info);
		frame_index += 1;
	}
	unsafe {
		vkraw::vkDeviceWaitIdle(device.device);
	}
}
