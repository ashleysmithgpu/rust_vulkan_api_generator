
extern crate vkrust;

extern crate libc;

extern crate glm;
extern crate num;

#[cfg(feature="xcb")]
extern crate xcb;

use std::ptr;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[cfg(feature="xcb")]
fn create_wsi(instance: vkrust::vkrust::VkInstance) -> (xcb::Connection, u32, u64) {

	let mut surface: vkrust::vkrust::VkSurfaceKHR = 0;
	println!("Creating XCB window");
	let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
	let win;
	{
		let setup = conn.get_setup();
		let screen = setup.roots().nth(screen_num as usize).unwrap();

		win = conn.generate_id();
		xcb::create_window(&conn,
			xcb::COPY_FROM_PARENT as u8,
			win,
			screen.root(),
			0, 0,
			WIDTH as u16, HEIGHT as u16,
			0,
			xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
			screen.root_visual(), &vec![]
		);
		xcb::map_window(&conn, win);
		conn.flush();

		let surface_create_info = vkrust::vkrust::VkXcbSurfaceCreateInfoKHR {
			sType: vkrust::vkrust::VkStructureType::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
			pNext: ptr::null(),
			flags: 0,
			connection: conn.get_raw_conn(),
			window: win
		};

		let res;
		unsafe {
			res = vkrust::vkrust::vkCreateXcbSurfaceKHR(instance, &surface_create_info, ptr::null(), &mut surface);
		}
		assert!(res == vkrust::vkrust::VkResult::VK_SUCCESS);
	}

	(conn, win, surface)
}

fn get_memory_type(type_bits: u32, properties: vkrust::vkrust::VkMemoryPropertyFlags, device_memory_properties: &vkrust::vkrust::VkPhysicalDeviceMemoryProperties) -> Option<u32> {
	let mut type_bits_mut = type_bits.clone();
	for i in 0..device_memory_properties.memoryTypeCount {
		if (type_bits_mut & 1) == 1 {
			if (device_memory_properties.memoryTypes[i as usize].propertyFlags & properties) == properties {
				return Some(i)
			}
		}
		type_bits_mut >>= 1;
	}
	None
}

