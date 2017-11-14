
extern crate vkraw;
extern crate libc;
extern crate glm;
extern crate num;
#[cfg(feature="xcb")]
extern crate xcb;

use std::ptr;
use std::io::prelude::*;
use std::fs::File;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[cfg(feature="xcb")]
fn create_wsi(instance: vkraw::vkraw::VkInstance, vk: &vkraw::vkraw::VulkanFunctionPointers) -> (xcb::Connection, u32, u64) {

	let mut surface: vkraw::vkraw::VkSurfaceKHR = 0;
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
			screen.root_visual(), &[
				(xcb::CW_BACK_PIXEL, screen.white_pixel()),
				(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS),
			]
		);
		xcb::map_window(&conn, win);
		conn.flush();

		let surface_create_info = vkraw::vkraw::VkXcbSurfaceCreateInfoKHR {
			sType: vkraw::vkraw::VkStructureType::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
			pNext: ptr::null(),
			flags: 0,
			connection: conn.get_raw_conn(),
			window: win
		};

		assert!(vk.CreateXcbSurfaceKHR.is_some());
		let res = vk.CreateXcbSurfaceKHR.unwrap()(instance, &surface_create_info, ptr::null(), &mut surface);
		assert!(res == vkraw::vkraw::VkResult::VK_SUCCESS);
	}

	(conn, win, surface)
}

fn load_spirv_shader_from_disk(device: vkraw::vkraw::VkDevice, filename: &str) -> Option<vkraw::vkraw::VkShaderModule> {

	// Load file contents in to buffer
	let mut f = File::open(filename).unwrap();
	let mut buffer = Vec::new();
	f.read_to_end(&mut buffer).unwrap();

	let mut shader_mod: vkraw::vkraw::VkShaderModule = 0;

	let mod_create_info = vkraw::vkraw::VkShaderModuleCreateInfo {
		sType:  vkraw::vkraw::VkStructureType::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
		pNext: ptr::null(),
		flags: 0,
		codeSize: buffer.len() as u64,
		pCode: buffer.as_mut_ptr() as *mut u32
	};

	let res;
	unsafe {
		res = vkraw::vkraw::vkCreateShaderModule(device, &mod_create_info, ptr::null(), &mut shader_mod);
	}
	if res == vkraw::vkraw::VkResult::VK_SUCCESS {
		Some(shader_mod)
	} else {
		None
	}
}

