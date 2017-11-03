# vkrust

Plan to expose an unsafe interface that basically mirrors the C interface, but in rust and a safe interface that wraps the unsafe operations as much as possible and moves vkCreate*, vkDestroy* to constructors/destructors.

Currently working:

Binary to convert the vulkan vk.xml file to an API usable in rust.

I.e.
```bash
wget https://raw.githubusercontent.com/KhronosGroup/Vulkan-LoaderAndValidationLayers/master/scripts/vk.xml
./rust_vulkan_api_generator vk.xml -o src/lib.rs
```

Unsafe library to use vulkan in rust.

I.e.
```rust
extern crate vkrust;

fn main() {
	let res: vkrust::VkResult;
	let mut instance: vkrust::VkInstance = 0;
	let application_info = vkrust::VkApplicationInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_APPLICATION_INFO,
		pNext: ptr::null(),
		pApplicationName: "app name".as_ptr(),
		applicationVersion: vkrust::VK_MAKE_VERSION(1,0,0),
		pEngineName: "engine name".as_ptr(),
		engineVersion: vkrust::VK_MAKE_VERSION(1,0,0),
		apiVersion: vkrust::VK_MAKE_VERSION(1,0,0),
	};
	let create_info = vkrust::VkInstanceCreateInfo {
		sType: vkrust::VkStructureType::VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
		pNext: ptr::null(),
		flags: 0,
		pApplicationInfo: &application_info,
		enabledLayerCount: 0,
		ppEnabledLayerNames: ptr::null(),
		enabledExtensionCount: 0,
		ppEnabledExtensionNames: ptr::null(),
	};
	unsafe {
		res = vkrust::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
	assert!(res == vkrust::VkResult::VK_SUCCESS);
	unsafe {
		vkrust::vkDestroyInstance(instance, ptr::null());
	}
}
```

# Todo

- [x] XML parsing
- [ ] Unsafe raw interface
- [ ] Safe interface
- [ ] Extension support
- [ ] Loader implementation
- [ ] Tests

## Note

This is my attempt to learn rust... You probably don't want to use this code yet :)

## Interesting stuff

I'm really liking rust, imagine the bugs that this eliminates:
```rust
pub enum VkStructureType {
	STANDARD_ENUM_VALUES=0,
	//...
#[cfg(feature="xcb")]
	XCB_SPECIFIC_ENUM_VALUE=123
}
```
Rust disallows enum values not present by default so we can only pass valid values to whatever takes a VkStructureType.
Match (switch) statements that don't handle all values (cases) are known at compile time.

