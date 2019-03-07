
use std::ptr;
use std::ffi::CString;
use std::mem;

#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
	use std::os::windows::ffi::OsStrExt;
	std::ffi::OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
}

pub struct Instance {
	pub instance: vkraw::VkInstance,
	vk: vkraw::VulkanFunctionPointers,
	callback: vkraw::VkDebugReportCallbackEXT,
}

pub struct Device<'a> {
	pub device: vkraw::VkDevice,
	pub instance: &'a Instance
}

pub struct Buffer<'a> {
	pub buffer: vkraw::VkBuffer,
	pub device: &'a Device<'a>
}

pub struct Surface<'a> {
	pub surface: vkraw::VkSurfaceKHR,
	instance: &'a Instance
}

pub struct PhysicalDevice<'a> {
	pub physical_device: vkraw::VkPhysicalDevice,
	pub instance: &'a Instance
}

pub struct Swapchain<'a, 'b> {
	pub swapchain: vkraw::VkSwapchainKHR,
	pub device: &'a Device<'b>
}

fn debug_message_callback(flags: libc::c_int, otype: libc::c_int, srco: u64, loc: usize, msgcode: u32, layer: *const libc::c_char, msg: *const libc::c_char, _userdata: *mut libc::c_void) -> bool {

	let c_s = unsafe { std::ffi::CStr::from_ptr(msg) };
	let c_sl: &str = c_s.to_str().unwrap();

	let c_l = unsafe { std::ffi::CStr::from_ptr(layer) };
	let c_ll: &str = c_l.to_str().unwrap();

	let flags = vkraw::VkDebugReportFlagsEXT::from_bits_truncate(flags as u32);
	let obj_type: vkraw::VkDebugReportObjectTypeEXT = unsafe { std::mem::transmute(otype) };

	return rust_debug_message_callback(flags, obj_type, srco, loc, msgcode, c_ll.to_string(), c_sl.to_string());
}

pub fn rust_debug_message_callback(flags: vkraw::VkDebugReportFlagsEXT, obj_type: vkraw::VkDebugReportObjectTypeEXT, src_obj: u64, location: usize, msg_code: u32, layer: String, message: String) -> bool {

	println!("f:{:?}, ot:{:?}, o:{:?}, loc:{:?}, c:{:?}, l:{:?}:\n {}", flags, obj_type, src_obj, location, msg_code, layer, message);
	true
}

pub struct InstanceBuilder {
	pub layers: Vec<String>,
	pub extensions: Vec<String>,
	pub application_name: String,
	pub debug_message_callback: fn(vkraw::VkDebugReportFlagsEXT, vkraw::VkDebugReportObjectTypeEXT, u64, usize, u32, String, String) -> bool
}

impl Default for InstanceBuilder {
	fn default() -> Self {
		Self {
			layers: vec![ 
				#[cfg(debug_assertions)]
				"VK_LAYER_LUNARG_standard_validation".to_string(),
				//"VkLayer_core_validation".to_string(),
				//"VkLayer_object_lifetimes".to_string(),
				//"VkLayer_stateless_validation".to_string(),
				//"VkLayer_thread_safety".to_string(),
				//"VkLayer_unique_objects".to_string(),
			],
			extensions: vec![
				#[cfg(debug_assertions)]
				"VK_EXT_debug_report".to_string(),
				"VK_KHR_surface".to_string(),
				#[cfg(windows)]
				"VK_KHR_win32_surface".to_string(),
				#[cfg(unix)]
				"VK_KHR_xcb_surface".to_string(),
				//"VK_KHR_swapchain".to_string());
				//"VK_KHR_display".to_string());
				//"VK_KHR_display_swapchain".to_string());
			],
			application_name: "rust vulkan application".to_string(),
			debug_message_callback: rust_debug_message_callback
		}
	}
}

