
extern crate vkrust;

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

fn get_memory_type(type_bits: u32, properties: vkrust::vkrust::VkMemoryPropertyFlags, device_memory_properties: vkrust::vkrust::VkPhysicalDeviceMemoryProperties) -> Option<u32> {
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
				mem_alloc.memoryTypeIndex = get_memory_type(mem_reqs.memoryTypeBits, vkrust::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, global_memory_properties).unwrap();
				unsafe {
					res = vkrust::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut ds_mem);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkBindImageMemory(device, ds_image, ds_mem, 0);
					assert!(res == vkrust::VkResult::VK_SUCCESS);
					res = vkrust::vkCreateImageView(device, &ds_view, ptr::null(), &mut ds_image_view);
				}
				assert!(res == vkrust::VkResult::VK_SUCCESS);
			}

			unsafe {
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
