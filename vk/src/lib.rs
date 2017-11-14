
extern crate vkraw;

pub mod vk {

	use std::ptr;
	use std::ffi::CString;
	use vkraw::*;

	pub struct Instance {
		instance: vkraw::VkInstance,
		vk: vkraw::VulkanFunctionPointers
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
				Ok(Instance { instance: instance, vk: vkraw::VulkanFunctionPointers::new(instance) })
			} else {
				Err(res)
			}
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
}