impl InstanceBuilder {
	pub fn new() -> InstanceBuilder {
		Self::default()
	}
	pub fn create_instance(&self) -> Result<Instance, vkraw::VkResult> {

		let res: vkraw::VkResult;
		let mut instance: vkraw::VkInstance = 0;

		// Create copy of each of the strings as a null terminated string for C
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(self.layers.len());
		for l in &self.layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
		}

		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(self.extensions.len());
		for e in &self.extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
		}

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(enabled_layers_rust.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(enabled_extensions_rust.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}
		let app_name = CString::new(self.application_name.clone()).unwrap();
		let engine_name = CString::new("engine name").unwrap();
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
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr()
		};

		println!("vkCreateInstance");
		unsafe {
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};

		let vk = vkraw::VulkanFunctionPointers::new(instance);

		if res == vkraw::VkResult::VK_SUCCESS {
			assert!(instance != vkraw::VK_NULL_HANDLE);

			let mut callback: vkraw::VkDebugReportCallbackEXT;
			unsafe {
				callback = std::mem::uninitialized();
				let rust_fptr = debug_message_callback;
				let c_fptr: vkraw::PFN_vkDebugReportCallbackEXT = rust_fptr as *const libc::c_void;
				let drcci = vkraw::VkDebugReportCallbackCreateInfoEXT {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
					pNext: std::ptr::null(),
					flags: vkraw::VkDebugReportFlagBitsEXT::all() & !vkraw::VkDebugReportFlagBitsEXT::VK_DEBUG_REPORT_INFORMATION_BIT_EXT,
					pfnCallback: c_fptr,
					pUserData: std::ptr::null_mut()
				};
				let res2 = vk.CreateDebugReportCallbackEXT.unwrap()(instance, &drcci, ptr::null(), &mut callback);
				assert!(res2 == vkraw::VkResult::VK_SUCCESS);
			};

			Ok(Instance { instance: instance, vk: vk, callback: callback })
		} else {
			Err(res)
		}
	}
}

impl Drop for Instance {
	fn drop(&mut self) {
		assert!(self.instance != vkraw::VK_NULL_HANDLE);
		unsafe {
			self.vk.DestroyDebugReportCallbackEXT.unwrap()(self.instance, self.callback, ptr::null());

			println!("vkDestroyInstance");
			vkraw::vkDestroyInstance(self.instance, ptr::null());
		}
	}
}

impl Instance {
	pub fn vk(&self) -> &vkraw::VulkanFunctionPointers {
		&self.vk
	}

	pub fn physical_devices(&self) -> Vec<PhysicalDevice> {

		let mut num_physical_devices = 0;
		let mut res: vkraw::VkResult;
		unsafe {
			res = vkraw::vkEnumeratePhysicalDevices(self.instance, &mut num_physical_devices, 0 as *mut u64);
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);

		let mut physical_devices = Vec::<PhysicalDevice>::with_capacity(num_physical_devices as usize);

		let mut vk_physical_devices = Vec::<vkraw::VkPhysicalDevice>::with_capacity(num_physical_devices as usize);
		unsafe {
			vk_physical_devices.set_len(num_physical_devices as usize);
			res = vkraw::vkEnumeratePhysicalDevices(self.instance, &mut num_physical_devices, vk_physical_devices.as_mut_ptr());
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);

		for d in vk_physical_devices {
			assert!(d != vkraw::VK_NULL_HANDLE);

			physical_devices.push(PhysicalDevice {
				physical_device: d,
				instance: &self
			});
		}
		return physical_devices;
	}

	#[cfg(unix)]
	pub fn create_wsi(&self, width: u32, height: u32) -> (Surface, xcb::Connection, u32) {

		assert!(width <= std::u16::MAX as u32);
		assert!(height <= std::u16::MAX as u32);

		let mut surface: vkraw::VkSurfaceKHR = 0;
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
				width as u16, height as u16,
				0,
				xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
				screen.root_visual(), &[
					(xcb::CW_BACK_PIXEL, screen.white_pixel()),
					(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS),
				]
			);
			xcb::map_window(&conn, win);
			conn.flush();

