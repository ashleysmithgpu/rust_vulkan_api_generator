
extern crate vkraw;

#[cfg(feature="xcb")]
extern crate xcb;

use std::ptr;
use std::ffi::CString;
use std::mem;

pub struct Vk {
	pub instance_extensions_available: Vec<ExtensionProperties>,
	pub instance_layers_available: Vec<LayerProperties>,
}

pub struct ExtensionProperties {
	pub extension_name: String,
	pub spec_version: u32
}

pub struct LayerProperties {
	pub layer_name: String,
	pub spec_version: u32,
	pub implementation_version: u32,
	pub description: String
}

pub struct Instance {
	pub instance: vkraw::VkInstance,
	pub vk: vkraw::VulkanFunctionPointers,
	pub physical_devices: Vec<PhysicalDevice>
}

pub struct Device {
	pub device: vkraw::VkDevice
}

pub struct Surface<'a> {
	pub surface: vkraw::VkSurfaceKHR,
	instance: &'a Instance
}

pub struct PhysicalDeviceProperties {
	pub api_version: u32,
	pub driver_version: u32,
	pub vendor_id: u32,
	pub device_id: u32,
	pub device_type: vkraw::VkPhysicalDeviceType,
	pub device_name: String,
	pub pipeline_cache_uuid: [u8; vkraw::VK_UUID_SIZE],
	pub limits: vkraw::VkPhysicalDeviceLimits,
	pub sparse_properties: vkraw::VkPhysicalDeviceSparseProperties
}

pub struct PhysicalDevice {
	pub physical_device: vkraw::VkPhysicalDevice,
	pub memory_properties: vkraw::VkPhysicalDeviceMemoryProperties,
	pub queue_family_properties: Vec<vkraw::VkQueueFamilyProperties>,
	pub properties: PhysicalDeviceProperties,
	pub display_properties: Option<Vec<vkraw::VkDisplayPropertiesKHR>>
}

impl Vk {
	pub fn new() -> Vk {

		let mut result;

		let mut num_layers = 0;
		unsafe {
			num_layers = mem::uninitialized();
			result = vkraw::vkEnumerateInstanceLayerProperties(&mut num_layers, ptr::null_mut());
		}
		assert!(result == vkraw::VkResult::VK_SUCCESS);
		let mut instance_layers = Vec::<vkraw::VkLayerProperties>::with_capacity(num_layers as usize);
		unsafe {
			instance_layers.set_len(num_layers as usize);
			result = vkraw::vkEnumerateInstanceLayerProperties(&mut num_layers, instance_layers.as_mut_ptr());
		}
		assert!(result == vkraw::VkResult::VK_SUCCESS);

		let mut num_extensions = 0;
		unsafe {
			num_extensions = mem::uninitialized();
			result = vkraw::vkEnumerateInstanceExtensionProperties(ptr::null(), &mut num_extensions, ptr::null_mut());
		}
		assert!(result == vkraw::VkResult::VK_SUCCESS);
		let mut instance_extensions = Vec::<vkraw::VkExtensionProperties>::with_capacity(num_extensions as usize);
		unsafe {
			instance_extensions.set_len(num_extensions as usize);
			result = vkraw::vkEnumerateInstanceExtensionProperties(ptr::null(), &mut num_extensions, instance_extensions.as_mut_ptr());
		}
		assert!(result == vkraw::VkResult::VK_SUCCESS);

		let mut instance_layers_available = Vec::<LayerProperties>::with_capacity(num_layers as usize);
		for l in instance_layers {

			unsafe {
				instance_layers_available.push(LayerProperties {
					layer_name: std::ffi::CStr::from_ptr(l.layerName.as_ptr() as *const i8).to_str().unwrap().to_string(),
					spec_version: l.specVersion,
					implementation_version: l.implementationVersion,
					description: std::ffi::CStr::from_ptr(l.description.as_ptr() as *const i8).to_str().unwrap().to_string()
				});
			}
		}

		let mut instance_extensions_available = Vec::<ExtensionProperties>::with_capacity(num_extensions as usize);
		for e in instance_extensions {

			unsafe {
				instance_extensions_available.push(ExtensionProperties {
					extension_name: std::ffi::CStr::from_ptr(e.extensionName.as_ptr() as *const i8).to_str().unwrap().to_string(),
					spec_version: e.specVersion
				});
			}
		}

		Vk { instance_layers_available: instance_layers_available, instance_extensions_available: instance_extensions_available }
	}
}

impl Instance {
	pub fn new(application_name: &str, layers: Vec<String>, extensions: Vec<String>) -> Result<Instance, vkraw::VkResult> {
		let res: vkraw::VkResult;
		let mut instance: vkraw::VkInstance = 0;

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
			Ok(Instance { instance: instance, vk: vk, physical_devices: physical_devices })
		} else {
			Err(res)
		}
	}

	pub fn physical_devices(&self) -> &Vec<PhysicalDevice> {
		&self.physical_devices
	}

#[cfg(feature="xcb")]
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
			flags: 0,
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

impl Drop for Instance {
	fn drop(&mut self) {
		assert!(self.instance != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyInstance");
			vkraw::vkDestroyInstance(self.instance, ptr::null());
		}
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		assert!(self.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyDevice");
			vkraw::vkDestroyDevice(self.device, ptr::null());
		}
	}
}

impl<'a> Drop for Surface<'a> {
	fn drop(&mut self) {
		assert!(self.instance.instance != vkraw::VK_NULL_HANDLE);
		println!("vkDestroySurfaceKHR");
		assert!(self.instance.vk.DestroySurfaceKHR.is_some());
		self.instance.vk.DestroySurfaceKHR.unwrap()(self.instance.instance, self.surface, ptr::null());
	}
}
