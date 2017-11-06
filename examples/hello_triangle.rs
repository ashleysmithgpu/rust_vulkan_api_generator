
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

		unsafe {
			vkrust::vkrust::vkCreateXcbSurfaceKHR(instance, &surface_create_info, ptr::null(), &mut surface);
		}
	}

	(conn, win, surface)
}

fn main() {

	use vkrust::*;

	let mut res: vkrust::VkResult;
	let mut instance: vkrust::VkInstance = 0;
	{
		let enabled_layers_rust = vec![
			"VK_LAYER_LUNARG_standard_validation".to_string(),
		];
		let enabled_extensions_rust = vec![
			"VK_KHR_surface".to_string(),
			"VK_KHR_xcb_surface".to_string()
		];

		let enabled_layers: Vec<*const u8> = vec![
			enabled_layers_rust[0].as_ptr()
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
			"VK_LAYER_LUNARG_standard_validation".to_string(),
		];
		let enabled_extensions_rust = vec![
			"VK_KHR_swapchain".to_string()
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

			let number_of_swapchain_images = surface_capabilities.maxImageCount;

			let swapchain_transform = vkrust::VkSurfaceTransformFlagsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;

			let composite_alpha = vkrust::VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;

			let mut swapchain_create_info = vkrust::VkSwapchainCreateInfoKHR {
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
					vkrust::vkCreateSwapchainKHR(device, &mut swapchain_create_info, ptr::null(), &mut swapchain);
				}
			}

			unsafe {
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
