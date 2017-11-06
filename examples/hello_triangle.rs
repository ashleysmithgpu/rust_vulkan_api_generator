
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
			connection: &mut conn.get_raw_conn(),
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

	let mut device: vkrust::VkDevice = 0;

	unsafe {
		res = vkrust::vkCreateDevice(physical_device, &device_create_info, ptr::null(), &mut device);
	};
	assert!(device != vkrust::VK_NULL_HANDLE);
	assert!(res == vkrust::VkResult::VK_SUCCESS);

	{
		let wsi_info = create_wsi(instance);
		{
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