			let surface_create_info = vkraw::VkXcbSurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				connection: conn.get_raw_conn(),
				window: win
			};

			assert!(self.vk.CreateXcbSurfaceKHR.is_some());
			let res = self.vk.CreateXcbSurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		(Surface { surface: surface, instance: self }, conn, win)
	}

	#[cfg(windows)]
	pub fn create_wsi(&self, width: u32, height: u32) -> (Surface, winapi::shared::windef::HWND, winapi::shared::minwindef::HINSTANCE) {

		let hinstance;
		let handle;
		let mut surface: vkraw::VkSurfaceKHR = 0;
		unsafe {
			let name = win32_string("windoze");
			println!("Creating WIN32 window");
			hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null_mut());
			let wnd_class = winapi::um::winuser::WNDCLASSW {
				style : winapi::um::winuser::CS_HREDRAW | winapi::um::winuser::CS_VREDRAW,
				lpfnWndProc: Some(winapi::um::winuser::DefWindowProcW),
				hInstance: hinstance,
				lpszClassName: name.as_ptr(),
				cbClsExtra: 0,
				cbWndExtra: 0,
				hIcon: std::ptr::null_mut(),
				hCursor: std::ptr::null_mut(),
				hbrBackground: std::ptr::null_mut(),
				lpszMenuName: std::ptr::null_mut(),
			};
			winapi::um::winuser::RegisterClassW(&wnd_class);

			println!("Window {}x{}", width, height);

			let mut window_rect = winapi::shared::windef::RECT {
				left: 0,
				top: 0,
				right: width as i32,
				bottom: height as i32
			};

			let style = winapi::um::winuser::WS_OVERLAPPEDWINDOW | winapi::um::winuser::WS_CLIPSIBLINGS | winapi::um::winuser::WS_CLIPCHILDREN;
			let exstyle = winapi::um::winuser::WS_EX_APPWINDOW | winapi::um::winuser::WS_EX_WINDOWEDGE;
			winapi::um::winuser::AdjustWindowRectEx(&mut window_rect, style, 0, exstyle);

			handle = winapi::um::winuser::CreateWindowExW(
				0,
				name.as_ptr(),
				name.as_ptr(),
				style, 
				0,
				0,
				window_rect.right - window_rect.left,
				window_rect.bottom - window_rect.top,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				hinstance,
				std::ptr::null_mut());

			let surface_create_info = vkraw::VkWin32SurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				hinstance: hinstance as u64,
				hwnd: handle as u64
			};

			winapi::um::winuser::ShowWindow(handle, winapi::um::winuser::SW_SHOW);
			winapi::um::winuser::SetForegroundWindow(handle);
			winapi::um::winuser::SetFocus(handle);

			assert!(self.vk.CreateWin32SurfaceKHR.is_some());
			println!("vk.CreateWin32SurfaceKHR");
			let res = self.vk.CreateWin32SurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		(Surface { surface: surface, instance: self }, handle, hinstance)
	}
}

impl<'a> PhysicalDevice<'a> {
	pub fn queue_families(&self) -> Vec<vkraw::VkQueueFamilyProperties> {
		let mut num_physical_devices = 0;
		unsafe {
			vkraw::vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut num_physical_devices, ptr::null_mut());
		}
		let mut physical_devices = Vec::<vkraw::VkQueueFamilyProperties>::with_capacity(num_physical_devices as usize);
		unsafe {
			physical_devices.set_len(num_physical_devices as usize);
			vkraw::vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut num_physical_devices, physical_devices.as_mut_ptr());
		}
		return physical_devices;
	}

	pub fn physical_properties(&self) -> vkraw::VkPhysicalDeviceProperties {

		let mut props: vkraw::VkPhysicalDeviceProperties;
		unsafe {
			props = mem::uninitialized();
			vkraw::vkGetPhysicalDeviceProperties(self.physical_device, &mut props);
		}

		props
	}

	pub fn memory_properties(&self) -> (Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>) {
		let mut memory_properties: vkraw::VkPhysicalDeviceMemoryProperties;

		unsafe {
			memory_properties = mem::uninitialized();
			vkraw::vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut memory_properties);
		}

		let mut mt = Vec::<vkraw::VkMemoryType>::new();
		for i in 0..memory_properties.memoryTypeCount {
			mt.push(memory_properties.memoryTypes[i as usize]);
		}
		let mut mh = Vec::<vkraw::VkMemoryHeap>::new();
		for i in 0..memory_properties.memoryHeapCount {
			mh.push(memory_properties.memoryHeaps[i as usize]);
		}

		(mt, mh)
	}

	pub fn supported_surface_formats(&self, surface: &Surface) -> Result<Vec<vkraw::VkSurfaceFormatKHR>, vkraw::VkResult> {

		// Get a supported colour format and colour space
		let mut format_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
		self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface.surface, &mut format_count, ptr::null_mut());

		let mut surface_formats = Vec::<vkraw::VkSurfaceFormatKHR>::with_capacity(format_count as usize);
		unsafe {
			surface_formats.set_len(format_count as usize);
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface.surface, &mut format_count, surface_formats.as_mut_ptr());

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_formats)
		} else {
			Err(res)
		}
	}

	pub fn surface_capabilities(&self, surface: &Surface) -> Result<vkraw::VkSurfaceCapabilitiesKHR, vkraw::VkResult> {
		let mut surface_capabilities: vkraw::VkSurfaceCapabilitiesKHR;
		unsafe {
			surface_capabilities = std::mem::uninitialized();
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(self.physical_device, surface.surface, &mut surface_capabilities);
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_capabilities)
		} else {
			Err(res)
		}
	}

	pub fn present_modes(&self, surface: &Surface) -> Result<Vec<vkraw::VkPresentModeKHR>, vkraw::VkResult> {

		let mut present_mode_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
		let mut res = self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface.surface, &mut present_mode_count, ptr::null_mut());
		if res != vkraw::VkResult::VK_SUCCESS {
			return Err(res)
		}
		assert!(present_mode_count > 0);
		let mut present_modes = Vec::<vkraw::VkPresentModeKHR>::with_capacity(present_mode_count as usize);
		unsafe {
			present_modes.set_len(present_mode_count as usize);
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
		res = self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface.surface, &mut present_mode_count, present_modes.as_mut_ptr());
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(present_modes)
		} else {
			Err(res)
		}
	}
}

