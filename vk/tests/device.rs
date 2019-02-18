
extern crate vk;

#[cfg(test)]
mod tests {

#[test]
	fn device_test() {

		let instance = vk::Instance::new("my app", vec![], vec![]).expect("Failed to create instance");

		let physical_devices = instance.physical_devices();
		println!("Found {} physical devices", physical_devices.len());
		for physical_device in physical_devices.iter().enumerate() {

		
		}
	}
}
