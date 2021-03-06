
#[cfg(test)]
mod tests {

	use std::ptr;
	use std::vec;
	use std;
/*
enum VulkanParameter {
	ApplicationInfo { next: Option<VulkanParameter>, application_name: str, application_version: u32, engine_name: str, engine_version: u32, api_version: u32 },
	InstanceCreateInfo { next: Option<VulkanParameter>, flags: u32, application_info: VulkanParameter, enabled_layers: [str], enabled_extensions: [str] }
}


	fn create_instance(create_info: VulkanParameter) -> Result<vkraw::VkInstance, vkraw::VkResult> {

		unsafe {
			
		}
	}
*/

struct InstanceCreateInfo {
	pub flags: u32,
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

fn create_instance(create_info: InstanceCreateInfo) -> Result<vkraw::VkInstance, vkraw::VkResult> {

	let mut instance: vkraw::VkInstance = 0;
	let app_name: *const u8 = create_info.application_info.application_name.as_ptr();
	let engine_name: *const u8 = create_info.application_info.engine_name.as_ptr();

	let application_info = vkraw::VkApplicationInfo {
		sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
		pNext: ptr::null(),
		pApplicationName: app_name,
		applicationVersion: create_info.application_info.application_version,
		pEngineName: engine_name,
		engineVersion: create_info.application_info.engine_version,
		apiVersion: create_info.application_info.api_version,
	};
	let create_info = vkraw::VkInstanceCreateInfo {
		sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
		pNext: ptr::null(),
		flags: create_info.flags,
		pApplicationInfo: &application_info,
		enabledLayerCount: create_info.enabled_layers.len() as u32,
		ppEnabledLayerNames: create_info.enabled_layers.as_ptr() as *const*const u8,
		enabledExtensionCount: create_info.enabled_extensions.len() as u32,
		ppEnabledExtensionNames: create_info.enabled_extensions.as_ptr() as *const*const u8,
	};
	let res: vkraw::VkResult;
	unsafe {
		res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
	match res {
		vkraw::VkResult::VK_SUCCESS => Ok(instance),
		_ => Err(res)
	}
}

fn destroy_instance(instance: vkraw::VkInstance) {

	unsafe {
		vkraw::vkDestroyInstance(instance, ptr::null());
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
			Err(e) => assert!(e == vkraw::VkResult::VK_ERROR_EXTENSION_NOT_PRESENT),
			Ok(_) => {}
		}
		std::thread::sleep(std::time::Duration::from_millis(1000));
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
			Err(e) => assert!(e == vkraw::VkResult::VK_ERROR_LAYER_NOT_PRESENT),
			Ok(_) => {}
		}
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}

// TODO: no way to use i.e. should_fail since we segfault here
/*
#[test]
#[should_fail]
	fn test_device_bad_layers_segfault() {

		let res: vkraw::VkResult;
		let mut instance: vkraw::VkInstance = 0;
		let application_info = vkraw::VkApplicationInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
			pNext: ptr::null(),
			pApplicationName: "app name".as_ptr(),
			applicationVersion: vkraw::VK_MAKE_VERSION(1,0,0),
			pEngineName: "engine name".as_ptr(),
			engineVersion: vkraw::VK_MAKE_VERSION(1,0,0),
			apiVersion: vkraw::VK_MAKE_VERSION(1,0,0),
		};
		let create_info = vkraw::VkInstanceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			pApplicationInfo: &application_info,
			enabledLayerCount: 100,
			ppEnabledLayerNames: ptr::null(),
			enabledExtensionCount: 0,
			ppEnabledExtensionNames: ptr::null(),
		};
		unsafe {
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};
	}*/

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
				api_version: vkraw::VK_MAKE_VERSION(0, 0, 0),
			},
			enabled_layers: vec::Vec::new(),
			enabled_extensions: vec::Vec::new()
		};
		let res = create_instance(ici);

		assert!(res.is_err());

		match res {
			Err(e) => assert!(e == vkraw::VkResult::VK_ERROR_INCOMPATIBLE_DRIVER),
			Ok(_) => {}
		}
		std::thread::sleep(std::time::Duration::from_millis(1000));
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
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}

#[test]
	fn test_device_two_instances() {
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

		let ici2 = InstanceCreateInfo {
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
		let res2 = create_instance(ici2);

		assert!(res2.is_ok());
		destroy_instance(res2.unwrap());
		destroy_instance(res.unwrap());
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}
}