pub struct DeviceBuilder<'a> {
	instance: &'a Instance,
	pub layers: Vec<String>,
	pub extensions: Vec<String>,
	pub queue_create_infos: Vec<(u32, Vec<f32>)>,
	pub physical_device: Option<(PhysicalDevice<'a>, usize)> // Physical device and index
}

impl<'a> DeviceBuilder<'a> {
	pub fn new(instance: &Instance) -> DeviceBuilder {
		DeviceBuilder {
			instance: instance,
			layers: vec![ 
				#[cfg(debug_assertions)]
				"VK_LAYER_LUNARG_standard_validation".to_string(),
			],
			extensions: vec![
			],
			queue_create_infos: vec![(0, vec![1.0])],
			physical_device: None
		}
	}
	pub fn default_queues_physical_device<'y>(&'y mut self, surface: &Surface) -> &'y mut Self {

		let physical_devices = self.instance.physical_devices();
		assert!(physical_devices.len() > 0);
		let mut graphics_queue_family_index = Vec::<usize>::new();
		let mut compute_queue_family_index = Vec::<usize>::new();
		let mut transfer_queue_family_index = Vec::<usize>::new();

		// Loop through the physical devices
		let pd = physical_devices.iter().enumerate().filter_map(
			|(device_index, device)|
			{
				let qf = device.queue_families();

				let mut queue_supports_present = vkraw::VK_FALSE;

				// Loop through each of the family queues in the physical device
				graphics_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {

						// TODO: could want to present on the compute queue
						// Check if this queue supports presenting to the wsi surface
						assert!(self.instance.vk.GetPhysicalDeviceSurfaceSupportKHR.is_some());
						self.instance.vk.GetPhysicalDeviceSurfaceSupportKHR.unwrap()(device.physical_device, queue_family_index as u32, surface.surface, &mut queue_supports_present);

						// If we find a matching family, push the index on the Vec
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT) 
							&& queue_supports_present == vkraw::VK_TRUE {
							Some(queue_family_index)
						} else {
							None
						}
					}
				).collect();
				compute_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT) { Some(queue_family_index) } else { None }
					}
				).collect();
				transfer_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT) { Some(queue_family_index) } else { None }
					}
				).collect();

				// Return the first physical device that has at least one of each queue family
				if graphics_queue_family_index.len() > 0 && compute_queue_family_index.len() > 0 && transfer_queue_family_index.len() > 0 {
					Some((PhysicalDevice { physical_device: device.physical_device, instance: &self.instance }, device_index))
				} else {
					None
				}
			}).next().expect("Couldn't find a supported graphics queue");

		self.physical_device = Some(pd);

		assert!(graphics_queue_family_index.len() > 0);
		assert!(compute_queue_family_index.len() > 0);
		assert!(transfer_queue_family_index.len() > 0);

		// Use different queue families for better performance (first/last)
		self.queue_create_infos = vec![(graphics_queue_family_index.first().unwrap().clone() as u32, vec![1.0]),
			(compute_queue_family_index.last().unwrap().clone() as u32, vec![1.0]),
			(transfer_queue_family_index.last().unwrap().clone() as u32, vec![1.0])];

		self
	}
	pub fn create_device(&self) -> Result<Device<'a>, vkraw::VkResult> {

		let mut device: vkraw::VkDevice = 0;

		// Create copy of each of the strings as a null terminated string
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(self.layers.len());
		for l in &self.layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
		}
		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(self.extensions.len());
		for e in &self.extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
		}

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(enabled_layers_rust.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(enabled_extensions_rust.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}
		let mut queue_priorities = Vec::<Vec<f32>>::new();
		let mut queue_create_infos = Vec::<vkraw::VkDeviceQueueCreateInfo>::new();

		for i in &self.queue_create_infos {
			queue_priorities.push(i.1.clone());
			queue_create_infos.push(vkraw::VkDeviceQueueCreateInfo {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
				pNext: ptr::null(),
				flags: vkraw::VkDeviceQueueCreateFlags::_EMPTY,
				queueFamilyIndex: i.0,
				queueCount: i.1.len() as u32,
				pQueuePriorities: queue_priorities.last().unwrap().as_ptr() as *const f32
			});
		}
		let device_create_info = vkraw::VkDeviceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			queueCreateInfoCount: queue_create_infos.len() as u32,
			pQueueCreateInfos: queue_create_infos.as_ptr(),
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr(),
			pEnabledFeatures: ptr::null()
		};

		println!("vkCreateDevice");
		let res;
		unsafe {
			res = vkraw::vkCreateDevice(self.physical_device.as_ref().expect("No physical device").0.physical_device, &device_create_info, ptr::null(), &mut device);
		};

		if res == vkraw::VkResult::VK_SUCCESS {

			assert!(device != vkraw::VK_NULL_HANDLE);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			Ok(Device { device: device, instance: self.instance })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Device<'a> {
	fn drop(&mut self) {
		assert!(self.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyDevice");
			vkraw::vkDestroyDevice(self.device, ptr::null());
		}
	}
}

