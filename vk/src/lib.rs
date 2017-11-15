
extern crate vkraw;

use std::ptr;
use std::ffi::CString;
use std::mem;

pub struct Instance {
	instance: vkraw::VkInstance,
	vk: vkraw::VulkanFunctionPointers,
	physical_devices: Vec<PhysicalDevice>
}

pub struct PhysicalDevice {
	physical_device: vkraw::VkPhysicalDevice,
	memory_properties: vkraw::VkPhysicalDeviceMemoryProperties,
	queue_family_properties: Vec<vkraw::VkQueueFamilyProperties>
}

pub struct Queue {
	queue: vkraw::VkQueue
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
			ppEnabledLayerNames: enabled_layers.as_ptr() as *const u8,
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr() as *const u8
		};

		println!("Creating instance");
		unsafe {
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};

		if res == vkraw::VkResult::VK_SUCCESS {
			assert!(instance != vkraw::VK_NULL_HANDLE);


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

				physical_devices.push(PhysicalDevice { physical_device: d, memory_properties: memory_properties, queue_family_properties: queue_props });
			}

			Ok(Instance { instance: instance, vk: vkraw::VulkanFunctionPointers::new(instance), physical_devices: physical_devices })
		} else {
			Err(res)
		}
	}

	pub fn physical_devices(&self) -> &Vec<PhysicalDevice> {
		&self.physical_devices
	}
}

impl Drop for Instance {
	fn drop(&mut self) {
		assert!(self.instance != vkraw::VK_NULL_HANDLE);
		unsafe {
			vkraw::vkDestroyInstance(self.instance, ptr::null());
		}
	}
}
/*
impl PhysicalDevice {
	pub fn memory_properties(&self) -> vkraw::VkPhysicalDeviceMemoryProperties {

		let mut return_value: vkraw::VkPhysicalDeviceMemoryProperties;

		unsafe {
			return_value = mem::uninitialized();
			vkraw::vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut return_value);
		}

		return_value
	}
}*/