fn main() {

	use vkrust::*;

	let mut res: vkrust::VkResult;
	let mut instance: vkrust::VkInstance = 0;
	{
		// Don't enable this layer here, it seems to break the lunarg code
		/*let enabled_layers_rust = vec![
			"VK_LAYER_LUNARG_standard_validation".to_string(),
		];*/
		let enabled_extensions_rust = vec![
			"VK_KHR_surface\0".to_string(),
			"VK_KHR_xcb_surface\0".to_string()
		];

		let enabled_layers: Vec<*const u8> = vec![
		];
		let enabled_extensions: Vec<*const u8> = vec![
			enabled_extensions_rust[0].as_ptr(),
			enabled_extensions_rust[1].as_ptr()
		];

		let application_info = vkrust::VkApplicationInfo {
			sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
			pNext: ptr::null(),
			pApplicationName: "app name".as_ptr(),
			applicationVersion: vkrust::VK_MAKE_VERSION(1,0,0),
			pEngineName: "engine name".as_ptr(),
			engineVersion: vkrust::VK_MAKE_VERSION(1,0,0),
			apiVersion: vkrust::VK_MAKE_VERSION(1,0,0),
		};
		let create_info = vkrust::VkInstanceCreateInfo {
			sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			pApplicationInfo: &application_info,
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr() as *const u8,
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr() as *const u8
		};

		println!("Creating instance");
		unsafe {
			res = vkrust::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};
	}
	assert!(instance != vkrust::VK_NULL_HANDLE);
	assert!(res == vkrust::VkResult::VK_SUCCESS);


	let mut num_physical_devices = 0;

	unsafe {
		vkrust::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, 0 as *mut u64);
	}
	assert!(num_physical_devices > 0);

	let mut physical_device: vkrust::VkPhysicalDevice = 0;
	unsafe {
		vkrust::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, &mut physical_device);
	}

	assert!(physical_device != vkrust::VK_NULL_HANDLE);

	let mut global_memory_properties: vkrust::VkPhysicalDeviceMemoryProperties;

	unsafe {
		global_memory_properties = std::mem::uninitialized();
		vkrust::vkGetPhysicalDeviceMemoryProperties(physical_device, &mut global_memory_properties);
	}

	let priorities: [f32; 1] = [1.0];

	let queue_create_info = vkrust::VkDeviceQueueCreateInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
		pNext: ptr::null(),
		flags: 0,
		queueFamilyIndex: 0,
		queueCount: 1,
		pQueuePriorities: &priorities as *const f32
	};

	let mut device: vkrust::VkDevice = 0;
	{
		let enabled_layers_rust = vec![
			"VK_LAYER_LUNARG_standard_validation\0".to_string(),
		];
		let enabled_extensions_rust = vec![
			"VK_KHR_swapchain\0".to_string()
		];

		let enabled_layers: Vec<*const u8> = vec![
			enabled_layers_rust[0].as_ptr()
		];
		let enabled_extensions: Vec<*const u8> = vec![
			enabled_extensions_rust[0].as_ptr()
		];
		let device_create_info = vkrust::VkDeviceCreateInfo {
			sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			queueCreateInfoCount: 1,
			pQueueCreateInfos: &queue_create_info,
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr() as *const u8,
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr() as *const u8,
			pEnabledFeatures: ptr::null()
		};

		println!("Creating device");
		unsafe {
			res = vkrust::vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
		};
		assert!(device != vkrust::VK_NULL_HANDLE);
		assert!(res == vkrust::VkResult::VK_SUCCESS);
	}

	{
		let wsi_info = create_wsi(instance);
		{
			// Get present and graphics queue index
			let mut queue_count = 0;
			unsafe {
				vkrust::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, ptr::null_mut());
			}
			assert!(queue_count > 0);
			println!("Found {} queues:", queue_count);

			let mut queue_props = Vec::<vkrust::VkQueueFamilyProperties>::with_capacity(queue_count as usize);
			let mut queue_supports_present = Vec::<bool>::with_capacity(queue_count as usize);
			unsafe {
				vkrust::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, queue_props.as_mut_ptr());
				queue_props.set_len(queue_count as usize);
				queue_supports_present.set_len(queue_count as usize);
			}

			let mut graphics_and_present_queue_index = 0;
			let mut found_good_queue = false;
			for (i,prop) in queue_props.iter().enumerate() {
				print!(" Queue {} supports: ", i);
				unsafe {
					vkrust::vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, i as u32, wsi_info.2, &mut queue_supports_present[i as usize]);
				}
				if !(prop.queueFlags & vkrust::VkQueueFlags::VK_QUEUE_GRAPHICS_BIT).is_empty() {
					print!(" graphics, ");
				}
				if queue_supports_present[i as usize] {
					print!(" present, ");
				}
				if !(prop.queueFlags & vkrust::VkQueueFlags::VK_QUEUE_GRAPHICS_BIT).is_empty() && queue_supports_present[i] {
					graphics_and_present_queue_index = i;
					found_good_queue = true;
				}
				print!("\n");
			}
			assert!(found_good_queue);
			println!("Using queue index {}", graphics_and_present_queue_index);

			// Get a supported colour format and colour space
			let mut format_count = 0;
			unsafe {
				vkrust::vkGetPhysicalDeviceSurfaceFormatsKHR(physical_device, wsi_info.2, &mut format_count, ptr::null_mut());
			}
			assert!(format_count > 0);
			println!("Found {} surface formats", format_count);

			let mut surface_formats = Vec::<vkrust::VkSurfaceFormatKHR>::with_capacity(format_count as usize);
			unsafe {
				surface_formats.set_len(format_count as usize);
				vkrust::vkGetPhysicalDeviceSurfaceFormatsKHR(physical_device, wsi_info.2, &mut format_count, surface_formats.as_mut_ptr());
			}

			let mut colour_format = vkrust::VkFormat::VK_FORMAT_B8G8R8A8_UNORM;
			let mut colour_space = vkrust::VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
			if format_count == 1 && surface_formats[0].format == vkrust::VkFormat::VK_FORMAT_UNDEFINED {

				colour_space = surface_formats[0].colorSpace.clone();
			} else {

				let mut found_b8g8r8a8_unorm = false;
				for fmt in &surface_formats {
					if fmt.format == vkrust::VkFormat::VK_FORMAT_B8G8R8A8_UNORM {
						colour_format = fmt.format.clone();
						colour_space = fmt.colorSpace.clone();
						found_b8g8r8a8_unorm = true;
						break;
					}
				}
				if !found_b8g8r8a8_unorm {
					colour_format = surface_formats[0].format.clone();
					colour_space = surface_formats[0].colorSpace.clone();
				}
			}

			println!("Using colour format {:?} colour space {:?}", colour_format, colour_space);

			// Create swapchain and get images
			let mut surface_capabilities: vkrust::VkSurfaceCapabilitiesKHR;
			unsafe {
				surface_capabilities = std::mem::uninitialized();
				vkrust::vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, wsi_info.2, &mut surface_capabilities);
			}

			let mut present_mode_count = 0;
			unsafe {
				vkrust::vkGetPhysicalDeviceSurfacePresentModesKHR(physical_device, wsi_info.2, &mut present_mode_count, ptr::null_mut());
			}
			assert!(present_mode_count > 0);
			let mut present_modes = Vec::<vkrust::VkPresentModeKHR>::with_capacity(present_mode_count as usize);
			unsafe {
				present_modes.set_len(present_mode_count as usize);
				vkrust::vkGetPhysicalDeviceSurfacePresentModesKHR(physical_device, wsi_info.2, &mut present_mode_count, present_modes.as_mut_ptr());
			}

			println!("Found {} present modes", present_mode_count);

			// TODO: check these properly
			let present_mode = vkrust::VkPresentModeKHR::VK_PRESENT_MODE_FIFO_KHR;

			let number_of_swapchain_images = surface_capabilities.minImageCount;

			let swapchain_transform = vkrust::VkSurfaceTransformFlagsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;

			let composite_alpha = vkrust::VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;

			let swapchain_create_info = vkrust::VkSwapchainCreateInfoKHR {
				sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: vkrust::VkSwapchainCreateFlagBitsKHR::_EMPTY,
				surface: wsi_info.2,
				minImageCount: number_of_swapchain_images,
				imageFormat: colour_format,
				imageColorSpace: colour_space,
				imageExtent: vkrust::VkExtent2D{ width: WIDTH, height: HEIGHT },
				imageArrayLayers: 1,
				imageUsage: vkrust::VkImageUsageFlags::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
				imageSharingMode: vkrust::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
				queueFamilyIndexCount: 0,
				pQueueFamilyIndices: ptr::null(),
				preTransform: swapchain_transform,
				compositeAlpha: composite_alpha,
				presentMode: present_mode,
				clipped: true,
				oldSwapchain: vkrust::VK_NULL_HANDLE
			};

			let mut swapchain: vkrust::VkSwapchainKHR = 0;
			{
				unsafe {
					res = vkrust::vkCreateSwapchainKHR(device, &swapchain_create_info, ptr::null(), &mut swapchain);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			let mut swapchain_image_count = 0;
			unsafe {
				vkrust::vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, ptr::null_mut());
			}
			assert!(swapchain_image_count > 0);
			println!("Creating {} swapchain images", swapchain_image_count);
			let mut swapchain_images = Vec::<vkrust::VkImage>::with_capacity(swapchain_image_count as usize);
			unsafe {
				swapchain_images.set_len(swapchain_image_count as usize);
				vkrust::vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr());
			}

			let mut swapchain_image_views = Vec::<vkrust::VkImageView>::with_capacity(swapchain_image_count as usize);
			unsafe {
				swapchain_image_views.set_len(swapchain_image_count as usize);
			}
			for i in 0..swapchain_image_count {
				let img_create_info = vkrust::VkImageViewCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
					pNext: ptr::null(),
					//flags: vkrust::VkImageViewCreateFlags::_EMPTY,
					flags: 0,
					image: swapchain_images[i as usize],
					viewType: vkrust::VkImageViewType::VK_IMAGE_VIEW_TYPE_2D,
					format: colour_format,
					components: vkrust::VkComponentMapping {
						r: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_R,
						g: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_G,
						b: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_B,
						a: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_A
					},
					subresourceRange: vkrust::VkImageSubresourceRange {
						aspectMask: vkrust::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
						baseMipLevel: 0,
						levelCount: 1,
						baseArrayLayer: 0,
						layerCount: 1
					}
				};

				unsafe{
					res = vkrust::vkCreateImageView(device, &img_create_info, ptr::null(), &mut swapchain_image_views[i as usize]);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Create command pool
			println!("Creating command pool");
			let mut command_pool: vkrust::VkCommandPool = 0;
			{
				let pool_create_info = vkrust::VkCommandPoolCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkrust::VkCommandPoolCreateFlags::VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
					queueFamilyIndex: 0
				};
				unsafe {
					res = vkrust::vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}


			// Create command buffers
			println!("Creating command buffers");
			let mut command_buffers = Vec::<vkrust::VkCommandBuffer>::with_capacity(swapchain_image_count as usize);
			{
				let cmd_buf_create_info = vkrust::VkCommandBufferAllocateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
					pNext: ptr::null(),
					commandPool: command_pool,
					level: vkrust::VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
					commandBufferCount: swapchain_image_count
				};

				unsafe {
					command_buffers.set_len(swapchain_image_count as usize);
					res = vkrust::vkAllocateCommandBuffers(device, &cmd_buf_create_info, command_buffers.as_mut_ptr());
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Create depth stencil
			println!("Creating depth/stencil images");
			let mut ds_image: vkrust::VkImage = 0;
			let mut ds_image_view: vkrust::VkImageView = 0;
			let mut ds_mem: vkrust::VkDeviceMemory = 0;
			let depth_format = vkrust::VkFormat::VK_FORMAT_D24_UNORM_S8_UINT;
			{
				let image_create_info = vkrust::VkImageCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkrust::VkImageCreateFlags::empty(),
					imageType: vkrust::VkImageType::VK_IMAGE_TYPE_2D,
					format: depth_format,
					extent: vkrust::VkExtent3D { width: WIDTH, height: HEIGHT, depth: 1 },
					mipLevels: 1,
					arrayLayers: 1,
					samples: vkrust::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
					tiling: vkrust::VkImageTiling::VK_IMAGE_TILING_OPTIMAL,
					usage: vkrust::VkImageUsageFlags::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT | vkrust::VkImageUsageFlags::VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
					sharingMode: vkrust::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
					queueFamilyIndexCount: 0,
					pQueueFamilyIndices: ptr::null(),
					initialLayout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED
				};
				let mut mem_alloc = vkrust::VkMemoryAllocateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
					pNext: ptr::null(),
					allocationSize: 0,
					memoryTypeIndex: 0
				};
				let mut mem_reqs: vkrust::VkMemoryRequirements;

				unsafe {
					res = vkrust::vkCreateImage(device, &image_create_info, ptr::null(), &mut ds_image);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					mem_reqs = std::mem::uninitialized();
					vkrust::vkGetImageMemoryRequirements(device, ds_image, &mut mem_reqs);
				}
				let ds_view = vkrust::VkImageViewCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					image: ds_image,
					viewType: vkrust::VkImageViewType::VK_IMAGE_VIEW_TYPE_2D,
					format: depth_format,
					components: vkrust::VkComponentMapping {
						r: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_R,
						g: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY,
						b: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY,
						a: vkrust::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY
					},
					subresourceRange: vkrust::VkImageSubresourceRange {
						aspectMask: vkrust::VkImageAspectFlags::VK_IMAGE_ASPECT_DEPTH_BIT | vkrust::VkImageAspectFlags::VK_IMAGE_ASPECT_STENCIL_BIT,
						baseMipLevel: 0,
						levelCount: 1,
						baseArrayLayer: 0,
						layerCount: 1
					}
				};
				mem_alloc.allocationSize = mem_reqs.size;
				mem_alloc.memoryTypeIndex = get_memory_type(mem_reqs.memoryTypeBits, vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, &global_memory_properties).unwrap();
				unsafe {
					res = vkrust::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut ds_mem);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkBindImageMemory(device, ds_image, ds_mem, 0);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkCreateImageView(device, &ds_view, ptr::null(), &mut ds_image_view);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Create render pass
			println!("Creating render pass");
			let mut render_pass: vkrust::VkRenderPass = 0;
			{
				let attachments = [
					vkrust::VkAttachmentDescription {
						flags: vkrust::VkAttachmentDescriptionFlags::_EMPTY,
						format: colour_format,
						samples: vkrust::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
						loadOp: vkrust::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						storeOp: vkrust::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
						stencilLoadOp: vkrust::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
						stencilStoreOp: vkrust::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
						initialLayout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						finalLayout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
					},
					vkrust::VkAttachmentDescription {
						flags: vkrust::VkAttachmentDescriptionFlags::_EMPTY,
						format: depth_format,
						samples: vkrust::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
						loadOp: vkrust::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						storeOp: vkrust::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
						stencilLoadOp: vkrust::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						stencilStoreOp: vkrust::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
						initialLayout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						finalLayout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
					}
				];

				let colour_reference = vkrust::VkAttachmentReference {
					attachment: 0,
					layout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
				};
				let depth_reference = vkrust::VkAttachmentReference {
					attachment: 1,
					layout: vkrust::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
				};
				let subpass = vkrust::VkSubpassDescription {
					flags: vkrust::VkSubpassDescriptionFlags::_EMPTY,
					pipelineBindPoint: vkrust::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS,
					inputAttachmentCount: 0,
					pInputAttachments: ptr::null(),
					colorAttachmentCount: 1,
					pColorAttachments: &colour_reference,
					pResolveAttachments: ptr::null(),
					pDepthStencilAttachment: &depth_reference,
					preserveAttachmentCount: 0,
					pPreserveAttachments: ptr::null()
				};
				let dependencies = [
					vkrust::VkSubpassDependency {
						srcSubpass: vkrust::VK_SUBPASS_EXTERNAL as u32,
						dstSubpass: 0,
						srcStageMask: vkrust::VkPipelineStageFlags::VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
						dstStageMask: vkrust::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
						srcAccessMask: vkrust::VkAccessFlags::VK_ACCESS_MEMORY_READ_BIT,
						dstAccessMask: vkrust::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | vkrust::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
						dependencyFlags: vkrust::VkDependencyFlags::VK_DEPENDENCY_BY_REGION_BIT,
					},
					vkrust::VkSubpassDependency {
						srcSubpass: 0,
						dstSubpass: vkrust::VK_SUBPASS_EXTERNAL as u32,
						srcStageMask: vkrust::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
						dstStageMask: vkrust::VkPipelineStageFlags::VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
						srcAccessMask: vkrust::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | vkrust::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
						dstAccessMask: vkrust::VkAccessFlags::VK_ACCESS_MEMORY_READ_BIT,
						dependencyFlags: vkrust::VkDependencyFlags::VK_DEPENDENCY_BY_REGION_BIT,
					}
				];
				let render_pass_create_info = vkrust::VkRenderPassCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					attachmentCount: attachments.len() as u32,
					pAttachments: attachments.as_ptr(),
					subpassCount: 1,
					pSubpasses: &subpass,
					dependencyCount: dependencies.len() as u32,
					pDependencies: dependencies.as_ptr()
				};

				unsafe {
					res = vkrust::vkCreateRenderPass(device, &render_pass_create_info, ptr::null(), &mut render_pass);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Pipeline cache
			println!("Creating pipeline cache");
			let mut pipeline_cache: vkrust::VkPipeline = 0;
			{
				let pipeline_create_info = vkrust::VkPipelineCacheCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					initialDataSize: 0,
					pInitialData: ptr::null()
				};
				unsafe {
					res = vkrust::vkCreatePipelineCache(device, &pipeline_create_info, ptr::null(), &mut pipeline_cache);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Framebuffer
			println!("Creating framebuffers");
			let mut framebuffers = Vec::<vkrust::VkFramebuffer>::with_capacity(swapchain_image_count as usize);
			{
				unsafe {
					framebuffers.set_len(swapchain_image_count as usize);
				}
				let mut attachments = [vkrust::VK_NULL_HANDLE, ds_image_view];
				let fb_create_info = vkrust::VkFramebufferCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					renderPass: render_pass,
					attachmentCount: attachments.len() as u32,
					pAttachments: attachments.as_ptr(),
					width: WIDTH,
					height: HEIGHT,
					layers: 1
				};
				for i in 0..swapchain_image_count {
					attachments[0] = swapchain_image_views[i as usize];

					unsafe {
						res = vkrust::vkCreateFramebuffer(device, &fb_create_info, ptr::null(), &mut framebuffers[i as usize]);
					}
					assert!(res == vkrust::VkResult::VK_SUCCESS);
				};
			}

			// Fences and semaphores
			println!("Creating sync prims");
			let mut present_complete_sem: vkrust::VkSemaphore = 0;
			let mut render_complete_sem: vkrust::VkSemaphore = 0;
			let mut fences = Vec::<vkrust::VkFence>::with_capacity(swapchain_image_count as usize);
			{
				let sem_create_info = vkrust::VkSemaphoreCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0
				};
				unsafe {
					fences.set_len(swapchain_image_count as usize);
					res = vkrust::vkCreateSemaphore(device, &sem_create_info, ptr::null(), &mut present_complete_sem);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkCreateSemaphore(device, &sem_create_info, ptr::null(), &mut render_complete_sem);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);

				let fence_create_info = vkrust::VkFenceCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkrust::VkFenceCreateFlags::_EMPTY
				};
				for i in 0..swapchain_image_count {
					unsafe {
						res = vkrust::vkCreateFence(device, &fence_create_info, ptr::null(), &mut fences[i as usize]);
					}
					assert!(res == vkrust::VkResult::VK_SUCCESS);
				}
			}

			let use_staging = false;

			// Vertex/index data
			println!("Creating verticies/indicies");
			let mut vertex_buffer: vkrust::VkBuffer = 0;
			let num_vertices = 3;
			let vertex_size = std::mem::size_of::<f32>() * 6;
			let mut vertex_mem: vkrust::VkDeviceMemory = 0;

			let mut index_buffer: vkrust::VkBuffer = 0;
			let num_indicies = 3;
			let index_size = std::mem::size_of::<u32>();
			let mut index_mem: vkrust::VkDeviceMemory = 0;
			{
				let vertices: [f32; 18] = [
					1.0, 1.0, 0.0,	1.0, 0.0, 0.0,
					-1.0, 1.0, 0.0,	0.0, 1.0, 0.0,
					0.0, -1.0, 0.0,	0.0, 0.0, 1.0
				];

				let indicies: [u32; 3] = [0, 1, 2];

				if use_staging {
					// TODO
				} else {
					{
						let vb_create_info = vkrust::VkBufferCreateInfo {
							sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
							pNext: ptr::null(),
							flags: vkrust::VkBufferCreateFlags::_EMPTY,
							size: num_vertices * vertex_size as u64,
							usage: vkrust::VkBufferUsageFlags::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
							sharingMode: vkrust::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
							queueFamilyIndexCount: 0,
							pQueueFamilyIndices: ptr::null()
						};

						unsafe {
							res = vkrust::vkCreateBuffer(device, &vb_create_info, ptr::null(), &mut vertex_buffer);
						}
						assert!(res == vkrust::VkResult::VK_SUCCESS);

						let mut mem_reqs: vkrust::VkMemoryRequirements;
						unsafe {
							mem_reqs = std::mem::uninitialized();
							vkrust::vkGetBufferMemoryRequirements(device, vertex_buffer, &mut mem_reqs);
						}
						let mut mem_alloc = vkrust::VkMemoryAllocateInfo {
							sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
							pNext: ptr::null(),
							allocationSize: mem_reqs.size,
							memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
						};
						unsafe {
							res = vkrust::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut vertex_mem);
							assert!(res == vkrust::VkResult::VK_SUCCESS);
							let mut data: *mut libc::c_void = ptr::null_mut();
							res = vkrust::vkMapMemory(device, vertex_mem, 0, mem_alloc.allocationSize, 0, &mut data);
							assert!(res == vkrust::VkResult::VK_SUCCESS);
							assert!(data != ptr::null_mut());
							libc::memcpy(data, vertices.as_ptr() as *mut libc::c_void, (num_vertices as usize * vertex_size as usize) as libc::size_t);
							vkrust::vkUnmapMemory(device, vertex_mem);
							res = vkrust::vkBindBufferMemory(device, vertex_buffer, vertex_mem, 0);
						}
						assert!(res == vkrust::VkResult::VK_SUCCESS);
					}

					{
						let ib_create_info = vkrust::VkBufferCreateInfo {
							sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
							pNext: ptr::null(),
							flags: vkrust::VkBufferCreateFlags::_EMPTY,
							size: num_indicies * index_size as u64,
							usage: vkrust::VkBufferUsageFlags::VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
							sharingMode: vkrust::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
							queueFamilyIndexCount: 0,
							pQueueFamilyIndices: ptr::null()
						};

						unsafe {
							res = vkrust::vkCreateBuffer(device, &ib_create_info, ptr::null(), &mut index_buffer);
						}
						assert!(res == vkrust::VkResult::VK_SUCCESS);

						let mut mem_reqs: vkrust::VkMemoryRequirements;
						unsafe {
							mem_reqs = std::mem::uninitialized();
							vkrust::vkGetBufferMemoryRequirements(device, index_buffer, &mut mem_reqs);
						}
						let mem_alloc = vkrust::VkMemoryAllocateInfo {
							sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
							pNext: ptr::null(),
							allocationSize: mem_reqs.size,
							memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
						};
						unsafe {
							res = vkrust::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut index_mem);
							assert!(res == vkrust::VkResult::VK_SUCCESS);
							let mut data: *mut libc::c_void = ptr::null_mut();
							res = vkrust::vkMapMemory(device, index_mem, 0, mem_alloc.allocationSize, 0, &mut data);
							assert!(res == vkrust::VkResult::VK_SUCCESS);
							assert!(data != ptr::null_mut());
							libc::memcpy(data, indicies.as_ptr() as *mut libc::c_void, (num_indicies as usize * index_size as usize) as libc::size_t);
							vkrust::vkUnmapMemory(device, index_mem);
							res = vkrust::vkBindBufferMemory(device, index_buffer, index_mem, 0);
						}
						assert!(res == vkrust::VkResult::VK_SUCCESS);
					}
				}
			}

			#[repr(C)]
			struct UniformBufferData {
				projection_from_view: glm::Mat4,
				view_from_model: glm::Mat4,
				world_from_model: glm::Mat4
			};

			println!("ubsz {}", std::mem::size_of::<UniformBufferData>());

			// Uniform buffers
			println!("Creating uniform buffers");
			let mut uniform_buffer: vkrust::VkBuffer = 0;
			let mut uniform_buffer_mem: vkrust::VkDeviceMemory = 0;
			{
				let ub_create_info = vkrust::VkBufferCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkrust::VkBufferCreateFlags::_EMPTY,
					size: std::mem::size_of::<UniformBufferData>() as u64,
					usage: vkrust::VkBufferUsageFlags::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
					sharingMode: vkrust::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
					queueFamilyIndexCount: 0,
					pQueueFamilyIndices: ptr::null()
				};

				unsafe {
					res = vkrust::vkCreateBuffer(device, &ub_create_info, ptr::null(), &mut uniform_buffer);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
				}

				let mut mem_reqs: vkrust::VkMemoryRequirements;
				unsafe {
					mem_reqs = std::mem::uninitialized();
					vkrust::vkGetBufferMemoryRequirements(device, uniform_buffer, &mut mem_reqs);
				}
				let mem_alloc = vkrust::VkMemoryAllocateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
					pNext: ptr::null(),
					allocationSize: mem_reqs.size,
					memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
				};
				unsafe {
					res = vkrust::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut uniform_buffer_mem);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkBindBufferMemory(device, uniform_buffer, uniform_buffer_mem, 0);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);


				// Update uniform buffer
				let projection = glm::ext::perspective(glm::radians(60.0), WIDTH as f32 / HEIGHT as f32, 0.01, 100.0);
				let view = glm::ext::translate(&num::one(), glm::vec3(0.0, 0.0, -2.5));
				let model: glm::Mat4 = num::one();
				let mut ub_data = UniformBufferData {
					projection_from_view: projection,
					view_from_model: view,
					world_from_model: model
				};

				unsafe {
					let mut data: *mut libc::c_void = ptr::null_mut();
					res = vkrust::vkMapMemory(device, uniform_buffer_mem, 0, mem_alloc.allocationSize, 0, &mut data);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					assert!(data != ptr::null_mut());
					libc::memcpy(data, (&mut ub_data as *mut UniformBufferData) as *mut libc::c_void, std::mem::size_of::<UniformBufferData>() as libc::size_t);
					vkrust::vkUnmapMemory(device, index_mem);
				}
			}

			// Descriptor set layout
			println!("Creating descriptor set layout");
			let mut descriptor_set_layout: vkrust::VkDescriptorSetLayout = 0;
			let mut pipeline_layout: vkrust::VkPipelineLayout = 0;
			{
				let dsl_binding = vkrust::VkDescriptorSetLayoutBinding {
					binding: 0,
					descriptorType: vkrust::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
					descriptorCount: 1,
					stageFlags: vkrust::VkShaderStageFlags::VK_SHADER_STAGE_VERTEX_BIT,
					pImmutableSamplers: ptr::null()
				};
				let dsl_create_info = vkrust::VkDescriptorSetLayoutCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkrust::VkDescriptorSetLayoutCreateFlags::_EMPTY,
					bindingCount: 1,
					pBindings: &dsl_binding
				};
				unsafe {
					res = vkrust::vkCreateDescriptorSetLayout(device, &dsl_create_info, ptr::null(), &mut descriptor_set_layout);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);

				let pl_create_info = vkrust::VkPipelineLayoutCreateInfo {
					sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					setLayoutCount: 1,
					pSetLayouts: &descriptor_set_layout,
					pushConstantRangeCount: 0,
					pPushConstantRanges: ptr::null()
				};
				unsafe {
					res = vkrust::vkCreatePipelineLayout(device, &pl_create_info, ptr::null(), &mut pipeline_layout);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			// Pipelines
			println!("Creating pipeline");
			{
			}

			unsafe {
				for i in 0..swapchain_image_count {
					vkrust::vkDestroyFramebuffer(device, framebuffers[i as usize], ptr::null());
				};
				vkrust::vkDestroyRenderPass(device, render_pass, ptr::null());

				vkrust::vkDestroyImage(device, ds_image, ptr::null());
				vkrust::vkFreeMemory(device, ds_mem, ptr::null());
				vkrust::vkDestroyImageView(device, ds_image_view, ptr::null());

				vkrust::vkFreeCommandBuffers(device, command_pool, command_buffers.len() as u32, command_buffers.as_ptr());
				vkrust::vkDestroyCommandPool(device, command_pool, ptr::null());
				for i in 0..swapchain_image_count {
					vkrust::vkDestroyImageView(device, swapchain_image_views[i as usize], ptr::null());
				}
				vkrust::vkDestroySwapchainKHR(device, swapchain, ptr::null());
			}
		}
		unsafe {
			vkrust::vkDestroySurfaceKHR(instance, wsi_info.2, ptr::null());
		}
	}

	unsafe {
		vkrust::vkDestroyDevice(device, ptr::null());
		vkrust::vkDestroyInstance(instance, ptr::null());
	}
}
