# vkrust

Binary to convert the vulkan vk.xml file to an API usable in rust.

I.e.
```bash
./rust_vulkan_api_generator path_to/vk.xml -o src/lib.rs
```

Library to use vulkan in rust.

I.e.
```rust
extern crate vkrust;

fn main() {
	let res: vkrust::VkResult;
	unsafe {
		res = vkrust::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
	match res {
		vkrust::VkResult::VK_SUCCESS => Ok(instance),
		_ => Err(res)
	}
}
```

## Note

This is my attempt to learn rust... You probably don't want to use this code yet :)