impl<'a> Device<'a> {
	pub fn get_queue(&self, queue_family_index: u32, queue_index: u32) -> vkraw::VkQueue {
		let mut queue;
		unsafe {
			queue = std::mem::uninitialized();
			println!("vkGetDeviceQueue");
			vkraw::vkGetDeviceQueue(self.device, queue_family_index, queue_index, &mut queue);
		}
		assert!(queue != vkraw::VK_NULL_HANDLE);
		queue
	}
	pub fn create_buffer(&self, size: usize, flags: vkraw::VkBufferUsageFlags) -> Result<Buffer, vkraw::VkResult> {
		let buf_create_info = vkraw::VkBufferCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkBufferCreateFlags::_EMPTY,
			size: size as u64,
			usage: flags,
			sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
			queueFamilyIndexCount: 0,
			pQueueFamilyIndices: ptr::null()
		};

		let mut buffer: vkraw::VkBuffer = 0;
		let res;
		println!("vkCreateBuffer");
		unsafe {
			res = vkraw::vkCreateBuffer(self.device, &buf_create_info, ptr::null(), &mut buffer);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Buffer { buffer: buffer, device: &self })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Surface<'a> {
	fn drop(&mut self) {
		assert!(self.instance.instance != vkraw::VK_NULL_HANDLE);
		println!("vk.DestroySurfaceKHR");
		assert!(self.instance.vk.DestroySurfaceKHR.is_some());
		self.instance.vk.DestroySurfaceKHR.unwrap()(self.instance.instance, self.surface, ptr::null());
	}
}

impl<'a> Drop for Buffer<'a> {
	fn drop(&mut self) {
		println!("vkDestroyBuffer");
		unsafe {
			vkraw::vkDestroyBuffer(self.device.device, self.buffer, ptr::null());
		}
	}
}

pub struct SwapchainBuilder<'a> {
	device: &'a Device<'a>
}

impl<'a> SwapchainBuilder<'a> {
	pub fn new(device: &'a Device) -> SwapchainBuilder<'a> {
		SwapchainBuilder {
			device: &device
		}
	}
	pub fn create(&self) -> Result<Swapchain, vkraw::VkResult> {
		Ok(Swapchain { device: &self.device, swapchain: 0 })
	}
}

pub struct MemoryAllocator<'a> {
	device: &'a Device<'a>
}

pub fn staging_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a staging memory type")
}

pub fn gpu_only_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a gpu only memory type")
}

pub fn gpu_to_cpu_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_CACHED_BIT | vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a gpu to cpu memory type")
}

impl<'a> MemoryAllocator<'a> {
	pub fn new(device: &'a Device) -> MemoryAllocator<'a> {
		MemoryAllocator {
			device: device
		}
	}
	pub fn allocate_buffer_memory(&self, buffer: &Buffer, memory_type_index: usize) -> Result<Mem, vkraw::VkResult> {

		let mut mem_reqs: vkraw::VkMemoryRequirements;
		unsafe {
			mem_reqs = std::mem::uninitialized();
			vkraw::vkGetBufferMemoryRequirements(self.device.device, buffer.buffer, &mut mem_reqs);
		}
		let mem_alloc = vkraw::VkMemoryAllocateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
			pNext: ptr::null(),
			allocationSize: mem_reqs.size,
			memoryTypeIndex: memory_type_index as u32
		};
		let mut memory: vkraw::VkDeviceMemory = 0;
		let mut res;
		unsafe {
			res = vkraw::vkAllocateMemory(self.device.device, &mem_alloc, ptr::null(), &mut memory);
			assert!(res == vkraw::VkResult::VK_SUCCESS);

			if res != vkraw::VkResult::VK_SUCCESS {
				return Err(res)
			}

			// TODO: do this here?
			res = vkraw::vkBindBufferMemory(self.device.device, buffer.buffer, memory, 0);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Mem { memory_allocator: self, mem: memory, ptr: 0 })
		} else {
			Err(res)
		}
	}
}