// Finds a memory type that supports exactly the properties we want
fn get_memory_type(type_bits: u32, properties: vkraw::vkraw::VkMemoryPropertyFlags, device_memory_properties: &vkraw::vkraw::VkPhysicalDeviceMemoryProperties) -> Option<u32> {

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

	use vkraw::*;

	// Create the instance, potentially enabling the validation layers
	let mut res: vkraw::VkResult;
	let mut instance: vkraw::VkInstance = 0;
	{
		let enabled_layers_rust = vec![
			std::ffi::CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()
		];
		let enabled_extensions_rust = vec![
			std::ffi::CString::new("VK_KHR_surface").unwrap(),
			std::ffi::CString::new("VK_KHR_xcb_surface").unwrap()
		];

#[cfg(debug_assertions)]
		let enabled_layers: Vec<*const u8> = vec![
			enabled_layers_rust[0].as_ptr() as *const u8
		];
#[cfg(not(debug_assertions))]
		let enabled_layers: Vec<*const u8> = vec![
		];
		let enabled_extensions: Vec<*const u8> = vec![
			enabled_extensions_rust[0].as_ptr() as *const u8,
			enabled_extensions_rust[1].as_ptr() as *const u8
		];

		let app_name = std::ffi::CString::new("app name").unwrap();
		let engine_name = std::ffi::CString::new("engine name").unwrap();
		let application_info = vkraw::VkApplicationInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
			pNext: ptr::null(),
			pApplicationName: app_name.as_ptr() as *const u8,
			applicationVersion: vkraw::VK_MAKE_VERSION(1,0,0),
			pEngineName: engine_name.as_ptr() as *const u8,
			engineVersion: vkraw::VK_MAKE_VERSION(1,0,0),
			apiVersion: vkraw::VK_MAKE_VERSION(1,0,0),
		};
		let create_info = vkraw::VkInstanceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
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
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};
	}
	assert!(instance != vkraw::VK_NULL_HANDLE);
	assert!(res == vkraw::VkResult::VK_SUCCESS);

	// This will load all of the extension function pointers that we know about
	let vk = vkraw::VulkanFunctionPointers::new(instance);

	let mut num_physical_devices = 0;

	unsafe {
		vkraw::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, 0 as *mut u64);
	}
	assert!(num_physical_devices > 0);

	let mut physical_device: vkraw::VkPhysicalDevice = 0;
	unsafe {
		vkraw::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, &mut physical_device);
	}

	assert!(physical_device != vkraw::VK_NULL_HANDLE);

	let mut global_memory_properties: vkraw::VkPhysicalDeviceMemoryProperties;

	unsafe {
		global_memory_properties = std::mem::uninitialized();
		vkraw::vkGetPhysicalDeviceMemoryProperties(physical_device, &mut global_memory_properties);
	}

	let priorities: [f32; 1] = [1.0];

	let queue_create_info = vkraw::VkDeviceQueueCreateInfo {
		sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
		pNext: ptr::null(),
		flags: 0,
		queueFamilyIndex: 0,
		queueCount: 1,
		pQueuePriorities: &priorities as *const f32
	};

	// Create the device
	let mut device: vkraw::VkDevice = 0;
	{
		let enabled_extensions_rust = vec![
			std::ffi::CString::new("VK_KHR_swapchain").unwrap()
		];

		let enabled_layers: Vec<*const u8> = vec![
		];
		let enabled_extensions: Vec<*const u8> = vec![
			enabled_extensions_rust[0].as_ptr() as *const u8
		];
		let device_create_info = vkraw::VkDeviceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
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
			res = vkraw::vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
		};
		assert!(device != vkraw::VK_NULL_HANDLE);
		assert!(res == vkraw::VkResult::VK_SUCCESS);
	}

	{
		// Create the window system
		let wsi_info = create_wsi(instance, &vk);
		{
			// Get present and graphics queue index
			let mut queue_count = 0;
			unsafe {
				vkraw::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, ptr::null_mut());
			}
			assert!(queue_count > 0);
			println!("Found {} queues:", queue_count);

			let mut queue_props = Vec::<vkraw::VkQueueFamilyProperties>::with_capacity(queue_count as usize);
			let mut queue_supports_present = Vec::<vkraw::VkBool32>::with_capacity(queue_count as usize);
			unsafe {
				vkraw::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, queue_props.as_mut_ptr());
				queue_props.set_len(queue_count as usize);
				queue_supports_present.set_len(queue_count as usize);
			}

			let mut graphics_and_present_queue_index = 0;
			let mut found_good_queue = false;
			for (i,prop) in queue_props.iter().enumerate() {
				print!(" Queue {} supports: ", i);
				assert!(vk.GetPhysicalDeviceSurfaceSupportKHR.is_some());
				vk.GetPhysicalDeviceSurfaceSupportKHR.unwrap()(physical_device, i as u32, wsi_info.2, &mut queue_supports_present[i as usize]);
				if !(prop.queueFlags & vkraw::VkQueueFlags::VK_QUEUE_GRAPHICS_BIT).is_empty() {
					print!(" graphics, ");
				}
				if queue_supports_present[i as usize] > 0 {
					print!(" present, ");
				}
				if !(prop.queueFlags & vkraw::VkQueueFlags::VK_QUEUE_GRAPHICS_BIT).is_empty() && queue_supports_present[i] > 0 {
					graphics_and_present_queue_index = i;
					found_good_queue = true;
				}
				print!("\n");
			}
			assert!(found_good_queue);
			println!("Using queue index {}", graphics_and_present_queue_index);

			let mut queue;
			unsafe {
				queue = std::mem::uninitialized();
				vkraw::vkGetDeviceQueue(device, graphics_and_present_queue_index as u32, 0, &mut queue);
			}

			// Get a supported colour format and colour space
			let mut format_count = 0;
			assert!(vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
			vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(physical_device, wsi_info.2, &mut format_count, ptr::null_mut());
			assert!(format_count > 0);
			println!("Found {} surface formats", format_count);

			let mut surface_formats = Vec::<vkraw::VkSurfaceFormatKHR>::with_capacity(format_count as usize);
			unsafe {
				surface_formats.set_len(format_count as usize);
			}
			assert!(vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
			vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(physical_device, wsi_info.2, &mut format_count, surface_formats.as_mut_ptr());

			let mut colour_format = vkraw::VkFormat::VK_FORMAT_B8G8R8A8_UNORM;
			let mut colour_space = vkraw::VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
			if format_count == 1 && surface_formats[0].format == vkraw::VkFormat::VK_FORMAT_UNDEFINED {

				colour_space = surface_formats[0].colorSpace.clone();
			} else {

				let mut found_b8g8r8a8_unorm = false;
				for fmt in &surface_formats {
					if fmt.format == vkraw::VkFormat::VK_FORMAT_B8G8R8A8_UNORM {
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
			let mut surface_capabilities: vkraw::VkSurfaceCapabilitiesKHR;
			unsafe {
				surface_capabilities = std::mem::uninitialized();
			}
			assert!(vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.is_some());
			vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(physical_device, wsi_info.2, &mut surface_capabilities);

			let mut present_mode_count = 0;
			assert!(vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
			vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(physical_device, wsi_info.2, &mut present_mode_count, ptr::null_mut());
			assert!(present_mode_count > 0);
			let mut present_modes = Vec::<vkraw::VkPresentModeKHR>::with_capacity(present_mode_count as usize);
			unsafe {
				present_modes.set_len(present_mode_count as usize);
			}
			assert!(vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
			vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(physical_device, wsi_info.2, &mut present_mode_count, present_modes.as_mut_ptr());

			println!("Found {} present modes", present_mode_count);

			// TODO: check these properly
			let present_mode = vkraw::VkPresentModeKHR::VK_PRESENT_MODE_FIFO_KHR;

			let number_of_swapchain_images = surface_capabilities.minImageCount;

			let swapchain_transform = vkraw::VkSurfaceTransformFlagsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;

			let composite_alpha = vkraw::VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;

			let swapchain_create_info = vkraw::VkSwapchainCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: vkraw::VkSwapchainCreateFlagBitsKHR::_EMPTY,
				surface: wsi_info.2,
				minImageCount: number_of_swapchain_images,
				imageFormat: colour_format,
				imageColorSpace: colour_space,
				imageExtent: vkraw::VkExtent2D{ width: WIDTH, height: HEIGHT },
				imageArrayLayers: 1,
				imageUsage: vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
				imageSharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
				queueFamilyIndexCount: 0,
				pQueueFamilyIndices: ptr::null(),
				preTransform: swapchain_transform,
				compositeAlpha: composite_alpha,
				presentMode: present_mode,
				clipped: vkraw::VK_TRUE,
				oldSwapchain: vkraw::VK_NULL_HANDLE
			};

			let mut swapchain: vkraw::VkSwapchainKHR = 0;
			{
				assert!(vk.CreateSwapchainKHR.is_some());
				res = vk.CreateSwapchainKHR.unwrap()(device, &swapchain_create_info, ptr::null(), &mut swapchain);
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			let mut swapchain_image_count = 0;
			assert!(vk.GetSwapchainImagesKHR.is_some());
			vk.GetSwapchainImagesKHR.unwrap()(device, swapchain, &mut swapchain_image_count, ptr::null_mut());
			assert!(swapchain_image_count > 0);
			println!("Creating {} swapchain images", swapchain_image_count);
			let mut swapchain_images = Vec::<vkraw::VkImage>::with_capacity(swapchain_image_count as usize);
			unsafe {
				swapchain_images.set_len(swapchain_image_count as usize);
			}
			assert!(vk.GetSwapchainImagesKHR.is_some());
			vk.GetSwapchainImagesKHR.unwrap()(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr());

			let mut swapchain_image_views = Vec::<vkraw::VkImageView>::with_capacity(swapchain_image_count as usize);
			unsafe {
				swapchain_image_views.set_len(swapchain_image_count as usize);
			}
			for i in 0..swapchain_image_count {
				let img_create_info = vkraw::VkImageViewCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					image: swapchain_images[i as usize],
					viewType: vkraw::VkImageViewType::VK_IMAGE_VIEW_TYPE_2D,
					format: colour_format,
					components: vkraw::VkComponentMapping {
						r: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_R,
						g: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_G,
						b: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_B,
						a: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_A
					},
					subresourceRange: vkraw::VkImageSubresourceRange {
						aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
						baseMipLevel: 0,
						levelCount: 1,
						baseArrayLayer: 0,
						layerCount: 1
					}
				};

				unsafe{
					res = vkraw::vkCreateImageView(device, &img_create_info, ptr::null(), &mut swapchain_image_views[i as usize]);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Create command pool
			println!("Creating command pool");
			let mut command_pool: vkraw::VkCommandPool = 0;
			{
				let pool_create_info = vkraw::VkCommandPoolCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkCommandPoolCreateFlags::VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
					queueFamilyIndex: 0
				};
				unsafe {
					res = vkraw::vkCreateCommandPool(device, &pool_create_info, ptr::null(), &mut command_pool);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}


			// Create command buffers
			println!("Creating command buffers");
			let mut command_buffers = Vec::<vkraw::VkCommandBuffer>::with_capacity(swapchain_image_count as usize);
			{
				let cmd_buf_create_info = vkraw::VkCommandBufferAllocateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
					pNext: ptr::null(),
					commandPool: command_pool,
					level: vkraw::VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
					commandBufferCount: swapchain_image_count
				};

				unsafe {
					command_buffers.set_len(swapchain_image_count as usize);
					res = vkraw::vkAllocateCommandBuffers(device, &cmd_buf_create_info, command_buffers.as_mut_ptr());
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Create depth stencil
			println!("Creating depth/stencil images");
			let mut ds_image: vkraw::VkImage = 0;
			let mut ds_image_view: vkraw::VkImageView = 0;
			let mut ds_mem: vkraw::VkDeviceMemory = 0;
			let depth_format = vkraw::VkFormat::VK_FORMAT_D24_UNORM_S8_UINT;
			{
				let image_create_info = vkraw::VkImageCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkImageCreateFlags::empty(),
					imageType: vkraw::VkImageType::VK_IMAGE_TYPE_2D,
					format: depth_format,
					extent: vkraw::VkExtent3D { width: WIDTH, height: HEIGHT, depth: 1 },
					mipLevels: 1,
					arrayLayers: 1,
					samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
					tiling: vkraw::VkImageTiling::VK_IMAGE_TILING_OPTIMAL,
					usage: vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT | vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
					sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
					queueFamilyIndexCount: 0,
					pQueueFamilyIndices: ptr::null(),
					initialLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED
				};
				let mut mem_alloc = vkraw::VkMemoryAllocateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
					pNext: ptr::null(),
					allocationSize: 0,
					memoryTypeIndex: 0
				};
				let mut mem_reqs: vkraw::VkMemoryRequirements;

				unsafe {
					res = vkraw::vkCreateImage(device, &image_create_info, ptr::null(), &mut ds_image);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					mem_reqs = std::mem::uninitialized();
					vkraw::vkGetImageMemoryRequirements(device, ds_image, &mut mem_reqs);
				}
				let ds_view = vkraw::VkImageViewCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					image: ds_image,
					viewType: vkraw::VkImageViewType::VK_IMAGE_VIEW_TYPE_2D,
					format: depth_format,
					components: vkraw::VkComponentMapping {
						r: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_R,
						g: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY,
						b: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY,
						a: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_IDENTITY
					},
					subresourceRange: vkraw::VkImageSubresourceRange {
						aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_DEPTH_BIT | vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_STENCIL_BIT,
						baseMipLevel: 0,
						levelCount: 1,
						baseArrayLayer: 0,
						layerCount: 1
					}
				};
				mem_alloc.allocationSize = mem_reqs.size;
				mem_alloc.memoryTypeIndex = get_memory_type(mem_reqs.memoryTypeBits, vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, &global_memory_properties).unwrap();
				unsafe {
					res = vkraw::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut ds_mem);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					res = vkraw::vkBindImageMemory(device, ds_image, ds_mem, 0);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					res = vkraw::vkCreateImageView(device, &ds_view, ptr::null(), &mut ds_image_view);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Create render pass
			println!("Creating render pass");
			let mut render_pass: vkraw::VkRenderPass = 0;
			{
				let attachments = [
					vkraw::VkAttachmentDescription {
						flags: vkraw::VkAttachmentDescriptionFlags::_EMPTY,
						format: colour_format,
						samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
						loadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						storeOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
						stencilLoadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
						stencilStoreOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
						initialLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						finalLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
					},
					vkraw::VkAttachmentDescription {
						flags: vkraw::VkAttachmentDescriptionFlags::_EMPTY,
						format: depth_format,
						samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
						loadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						storeOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
						stencilLoadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
						stencilStoreOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
						initialLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						finalLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
					}
				];

				let colour_reference = vkraw::VkAttachmentReference {
					attachment: 0,
					layout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
				};
				let depth_reference = vkraw::VkAttachmentReference {
					attachment: 1,
					layout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
				};
				let subpass = vkraw::VkSubpassDescription {
					flags: vkraw::VkSubpassDescriptionFlags::_EMPTY,
					pipelineBindPoint: vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS,
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
					vkraw::VkSubpassDependency {
						srcSubpass: vkraw::VK_SUBPASS_EXTERNAL as u32,
						dstSubpass: 0,
						srcStageMask: vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
						dstStageMask: vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
						srcAccessMask: vkraw::VkAccessFlags::VK_ACCESS_MEMORY_READ_BIT,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | vkraw::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
						dependencyFlags: vkraw::VkDependencyFlags::VK_DEPENDENCY_BY_REGION_BIT,
					},
					vkraw::VkSubpassDependency {
						srcSubpass: 0,
						dstSubpass: vkraw::VK_SUBPASS_EXTERNAL as u32,
						srcStageMask: vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
						dstStageMask: vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
						srcAccessMask: vkraw::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | vkraw::VkAccessFlags::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_MEMORY_READ_BIT,
						dependencyFlags: vkraw::VkDependencyFlags::VK_DEPENDENCY_BY_REGION_BIT,
					}
				];
				let render_pass_create_info = vkraw::VkRenderPassCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
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
					res = vkraw::vkCreateRenderPass(device, &render_pass_create_info, ptr::null(), &mut render_pass);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Pipeline cache
			println!("Creating pipeline cache");
			let mut pipeline_cache: vkraw::VkPipeline = 0;
			{
				let pipeline_create_info = vkraw::VkPipelineCacheCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					initialDataSize: 0,
					pInitialData: ptr::null()
				};
				unsafe {
					res = vkraw::vkCreatePipelineCache(device, &pipeline_create_info, ptr::null(), &mut pipeline_cache);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Framebuffer
			println!("Creating framebuffers");
			let mut framebuffers = Vec::<vkraw::VkFramebuffer>::with_capacity(swapchain_image_count as usize);
			{
				unsafe {
					framebuffers.set_len(swapchain_image_count as usize);
				}
				let mut attachments = [vkraw::VK_NULL_HANDLE, ds_image_view];
				let fb_create_info = vkraw::VkFramebufferCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
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
						res = vkraw::vkCreateFramebuffer(device, &fb_create_info, ptr::null(), &mut framebuffers[i as usize]);
					}
					assert!(res == vkraw::VkResult::VK_SUCCESS);
				};
			}

			// Fences and semaphores
			println!("Creating sync prims");
			let mut present_complete_sem: vkraw::VkSemaphore = 0;
			let mut render_complete_sem: vkraw::VkSemaphore = 0;
			let mut fences = Vec::<vkraw::VkFence>::with_capacity(swapchain_image_count as usize);
			{
				let sem_create_info = vkraw::VkSemaphoreCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0
				};
				unsafe {
					fences.set_len(swapchain_image_count as usize);
					res = vkraw::vkCreateSemaphore(device, &sem_create_info, ptr::null(), &mut present_complete_sem);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					res = vkraw::vkCreateSemaphore(device, &sem_create_info, ptr::null(), &mut render_complete_sem);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);

				let fence_create_info = vkraw::VkFenceCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkFenceCreateFlags::_EMPTY
				};
				for i in 0..swapchain_image_count {
					unsafe {
						res = vkraw::vkCreateFence(device, &fence_create_info, ptr::null(), &mut fences[i as usize]);
					}
					assert!(res == vkraw::VkResult::VK_SUCCESS);
				}
			}

			let use_staging = false;

			// Vertex/index data
			println!("Creating verticies/indices");
			let mut vertex_buffer: vkraw::VkBuffer = 0;
			let num_vertices = 3;
			let vertex_size = std::mem::size_of::<f32>() * 6;
			let mut vertex_mem: vkraw::VkDeviceMemory = 0;

			let mut index_buffer: vkraw::VkBuffer = 0;
			let num_indices = 3;
			let index_size = std::mem::size_of::<u32>();
			let mut index_mem: vkraw::VkDeviceMemory = 0;
			{
				let vertices: [f32; 18] = [
					1.0, 1.0, 0.0,	1.0, 0.0, 0.0,
					-1.0, 1.0, 0.0,	0.0, 1.0, 0.0,
					0.0, -1.0, 0.0,	0.0, 0.0, 1.0
				];

				let indices: [u32; 3] = [0, 1, 2];

				if use_staging {
					// TODO
				} else {
					{
						let vb_create_info = vkraw::VkBufferCreateInfo {
							sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
							pNext: ptr::null(),
							flags: vkraw::VkBufferCreateFlags::_EMPTY,
							size: num_vertices * vertex_size as u64,
							usage: vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
							sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
							queueFamilyIndexCount: 0,
							pQueueFamilyIndices: ptr::null()
						};

						unsafe {
							res = vkraw::vkCreateBuffer(device, &vb_create_info, ptr::null(), &mut vertex_buffer);
						}
						assert!(res == vkraw::VkResult::VK_SUCCESS);

						let mut mem_reqs: vkraw::VkMemoryRequirements;
						unsafe {
							mem_reqs = std::mem::uninitialized();
							vkraw::vkGetBufferMemoryRequirements(device, vertex_buffer, &mut mem_reqs);
						}
						let mem_alloc = vkraw::VkMemoryAllocateInfo {
							sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
							pNext: ptr::null(),
							allocationSize: mem_reqs.size,
							memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
						};
						unsafe {
							res = vkraw::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut vertex_mem);
							assert!(res == vkraw::VkResult::VK_SUCCESS);
							let mut data: *mut libc::c_void = ptr::null_mut();
							res = vkraw::vkMapMemory(device, vertex_mem, 0, mem_alloc.allocationSize, 0, &mut data);
							assert!(res == vkraw::VkResult::VK_SUCCESS);
							assert!(data != ptr::null_mut());
							libc::memcpy(data, vertices.as_ptr() as *mut libc::c_void, (num_vertices as usize * vertex_size as usize) as libc::size_t);
							vkraw::vkUnmapMemory(device, vertex_mem);
							res = vkraw::vkBindBufferMemory(device, vertex_buffer, vertex_mem, 0);
						}
						assert!(res == vkraw::VkResult::VK_SUCCESS);
					}

					{
						let ib_create_info = vkraw::VkBufferCreateInfo {
							sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
							pNext: ptr::null(),
							flags: vkraw::VkBufferCreateFlags::_EMPTY,
							size: num_indices * index_size as u64,
							usage: vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
							sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
							queueFamilyIndexCount: 0,
							pQueueFamilyIndices: ptr::null()
						};

						unsafe {
							res = vkraw::vkCreateBuffer(device, &ib_create_info, ptr::null(), &mut index_buffer);
						}
						assert!(res == vkraw::VkResult::VK_SUCCESS);

						let mut mem_reqs: vkraw::VkMemoryRequirements;
						unsafe {
							mem_reqs = std::mem::uninitialized();
							vkraw::vkGetBufferMemoryRequirements(device, index_buffer, &mut mem_reqs);
						}
						let mem_alloc = vkraw::VkMemoryAllocateInfo {
							sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
							pNext: ptr::null(),
							allocationSize: mem_reqs.size,
							memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
						};
						unsafe {
							res = vkraw::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut index_mem);
							assert!(res == vkraw::VkResult::VK_SUCCESS);
							let mut data: *mut libc::c_void = ptr::null_mut();
							res = vkraw::vkMapMemory(device, index_mem, 0, mem_alloc.allocationSize, 0, &mut data);
							assert!(res == vkraw::VkResult::VK_SUCCESS);
							assert!(data != ptr::null_mut());
							libc::memcpy(data, indices.as_ptr() as *mut libc::c_void, (num_indices as usize * index_size as usize) as libc::size_t);
							vkraw::vkUnmapMemory(device, index_mem);
							res = vkraw::vkBindBufferMemory(device, index_buffer, index_mem, 0);
						}
						assert!(res == vkraw::VkResult::VK_SUCCESS);
					}
				}
			}

			#[repr(C)]
			struct UniformBufferData {
				projection_from_view: glm::Mat4,
				view_from_model: glm::Mat4,
				world_from_model: glm::Mat4
			};

			// Uniform buffers
			println!("Creating uniform buffers");
			let mut uniform_buffers = Vec::<vkraw::VkBuffer>::with_capacity(swapchain_image_count as usize);
			let mut uniform_buffers_mem = Vec::<vkraw::VkDeviceMemory>::with_capacity(swapchain_image_count as usize);
			unsafe {
				uniform_buffers.set_len(swapchain_image_count as usize);
				uniform_buffers_mem.set_len(swapchain_image_count as usize);
			}
			for i in 0..swapchain_image_count {
				let ub_create_info = vkraw::VkBufferCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkBufferCreateFlags::_EMPTY,
					size: std::mem::size_of::<UniformBufferData>() as u64,
					usage: vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
					sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
					queueFamilyIndexCount: 0,
					pQueueFamilyIndices: ptr::null()
				};

				unsafe {
					res = vkraw::vkCreateBuffer(device, &ub_create_info, ptr::null(), &mut uniform_buffers[i as usize]);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
				}

				let mut mem_reqs: vkraw::VkMemoryRequirements;
				unsafe {
					mem_reqs = std::mem::uninitialized();
					vkraw::vkGetBufferMemoryRequirements(device, uniform_buffers[i as usize], &mut mem_reqs);
				}
				let mem_alloc = vkraw::VkMemoryAllocateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
					pNext: ptr::null(),
					allocationSize: mem_reqs.size,
					memoryTypeIndex: get_memory_type(mem_reqs.memoryTypeBits, vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkraw::VkMemoryPropertyFlagBits::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, &global_memory_properties).unwrap()
				};
				unsafe {
					res = vkraw::vkAllocateMemory(device, &mem_alloc, ptr::null(), &mut uniform_buffers_mem[i as usize]);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					res = vkraw::vkBindBufferMemory(device, uniform_buffers[i as usize], uniform_buffers_mem[i as usize], 0);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);


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
					res = vkraw::vkMapMemory(device, uniform_buffers_mem[i as usize], 0, mem_alloc.allocationSize, 0, &mut data);
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					assert!(data != ptr::null_mut());
					libc::memcpy(data, (&mut ub_data as *mut UniformBufferData) as *mut libc::c_void, std::mem::size_of::<UniformBufferData>() as libc::size_t);
					vkraw::vkUnmapMemory(device, uniform_buffers_mem[i as usize]);
				}
			}

			// Descriptor set layout
			println!("Creating descriptor set layout");
			let mut descriptor_set_layout: vkraw::VkDescriptorSetLayout = 0;
			let mut pipeline_layout: vkraw::VkPipelineLayout = 0;
			{
				let dsl_binding = vkraw::VkDescriptorSetLayoutBinding {
					binding: 0,
					descriptorType: vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
					descriptorCount: 1,
					stageFlags: vkraw::VkShaderStageFlags::VK_SHADER_STAGE_VERTEX_BIT,
					pImmutableSamplers: ptr::null()
				};
				let dsl_create_info = vkraw::VkDescriptorSetLayoutCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkDescriptorSetLayoutCreateFlags::_EMPTY,
					bindingCount: 1,
					pBindings: &dsl_binding
				};
				unsafe {
					res = vkraw::vkCreateDescriptorSetLayout(device, &dsl_create_info, ptr::null(), &mut descriptor_set_layout);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);

				let pl_create_info = vkraw::VkPipelineLayoutCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					setLayoutCount: 1,
					pSetLayouts: &descriptor_set_layout,
					pushConstantRangeCount: 0,
					pPushConstantRanges: ptr::null()
				};
				unsafe {
					res = vkraw::vkCreatePipelineLayout(device, &pl_create_info, ptr::null(), &mut pipeline_layout);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Pipelines
			println!("Creating pipeline");
			let mut pipeline: vkraw::VkPipeline = 0;
			{
				let shader_entry_point = std::ffi::CString::new("main").unwrap();
				let shader_stages = [vkraw::VkPipelineShaderStageCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_VERTEX_BIT,
					module: load_spirv_shader_from_disk(device, "triangle.vert.spv").unwrap(),
					pName: shader_entry_point.as_ptr() as *const u8,
					pSpecializationInfo: ptr::null()
				},
				vkraw::VkPipelineShaderStageCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_FRAGMENT_BIT,
					module: load_spirv_shader_from_disk(device, "triangle.frag.spv").unwrap(),
					pName: shader_entry_point.as_ptr() as *const u8,
					pSpecializationInfo: ptr::null()
				}];
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
					width: WIDTH as f32,
					height: HEIGHT as f32,
					minDepth: 0.0,
					maxDepth: 1.0
				};
				let scissors = vkraw::VkRect2D {
					offset: vkraw::VkOffset2D {
						x: 0,
						y: 0
					},
					extent: vkraw::VkExtent2D {
						width: WIDTH,
						height: HEIGHT
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
					vkraw::VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT,
					vkraw::VkDynamicState::VK_DYNAMIC_STATE_VIEWPORT
				];
				let dynamic = vkraw::VkPipelineDynamicStateCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
					pNext: ptr::null(),
					flags: 0,
					dynamicStateCount: 1,
					pDynamicStates: dynamic_states.as_ptr()
				};

				let pipeline_create_info = vkraw::VkGraphicsPipelineCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkPipelineCreateFlags::_EMPTY,
					stageCount: shader_stages.len() as u32,
					pStages: shader_stages.as_ptr(),
					pVertexInputState: &vertex_input,
					pInputAssemblyState: &input_assembly,
					pTessellationState: ptr::null(),
					pViewportState: &viewport,
					pRasterizationState: &rasterisation,
					pMultisampleState: &multisample,
					pDepthStencilState: &depth_stencil,
					pColorBlendState: &colour_blend,
					pDynamicState: &dynamic,
					layout: pipeline_layout,
					renderPass: render_pass,
					subpass: 0,
					basePipelineHandle: vkraw::VK_NULL_HANDLE,
					basePipelineIndex: 0
				};

				unsafe {
					res = vkraw::vkCreateGraphicsPipelines(device, pipeline_cache, 1, &pipeline_create_info, ptr::null(), &mut pipeline);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);

				// Shader modules are no longer needed (They are baked in to the pipeline)
				unsafe {
					for i in 0..shader_stages.len() {
						vkraw::vkDestroyShaderModule(device, shader_stages[i].module, ptr::null());
					}
				}
			}

			// Descriptor pool
			println!("Creating descriptor pool");
			let mut descriptor_pool: vkraw::VkDescriptorPool = 0;
			{
				let dtypes = vkraw::VkDescriptorPoolSize {
					_type: vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
					descriptorCount: swapchain_image_count
				};
				let pool_create_info = vkraw::VkDescriptorPoolCreateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
					pNext: ptr::null(),
					flags: vkraw::VkDescriptorPoolCreateFlags::_EMPTY,
					maxSets: swapchain_image_count,
					poolSizeCount: 1,
					pPoolSizes: &dtypes
				};
				unsafe {
					res = vkraw::vkCreateDescriptorPool(device, &pool_create_info, ptr::null(), &mut descriptor_pool);
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

			// Descriptor sets
			println!("Creating descriptor sets");
			let mut descriptor_sets = Vec::<vkraw::VkDescriptorSet>::with_capacity(swapchain_image_count as usize);
			{
				let set_layouts = vec![descriptor_set_layout; swapchain_image_count as usize];
				let ds_alloc = vkraw::VkDescriptorSetAllocateInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
					pNext: ptr::null(),
					descriptorPool: descriptor_pool,
					descriptorSetCount: swapchain_image_count,
					pSetLayouts: set_layouts.as_ptr()
				};
				unsafe {
					descriptor_sets.set_len(swapchain_image_count as usize);
					res = vkraw::vkAllocateDescriptorSets(device, &ds_alloc, descriptor_sets.as_mut_ptr());
				}
				assert!(res == vkraw::VkResult::VK_SUCCESS);
				for i in 0..swapchain_image_count {
					let buffer_info = vkraw::VkDescriptorBufferInfo {
						buffer: uniform_buffers[i as usize],
						offset: 0,
						range: std::mem::size_of::<UniformBufferData>() as u64,
					};
					let write_ds = vkraw::VkWriteDescriptorSet {
						sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
						pNext: ptr::null(),
						dstSet: descriptor_sets[i as usize],
						dstBinding: 0,
						dstArrayElement: 0,
						descriptorCount: 1,
						descriptorType: vkraw::VkDescriptorType::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
						pImageInfo: ptr::null(),
						pBufferInfo: &buffer_info,
						pTexelBufferView: ptr::null()
					};
					unsafe {
						vkraw::vkUpdateDescriptorSets(device, 1, &write_ds, 0, ptr::null());
					}
				}
			}

			// Buliding command buffers
			println!("Building command buffers");
			{
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
					renderPass: render_pass,
					framebuffer: vkraw::VK_NULL_HANDLE,
					renderArea: vkraw::VkRect2D {
						offset: vkraw::VkOffset2D {
							x: 0,
							y: 0
						},
						extent: vkraw::VkExtent2D {
							width: WIDTH,
							height: HEIGHT
						}
					},
					clearValueCount: 2,
					pClearValues: clear_values.as_ptr()
				};
				for i in 0..swapchain_image_count {
					rp_begin_info.framebuffer = framebuffers[i as usize];

					unsafe {
						res = vkraw::vkBeginCommandBuffer(command_buffers[i as usize], &begin_info);
					}
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					unsafe {
						vkraw::vkCmdBeginRenderPass(command_buffers[i as usize], &rp_begin_info, vkraw::VkSubpassContents::VK_SUBPASS_CONTENTS_INLINE);
					}
					let vp = vkraw::VkViewport {
						x: 0.0,
						y: 0.0,
						width: WIDTH as f32,
						height: HEIGHT as f32,
						minDepth: 0.0,
						maxDepth: 1.0,
					};
					unsafe {
						vkraw::vkCmdSetViewport(command_buffers[i as usize], 0, 1, &vp);
					}
					let sc = vkraw::VkRect2D {
						offset: vkraw::VkOffset2D {
							x: 0,
							y: 0
						},
						extent: vkraw::VkExtent2D {
							width: WIDTH,
							height: HEIGHT
						}
					};
					let offset = 0;
					unsafe {
						vkraw::vkCmdSetScissor(command_buffers[i as usize], 0, 1, &sc);
						vkraw::vkCmdBindDescriptorSets(command_buffers[i as usize], vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_layout, 0, 1, &descriptor_sets[(i % 2) as usize], 0, ptr::null());
						vkraw::vkCmdBindPipeline(command_buffers[i as usize], vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline);
						vkraw::vkCmdBindVertexBuffers(command_buffers[i as usize], 0, 1, &vertex_buffer, &offset);
						vkraw::vkCmdBindIndexBuffer(command_buffers[i as usize], index_buffer, 0, vkraw::VkIndexType::VK_INDEX_TYPE_UINT32);
						vkraw::vkCmdDrawIndexed(command_buffers[i as usize], num_indices as u32, 1, 0, 0, 1);
						vkraw::vkCmdEndRenderPass(command_buffers[i as usize]);
						vkraw::vkEndCommandBuffer(command_buffers[i as usize]);
					}
				}
			}


			let mut current_buffer = 0;
			let mut frame_index = 0;

			let mut rotation = 0.0;

			// Render loop
			loop {

				println!("Frame {}", frame_index);

				let event = wsi_info.0.poll_for_event();
				match event {
					None => {}
					Some(event) => {
						let r = event.response_type() & !0x80;
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
							_ => {}
						}
					}
				}

				assert!(vk.AcquireNextImageKHR.is_some());
				res = vk.AcquireNextImageKHR.unwrap()(device, swapchain, std::u64::MAX, present_complete_sem, vkraw::VK_NULL_HANDLE, &mut current_buffer);
				assert!(res == vkraw::VkResult::VK_SUCCESS);

				if frame_index > 1 {
					unsafe {
						res = vkraw::vkWaitForFences(device, 1, &fences[current_buffer as usize], vkraw::VK_TRUE, std::u64::MAX);
					}
					assert!(res == vkraw::VkResult::VK_SUCCESS);
					unsafe {
						res = vkraw::vkResetFences(device, 1, &fences[current_buffer as usize]);
					}
					assert!(res == vkraw::VkResult::VK_SUCCESS);
				}


				// Per frame logic
				// TODO: index returned from vkAcquireNextImageKHR may not be sequential
				// need to allocate uniform buffers[num swapchain images][num frames in flight at once]
				{
					let projection = glm::ext::perspective(glm::radians(60.0), WIDTH as f32 / HEIGHT as f32, 0.01, 100.0);
					let view = glm::ext::translate(&num::one(), glm::vec3(0.0, 0.0, -2.5));
					let model: glm::Mat4 = glm::ext::rotate(&num::one(), rotation, glm::vec3(0.0, 0.0, 1.0));
					let mut ub_data = UniformBufferData {
						projection_from_view: projection,
						view_from_model: view,
						world_from_model: model
					};
					rotation += 0.01;

					unsafe {
						let mut data: *mut libc::c_void = ptr::null_mut();
						res = vkraw::vkMapMemory(device, uniform_buffers_mem[current_buffer as usize], 0, std::mem::size_of::<UniformBufferData>() as u64, 0, &mut data);
						assert!(res == vkraw::VkResult::VK_SUCCESS);
						assert!(data != ptr::null_mut());
						libc::memcpy(data, (&mut ub_data as *mut UniformBufferData) as *mut libc::c_void, std::mem::size_of::<UniformBufferData>() as libc::size_t);
						vkraw::vkUnmapMemory(device, uniform_buffers_mem[current_buffer as usize]);
					}
				}

				let wait_stage_mask = vkraw::VkPipelineStageFlags::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
				let submit_info = vkraw::VkSubmitInfo {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SUBMIT_INFO,
					pNext: ptr::null(),
					waitSemaphoreCount: 1,
					pWaitSemaphores: &present_complete_sem,
					pWaitDstStageMask: &wait_stage_mask,
					commandBufferCount: 1,
					pCommandBuffers: &command_buffers[current_buffer as usize],
					signalSemaphoreCount: 1,
					pSignalSemaphores: &render_complete_sem
				};

				unsafe {
					vkraw::vkQueueSubmit(queue, 1, &submit_info, fences[current_buffer as usize]);
				}

				let mut result = vkraw::VkResult::VK_SUCCESS;
				let mut image_indices = current_buffer;
				let present_info = vkraw::VkPresentInfoKHR {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
					pNext: ptr::null(),
					waitSemaphoreCount: 1,
					pWaitSemaphores: &render_complete_sem,
					swapchainCount: 1,
					pSwapchains: &swapchain,
					pImageIndices: &mut image_indices,
					pResults: &mut result
				};
				assert!(vk.QueuePresentKHR.is_some());
				vk.QueuePresentKHR.unwrap()(queue, &present_info);
				frame_index += 1;
			}

			unsafe {
				vkraw::vkDeviceWaitIdle(device);

				for i in 0..swapchain_image_count {
					vkraw::vkDestroyFramebuffer(device, framebuffers[i as usize], ptr::null());
				};
				vkraw::vkDestroyRenderPass(device, render_pass, ptr::null());

				vkraw::vkDestroyImage(device, ds_image, ptr::null());
				vkraw::vkFreeMemory(device, ds_mem, ptr::null());
				vkraw::vkDestroyImageView(device, ds_image_view, ptr::null());

				vkraw::vkFreeCommandBuffers(device, command_pool, command_buffers.len() as u32, command_buffers.as_ptr());
				vkraw::vkDestroyCommandPool(device, command_pool, ptr::null());
				for i in 0..swapchain_image_count {
					vkraw::vkDestroyImageView(device, swapchain_image_views[i as usize], ptr::null());
				}
			}
			assert!(vk.DestroySwapchainKHR.is_some());
			vk.DestroySwapchainKHR.unwrap()(device, swapchain, ptr::null());
		}
		assert!(vk.DestroySurfaceKHR.is_some());
		vk.DestroySurfaceKHR.unwrap()(instance, wsi_info.2, ptr::null());
	}

	unsafe {
		vkraw::vkDestroyDevice(device, ptr::null());
		vkraw::vkDestroyInstance(instance, ptr::null());
	}
}
