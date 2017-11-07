# vkrust

This library exposes an unsafe Vulkan interface that mirrors the C interface in rust. It will also expose a safe interface that wraps the unsafe operations as much as possible and handles lifetimes of Vulkan objects.

## Note

This is my attempt to learn rust... You probably don't want to use this code yet :)

# How to use

Binary to convert the vulkan vk.xml file to an API usable in rust.

I.e.
```bash
touch src/lib.rs
wget https://raw.githubusercontent.com/KhronosGroup/Vulkan-LoaderAndValidationLayers/master/scripts/vk.xml
cargo build
target/debug/rust_vulkan_api_generator vk.xml -o src/lib.rs
cargo build --examples
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

# Debugging

Occasionally you will need to debug why the usermode driver of a particular vendor crashes because the layers do not catch the error. At least for intel and AMD you can do this.

## Intel

You will need to compile mesa with debugging symbols and optimisations turned off. First clone mesa somewhere
```git://anongit.freedesktop.org/mesa/mesa```
Now disable optimisations via CFLAGS and CXXFLAGS:
```export CFLAGS="-g -O0"
export CXXFLAGS="-g -O0"```
Then configure it to enable the vulkan library and debugging symbols:
```./configure --with-dri-drivers=i915 --with-vulkan-drivers=intel --enable-gles2 --with-gallium-drivers= --enable-debug```