// The lifetime specifiers ('a) here say "Mem should live a shorter lifetime than Instance because we have a reference to it"
pub struct Mem<'a> {
	memory_allocator: &'a MemoryAllocator<'a>,
	mem: vkraw::VkDeviceMemory,
	ptr: u64
}
impl<'a> Mem<'a> {
	// &mut self means that we can only ever have one MappedMem instance (Cannot map() twice)
	pub fn map<T>(&mut self) -> MappedMem<T> {

		let mut data: *mut libc::c_void = ptr::null_mut();
		let res;
		unsafe {
			res = vkraw::vkMapMemory(self.memory_allocator.device.device, self.mem, 0, std::mem::size_of::<T>() as u64, 0, &mut data);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			assert!(data != ptr::null_mut());
		}
		self.ptr = data as u64;
		println!("Mem::map()");
		MappedMem { mem: self, ptr: &self.ptr, _phantom: std::marker::PhantomData }
	}
}
impl<'a> Drop for Mem<'a> {
	fn drop(&mut self) {
		println!("Mem::drop()");
		unsafe {
			vkraw::vkFreeMemory(self.memory_allocator.device.device, self.mem, ptr::null());
		}
	}
}

// MappedMem should live a shorter lifetime than Mem
pub struct MappedMem<'a, T> {
	mem: &'a Mem<'a>,
	ptr: &'a u64,
	_phantom: std::marker::PhantomData<T>
}
impl<'a, T> std::ops::Deref for MappedMem<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe { std::mem::transmute::<u64, &T>(*self.ptr) }
	}
}
impl<'a, T> std::ops::DerefMut for MappedMem<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { std::mem::transmute::<u64, &mut T>(*self.ptr) }
	}
}
impl<'a, T> MappedMem<'a, T> {
	pub fn get_ptr(&mut self) -> *mut T {
		println!("MappedMem::get_ptr()");
		unsafe { std::mem::transmute::<u64, *mut T>(*self.ptr) }
	}
}
impl<'a, T> Drop for MappedMem<'a, T> {
	fn drop(&mut self) {
		println!("MappedMem::drop()");

		unsafe {
			vkraw::vkUnmapMemory(self.mem.memory_allocator.device.device, self.mem.mem);
		}
	}
}


/*
// The lifetime specifiers ('a) here say "MemSafe should live a shorter lifetime than Instance because we have a reference to it"
pub struct MemSafe<'a, T> {
	memory_allocator: &'a MemoryAllocator<'a>,
	mem: std::pin::Pin<T>
}
impl<'a, T> MemSafe<'a, T> {

	pub fn map(&mut self) -> std::pin::Pin<T> {
		println!("MemSafe::map()");
		self.mem
	}
}
impl<'a, T> Drop for MemSafe<'a, T> {
	fn drop(&mut self) {
		println!("MemSafe::drop()");
	}
}
*/



