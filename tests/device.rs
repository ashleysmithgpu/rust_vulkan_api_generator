
extern crate vkrust;

extern crate libc;

extern crate bitflags;

#[cfg(test)]
mod tests {

	use std::ptr;
	use vkrust::vkrust;

	use std::vec;
/*
enum VulkanParameter {
	ApplicationInfo { next: Option<VulkanParameter>, application_name: str, application_version: u32, engine_name: str, engine_version: u32, api_version: u32 },
	InstanceCreateInfo { next: Option<VulkanParameter>, flags: u32, application_info: VulkanParameter, enabled_layers: [str], enabled_extensions: [str] }
}


	fn create_instance(create_info: VulkanParameter) -> Result<vkrust::VkInstance, vkrust::VkResult> {

		unsafe {
			
		}
	}
*/

struct InstanceCreateInfo {
	pub flags: i32,
	pub application_info: ApplicationInfo,
	pub enabled_layers: vec::Vec<String>,
	pub enabled_extensions: vec::Vec<String>,
}
struct ApplicationInfo {
	pub application_name: String,
	pub application_version: u32,
	pub engine_name: String,
	pub engine_version: u32,
	pub api_version: u32,
}

fn create_instance(create_info: InstanceCreateInfo) -> Result<vkrust::VkInstance, vkrust::VkResult> {

	let mut instance: vkrust::VkInstance = 0;
	let app_name: *const u8 = create_info.application_info.application_name.as_ptr();
	let engine_name: *const u8 = create_info.application_info.engine_name.as_ptr();

	let application_info = vkrust::VkApplicationInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
		pNext: ptr::null(),
		pApplicationName: app_name,
		applicationVersion: create_info.application_info.application_version,
		pEngineName: engine_name,
		engineVersion: create_info.application_info.engine_version,
		apiVersion: create_info.application_info.api_version,
	};
	let create_info = vkrust::VkInstanceCreateInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
		pNext: ptr::null(),
		flags: create_info.flags,
		pApplicationInfo: &application_info,
		enabledLayerCount: create_info.enabled_layers.len() as u32,
		ppEnabledLayerNames: create_info.enabled_layers.as_ptr() as *const u8,
		enabledExtensionCount: create_info.enabled_extensions.len() as u32,
		ppEnabledExtensionNames: create_info.enabled_extensions.as_ptr() as *const u8,
	};
	let res: vkrust::VkResult;
	unsafe {
		res = vkrust::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
	match res {
		vkrust::VkResult::VK_SUCCESS => Ok(instance),
		_ => Err(res)
	}
}

fn destroy_instance(instance: vkrust::VkInstance) {

	unsafe {
		vkrust::vkDestroyInstance(instance, ptr::null());
	}
}

#[test]
	fn test_device_bad_extensions() {

		// Bad extensions
		let ici = InstanceCreateInfo {
			flags: 0,
			application_info: ApplicationInfo {
				application_name: "test".to_string(),
				application_version: 1,
				engine_name: "test_engine".to_string(),
				engine_version: 0,
				api_version: 0,
			},
			enabled_layers: vec::Vec::new(),
			enabled_extensions: vec!["non_existant_extension".to_string()],
		};
		let res = create_instance(ici);

		assert!(res.is_err());

		match res {
			Err(e) => assert!(e == vkrust::VkResult::VK_ERROR_EXTENSION_NOT_PRESENT),
			Ok(_) => {}
		}
	}

#[test]
	fn test_device_bad_layers() {

		// Bad layers
		let ici = InstanceCreateInfo {
			flags: 0,
			application_info: ApplicationInfo {
				application_name: "test".to_string(),
				application_version: 1,
				engine_name: "test_engine".to_string(),
				engine_version: 0,
				api_version: 0,
			},
			enabled_layers: vec!["non_existant_layer".to_string()],
			enabled_extensions: vec::Vec::new()
		};
		let res = create_instance(ici);

		assert!(res.is_err());

		match res {
			Err(e) => assert!(e == vkrust::VkResult::VK_ERROR_LAYER_NOT_PRESENT),
			Ok(_) => {}
		}
	}

#[test]
	fn test_device_bad_version() {
		// Bad api version
		let ici = InstanceCreateInfo {
			flags: 0,
			application_info: ApplicationInfo {
				application_name: "test".to_string(),
				application_version: 1,
				engine_name: "test_engine".to_string(),
				engine_version: 0,
				api_version: 93499348,
			},
			enabled_layers: vec::Vec::new(),
			enabled_extensions: vec::Vec::new()
		};
		let res = create_instance(ici);

		assert!(res.is_err());

		match res {
			Err(e) => assert!(e == vkrust::VkResult::VK_ERROR_INCOMPATIBLE_DRIVER),
			Ok(_) => {}
		}
	}

#[test]
	fn test_ok_create_destroy() {
		// Ok create destroy
		let ici = InstanceCreateInfo {
			flags: 0,
			application_info: ApplicationInfo {
				application_name: "test".to_string(),
				application_version: 1,
				engine_name: "test_engine".to_string(),
				engine_version: 0,
				api_version: 0,
			},
			enabled_layers: vec::Vec::new(),
			enabled_extensions: vec::Vec::new()
		};
		let res = create_instance(ici);

		assert!(res.is_ok());

		destroy_instance(res.unwrap());
	}
}
