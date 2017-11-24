
extern crate vk;

fn main() {

	let vk = vk::Vk::new();

	println!("Found {} instance extensions:", vk.instance_extensions_available.len());
	for e in vk.instance_extensions_available {

		println!(" Name {}", e.extension_name);
		println!(" Version {}", e.spec_version);
	}

	println!("Found {} instance layers:", vk.instance_layers_available.len());
	for e in vk.instance_layers_available {

		println!(" Name {}", e.layer_name);
		println!(" Version {}", e.spec_version);
		println!(" Impl version {}", e.implementation_version);
		println!(" Description {}", e.description);
	}

	println!("Creating vulkan instance");
	let instance = vk::Instance::new("my app",
#[cfg(debug_assertions)]
		vec!["VK_LAYER_LUNARG_standard_validation".to_string()],
#[cfg(not(debug_assertions))]
		vec![],
		vec![
			"VK_KHR_surface".to_string(),
			"VK_KHR_xcb_surface".to_string(),
		]).expect("Failed to create instance");

	println!("Creating WSI");
	let wsi_info = instance.create_wsi(800, 600);

	let mut physical_device_to_use = None;
	let mut queue_family_index_to_use = None;
	let mut num_queues = None;

	let physical_devices = instance.physical_devices();
	println!("Found {} physical devices", physical_devices.len());
	for (i,physical_device) in physical_devices.iter().enumerate() {

		println!(" Physical device {}:", i);
		println!("  Name {}", physical_device.properties.device_name);
		println!("  Type {:?}", physical_device.properties.device_type);
		println!("  Queue families {}:", physical_device.queue_family_properties.len());

		for (j,queue_family) in physical_device.queue_family_properties.iter().enumerate() {

			println!("   Queue family {}:", j);
			println!("    Queue count {}", queue_family.queueCount);
			println!("    Flags {:?}", queue_family.queueFlags);

			if instance.queue_family_supports_surface(physical_device, j as u32, &wsi_info.2) {

				println!("    Queue family supports surface");

				physical_device_to_use = Some(physical_device);
				queue_family_index_to_use = Some(j);
				num_queues = Some(queue_family.queueCount);
			}
		}

		println!("  Memory types {}:", physical_device.memory_properties.memoryTypeCount);
		for t in 0..physical_device.memory_properties.memoryTypeCount as usize {
			println!("    Memory type {}:", t);
			println!("     Flags {:?}", physical_device.memory_properties.memoryTypes[t].propertyFlags);
			println!("     Heap index {}", physical_device.memory_properties.memoryTypes[t].heapIndex);
		}
		println!("  Memory heaps {}:", physical_device.memory_properties.memoryTypeCount);
		for h in 0..physical_device.memory_properties.memoryHeapCount as usize {
			println!("    Memory heap {}:", h);
			println!("     Flags {:?}", physical_device.memory_properties.memoryHeaps[h].flags);
			println!("     Size {}", physical_device.memory_properties.memoryHeaps[h].size);
		}

		println!("  VK_KHR_display: {}", if physical_device.display_properties.is_some() { "supported" } else { "not supported" });
	}

	if physical_device_to_use.is_none() || queue_family_index_to_use.is_none() || num_queues.is_none() {

		println!("Could not find suitable device or queue family index");
	} else {

		println!("Creating device");
		let device = physical_device_to_use.unwrap().create_device(
			vec![],
			vec!["VK_KHR_swapchain".to_string()],
			queue_family_index_to_use.unwrap() as u32,
			vec![1.0; num_queues.unwrap() as usize]).expect("Failed to create device");
	}
}