/*

impl Instance2 {
	pub fn new(application_name: &str, layers: Vec<String>, extensions: Vec<String>) -> Result<Instance2, vkraw::VkResult> {
		let res: vkraw::VkResult;
		let mut instance: vkraw::VkInstance = 0;

		// Create copy of each of the strings as a null terminated string
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(layers.len());
		for l in &layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
		}

		#[cfg(debug_assertions)]
		enabled_layers_rust.push(CString::new("VK_LAYER_LUNARG_standard_validation").unwrap());
		
		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(extensions.len());
		for e in &extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
		}
		
		#[cfg(debug_assertions)]
		enabled_extensions_rust.push(CString::new("VK_EXT_debug_report").unwrap());
		enabled_extensions_rust.push(CString::new("VK_KHR_surface").unwrap());
		//enabled_extensions_rust.push(CString::new("VK_KHR_swapchain").unwrap());
		//enabled_extensions_rust.push(CString::new("VK_KHR_display").unwrap());
		//enabled_extensions_rust.push(CString::new("VK_KHR_display_swapchain").unwrap());
		#[cfg(windows)]
		enabled_extensions_rust.push(CString::new("VK_KHR_win32_surface").unwrap());
		#[cfg(unix)]
		enabled_extensions_rust.push(CString::new("VK_KHR_xcb_surface").unwrap());
		
		println!("extensions {:?}", enabled_extensions_rust);

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(layers.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(extensions.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}

		let app_name = CString::new(application_name.clone()).unwrap();
		let engine_name = CString::new("engine name").unwrap();
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
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr()
		};

		println!("Creating instance");
		unsafe {
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};

		if res == vkraw::VkResult::VK_SUCCESS {
			assert!(instance != vkraw::VK_NULL_HANDLE);

			let vk = vkraw::VulkanFunctionPointers::new(instance);

			let mut num_physical_devices = 0;
			let mut res: vkraw::VkResult;
			unsafe {
				res = vkraw::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, 0 as *mut u64);
			}
			assert!(res == vkraw::VkResult::VK_SUCCESS);

			let mut physical_devices = Vec::<PhysicalDevice>::with_capacity(num_physical_devices as usize);

			let mut vk_physical_devices = Vec::<vkraw::VkPhysicalDevice>::with_capacity(num_physical_devices as usize);
			unsafe {
				vk_physical_devices.set_len(num_physical_devices as usize);
				res = vkraw::vkEnumeratePhysicalDevices(instance, &mut num_physical_devices, vk_physical_devices.as_mut_ptr());
			}
			assert!(res == vkraw::VkResult::VK_SUCCESS);

			for d in vk_physical_devices {
				assert!(d != vkraw::VK_NULL_HANDLE);

				let mut memory_properties: vkraw::VkPhysicalDeviceMemoryProperties;

				unsafe {
					memory_properties = mem::uninitialized();
					vkraw::vkGetPhysicalDeviceMemoryProperties(d, &mut memory_properties);
				}

				let mut queue_count = 0;
				unsafe {
					vkraw::vkGetPhysicalDeviceQueueFamilyProperties(d, &mut queue_count, ptr::null_mut());
				}
				assert!(queue_count > 0);

				let mut queue_props = Vec::<vkraw::VkQueueFamilyProperties>::with_capacity(queue_count as usize);
				unsafe {
					vkraw::vkGetPhysicalDeviceQueueFamilyProperties(d, &mut queue_count, queue_props.as_mut_ptr());
					queue_props.set_len(queue_count as usize);
				}

				let mut props: vkraw::VkPhysicalDeviceProperties;
				let device_name;
				unsafe {
					props = mem::uninitialized();
					vkraw::vkGetPhysicalDeviceProperties(d, &mut props);
					device_name = std::ffi::CStr::from_ptr(props.deviceName.as_ptr() as *const i8).to_str().unwrap().to_string();
				}

				let properties = PhysicalDeviceProperties {
					api_version: props.apiVersion,
					driver_version: props.driverVersion,
					vendor_id: props.vendorID,
					device_id: props.deviceID,
					device_type: props.deviceType,
					device_name: device_name,
					pipeline_cache_uuid: props.pipelineCacheUUID,
					limits: props.limits,
					sparse_properties: props.sparseProperties
				};

				let mut num_displays = 0;
				let mut display_properties = None;
				if vk.GetPhysicalDeviceDisplayPropertiesKHR.is_some() {
					assert!(vk.GetPhysicalDeviceDisplayPropertiesKHR.is_some());
					vk.GetPhysicalDeviceDisplayPropertiesKHR.unwrap()(d, &mut num_displays, ptr::null_mut());

					if num_displays > 0 {

						display_properties = Some(Vec::<vkraw::VkDisplayPropertiesKHR>::with_capacity(num_displays as usize));
						unsafe {
							display_properties.as_mut().unwrap().set_len(num_displays as usize);
						}
						assert!(vk.GetPhysicalDeviceDisplayPropertiesKHR.is_some());
						vk.GetPhysicalDeviceDisplayPropertiesKHR.unwrap()(d, &mut num_displays, display_properties.as_mut().unwrap().as_mut_ptr());
					}
				}

				physical_devices.push(PhysicalDevice {
					physical_device: d,
					memory_properties: memory_properties,
					queue_family_properties: queue_props,
					properties: properties,
					display_properties: display_properties
				});
			}
			Ok(Instance2 { instance: instance, vk: vk, physical_devices: physical_devices })
		} else {
			Err(res)
		}
	}

	pub fn physical_devices(&self) -> &Vec<PhysicalDevice> {
		&self.physical_devices
	}

	#[cfg(unix)]
	pub fn create_wsi(&self, width: u32, height: u32) -> (xcb::Connection, u32, Surface) {

		assert!(width <= std::u16::MAX as u32);
		assert!(height <= std::u16::MAX as u32);

		let mut surface: vkraw::VkSurfaceKHR = 0;
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
				width as u16, height as u16,
				0,
				xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
				screen.root_visual(), &[
					(xcb::CW_BACK_PIXEL, screen.white_pixel()),
					(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS),
				]
			);
			xcb::map_window(&conn, win);
			conn.flush();

			let surface_create_info = vkraw::VkXcbSurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				connection: conn.get_raw_conn(),
				window: win
			};

			assert!(self.vk.CreateXcbSurfaceKHR.is_some());
			let res = self.vk.CreateXcbSurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		(conn, win, Surface { surface: surface, instance: self })
	}

	#[cfg(windows)]
	pub fn create_wsi(&self, width: u32, height: u32) -> (winapi::shared::windef::HWND, winapi::shared::minwindef::HINSTANCE, Surface) {

		let hinstance;
		let handle;
		let mut surface: vkraw::VkSurfaceKHR = 0;
		unsafe {
			let name = win32_string("windoze");
			println!("Creating WIN32 window");
			hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null_mut());
			let wnd_class = winapi::um::winuser::WNDCLASSW {
				style : winapi::um::winuser::CS_HREDRAW | winapi::um::winuser::CS_VREDRAW,
				lpfnWndProc: Some(winapi::um::winuser::DefWindowProcW),
				hInstance: hinstance,
				lpszClassName: name.as_ptr(),
				cbClsExtra: 0,
				cbWndExtra: 0,
				hIcon: std::ptr::null_mut(),
				hCursor: std::ptr::null_mut(),
				hbrBackground: std::ptr::null_mut(),
				lpszMenuName: std::ptr::null_mut(),
			};
			winapi::um::winuser::RegisterClassW(&wnd_class);
			
			println!("Window {}x{}", width, height);
			
			let mut window_rect = winapi::shared::windef::RECT {
				left: 0,
				top: 0,
				right: width as i32,
				bottom: height as i32
			};

			let style = winapi::um::winuser::WS_OVERLAPPEDWINDOW | winapi::um::winuser::WS_CLIPSIBLINGS | winapi::um::winuser::WS_CLIPCHILDREN;
			let exstyle = winapi::um::winuser::WS_EX_APPWINDOW | winapi::um::winuser::WS_EX_WINDOWEDGE;
			winapi::um::winuser::AdjustWindowRectEx(&mut window_rect, style, 0, exstyle);
			
			handle = winapi::um::winuser::CreateWindowExW(
				0,
				name.as_ptr(),
				name.as_ptr(),
				style, 
				0,
				0,
				window_rect.right - window_rect.left,
				window_rect.bottom - window_rect.top,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				hinstance,
				std::ptr::null_mut());
		
			let surface_create_info = vkraw::VkWin32SurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				hinstance: hinstance as u64,
				hwnd: handle as u64
			};
			
			winapi::um::winuser::ShowWindow(handle, winapi::um::winuser::SW_SHOW);
			winapi::um::winuser::SetForegroundWindow(handle);
			winapi::um::winuser::SetFocus(handle);

			assert!(self.vk.CreateWin32SurfaceKHR.is_some());
			let res = self.vk.CreateWin32SurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		(handle, hinstance, Surface { surface: surface, instance: self })
	}

	pub fn queue_family_supports_surface(&self, physical_device: &PhysicalDevice, queue_family_index: u32, surface: &Surface) -> bool {

		let mut supported: u32;
		assert!(self.vk.GetPhysicalDeviceSurfaceSupportKHR.is_some());
		unsafe {
			supported = mem::uninitialized();
		}
		let result = self.vk.GetPhysicalDeviceSurfaceSupportKHR.unwrap()(physical_device.physical_device, queue_family_index, surface.surface, &mut supported);
		assert!(result == vkraw::VkResult::VK_SUCCESS);

		if supported > 0 { true } else { false }
	}
}

impl PhysicalDevice {
	pub fn create_device(&self, layers: Vec<String>, extensions: Vec<String>, queue_family_index: u32, queue_priorities: Vec<f32>) -> Result<Device, vkraw::VkResult> {
		let mut device: vkraw::VkDevice = 0;

		// Create copy of each of the strings as a null terminated string
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(layers.len());
		for l in &layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
		}
		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(extensions.len());
		for e in &extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
		}

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(layers.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(extensions.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}

		let queue_create_info = vkraw::VkDeviceQueueCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkDeviceQueueCreateFlags::_EMPTY,
			queueFamilyIndex: queue_family_index,
			queueCount: queue_priorities.len() as u32,
			pQueuePriorities: queue_priorities.as_ptr() as *const f32
		};
		let device_create_info = vkraw::VkDeviceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			queueCreateInfoCount: 1,
			pQueueCreateInfos: &queue_create_info,
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr(),
			pEnabledFeatures: ptr::null()
		};

		let res;
		unsafe {
			res = vkraw::vkCreateDevice(self.physical_device, &device_create_info, ptr::null(), &mut device);
		};

		if res == vkraw::VkResult::VK_SUCCESS {

			assert!(device != vkraw::VK_NULL_HANDLE);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			Ok(Device { device: device })
		} else {
			Err(res)
		}
	}
}

impl Drop for Instance2 {
	fn drop(&mut self) {
		assert!(self.instance != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyInstance");
			vkraw::vkDestroyInstance(self.instance, ptr::null());
		}
	}
}
*/
