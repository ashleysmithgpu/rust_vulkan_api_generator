
extern crate vkrust;

#[cfg(feature="xcb")]
extern crate xcb;

use std::ptr;

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
			150, 150,
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

	let mut res: vkrust::VkResult;
	let mut instance: vkrust::VkInstance = 0;
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

	// Create instance
	unsafe {
		res = vkrust::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
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

	let device_create_info = vkrust::VkDeviceCreateInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
		pNext: ptr::null(),
		flags: 0,
		queueCreateInfoCount: 1,
		pQueueCreateInfos: &queue_create_info,
		enabledLayerCount: enabled_layers.len() as u32,
		ppEnabledLayerNames: enabled_layers.as_ptr() as *const u8,
		enabledExtensionCount: 0,
		ppEnabledExtensionNames: ptr::null(),
		pEnabledFeatures: ptr::null()
	};

	// Create device
	let mut device: vkrust::VkDevice = 0;
	unsafe {
		res = vkrust::vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
	};
	assert!(device != vkrust::VK_NULL_HANDLE);
	assert!(res == vkrust::VkResult::VK_SUCCESS);

	{
		let wsi_info = create_wsi(instance);
		{
			// Get present and graphics queue index

			let mut queue_count = 0;
			unsafe {
				vkrust::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, ptr::null_mut());
			}
			assert!(queue_count > 0);

			let mut queue_props = Vec::<vkrust::VkQueueFamilyProperties>::with_capacity(queue_count as usize);
			let mut queue_supports_present = Vec::<bool>::with_capacity(queue_count as usize);
			unsafe {
				vkrust::vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_count, queue_props.as_mut_ptr());
				queue_props.set_len(queue_count as usize);
				queue_supports_present.set_len(queue_count as usize);
			}

			for i in 0..queue_count {
				unsafe {
					vkrust::vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, i, wsi_info.2, &mut queue_supports_present[i as usize]);
				}
			}

			let mut graphics_and_present_queue_index = 0;
			let mut found_good_queue = false;
			for (i,prop) in queue_props.iter().enumerate() {
				if !(prop.queueFlags & vkrust::VkQueueFlags::VK_QUEUE_GRAPHICS_BIT).is_empty() && queue_supports_present[i] {
					graphics_and_present_queue_index = i;
					found_good_queue = true;
				}
			}
			assert!(found_good_queue);

			// Get a supported colour format and colour space
			let mut format_count = 0;
			unsafe {
				vkrust::vkGetPhysicalDeviceSurfaceFormatsKHR(physical_device, wsi_info.2, &mut format_count, ptr::null_mut());
			}
			assert!(format_count > 0);

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
