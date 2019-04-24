
extern crate vk;

use rspirv::binary::Assemble;
use rspirv::binary::Disassemble;

use std::ptr;

fn main() {

	let args: Vec<String> = std::env::args().collect();
	
	let mut hdr = args.iter().find(|&x| x == "hdr").is_some();
	let mut fullscreen = args.iter().find(|&x| x == "fullscreen").is_some();

	let instance = vk::InstanceBuilder {
		application_name: "hello triangle".to_string(),
		args: args.clone(),
		.. Default::default()
	}.create_instance().expect("Couldn't create instance");

	let mut width = 2560;
	let mut height = 1440;

	let wsi_info = instance.create_wsi(width, height, fullscreen);

	#[cfg(feature = "xcb")]
	let (protocols, wm_delete_window, wm_protocols) = {
		let (wm_protocols, wm_delete_window) = {
			let pc = xcb::intern_atom(&wsi_info.1, false, "WM_PROTOCOLS");
			let dwc = xcb::intern_atom(&wsi_info.1, false, "WM_DELETE_WINDOW");

			let p = match pc.get_reply() {
				Ok(p) => p.atom(),
				Err(_) => panic!("could not load WM_PROTOCOLS atom")
			};
			let dw = match dwc.get_reply() {
				Ok(dw) => dw.atom(),
				Err(_) => panic!("could not load WM_DELETE_WINDOW atom")
			};
			(p, dw)
		};

		xcb::change_property(&wsi_info.1, xcb::PROP_MODE_REPLACE as u8, wsi_info.2, wm_protocols, xcb::ATOM_ATOM, 32, &protocols);
		
		([wm_delete_window], wm_delete_window, wm_protocols)
	};
	
	let device: vk::Device;

	let (physical_device, graphics_queue, compute_queue, transfer_queue) = {
		let mut db = vk::DeviceBuilder::new(&instance);
		device = db/*.use_device_named("Vega".to_string())*/.default_queues_physical_device(&wsi_info.0).create_device().expect("Couldn't create logical device");

		let (physical_device, physical_device_index) = db.physical_device.unwrap();

		println!("Using device index {:?}, graphics, compute, transfer queue family inices: {}, {}, {}",
			physical_device_index, db.queue_create_infos[0].0, db.queue_create_infos[0].0, db.queue_create_infos[0].0);

		let graphics_queue = device.get_queue(db.queue_create_infos[0].0, 0).unwrap();
		let compute_queue = device.get_queue(db.queue_create_infos[0].0, 0).unwrap();
		let transfer_queue = device.get_queue(db.queue_create_infos[0].0, 0).unwrap();
		
		(physical_device, graphics_queue, compute_queue, transfer_queue)
	};
	
	let caps = physical_device.surface_capabilities(&wsi_info.0).unwrap();
	width = std::cmp::max(width, caps.minImageExtent.width);
	height = std::cmp::max(height, caps.minImageExtent.height);
	width = std::cmp::min(width, caps.maxImageExtent.width);
	height = std::cmp::min(height, caps.maxImageExtent.height);
	
	println!("Capping window size to {}x{}", width, height);

	let heaps = physical_device.memory_properties();
	let mem = vk::MemoryAllocator::new(&device);

	let modes = physical_device.present_modes(&wsi_info.0).unwrap();
	
	let present_complete_sem = device.create_semaphore().unwrap();
	let render_complete_sem = device.create_semaphore().unwrap();

	let fences = vec![
		device.create_fence().unwrap(),
		device.create_fence().unwrap(),
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

	let fshspirv = {
		let mut b = rspirv::mr::Builder::new();
		b.set_version(1, 0);
		b.capability(spirv_headers::Capability::Shader);
		b.ext_inst_import("GLSL.std.450");
		b.memory_model(spirv_headers::AddressingModel::Logical, spirv_headers::MemoryModel::GLSL450);
		b.source::<String>(spirv_headers::SourceLanguage::GLSL, 450, None, None);
		let output = b.id();
		b.name(output, "out_colour");
		let input = b.id();
		b.name(input, "in_colour");
		b.decorate(output, spirv_headers::Decoration::Location, vec![rspirv::mr::Operand::from(0u32)]);
		b.decorate(input, spirv_headers::Decoration::Location, vec![rspirv::mr::Operand::from(1u32)]);
		let void = b.type_void();
		let float32 = b.type_float(32);
		let voidfvoid = b.type_function(void, vec![]);
		let vec4 = b.type_vector(float32, 4);
		let pvec4 = b.type_pointer(None, spirv_headers::StorageClass::Output, vec4);
		b.variable(pvec4, Some(output), spirv_headers::StorageClass::Output, None);
		let vec3 = b.type_vector(float32, 3);
		let pvec3 = b.type_pointer(None, spirv_headers::StorageClass::Input, vec3);
		b.variable(pvec3, Some(input), spirv_headers::StorageClass::Input, None);
		let one_point_zero_f = b.constant_f32(float32, 1.0);
		let f = b.begin_function(void, None, spirv_headers::FunctionControl::NONE, voidfvoid).unwrap();
		b.begin_basic_block(None).unwrap();

		let in_colour_loaded = b.load(vec3, None, input, None, vec![]).unwrap();
		let in_colour_x = b.composite_extract(float32, None, in_colour_loaded, vec![0]).unwrap();
		let in_colour_y = b.composite_extract(float32, None, in_colour_loaded, vec![0]).unwrap();
		let in_colour_z = b.composite_extract(float32, None, in_colour_loaded, vec![0]).unwrap();
		let result = b.composite_construct(vec4, None, vec![in_colour_x, in_colour_y, in_colour_z, one_point_zero_f]).unwrap();
		let _ = b.store(output, result, None, vec![]);
		b.ret().unwrap();
		b.end_function().unwrap();

		b.entry_point(spirv_headers::ExecutionModel::Fragment, f, "main", vec![output, input]);
		b.execution_mode(f, spirv_headers::ExecutionMode::OriginUpperLeft, vec![]);
		b.name(f, "main");
		b.module().assemble()
	};

	let descriptor_pool = device.create_descriptor_pool(2, vec![(2, vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER)]).unwrap();
	let descriptor_sets = descriptor_pool.create_descriptor_sets(vec![&dsl; 2]).unwrap();

	let mut vertex_buffer = device.create_buffer(std::mem::size_of::<[f32;24]>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT).unwrap();
	let aspect = width as f32 / height as f32;
	let mut vertex_memory = mem.allocate_buffer_memory(&vertex_buffer, vk::staging_memory(&heaps)).unwrap();
	//let mut gpu_only_data = mem.allocate_buffer_memory(std::mem::size_of::<GPUData>(), vk::gpu_only_memory(&heaps));
	{
		let mut mapped = vertex_memory.map::<[f32; 24]>();
		(*mapped) = [
			aspect, 1.0, 0.0,	1.0, 0.0, 0.0,
			-aspect, 1.0, 0.0,	0.0, 1.0, 0.0,
			aspect, -1.0, 0.0,	1.0, 0.0, 1.0,
			-aspect, -1.0, 0.0,0.0, 1.0, 1.0,
		];
	}
	let mut index_buffer = device.create_buffer(std::mem::size_of::<[u32; 6]>(), vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_INDEX_BUFFER_BIT).unwrap();
	let mut index_memory = mem.allocate_buffer_memory(&index_buffer, vk::staging_memory(&heaps)).unwrap();
	{
		let mut mapped = index_memory.map::<[u32; 6]>();
		(*mapped) = [0, 1, 2, 1, 2, 3];
	}

	#[repr(C)]
	struct UniformBufferData {
		projection_from_view: cgmath::Matrix4<f32>,
		view_from_model: cgmath::Matrix4<f32>,
		world_from_model: cgmath::Matrix4<f32>
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
	
	let command_pool = device.create_command_pool().unwrap();

	let mut quit = false;
	
	let mut swapchain = None;

	'swapchain_setup: while !quit {

		let formats = physical_device.supported_surface_formats2(hdr, &wsi_info).unwrap();
		
		let format;
		let colour_space;

		if hdr && formats.iter().find(|&x| x.surfaceFormat.format == vkraw::VkFormat::VK_FORMAT_A2R10G10B10_UNORM_PACK32).is_some() {
			format = vkraw::VkFormat::VK_FORMAT_A2R10G10B10_UNORM_PACK32;
			colour_space = vkraw::VkColorSpaceKHR::VK_COLOR_SPACE_BT2020_LINEAR_EXT;
		} else {
			format = formats[0].surfaceFormat.format;
			colour_space = formats[0].surfaceFormat.colorSpace;
		}

		println!("Using format {:?} and colour_space {:?}", format, colour_space);

		swapchain = Some({
			let mut sb = vk::SwapchainBuilder::new(&device, &wsi_info.0);
			sb.width = width;
			sb.height = height;
			sb.num_swapchain_images = 2;
			sb.colour_format = format;
			sb.colour_space = colour_space;
			sb.present_mode = modes[0];
			sb.create(&swapchain).unwrap()
		});
		
		let swapchain = swapchain.as_ref().unwrap();

		let mut fences = vec![
			device.create_fence().unwrap(),
			device.create_fence().unwrap(),
			device.create_fence().unwrap()
		];

		let swapchain_images = swapchain.get_swapchain_images();
		let swapchain_image_views: Vec<vk::ImageView> = swapchain_images.iter().map(|x| vk::ImageViewBuilder::new(x, format).create().unwrap()).collect();

		let mut command_buffers = command_pool.create_command_buffers(swapchain_images.len()).unwrap();

		let ds_image = {
			let mut ib = vk::ImageBuilder::new(&device);
			ib.extent.width = width;
			ib.extent.height = height;
			ib.format = vkraw::VkFormat::VK_FORMAT_D32_SFLOAT;
			ib.usage = vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
			ib.create().unwrap()
		};

		let ds_image_mem = mem.allocate_image_memory(&ds_image, vk::staging_memory(&heaps));
		let ds_image_view = vk::ImageViewBuilder::new(&ds_image, vkraw::VkFormat::VK_FORMAT_D32_SFLOAT).depth_view().create().unwrap();

		let render_pass = vk::RenderPassBuilder::new(&device).default_single_colour_depth(format, vkraw::VkFormat::VK_FORMAT_D32_SFLOAT).create().unwrap();

		let framebuffers = {
			let mut fbb = vk::FramebufferBuilder::new(&device);
			fbb.width = width as usize;
			fbb.height = height as usize;
			fbb.image_view_attachments = vec![&swapchain_image_views[0], &ds_image_view];
			fbb.render_pass = Some(&render_pass);
			vec![
				fbb.create().unwrap(),
				fbb.set_attachments(vec![&swapchain_image_views[1], &ds_image_view]).create().unwrap()]
		};

		let pipeline = {
			let mut pb = vk::PipelineBuilder::new(&device, &pipeline_layout, &render_pass);
			let vsh = device.load_spirv_shader_from_disk("triangle.vert.spv").unwrap();
			let fsh = device.load_spirv_shader_from_buffer(&fshspirv).unwrap();
			pb.default_graphics(vsh, fsh, width, height).create().unwrap()
		};

		let mut rotation_start = std::time::Instant::now();
		let mut frame_index = 0;

		let mut rotate = false;
		let mut recreate_swapchain = false;

		// Render loop
		'render_loop: while !quit {

			#[cfg(feature = "xcb")]
			{
				let event = wsi_info.1.poll_for_event();
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
								if key_press.detail() == 27 {
									rotate = true;
								}
							},
							xcb::KEY_RELEASE => {
								let key_release : &xcb::KeyReleaseEvent = unsafe {
									xcb::cast_event(&event)
								};
								println!("Key {} released", key_release.detail());
								if key_release.detail() == 27 {
									rotate = false;
								}
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
					match message.message {
						winapi::um::winuser::WM_QUIT => {
							println!("WM_QUIT");
							quit = true;
							break;
						},
						winapi::um::winuser::WM_KEYDOWN => {
							println!("WM_KEYDOWN {}", message.wParam);
							if message.wParam == 32 {
							} else if message.wParam == 82 {
								rotate = true;
							} else {
								quit = true;
							}
							break;
						},
						winapi::um::winuser::WM_KEYUP => {
							println!("WM_KEYUP");
							if message.wParam == 82 {
								rotate = false;
							} else if message.wParam == 32 {
								hdr = !hdr;
								recreate_swapchain = true;
							} else {
								quit = true;
							}
							break;
						},
						_ => {
						}
					};
				}

				if winapi::um::winuser::IsIconic(wsi_info.1) > 0 {
					continue;
				}
			}

			if quit {
				println!("WSI requested quit");
				continue;
			}

			if recreate_swapchain {
				println!("Recreating swapchain");
				break; // Out of render loop and re-create swapchain
			}

			let mut current_buffer = 0;
			let res = graphics_queue.acquire(&swapchain, std::u64::MAX, Some(&present_complete_sem), None);
			match res {
				Ok(image_index) => current_buffer = image_index,
				Err(res) => {
					if res == vkraw::VkResult::VK_ERROR_OUT_OF_DATE_KHR {
						recreate_swapchain = true;
					} else {
						println!("present err {:?}", res);
						quit = true;
					}
				}
			}

			if recreate_swapchain {
				println!("Recreating swapchain");
				break; // Out of render loop and re-create swapchain
			}

			if frame_index > 1 {
				fences[current_buffer as usize].wait(std::u64::MAX).unwrap()
					.reset().unwrap();
			}

			{
				if !rotate {
					rotation_start = std::time::Instant::now();
				}
				let elapsed = rotation_start.elapsed();
				let rotation = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
				let rotation = cgmath::Matrix3::from_angle_z(cgmath::Rad(rotation as f32));

				let aspect_ratio = width as f32 / height as f32;
				let projection = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
				let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, 1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -1.0, 0.0));

				let mut ub_data = UniformBufferData {
					projection_from_view: projection.into(),
					view_from_model: view.into(),
					world_from_model: rotation.into()
				};

				{
					let mut mapped = uniform_memory[current_buffer as usize].map::<UniformBufferData>();
					unsafe {
						libc::memcpy(mapped.get_ptr() as *mut core::ffi::c_void, (&mut ub_data as *mut UniformBufferData) as *mut libc::c_void, std::mem::size_of::<UniformBufferData>() as libc::size_t);
					}
				}
			}

			command_buffers[current_buffer as usize]
				.begin().unwrap()
				.begin_render_pass(width, height, &render_pass, vec![
					vk::ClearValue::Colourf32([0.0, 0.0, 0.0, 0.0]),
					vk::ClearValue::DepthStencil{ depth: 1.0, stencil: 0 }], Some(&framebuffers[current_buffer as usize]))
				.bind_descriptor_sets(vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, &pipeline_layout, 0, vec![&descriptor_sets[current_buffer as usize]], vec![])
				.bind_pipeline(vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, &pipeline)
				.bind_vertex_buffers(0, vec![(&vertex_buffer, 0)])
				.bind_index_buffer(&index_buffer, 0, vkraw::VkIndexType::VK_INDEX_TYPE_UINT32)
				.draw_indexed(6, 1, 0, 0, 1)
				.end_render_pass()
				.end_command_buffer();

			graphics_queue.submit(vec![&command_buffers[current_buffer as usize]], Some(&fences[current_buffer as usize]),
				vec![(&present_complete_sem, vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)], 
				vec![&render_complete_sem]);

			let res = graphics_queue.present(vec![(&render_complete_sem, &swapchain, current_buffer)]);
			match res {
				Ok(ress) => {
					for res2 in ress {
						if res2 == vkraw::VkResult::VK_ERROR_OUT_OF_DATE_KHR {
							recreate_swapchain = true;
						} else {
							println!("present err {:?}", res2);
							quit = true;
						}
					}
				},
				Err(res) => {
					if res == vkraw::VkResult::VK_ERROR_OUT_OF_DATE_KHR {
						recreate_swapchain = true;
					} else {
						println!("present err {:?}", res);
						quit = true;
					}
				}
			}

			if recreate_swapchain {
				println!("Recreating swapchain");
				break; // Out of render loop and re-create swapchain
			}

			frame_index += 1;
		}
		device.wait_idle();
	}
	device.wait_idle();
}
