
extern crate vk;

#[cfg(test)]
mod tests {

#[test]
	fn device_test() {

		let instance = vk::Instance::new("my app",
#[cfg(debug_assertions)]
			vec!["VK_LAYER_LUNARG_standard_validation".to_string()],
#[cfg(not(debug_assertions))]
			vec![],
			vec!["VK_KHR_surface".to_string(), "VK_KHR_xcb_surface".to_string()]).expect("Failed to create instance");

		let physical_devices = instance.physical_devices();
		println!("Found {} physical devices", physical_devices.len());
		for physical_device in physical_devices {

			
		}
	}
}
