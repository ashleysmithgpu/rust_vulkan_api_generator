
extern crate vk;

#[cfg(test)]
mod tests {

	use vk::*;

#[test]
	fn device_test() {

		let instance = vk::Instance::new("my app",
#[cfg(debug_assertions)]
			vec!["VK_LAYER_LUNARG_standard_validation".to_string()],
#[cfg(not(debug_assertions))]
			vec![],
			vec!["VK_KHR_surface".to_string(), "VK_KHR_xcb_surface".to_string()]);

		instance.expect("Failed to create instance");
	}
}
