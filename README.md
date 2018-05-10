# vkrust

This project generates an unsafe Vulkan interface that mirrors the C interface in rust from a vk.xml. It will also expose a safe interface that wraps the unsafe operations as much as possible and handles lifetimes of Vulkan objects.

## Note

This is my attempt to learn rust... You probably don't want to use this code yet :)

# How to use

vkgen is a binary to convert the vulkan vk.xml file to an API usable in rust.

I.e.
```bash
cd vkgen
wget https://raw.githubusercontent.com/KhronosGroup/Vulkan-LoaderAndValidationLayers/master/scripts/vk.xml
cargo build
mkdir ../vkraw/src/
target/debug/vkgen vk.xml -o ../vkraw/src/lib.rs
```

vkraw is an unsafe library to use vulkan in rust.

I.e.
```rust
extern crate vkraw;

fn main() {
	let app_name = std::ffi::CString::new("app name").unwrap();
	let engine_name = std::ffi::CString::new("engine name").unwrap();
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
		enabledLayerCount: 0,
		ppEnabledLayerNames: ptr::null(),
		enabledExtensionCount: 0,
		ppEnabledExtensionNames: ptr::null(),
	};

	println!("Creating instance");
	let res: vkraw::VkResult;
	let mut instance: vkraw::VkInstance = 0;
	unsafe {
		res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
	};
	assert!(res == vkraw::VkResult::VK_SUCCESS);
	unsafe {
		vkraw::vkDestroyInstance(instance, ptr::null());
	}
}
```

vk is a wrapped, safe interface to vulkan in rust.

I.e.
```rust
extern crate vk;

fn main() {
	let instance = vk::Instance::new("my app",
#[cfg(debug_assertions)]
		vec!["VK_LAYER_LUNARG_standard_validation".to_string()],
#[cfg(not(debug_assertions))]
		vec![],
		vec!["VK_KHR_surface".to_string(), "VK_KHR_xcb_surface".to_string()]);

	instance.expect("Failed to create instance");
}
```

# Todo

- [x] XML parsing
- [x] Unsafe raw interface
- [x] Extension support
- [x] Function pointer loading
- [ ] Dynamic loading of libvulkan.so
- [ ] Safe interface
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

    git clone git://anongit.freedesktop.org/mesa/mesa
Now disable optimisations via CFLAGS and CXXFLAGS:

    export CFLAGS="-g -O0"
    export CXXFLAGS="-g -O0"
Then configure it to enable the vulkan library and debugging symbols and make:

    ./configure --with-dri-drivers=i915 --with-vulkan-drivers=intel --enable-gles2 --with-gallium-drivers= --enable-debug
	make
Now create an ICD (Installable Client Driver) json file somewhere (debug_intel.json)

    {
      "file_format_version": "1.0.0",
      "ICD": {
        "library_path": "/path/to/libvulkan_intel.so",
        "api_version": "1.0.3"
      }
    }
Point your application at it with an environment variable:

	export VK_ICD_FILENAMES=/path/to/debug_intel.json

# Interface tests

```rust
extern crate crossbeam;

struct Instance;
impl Instance {
	fn new() -> Instance {
		println!("Instance::new()");
		Instance {}
	}

	fn create_mem(&self) -> Mem {
		println!("Instance::create_mem()");
		Mem { instance: self }
	}
}
impl Drop for Instance {
	fn drop(&mut self) {
		println!("Instance::drop()");
	}
}

// The lifetime specifiers ('a) here say "Mem should live a shorter lifetime than Instance because we have a reference to it"
struct Mem<'a> {
	instance: &'a Instance
}
impl<'a> Mem<'a> {
	// &mut self means that we can only ever have one MappedMem instance (Cannot map() twice)
	fn map(&mut self) -> MappedMem {
		println!("Mem::map()");
		MappedMem { mem: self }
	}
}
impl<'a> Drop for Mem<'a> {
	fn drop(&mut self) {
		println!("Mem::drop()");
	}
}

// MappedMem should live a shorter lifetime than Mem
struct MappedMem<'a> {
	mem: &'a Mem<'a>
}
impl<'a> MappedMem<'a> {
	fn get_ptr<T>(&mut self) -> *mut T {
		println!("MappedMem::get_ptr()");
		0 as *mut T
	}
}
impl<'a> Drop for MappedMem<'a> {
	fn drop(&mut self) {
		println!("MappedMem::drop()");
	}
}

fn main() {
	let instance_ptr = std::sync::Arc::new(Instance::new());

	// We can create multiple instances of Mem
	// if we want to transfer them across thread boundaries then we need to wrap them in a mutex/ref count
	let mut mem = instance_ptr.create_mem();
	let mut mem2 = instance_ptr.create_mem();

	// We can only map memory from a Mem once (because map is &mut self), before it gets unmapped by the Drop
	// if we want to transfer the MappedMem across thread boundaries it also needs to be wrapped in a mutex/ref count
	let mapped_mem_ptr = std::sync::Arc::new(std::sync::Mutex::new(mem.map()));
	//let test = mem.map(); // Error

	{
		//let mut mem3;

		let instance_ptr_clone = instance_ptr.clone();
		let mapped_mem_ptr_clone = mapped_mem_ptr.clone();

		println!("start of thread");
		crossbeam::scope(|scope| {
			scope.spawn(move || {

				// Accessing the MappedMem from another thread by using a mutex
				let mut m = mapped_mem_ptr_clone.lock().unwrap();
				let asdf: *mut u32 = m.get_ptr();
				//unsafe { *asdf = 1; }
				let mut mem4 = instance_ptr_clone.create_mem();
				// TODO: cant create memory here and pass it outside of the thread boundary because we need a
				// reference to Instance. We have a reference counted pointer to an Instance. So if we could get a reference
				// to it, it could be destroyed while we still have a reference
				//mem3 = instance_ptr_clone.create_mem();
			});
		});
		println!("end of thread");

		let mut m = mapped_mem_ptr.lock().unwrap();
		let asdf: *mut u32 = m.get_ptr();
	}
}
