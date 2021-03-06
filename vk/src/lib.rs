
use colored::*;

use std::ptr;
use std::ffi::CString;
use std::mem;
use std::io::prelude::*;

use rspirv::binary::Disassemble;

#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
	use std::os::windows::ffi::OsStrExt;
	std::ffi::OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
}

pub struct Instance {
	pub instance: vkraw::VkInstance,
	pub vk: vkraw::VulkanFunctionPointers,
	pub callback: vkraw::VkDebugReportCallbackEXT,
}

pub struct Device<'a> {
	pub device: vkraw::VkDevice,
	pub instance: &'a Instance
}

pub struct Buffer<'a> {
	pub buffer: vkraw::VkBuffer,
	pub device: &'a Device<'a>
}

pub struct Queue<'a> {
	pub queue: vkraw::VkQueue,
	pub device: &'a Device<'a>
}

pub struct Surface<'a> {
	pub surface: vkraw::VkSurfaceKHR,
	instance: &'a Instance
}

pub struct PhysicalDevice<'a> {
	pub physical_device: vkraw::VkPhysicalDevice,
	pub instance: &'a Instance
}

pub struct Swapchain<'a> {
	pub swapchain: vkraw::VkSwapchainKHR,
	pub device: &'a Device<'a>
}

pub struct Image<'a> {
	pub image: vkraw::VkImage,
	pub device: &'a Device<'a>,
	swapchain_image: bool
}

pub struct ImageView<'a> {
	pub image_view: vkraw::VkImageView,
	pub image: &'a Image<'a>
}

pub struct CommandPool<'a> {
	pub command_pool: vkraw::VkCommandPool,
	pub device: &'a Device<'a>
}

pub struct CommandBuffer<'a> {
	pub command_buffer: vkraw::VkCommandBuffer,
	pub command_pool: &'a CommandPool<'a>
}

pub struct RenderPass<'a> {
	pub render_pass: vkraw::VkRenderPass,
	pub device: &'a Device<'a>,
}

pub struct Framebuffer<'a> {
	pub framebuffer: vkraw::VkFramebuffer,
	pub device: &'a Device<'a>,
}

pub struct Semaphore<'a> {
	pub semaphore: vkraw::VkSemaphore,
	pub device: &'a Device<'a>,
}

pub struct Fence<'a> {
	pub fence: vkraw::VkFence,
	pub device: &'a Device<'a>,
}

pub struct DescriptorSetLayout<'a> {
	pub dsl: vkraw::VkDescriptorSetLayout,
	pub device: &'a Device<'a>,
}

pub struct PipelineLayout<'a> {
	pub pipeline_layout: vkraw::VkPipelineLayout,
	pub dsls: Vec<&'a DescriptorSetLayout<'a>>,
	pub device: &'a Device<'a>,
}

pub struct Pipeline<'a> {
	pub pipeline: vkraw::VkPipeline,
	pub device: &'a Device<'a>,
}

pub struct ComputePipeline<'a> {
	pub pipeline: vkraw::VkPipeline,
	pub device: &'a Device<'a>,
}

pub struct ShaderModule<'a> {
	pub module: vkraw::VkShaderModule,
	pub device: &'a Device<'a>,
}

pub struct DescriptorPool<'a> {
	pub descriptor_pool: vkraw::VkDescriptorPool,
	pub device: &'a Device<'a>,
}

pub struct DescriptorSet<'a> {
	pub descriptor_set: vkraw::VkDescriptorSet,
	pub descriptor_pool: &'a DescriptorPool<'a>,
	pub set_layouts: &'a DescriptorSetLayout<'a>
}

pub struct Sampler<'a> {
	pub sampler: vkraw::VkSampler,
	pub device: &'a Device<'a>,
}

pub enum ClearValue {
	Colourf32([f32; 4]),
	Colouri32([i32; 4]),
	Colouru32([u32; 4]),
	DepthStencil{ depth: f32, stencil: u32 }
}

fn debug_message_callback(flags: libc::c_int, otype: libc::c_int, srco: u64, loc: usize, msgcode: u32, layer: *const libc::c_char, msg: *const libc::c_char, _userdata: *mut libc::c_void) -> bool {

	let c_s = unsafe { std::ffi::CStr::from_ptr(msg) };
	let c_sl: &str = c_s.to_str().unwrap();

	let c_l = unsafe { std::ffi::CStr::from_ptr(layer) };
	let c_ll: &str = c_l.to_str().unwrap();

	let flags = vkraw::VkDebugReportFlagsEXT::from_bits_truncate(flags as u32);
	let obj_type: vkraw::VkDebugReportObjectTypeEXT = unsafe { std::mem::transmute(otype) };

	return rust_debug_message_callback(flags, obj_type, srco, loc, msgcode, c_ll.to_string(), c_sl.to_string());
}

pub fn rust_debug_message_callback(flags: vkraw::VkDebugReportFlagsEXT, obj_type: vkraw::VkDebugReportObjectTypeEXT, src_obj: u64, location: usize, msg_code: u32, layer: String, message: String) -> bool {

	println!("{}:\n {}", format!("f:{:?}, ot:{:?}, o:{:?}, loc:{:?}, c:{:?}, l:{:?}", flags, obj_type, src_obj, location, msg_code, layer).red(), message);
	true
}

pub struct InstanceBuilder {
	pub layers: Vec<String>,
	pub extensions: Vec<String>,
	pub application_name: String,
	pub debug_message_callback: fn(vkraw::VkDebugReportFlagsEXT, vkraw::VkDebugReportObjectTypeEXT, u64, usize, u32, String, String) -> bool,
	pub args: Vec<String>
}

impl Default for InstanceBuilder {
	fn default() -> Self {
		Self {
			layers: vec![ 
				#[cfg(debug_assertions)]
				"VK_LAYER_LUNARG_standard_validation".to_string(),
				//"VkLayer_core_validation".to_string(),
				//"VkLayer_object_lifetimes".to_string(),
				//"VkLayer_stateless_validation".to_string(),
				//"VkLayer_thread_safety".to_string(),
				//"VkLayer_unique_objects".to_string(),
			],
			extensions: vec![
				#[cfg(debug_assertions)]
				"VK_EXT_debug_report".to_string(),
				"VK_KHR_surface".to_string(),
				#[cfg(windows)]
				"VK_KHR_win32_surface".to_string(),
				#[cfg(unix)]
				"VK_KHR_xcb_surface".to_string(),
				"VK_KHR_get_surface_capabilities2".to_string(),

				//"VK_EXT_swapchain_colorspace".to_string(),

				"VK_KHR_get_physical_device_properties2".to_string(),
				//"VK_KHR_swapchain".to_string(),
				//"VK_EXT_full_screen_exclusive".to_string(),
				
				//"VK_KHR_display".to_string(),
				//"VK_KHR_display_swapchain".to_string(),
			],
			application_name: "rust vulkan application".to_string(),
			debug_message_callback: rust_debug_message_callback,
			args: Vec::<String>::new()
		}
	}
}

impl InstanceBuilder {
	pub fn new() -> InstanceBuilder {
		Self::default()
	}
	pub fn create_instance(&self) -> Result<Instance, vkraw::VkResult> {

		// Check available layers/extensions
		let available_layers: Vec<String>;
		let available_extensions: Vec<String>;

		let mut num_available_extensions: u32 = 0;
		let mut num_available_layers: u32 = 0;
		let ext_res;
		let layer_res;
		unsafe {
			ext_res = vkraw::vkEnumerateInstanceExtensionProperties(ptr::null_mut(), &mut num_available_extensions, ptr::null_mut());
			layer_res = vkraw::vkEnumerateInstanceLayerProperties(&mut num_available_layers, ptr::null_mut());
		};

		let mut available_layers_struct = Vec::<vkraw::VkLayerProperties>::with_capacity(num_available_layers as usize);
		let mut available_extensions_struct = Vec::<vkraw::VkExtensionProperties>::with_capacity(num_available_extensions as usize);
		unsafe {
			if ext_res == vkraw::VkResult::VK_SUCCESS {
				let res = vkraw::vkEnumerateInstanceExtensionProperties(ptr::null_mut(), &mut num_available_extensions, available_extensions_struct.as_mut_ptr());
				if res == vkraw::VkResult::VK_SUCCESS {
					available_extensions_struct.set_len(num_available_extensions as usize);
				}
			}
			available_extensions = available_extensions_struct.iter().map(|x| std::ffi::CStr::from_ptr(&x.extensionName[0] as *const u8 as *const i8).to_owned().into_string().unwrap()).collect();
			if layer_res == vkraw::VkResult::VK_SUCCESS {
				let res = vkraw::vkEnumerateInstanceLayerProperties(&mut num_available_layers, available_layers_struct.as_mut_ptr());
				if res == vkraw::VkResult::VK_SUCCESS {
					available_layers_struct.set_len(num_available_layers as usize);
				}
			}
			available_layers = available_layers_struct.iter().map(|x| std::ffi::CStr::from_ptr(&x.layerName[0] as *const u8 as *const i8).to_owned().into_string().unwrap()).collect();
		};

		let res: vkraw::VkResult;
		let mut instance: vkraw::VkInstance = 0;

		// Create copy of each of the strings as a null terminated string for C
		// warn about unavailable layers
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(self.layers.len());
		for l in &self.layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
			if !available_layers.iter().any(|x| x == l) {
				println!("{}", format!("Layer {} not available", l).red());
			}
		}

		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(self.extensions.len());
		for e in &self.extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
			if !available_extensions.iter().any(|x| x == e) {
				println!("{}", format!("Extension {} not available", e).red());
			}
		}

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(enabled_layers_rust.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(enabled_extensions_rust.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}
		let app_name = CString::new(self.application_name.clone()).unwrap();
		let engine_name = CString::new("engine name").unwrap();
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
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr()
		};

		println!("vkCreateInstance");
		unsafe {
			res = vkraw::vkCreateInstance(&create_info, ptr::null(), &mut instance);
		};

		let vk = vkraw::VulkanFunctionPointers::new(instance);

		if res == vkraw::VkResult::VK_SUCCESS {
			assert!(instance != vkraw::VK_NULL_HANDLE);

			let mut callback: vkraw::VkDebugReportCallbackEXT;
			unsafe {
				callback = std::mem::uninitialized();
				let rust_fptr = debug_message_callback;
				let c_fptr: vkraw::PFN_vkDebugReportCallbackEXT = rust_fptr as *const libc::c_void;
				let drcci = vkraw::VkDebugReportCallbackCreateInfoEXT {
					sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
					pNext: std::ptr::null(),
					flags: vkraw::VkDebugReportFlagBitsEXT::all() & !vkraw::VkDebugReportFlagBitsEXT::VK_DEBUG_REPORT_INFORMATION_BIT_EXT,
					pfnCallback: c_fptr,
					pUserData: std::ptr::null_mut()
				};
				let res2 = vk.CreateDebugReportCallbackEXT.unwrap()(instance, &drcci, ptr::null(), &mut callback);
				assert!(res2 == vkraw::VkResult::VK_SUCCESS);
			};

			Ok(Instance { instance: instance, vk: vk, callback: callback })
		} else {
			Err(res)
		}
	}
}

impl Drop for Instance {
	fn drop(&mut self) {
		assert!(self.instance != vkraw::VK_NULL_HANDLE);
		unsafe {
			self.vk.DestroyDebugReportCallbackEXT.unwrap()(self.instance, self.callback, ptr::null());

			println!("vkDestroyInstance");
			vkraw::vkDestroyInstance(self.instance, ptr::null());
		}
	}
}

impl Instance {
	pub fn vk(&self) -> &vkraw::VulkanFunctionPointers {
		&self.vk
	}

	pub fn physical_devices(&self) -> Vec<PhysicalDevice> {

		let mut num_physical_devices = 0;
		let mut res: vkraw::VkResult;
		unsafe {
			res = vkraw::vkEnumeratePhysicalDevices(self.instance, &mut num_physical_devices, 0 as *mut u64);
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);

		let mut physical_devices = Vec::<PhysicalDevice>::with_capacity(num_physical_devices as usize);

		let mut vk_physical_devices = Vec::<vkraw::VkPhysicalDevice>::with_capacity(num_physical_devices as usize);
		unsafe {
			vk_physical_devices.set_len(num_physical_devices as usize);
			res = vkraw::vkEnumeratePhysicalDevices(self.instance, &mut num_physical_devices, vk_physical_devices.as_mut_ptr());
		}
		assert!(res == vkraw::VkResult::VK_SUCCESS);

		for d in vk_physical_devices {
			assert!(d != vkraw::VK_NULL_HANDLE);

			physical_devices.push(PhysicalDevice {
				physical_device: d,
				instance: &self
			});
		}
		return physical_devices;
	}

	#[cfg(unix)]
	pub fn create_wsi(&self, width: u32, height: u32, fullscreen: bool) -> (Surface, xcb::Connection, u32) {

		assert!(width <= std::u16::MAX as u32);
		assert!(height <= std::u16::MAX as u32);

		let mut surface: vkraw::VkSurfaceKHR = 0;
		println!("Creating XCB window");
		let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
		let win;
		{
			let setup = conn.get_setup();
			let screen = setup.roots().nth(screen_num as usize).unwrap();

			win = conn.generate_id();
			xcb::create_window(&conn,
				xcb::COPY_FROM_PARENT as u8,
				win,
				screen.root(),
				0, 0,
				width as u16, height as u16,
				0,
				xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
				screen.root_visual(), &[
					(xcb::CW_BACK_PIXEL, screen.white_pixel()),
					(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS | xcb::EVENT_MASK_KEY_RELEASE),
				]
			);

			if fullscreen {
				let atom_wm_state = xcb::intern_atom(&conn, false, "_NET_WM_STATE");
				let atom_wm_fullscreen = xcb::intern_atom(&conn, false, "_NET_WM_STATE_FULLSCREEN");
				let s = match atom_wm_state.get_reply() {
					Ok(s) => s.atom(),
					Err(_) => panic!("could not load _NET_WM_STATE atom")
				};
				let sfs = match atom_wm_fullscreen.get_reply() {
					Ok(sfs) => sfs.atom(),
					Err(_) => panic!("could not load _NET_WM_STATE_FULLSCREEN atom")
				};
				let protocols = [sfs];
				xcb::change_property(&conn,
					xcb::PROP_MODE_REPLACE as u8,
					win, s,
					xcb::ATOM_ATOM, 32,
					&protocols);
			}

			xcb::map_window(&conn, win);
			conn.flush();

			let surface_create_info = vkraw::VkXcbSurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				connection: conn.get_raw_conn(),
				window: win
			};

			assert!(self.vk.CreateXcbSurfaceKHR.is_some());
			let res = self.vk.CreateXcbSurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		(Surface { surface: surface, instance: self }, conn, win)
	}

	#[cfg(windows)]
	pub fn create_wsi(&self, width: u32, height: u32, fullscreen: bool) -> (Surface, winapi::shared::windef::HWND, winapi::shared::minwindef::HINSTANCE) {

		let hinstance;
		let handle;
		let mut surface: vkraw::VkSurfaceKHR = 0;
		unsafe {

			winapi::um::shellscalingapi::SetProcessDpiAwareness(winapi::um::shellscalingapi::PROCESS_SYSTEM_DPI_AWARE);

			let name = win32_string("windoze");
			println!("Creating WIN32 window");
			hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null_mut());
			let wnd_class = winapi::um::winuser::WNDCLASSW {
				style : winapi::um::winuser::CS_HREDRAW | winapi::um::winuser::CS_VREDRAW,
				lpfnWndProc: Some(winapi::um::winuser::DefWindowProcW),
				hInstance: hinstance,
				lpszClassName: name.as_ptr(),
				cbClsExtra: 0,
				cbWndExtra: 0,
				hIcon: std::ptr::null_mut(),
				hCursor: std::ptr::null_mut(),
				hbrBackground: std::ptr::null_mut(),
				lpszMenuName: std::ptr::null_mut(),
			};
			winapi::um::winuser::RegisterClassW(&wnd_class);

			println!("Window {}x{}", width, height);

			let mut window_rect = winapi::shared::windef::RECT {
				left: 0,
				top: 0,
				right: width as i32,
				bottom: height as i32
			};

			let style = winapi::um::winuser::WS_OVERLAPPEDWINDOW | winapi::um::winuser::WS_CLIPSIBLINGS | winapi::um::winuser::WS_CLIPCHILDREN;
			let exstyle = winapi::um::winuser::WS_EX_APPWINDOW | winapi::um::winuser::WS_EX_WINDOWEDGE;
			winapi::um::winuser::AdjustWindowRectEx(&mut window_rect, style, 0, exstyle);

			handle = winapi::um::winuser::CreateWindowExW(
				0,
				name.as_ptr(),
				name.as_ptr(),
				style, 
				0,
				0,
				window_rect.right - window_rect.left,
				window_rect.bottom - window_rect.top,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				hinstance,
				std::ptr::null_mut());

			let surface_create_info = vkraw::VkWin32SurfaceCreateInfoKHR {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
				pNext: ptr::null(),
				flags: 0,
				hinstance: hinstance as u64,
				hwnd: handle as u64
			};

			winapi::um::winuser::ShowWindow(handle, winapi::um::winuser::SW_SHOW);
			winapi::um::winuser::SetForegroundWindow(handle);
			winapi::um::winuser::SetFocus(handle);

			if fullscreen {
				let style = /*winapi::shared::basetsd::LONG_PTR:*/ winapi::um::winuser::GetWindowLongPtrA(handle, winapi::um::winuser::GWL_STYLE);
				let exstyle = /*winapi::shared::basetsd::LONG_PTR:*/ winapi::um::winuser::GetWindowLongPtrA(handle, winapi::um::winuser::GWL_EXSTYLE);

				winapi::um::winuser::SetWindowLongPtrA(handle, winapi::um::winuser::GWL_STYLE, style & (!(winapi::um::winuser::WS_CAPTION | winapi::um::winuser::WS_THICKFRAME) as isize));
				winapi::um::winuser::SetWindowLongPtrA(handle, winapi::um::winuser::GWL_EXSTYLE, exstyle & (!(winapi::um::winuser::WS_EX_DLGMODALFRAME | winapi::um::winuser::WS_EX_WINDOWEDGE | winapi::um::winuser::WS_EX_CLIENTEDGE | winapi::um::winuser::WS_EX_STATICEDGE) as isize));

				let mut monitor_info: winapi::um::winuser::MONITORINFO;
				monitor_info = std::mem::uninitialized();
				monitor_info.cbSize = std::mem::size_of::<winapi::um::winuser::MONITORINFO>() as u32;
				winapi::um::winuser::GetMonitorInfoA(winapi::um::winuser::MonitorFromWindow(handle, winapi::um::winuser::MONITOR_DEFAULTTONEAREST), &mut monitor_info);
				winapi::um::winuser::SetWindowPos(handle, ptr::null_mut(), monitor_info.rcMonitor.left, monitor_info.rcMonitor.top, 
					monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
					monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
					winapi::um::winuser::SWP_NOZORDER | winapi::um::winuser::SWP_NOACTIVATE | winapi::um::winuser::SWP_FRAMECHANGED);
					}

				assert!(self.vk.CreateWin32SurfaceKHR.is_some());
				println!("vk.CreateWin32SurfaceKHR");
				let res = self.vk.CreateWin32SurfaceKHR.unwrap()(self.instance, &surface_create_info, ptr::null(), &mut surface);
				assert!(res == vkraw::VkResult::VK_SUCCESS);
			}

		(Surface { surface: surface, instance: self }, handle, hinstance)
	}
}

impl<'a> PhysicalDevice<'a> {
	pub fn queue_families(&self) -> Vec<vkraw::VkQueueFamilyProperties> {
		let mut num_physical_devices = 0;
		unsafe {
			vkraw::vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut num_physical_devices, ptr::null_mut());
		}
		let mut physical_devices = Vec::<vkraw::VkQueueFamilyProperties>::with_capacity(num_physical_devices as usize);
		unsafe {
			physical_devices.set_len(num_physical_devices as usize);
			vkraw::vkGetPhysicalDeviceQueueFamilyProperties(self.physical_device, &mut num_physical_devices, physical_devices.as_mut_ptr());
		}
		return physical_devices;
	}

	pub fn physical_properties(&self) -> vkraw::VkPhysicalDeviceProperties {

		let mut props: vkraw::VkPhysicalDeviceProperties;
		unsafe {
			props = mem::uninitialized();
			vkraw::vkGetPhysicalDeviceProperties(self.physical_device, &mut props);
		}

		props
	}

	pub fn memory_properties(&self) -> (Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>) {
		let mut memory_properties: vkraw::VkPhysicalDeviceMemoryProperties;

		unsafe {
			memory_properties = mem::uninitialized();
			vkraw::vkGetPhysicalDeviceMemoryProperties(self.physical_device, &mut memory_properties);
		}

		let mut mt = Vec::<vkraw::VkMemoryType>::new();
		for i in 0..memory_properties.memoryTypeCount {
			mt.push(memory_properties.memoryTypes[i as usize]);
		}
		let mut mh = Vec::<vkraw::VkMemoryHeap>::new();
		for i in 0..memory_properties.memoryHeapCount {
			mh.push(memory_properties.memoryHeaps[i as usize]);
		}

		(mt, mh)
	}

	pub fn supported_surface_formats(&self, surface: &Surface) -> Result<Vec<vkraw::VkSurfaceFormatKHR>, vkraw::VkResult> {

		// Get a supported colour format and colour space
		let mut format_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
		self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface.surface, &mut format_count, ptr::null_mut());

		let mut surface_formats = Vec::<vkraw::VkSurfaceFormatKHR>::with_capacity(format_count as usize);
		unsafe {
			surface_formats.set_len(format_count as usize);
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface.surface, &mut format_count, surface_formats.as_mut_ptr());

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_formats)
		} else {
			Err(res)
		}
	}

	#[cfg(unix)]
	pub fn supported_surface_formats2(&self, hdr: bool, wsi_info: &(Surface, xcb::Connection, u32)) -> Result<Vec<vkraw::VkSurfaceFormat2KHR>, vkraw::VkResult> {

		let fullscreen_info = vkraw::VkSurfaceFullScreenExclusiveInfoEXT {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SURFACE_FULL_SCREEN_EXCLUSIVE_INFO_EXT,
			pNext: ptr::null_mut(),
			fullScreenExclusive: vkraw::VkFullScreenExclusiveEXT::VK_FULL_SCREEN_EXCLUSIVE_APPLICATION_CONTROLLED_EXT
		};

		let surface_info = vkraw::VkPhysicalDeviceSurfaceInfo2KHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR,
			pNext: if hdr { unsafe { mem::transmute(&fullscreen_info) } } else { ptr::null_mut() },
			surface: wsi_info.0.surface
		};

		// Get a supported colour format and colour space
		let mut format_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.is_some());
		self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.unwrap()(self.physical_device, &surface_info, &mut format_count, ptr::null_mut());

		assert!(format_count > 0);

		let mut surface_formats = vec![vkraw::VkSurfaceFormat2KHR { sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SURFACE_FORMAT_2_KHR, pNext: ptr::null_mut(), surfaceFormat: vkraw::VkSurfaceFormatKHR { format: vkraw::VkFormat::VK_FORMAT_UNDEFINED, colorSpace: vkraw::VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR }, }; format_count as usize];

		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.unwrap()(self.physical_device, &surface_info, &mut format_count, surface_formats.as_mut_ptr());

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_formats)
		} else {
			Err(res)
		}
	}

	#[cfg(windows)]
	pub fn supported_surface_formats2(&self, hdr: bool, wsi_info: &(Surface, winapi::shared::windef::HWND, winapi::shared::minwindef::HINSTANCE)) -> Result<Vec<vkraw::VkSurfaceFormat2KHR>, vkraw::VkResult> {

		let fullscreen_info = vkraw::VkSurfaceFullScreenExclusiveInfoEXT {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SURFACE_FULL_SCREEN_EXCLUSIVE_INFO_EXT,
			pNext: ptr::null_mut(),
			fullScreenExclusive: vkraw::VkFullScreenExclusiveEXT::VK_FULL_SCREEN_EXCLUSIVE_APPLICATION_CONTROLLED_EXT
		};
		let win32_fullscreen_info = vkraw::VkSurfaceFullScreenExclusiveWin32InfoEXT {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SURFACE_FULL_SCREEN_EXCLUSIVE_WIN32_INFO_EXT,
			pNext: unsafe { mem::transmute(&fullscreen_info) },
			hmonitor: unsafe { winapi::um::winuser::MonitorFromWindow(wsi_info.1, winapi::um::winuser::MONITOR_DEFAULTTOPRIMARY) as u64 }
		};

		let surface_info = vkraw::VkPhysicalDeviceSurfaceInfo2KHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR,
			pNext: if hdr { unsafe { mem::transmute(&win32_fullscreen_info) } } else { ptr::null_mut() },
			surface: wsi_info.0.surface
		};
	
		// Get a supported colour format and colour space
		let mut format_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.is_some());
		self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.unwrap()(self.physical_device, &surface_info, &mut format_count, ptr::null_mut());

		assert!(format_count > 0);

		let mut surface_formats = vec![vkraw::VkSurfaceFormat2KHR { sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SURFACE_FORMAT_2_KHR, pNext: ptr::null_mut(), surfaceFormat: vkraw::VkSurfaceFormatKHR { format: vkraw::VkFormat::VK_FORMAT_UNDEFINED, colorSpace: vkraw::VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR }, }; format_count as usize];

		assert!(self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceFormats2KHR.unwrap()(self.physical_device, &surface_info, &mut format_count, surface_formats.as_mut_ptr());

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_formats)
		} else {
			Err(res)
		}
	}

	pub fn surface_capabilities(&self, surface: &Surface) -> Result<vkraw::VkSurfaceCapabilitiesKHR, vkraw::VkResult> {
		let mut surface_capabilities: vkraw::VkSurfaceCapabilitiesKHR;
		unsafe {
			surface_capabilities = std::mem::uninitialized();
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.is_some());
		let res = self.instance.vk.GetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(self.physical_device, surface.surface, &mut surface_capabilities);
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(surface_capabilities)
		} else {
			Err(res)
		}
	}

	pub fn present_modes(&self, surface: &Surface) -> Result<Vec<vkraw::VkPresentModeKHR>, vkraw::VkResult> {

		let mut present_mode_count = 0;
		assert!(self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
		let mut res = self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface.surface, &mut present_mode_count, ptr::null_mut());
		if res != vkraw::VkResult::VK_SUCCESS {
			return Err(res)
		}
		assert!(present_mode_count > 0);
		let mut present_modes = Vec::<vkraw::VkPresentModeKHR>::with_capacity(present_mode_count as usize);
		unsafe {
			present_modes.set_len(present_mode_count as usize);
		}
		assert!(self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.is_some());
		res = self.instance.vk.GetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface.surface, &mut present_mode_count, present_modes.as_mut_ptr());
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(present_modes)
		} else {
			Err(res)
		}
	}
}

pub struct DeviceBuilder<'a> {
	instance: &'a Instance,
	pub layers: Vec<String>,
	pub extensions: Vec<String>,
	pub queue_create_infos: Vec<(u32, Vec<f32>)>,
	pub want_device_name: String,
	pub physical_device: Option<(PhysicalDevice<'a>, usize)> // Physical device and index
}

impl<'a> DeviceBuilder<'a> {
	pub fn new(instance: &Instance) -> DeviceBuilder {
		DeviceBuilder {
			instance: instance,
			layers: vec![ 
				#[cfg(debug_assertions)]
				"VK_LAYER_LUNARG_standard_validation".to_string(),
			],
			extensions: vec![
				"VK_KHR_swapchain".to_string(),
				//"VK_EXT_full_screen_exclusive".to_string(),
				//"VK_AMD_display_native_hdr".to_string(),
				"VK_KHR_shader_float16_int8".to_string(),
			],
			queue_create_infos: vec![(0, vec![1.0])],
			want_device_name: String::new(),
			physical_device: None
		}
	}
	pub fn use_device_named<'y>(&'y mut self, name: String) -> &'y mut Self {
		self.want_device_name = name;
		self
	}
	pub fn default_queues_physical_device<'y>(&'y mut self, surface: &Surface) -> &'y mut Self {

		let physical_devices = self.instance.physical_devices();
		assert!(physical_devices.len() > 0);
		let mut graphics_queue_family_index = Vec::<usize>::new();
		let mut compute_queue_family_index = Vec::<usize>::new();
		let mut transfer_queue_family_index = Vec::<usize>::new();

		// Loop through the physical devices
		let pd = physical_devices.iter().enumerate().filter_map(
			|(device_index, device)|
			{
				let qf = device.queue_families();
				
				let s;
				unsafe {
					s = std::ffi::CStr::from_ptr(device.physical_properties().deviceName.as_ptr() as *const i8).to_str().unwrap().to_string();
					println!("Name: {}", s);
				}
				if s.contains("GeForce") {
					return None
				}

				let mut queue_supports_present = vkraw::VK_FALSE;

				// Loop through each of the family queues in the physical device
				graphics_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {

						// TODO: could want to present on the compute queue
						// Check if this queue supports presenting to the wsi surface
						assert!(self.instance.vk.GetPhysicalDeviceSurfaceSupportKHR.is_some());
						self.instance.vk.GetPhysicalDeviceSurfaceSupportKHR.unwrap()(device.physical_device, queue_family_index as u32, surface.surface, &mut queue_supports_present);

						// If we find a matching family, push the index on the Vec
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_GRAPHICS_BIT) 
							&& queue_supports_present == vkraw::VK_TRUE {
							Some(queue_family_index)
						} else {
							None
						}
					}
				).collect();
				compute_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_COMPUTE_BIT) { Some(queue_family_index) } else { None }
					}
				).collect();
				transfer_queue_family_index = qf.iter().enumerate().filter_map(
					|(queue_family_index, queue_family)| {
						if queue_family.queueFlags.intersects(vkraw::VkQueueFlagBits::VK_QUEUE_TRANSFER_BIT) { Some(queue_family_index) } else { None }
					}
				).collect();

				// Return the first physical device that has at least one of each queue family
				if graphics_queue_family_index.len() > 0 && compute_queue_family_index.len() > 0 && transfer_queue_family_index.len() > 0 &&
					(self.want_device_name.is_empty() || s.contains(&self.want_device_name)){
					Some((PhysicalDevice { physical_device: device.physical_device, instance: &self.instance }, device_index))
				} else {
					None
				}
			}).next().expect("Couldn't find a supported graphics queue");

		self.physical_device = Some(pd);

		assert!(graphics_queue_family_index.len() > 0);
		assert!(compute_queue_family_index.len() > 0);
		assert!(transfer_queue_family_index.len() > 0);

		// Use different queue families for better performance (first/last)
		self.queue_create_infos = vec![(graphics_queue_family_index.first().unwrap().clone() as u32, vec![1.0]),
			(compute_queue_family_index.last().unwrap().clone() as u32, vec![1.0]),
			(transfer_queue_family_index.last().unwrap().clone() as u32, vec![1.0])];
		self.queue_create_infos.dedup();

		self
	}
	pub fn create_device(&self) -> Result<Device<'a>, vkraw::VkResult> {

		let mut device: vkraw::VkDevice = 0;

		// Create copy of each of the strings as a null terminated string
		let mut enabled_layers_rust = Vec::<CString>::with_capacity(self.layers.len());
		for l in &self.layers {
			enabled_layers_rust.push(CString::new(l.clone()).unwrap());
		}
		let mut enabled_extensions_rust = Vec::<CString>::with_capacity(self.extensions.len());
		for e in &self.extensions {
			enabled_extensions_rust.push(CString::new(e.clone()).unwrap());
		}

		// Create a vector of pointers to the above
		let mut enabled_layers = Vec::<*const u8>::with_capacity(enabled_layers_rust.len());
		for l in &enabled_layers_rust {
			enabled_layers.push(l.as_ptr() as *const u8);
		}
		let mut enabled_extensions = Vec::<*const u8>::with_capacity(enabled_extensions_rust.len());
		for e in &enabled_extensions_rust {
			enabled_extensions.push(e.as_ptr() as *const u8);
		}
		let mut queue_priorities = Vec::<Vec<f32>>::new();
		let mut queue_create_infos = Vec::<vkraw::VkDeviceQueueCreateInfo>::new();

		for i in &self.queue_create_infos {
			queue_priorities.push(i.1.clone());
			queue_create_infos.push(vkraw::VkDeviceQueueCreateInfo {
				sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
				pNext: ptr::null(),
				flags: vkraw::VkDeviceQueueCreateFlags::_EMPTY,
				queueFamilyIndex: i.0,
				queueCount: i.1.len() as u32,
				pQueuePriorities: queue_priorities.last().unwrap().as_ptr() as *const f32
			});
		}
		let features = vkraw::VkPhysicalDeviceFeatures {
			robustBufferAccess: 0,
			fullDrawIndexUint32: 0,
			imageCubeArray: 0,
			independentBlend: 0,
			geometryShader: 0,
			tessellationShader: 0,
			sampleRateShading: 0,
			dualSrcBlend: 0,
			logicOp: 0,
			multiDrawIndirect: 0,
			drawIndirectFirstInstance: 0,
			depthClamp: 0,
			depthBiasClamp: 0,
			fillModeNonSolid: 0,
			depthBounds: 0,
			wideLines: 0,
			largePoints: 0,
			alphaToOne: 0,
			multiViewport: 0,
			samplerAnisotropy: 0,
			textureCompressionETC2: 0,
			textureCompressionASTC_LDR: 0,
			textureCompressionBC: 0,
			occlusionQueryPrecise: 0,
			pipelineStatisticsQuery: 0,
			vertexPipelineStoresAndAtomics: 0,
			fragmentStoresAndAtomics: 0,
			shaderTessellationAndGeometryPointSize: 0,
			shaderImageGatherExtended: 0,
			shaderStorageImageExtendedFormats: 1,
			shaderStorageImageMultisample: 0,
			shaderStorageImageReadWithoutFormat: 0,
			shaderStorageImageWriteWithoutFormat: 0,
			shaderUniformBufferArrayDynamicIndexing: 0,
			shaderSampledImageArrayDynamicIndexing: 0,
			shaderStorageBufferArrayDynamicIndexing: 0,
			shaderStorageImageArrayDynamicIndexing: 0,
			shaderClipDistance: 0,
			shaderCullDistance: 0,
			shaderFloat64: 0,
			shaderInt64: 0,
			shaderInt16: 0,
			shaderResourceResidency: 0,
			shaderResourceMinLod: 0,
			sparseBinding: 0,
			sparseResidencyBuffer: 0,
			sparseResidencyImage2D: 0,
			sparseResidencyImage3D: 0,
			sparseResidency2Samples: 0,
			sparseResidency4Samples: 0,
			sparseResidency8Samples: 0,
			sparseResidency16Samples: 0,
			sparseResidencyAliased: 0,
			variableMultisampleRate: 0,
			inheritedQueries: 0,
		};
		let extra_features = vkraw::VkPhysicalDeviceFloat16Int8FeaturesKHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FLOAT16_INT8_FEATURES_KHR,
			pNext: ptr::null_mut(),
			shaderFloat16: 1,
			shaderInt8: 1
		};

		let device_create_info = vkraw::VkDeviceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
			pNext: if true { unsafe { mem::transmute(&extra_features) } } else { ptr::null_mut() },
			flags: 0,
			queueCreateInfoCount: queue_create_infos.len() as u32,
			pQueueCreateInfos: queue_create_infos.as_ptr(),
			enabledLayerCount: enabled_layers.len() as u32,
			ppEnabledLayerNames: enabled_layers.as_ptr(),
			enabledExtensionCount: enabled_extensions.len() as u32,
			ppEnabledExtensionNames: enabled_extensions.as_ptr(),
			pEnabledFeatures: &features
		};

		println!("vkCreateDevice");
		let res;
		unsafe {
			res = vkraw::vkCreateDevice(self.physical_device.as_ref().expect("No physical device").0.physical_device, &device_create_info, ptr::null(), &mut device);
		};

		if res == vkraw::VkResult::VK_SUCCESS {

			assert!(device != vkraw::VK_NULL_HANDLE);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			Ok(Device { device: device, instance: self.instance })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Device<'a> {
	fn drop(&mut self) {
		assert!(self.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyDevice");
			vkraw::vkDestroyDevice(self.device, ptr::null());
		}
	}
}

impl<'a> Device<'a> {
	pub fn get_queue(&self, queue_family_index: u32, queue_index: u32) -> Option<Queue> {
		let mut queue;
		unsafe {
			queue = std::mem::uninitialized();
			println!("vkGetDeviceQueue");
			vkraw::vkGetDeviceQueue(self.device, queue_family_index, queue_index, &mut queue);
		}
		if queue != vkraw::VK_NULL_HANDLE {
			Some(Queue { device: &self, queue: queue })
		} else {
			None
		}
	}
	pub fn create_buffer(&self, size: usize, flags: vkraw::VkBufferUsageFlags) -> Result<Buffer, vkraw::VkResult> {
		let buf_create_info = vkraw::VkBufferCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkBufferCreateFlags::_EMPTY,
			size: size as u64,
			usage: flags,
			sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
			queueFamilyIndexCount: 0,
			pQueueFamilyIndices: ptr::null()
		};

		let mut buffer: vkraw::VkBuffer = 0;
		let res;
		println!("vkCreateBuffer");
		unsafe {
			res = vkraw::vkCreateBuffer(self.device, &buf_create_info, ptr::null(), &mut buffer);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Buffer { buffer: buffer, device: &self })
		} else {
			Err(res)
		}
	}

	pub fn create_command_pool(&self) -> Result<CommandPool, vkraw::VkResult> {
		// Create command pool
		println!("Creating command pool");
		let mut command_pool: vkraw::VkCommandPool = 0;
		let pool_create_info = vkraw::VkCommandPoolCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkCommandPoolCreateFlags::VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
			queueFamilyIndex: 0
		};
		let res;
		unsafe {
			res = vkraw::vkCreateCommandPool(self.device, &pool_create_info, ptr::null(), &mut command_pool);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(CommandPool { device: &self, command_pool: command_pool })
		} else {
			Err(res)
		}
	}

	pub fn create_semaphore(&self) -> Result<Semaphore, vkraw::VkResult> {
		let sem_create_info = vkraw::VkSemaphoreCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0
		};
		println!("vkCreateSemaphore");
		let res;
		let mut sem: vkraw::VkSemaphore = 0;
		unsafe {
			res = vkraw::vkCreateSemaphore(self.device, &sem_create_info, ptr::null(), &mut sem);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Semaphore { semaphore: sem, device: &self })
		} else {
			Err(res)
		}
	}

	pub fn create_fence(&self) -> Result<Fence, vkraw::VkResult> {
		let fence_create_info = vkraw::VkFenceCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkFenceCreateFlags::_EMPTY
		};
		println!("vkCreateFence");
		let res;
		let mut fence: vkraw::VkFence = 0;
		unsafe {
			res = vkraw::vkCreateFence(self.device, &fence_create_info, ptr::null(), &mut fence);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Fence { fence: fence, device: &self })
		} else {
			Err(res)
		}
	}

	pub fn load_spirv_shader_from_disk(&self, filename: &str) -> Result<ShaderModule, vkraw::VkResult> {

		let mut buffer = Vec::<u8>::new();

		// Load file contents in to buffer
		if let Ok(mut f) = std::fs::File::open(filename) {
			println!("Loaded {}", filename);
			f.read_to_end(&mut buffer).unwrap();
		} else if let Ok(mut f) = std::fs::File::open("examples/".to_owned() + filename) {
			println!("Loaded examples/{}", filename);
			f.read_to_end(&mut buffer).unwrap();
		} else {
			println!("Could not load file {}", filename);
			return Err(vkraw::VkResult::VK_RESULT_MAX_ENUM)
		}
		
		/*let dis = match rspirv::mr::load_bytes(buffer.clone()) {
			Ok(m) => m.disassemble(),
			Err(err) => format!("{}", err),
		};
		print!("{}: {}", filename, dis);*/
		
		/*println!("a {}", buffer.len());
		unsafe {
		let view = &buffer as *const _ as *const u8;
		for i in 0 .. buffer.len() {
			print!("{:02x} ", unsafe {*view.offset(i as isize)});
		}
		}
		print!("\n");*/

		let mut shader_mod: vkraw::VkShaderModule = 0;

		let mod_create_info = vkraw::VkShaderModuleCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			codeSize: buffer.len() as u64,
			pCode: buffer.as_mut_ptr() as *mut u32
		};

		let res;
		unsafe {
			res = vkraw::vkCreateShaderModule(self.device, &mod_create_info, ptr::null(), &mut shader_mod);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(ShaderModule { device: &self, module: shader_mod })
		} else {
			Err(res)
		}
	}

	pub fn load_spirv_shader_from_buffer(&self, buffer: &Vec<u32>) -> Result<ShaderModule, vkraw::VkResult> {

		let mut shader_mod: vkraw::VkShaderModule = 0;
		
		/*println!("a {}", buffer.len() * 4);
		unsafe {
		let view = &buffer as *const _ as *const u8;
		for i in 0 .. buffer.len() * 4 {
			print!("{:02x} ", unsafe {*view.offset(i as isize)});
		}
		}
		print!("\n");*/

		/*let dis = match rspirv::mr::load_words(buffer.clone()) {
			Ok(m) => m.disassemble(),
			Err(err) => format!("{}", err),
		};
		print!("buf: {}", dis);*/

		let mod_create_info = vkraw::VkShaderModuleCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			codeSize: (buffer.len() * 4) as u64,
			pCode: buffer.as_ptr() as *const u32
		};

		let res;
		unsafe {
			res = vkraw::vkCreateShaderModule(self.device, &mod_create_info, ptr::null(), &mut shader_mod);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(ShaderModule { device: &self, module: shader_mod })
		} else {
			Err(res)
		}
	}

	pub fn create_descriptor_pool(&self, max_sets: usize, pool_sizes: Vec<(usize, vkraw::VkDescriptorType)>) -> Result<DescriptorPool, vkraw::VkResult> {
		let mut descriptor_pool: vkraw::VkDescriptorPool = 0;
		let pools: Vec<vkraw::VkDescriptorPoolSize> = pool_sizes.iter().map(|x| vkraw::VkDescriptorPoolSize { _type: x.1, descriptorCount: x.0 as u32 }).collect();
		let pool_create_info = vkraw::VkDescriptorPoolCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkDescriptorPoolCreateFlags::_EMPTY,
			maxSets: max_sets as u32,
			poolSizeCount: pools.len() as u32,
			pPoolSizes: pools.as_ptr()
		};
		let res;
		unsafe {
			res = vkraw::vkCreateDescriptorPool(self.device, &pool_create_info, ptr::null(), &mut descriptor_pool);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(DescriptorPool { device: &self, descriptor_pool: descriptor_pool })
		} else {
			Err(res)
		}
	}

	pub fn create_sampler(&self) -> Result<Sampler, vkraw::VkResult> {

		// TODO
		let mut sampler: vkraw::VkSampler = 0;
		let sampler_create_info = vkraw::VkSamplerCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkSamplerCreateFlags::_EMPTY,
			magFilter: vkraw::VkFilter::VK_FILTER_NEAREST,
			minFilter: vkraw::VkFilter::VK_FILTER_NEAREST,
			mipmapMode: vkraw::VkSamplerMipmapMode::VK_SAMPLER_MIPMAP_MODE_NEAREST,
			addressModeU: vkraw::VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
			addressModeV: vkraw::VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
			addressModeW: vkraw::VkSamplerAddressMode::VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
			mipLodBias: 0.0,
			anisotropyEnable: false as u32,
			maxAnisotropy: 0.0,
			compareEnable: false as u32,
			compareOp: vkraw::VkCompareOp::VK_COMPARE_OP_NEVER,
			minLod: 0.0,
			maxLod: 0.0,
			borderColor: vkraw::VkBorderColor::VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK,
			unnormalizedCoordinates: false as u32
		};
		let res;
		unsafe {
			res = vkraw::vkCreateSampler(self.device, &sampler_create_info, ptr::null(), &mut sampler);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Sampler { device: &self, sampler: sampler })
		} else {
			Err(res)
		}
	}
	
	pub fn wait_idle(&self) {
		unsafe {
			vkraw::vkDeviceWaitIdle(self.device);
		}
	}

	pub fn load_ktx_texture_to_cpu_mem<'y>(&'a self, filename: String) -> Option<(KtxFile)> {

		let f = std::fs::File::open(&filename);
		if f.is_ok() {
			match KtxFile::deserialize(&mut f.unwrap()) {
				Ok(ktx_file) => {
					println!("ktx file\n{:?}", ktx_file);
					return Some(ktx_file)
				}
				Err(_e) => return None
			}
		}
		None
	}

	pub fn load_ktx_texture_create_img<'y>(&'a self, ktx_file: &KtxFile) -> Option<(Image<'a>)> {

		println!("Creating GPU image");
		let image = {
			let mut ib = ImageBuilder::new(&self);
			ib.extent.width = ktx_file.header.pixel_width as u32;
			ib.extent.height = ktx_file.header.pixel_height as u32;
			ib.format = ktx_file.header.get_vk_format().unwrap();
			ib.tiling = vkraw::VkImageTiling::VK_IMAGE_TILING_OPTIMAL;
			ib.usage = vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_TRANSFER_DST_BIT | vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_STORAGE_BIT;
			ib.create()
		};

		if image.is_ok() {
			Some(image.unwrap())
		} else {
			None
		}
	}

	pub fn load_ktx_texture_create_mem<'y>(&'a self, mem: &'y MemoryAllocator<'a>, image: &Image<'a>, gpu_only_memory_index: usize) -> Option<(Mem<'y>)> {

		let mem = mem.allocate_image_memory(&image, gpu_only_memory_index);

		if mem.is_ok() {
			Some(mem.unwrap())
		} else {
			None
		}
	}

	pub fn load_ktx_texture_upload_blocking<'y>(&'a self, ktx_file: &KtxFile, image: &Image<'a>, mem: &'y MemoryAllocator<'a>, staging_memory_index: usize, queue: &Queue, command_pool: &CommandPool) -> bool {

		let img_size = ktx_file.header.image_size_max();

		println!("Creating memory");
		let staging_buffer = self.create_buffer(img_size, vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_TRANSFER_SRC_BIT).unwrap();
		let staging_buffer_size = mem.get_buffer_memory_size_req(&staging_buffer);
		let mut staging_buffer_mem = mem.allocate_buffer_memory(&staging_buffer, staging_memory_index).unwrap();
		assert!(staging_buffer_size >= img_size as u64);

		println!("Coyping {}", img_size);
		{
			let mut mapped = staging_buffer_mem.map_raw(img_size);
			unsafe {
				libc::memcpy(mapped.get_ptr() as *mut libc::c_void, ktx_file.data.as_ref().unwrap()[0].data.as_ptr() as *mut libc::c_void, img_size as libc::size_t);
			}
		}
		let mut buffer_copy_regions = Vec::<vkraw::VkBufferImageCopy>::new();

		assert!(ktx_file.header.number_of_mipmap_levels == 1);
		//for mip in mips { // TODO
		{
			buffer_copy_regions.push(vkraw::VkBufferImageCopy {
				bufferOffset: 0,
				bufferRowLength: 0,
				bufferImageHeight: 0,
				imageSubresource: vkraw::VkImageSubresourceLayers {
					aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
					mipLevel: 0,
					baseArrayLayer: 0,
					layerCount: 1,
				},
				imageOffset: vkraw::VkOffset3D { x: 0, y: 0, z: 0 },
				imageExtent: vkraw::VkExtent3D { width: ktx_file.header.pixel_width as u32, height: ktx_file.header.pixel_height as u32, depth: 1 },
			});
		}
		{
			println!("Creating cmdb");
			let mut upload_cmdbs = command_pool.create_command_buffers(1).unwrap();
			let upload_cmdb = &mut upload_cmdbs[0];
			upload_cmdb
				.begin().unwrap()

				.pipeline_barrier(vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_HOST_BIT, vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_TRANSFER_BIT, vkraw::VkDependencyFlagBits::_EMPTY,
					vec![], vec![], vec![vkraw::VkImageMemoryBarrier {
						sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
						pNext: ptr::null(),
						srcAccessMask: vkraw::VkAccessFlags::_EMPTY,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_TRANSFER_WRITE_BIT,
						oldLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						newLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
						srcQueueFamilyIndex: 0,
						dstQueueFamilyIndex: 0,
						image: image.image,
						subresourceRange: vkraw::VkImageSubresourceRange {
							aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
							baseMipLevel: 0,
							levelCount: 1,
							baseArrayLayer: 0,
							layerCount: 1
						}
					}])
				.copy_buffer_to_image(&staging_buffer, &image, vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, buffer_copy_regions)
				.pipeline_barrier(vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_TRANSFER_BIT, vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, vkraw::VkDependencyFlagBits::_EMPTY,
					vec![], vec![], vec![vkraw::VkImageMemoryBarrier {
						sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
						pNext: ptr::null(),
						srcAccessMask: vkraw::VkAccessFlags::VK_ACCESS_TRANSFER_WRITE_BIT,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_SHADER_READ_BIT,
						oldLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
						newLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_GENERAL,
						srcQueueFamilyIndex: 0,
						dstQueueFamilyIndex: 0,
						image: image.image,
						subresourceRange: vkraw::VkImageSubresourceRange {
							aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
							baseMipLevel: 0,
							levelCount: 1,
							baseArrayLayer: 0,
							layerCount: 1
						}
					}])
				.end_command_buffer();

			println!("Submitting cmdb");
			let mut temp_fence = self.create_fence().unwrap();
			queue.submit(vec![&upload_cmdb], Some(&temp_fence), vec![], vec![]);
			println!("Waiting for results");
			temp_fence.wait(None).unwrap();
			println!("Done");
		}

		true
	}

	pub fn load_ktx_texture_create_image_view<'y>(&'a self, ktx_file: &KtxFile, image: &'a Image<'a>) -> Option<ImageView<'a>> {

		let image_view = ImageViewBuilder::new(&image, ktx_file.header.get_vk_format().unwrap()).create();

		if image_view.is_ok() {
			Some(image_view.unwrap())
		} else {
			None
		}
	}

/*	pub fn load_ktx_texture_to_gpu<'y>(&'a self, mem: &'y mut MemoryAllocator<'a>, filename: String, staging_memory_index: usize, gpu_only_memory_index: usize, queue: &Queue, command_pool: &CommandPool) -> Option<(Image<'a>, ImageView<'a>, Mem<'y>, KtxFile)> {

		let mut f = std::fs::File::open(&filename).unwrap();
		let ktx_file = KtxFile::deserialize(&mut f).unwrap();
		println!("ktx file\n{:?}", ktx_file);

		let input_size = (ktx_file.header.pixel_width, ktx_file.header.pixel_height);
		println!("{}x{}", input_size.0, input_size.1);
		let img_size = ktx_file.header.image_size_max();

		println!("Creating memory");
		let staging_buffer = self.create_buffer(img_size, vkraw::VkBufferUsageFlags::VK_BUFFER_USAGE_TRANSFER_SRC_BIT).unwrap();
		let staging_buffer_size = mem.get_buffer_memory_size_req(&staging_buffer);
		let mut staging_buffer_mem = mem.allocate_buffer_memory(&staging_buffer, staging_memory_index).unwrap();
		assert!(staging_buffer_size >= img_size as u64);

		println!("Coyping {}", img_size);
		{
			let mut mapped = staging_buffer_mem.map_raw(img_size);
			unsafe {
				libc::memcpy(mapped.get_ptr() as *mut libc::c_void, ktx_file.data.as_ref().unwrap()[0].data.as_ptr() as *mut libc::c_void, img_size as libc::size_t);
			}
		}

		let mut buffer_copy_regions = Vec::<vkraw::VkBufferImageCopy>::new();
		
		//for mip in mips { // TODO
		{
			buffer_copy_regions.push(vkraw::VkBufferImageCopy {
				bufferOffset: 0,
				bufferRowLength: 0,
				bufferImageHeight: 0,
				imageSubresource: vkraw::VkImageSubresourceLayers {
					aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
					mipLevel: 0,
					baseArrayLayer: 0,
					layerCount: 1,
				},
				imageOffset: vkraw::VkOffset3D { x: 0, y: 0, z: 0 },
				imageExtent: vkraw::VkExtent3D { width: input_size.0 as u32, height: input_size.1 as u32, depth: 1 },
			});
		}

		println!("Creating GPU image");
		let cs_input_image = {
			let mut ib = ImageBuilder::new(&self);
			ib.extent.width = input_size.0 as u32;
			ib.extent.height = input_size.1 as u32;
			ib.format = ktx_file.header.get_vk_format().unwrap();
			ib.tiling = vkraw::VkImageTiling::VK_IMAGE_TILING_OPTIMAL;
			ib.usage = vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_TRANSFER_DST_BIT | vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_STORAGE_BIT;
			ib.create().unwrap()
		};

		let cs_image_mem = mem.allocate_image_memory(&cs_input_image, gpu_only_memory_index).unwrap();
		//let cs_image_view = ImageViewBuilder::new(&cs_input_image, ktx_file.header.get_vk_format().unwrap()).create().unwrap();
		let cs_image_view = ImageView {
			image_view: 0,
			image: &cs_input_image
		};

		{
			println!("Creating cmdb");
			let mut upload_cmdbs = command_pool.create_command_buffers(1).unwrap();
			let upload_cmdb = &mut upload_cmdbs[0];
			upload_cmdb
				.begin().unwrap()

				.pipeline_barrier(vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_HOST_BIT, vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_TRANSFER_BIT, vkraw::VkDependencyFlagBits::_EMPTY,
					vec![], vec![], vec![vkraw::VkImageMemoryBarrier {
						sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
						pNext: ptr::null(),
						srcAccessMask: vkraw::VkAccessFlags::_EMPTY,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_TRANSFER_WRITE_BIT,
						oldLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
						newLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
						srcQueueFamilyIndex: 0,
						dstQueueFamilyIndex: 0,
						image: cs_input_image.image,
						subresourceRange: vkraw::VkImageSubresourceRange {
							aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
							baseMipLevel: 0,
							levelCount: 1,
							baseArrayLayer: 0,
							layerCount: 1
						}
					}])
				.copy_buffer_to_image(&staging_buffer, &cs_input_image, vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, buffer_copy_regions)
				.pipeline_barrier(vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_TRANSFER_BIT, vkraw::VkPipelineStageFlagBits::VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, vkraw::VkDependencyFlagBits::_EMPTY,
					vec![], vec![], vec![vkraw::VkImageMemoryBarrier {
						sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
						pNext: ptr::null(),
						srcAccessMask: vkraw::VkAccessFlags::VK_ACCESS_TRANSFER_WRITE_BIT,
						dstAccessMask: vkraw::VkAccessFlags::VK_ACCESS_SHADER_READ_BIT,
						oldLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
						newLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_GENERAL,
						srcQueueFamilyIndex: 0,
						dstQueueFamilyIndex: 0,
						image: cs_input_image.image,
						subresourceRange: vkraw::VkImageSubresourceRange {
							aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
							baseMipLevel: 0,
							levelCount: 1,
							baseArrayLayer: 0,
							layerCount: 1
						}
					}])
				.end_command_buffer();

			println!("Submitting cmdb");
			let mut temp_fence = self.create_fence().unwrap();
			queue.submit(vec![&upload_cmdb], Some(&temp_fence), vec![], vec![]);
			println!("Waiting for results");
			temp_fence.wait(None).unwrap();
			println!("Done");
		}

		Some((cs_input_image, cs_image_view, cs_image_mem, ktx_file))
	}*/
}

impl<'a> Queue<'a> {
	pub fn submit(&self, command_buffers: Vec<&CommandBuffer>, fence: Option<&Fence>, waits: Vec<(&Semaphore, vkraw::VkPipelineStageFlags)>, signals: Vec<&Semaphore>) -> vkraw::VkResult {

		let raw_wait_semaphores: Vec<vkraw::VkSemaphore> = waits.iter().map(|x| x.0.semaphore).collect();
		let raw_wait_masks: Vec<vkraw::VkPipelineStageFlags> = waits.iter().map(|x| x.1).collect();
		let raw_signal_semaphores: Vec<vkraw::VkSemaphore> = signals.iter().map(|x| x.semaphore).collect();
		let raw_cmdbs: Vec<vkraw::VkCommandBuffer> = command_buffers.iter().map(|x| x.command_buffer).collect();

		let submit_info = vkraw::VkSubmitInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SUBMIT_INFO,
			pNext: ptr::null(),
			waitSemaphoreCount: raw_wait_semaphores.len() as u32,
			pWaitSemaphores: raw_wait_semaphores.as_ptr(),
			pWaitDstStageMask: raw_wait_masks.as_ptr(),
			commandBufferCount: raw_cmdbs.len() as u32,
			pCommandBuffers: raw_cmdbs.as_ptr(),
			signalSemaphoreCount: raw_signal_semaphores.len() as u32,
			pSignalSemaphores: raw_signal_semaphores.as_ptr()
		};

		unsafe {
			vkraw::vkQueueSubmit(self.queue, 1, &submit_info, if fence.is_some() { fence.unwrap().fence } else { vkraw::VK_NULL_HANDLE })
		}		
	}
	pub fn present(&self, swaps: Vec<(&Semaphore, &Swapchain, u32)>) -> Result<Vec<vkraw::VkResult>, vkraw::VkResult> {
	
		let raw_wait_semaphores: Vec<vkraw::VkSemaphore> = swaps.iter().map(|x| x.0.semaphore).collect();
		let raw_swapchains: Vec<vkraw::VkSwapchainKHR> = swaps.iter().map(|x| x.1.swapchain).collect();
		let image_indices: Vec<u32> = swaps.iter().map(|x| x.2).collect();
		
		let mut results = Vec::<vkraw::VkResult>::with_capacity(swaps.len());
		
		let present_info = vkraw::VkPresentInfoKHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
			pNext: ptr::null(),
			waitSemaphoreCount: raw_wait_semaphores.len() as u32,
			pWaitSemaphores: raw_wait_semaphores.as_ptr(),
			swapchainCount: raw_swapchains.len() as u32,
			pSwapchains: raw_swapchains.as_ptr(),
			pImageIndices: image_indices.as_ptr(),
			pResults: results.as_mut_ptr()
		};
		assert!(self.device.instance.vk.QueuePresentKHR.is_some());
		let res = self.device.instance.vk.QueuePresentKHR.unwrap()(self.queue, &present_info);
		
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(results)
		} else {
			Err(res)
		}
	}
	pub fn acquire(&self, swapchain: &Swapchain, timeout: u64, semaphore: Option<&Semaphore>, fence: Option<Fence>) -> Result<u32, vkraw::VkResult> {
	
		let mut image_index = 0;
		assert!(self.device.instance.vk.AcquireNextImageKHR.is_some());
		let res = self.device.instance.vk.AcquireNextImageKHR.unwrap()(self.device.device, swapchain.swapchain, timeout, if semaphore.is_some() { semaphore.unwrap().semaphore } else { vkraw::VK_NULL_HANDLE }, if fence.is_some() { fence.unwrap().fence } else { vkraw::VK_NULL_HANDLE }, &mut image_index);
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(image_index)
		} else {
			Err(res)
		}
	}
}

pub struct ImageBuilder<'a> {
	pub device: &'a Device<'a>,
	pub image_type: vkraw::VkImageType,
	pub format: vkraw::VkFormat,
	pub extent: vkraw::VkExtent3D,
	pub mip_levels: usize,
	pub array_layers: usize,
	pub samples: vkraw::VkSampleCountFlagBits,
	pub tiling: vkraw::VkImageTiling,
	pub usage: vkraw::VkImageUsageFlags,
	pub initial_layout: vkraw::VkImageLayout
}

impl<'a> ImageBuilder<'a> {
	pub fn new(device: &'a Device) -> Self {
		ImageBuilder {
			device: device,
			image_type: vkraw::VkImageType::VK_IMAGE_TYPE_2D,
			format: vkraw::VkFormat::VK_FORMAT_B8G8R8A8_UNORM,
			extent: vkraw::VkExtent3D { width: 0, height: 0, depth: 1 },
			mip_levels: 1,
			array_layers: 1,
			samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
			tiling: vkraw::VkImageTiling::VK_IMAGE_TILING_OPTIMAL,
			usage: vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
			initial_layout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED
		}
	}
	pub fn create(&self) -> Result<Image<'a>, vkraw::VkResult> {
		let image_create_info = vkraw::VkImageCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkImageCreateFlags::empty(),
			imageType: self.image_type,
			format: self.format,
			extent: self.extent,
			mipLevels: self.mip_levels as u32,
			arrayLayers: self.array_layers as u32,
			samples: self.samples,
			tiling: self.tiling,
			usage: self.usage,
			sharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
			queueFamilyIndexCount: 0,
			pQueueFamilyIndices: ptr::null(),
			initialLayout: self.initial_layout
		};
		let res;
		let mut image: vkraw::VkImage;
		unsafe {
			image = std::mem::uninitialized();
			println!("vkCreateImage");
			res = vkraw::vkCreateImage(self.device.device, &image_create_info, ptr::null(), &mut image);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Image { device: self.device, image: image, swapchain_image: false })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Surface<'a> {
	fn drop(&mut self) {
		assert!(self.instance.instance != vkraw::VK_NULL_HANDLE);
		println!("vk.DestroySurfaceKHR");
		assert!(self.instance.vk.DestroySurfaceKHR.is_some());
		self.instance.vk.DestroySurfaceKHR.unwrap()(self.instance.instance, self.surface, ptr::null());
	}
}

impl<'a> Drop for Buffer<'a> {
	fn drop(&mut self) {
		println!("vkDestroyBuffer");
		unsafe {
			vkraw::vkDestroyBuffer(self.device.device, self.buffer, ptr::null());
		}
	}
}
pub struct SwapchainBuilder<'a> {
	pub device: &'a Device<'a>,
	pub surface: &'a Surface<'a>,
	pub height: u32,
	pub width: u32,
	pub num_swapchain_images: u32,
	pub colour_format: vkraw::VkFormat,
	pub colour_space: vkraw::VkColorSpaceKHR,
	pub swapchain_transform: vkraw::VkSurfaceTransformFlagsKHR,
	pub composite_alpha: vkraw::VkCompositeAlphaFlagBitsKHR,
	pub present_mode: vkraw::VkPresentModeKHR
}

impl<'a> SwapchainBuilder<'a> {
	pub fn new(device: &'a Device, surface: &'a Surface) -> SwapchainBuilder<'a> {
		SwapchainBuilder {
			device: &device,
			surface: &surface,
			height: 0,
			width: 0,
			num_swapchain_images: 0,
			colour_format: vkraw::VkFormat::VK_FORMAT_B8G8R8A8_UNORM,
			colour_space: vkraw::VkColorSpaceKHR::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
			swapchain_transform: vkraw::VkSurfaceTransformFlagsKHR::VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
			composite_alpha: vkraw::VkCompositeAlphaFlagBitsKHR::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
			present_mode: vkraw::VkPresentModeKHR::VK_PRESENT_MODE_FIFO_KHR
		}
	}
	pub fn create(&self, old_swapchain: &Option<Swapchain>) -> Result<Swapchain<'a>, vkraw::VkResult> {

		let swapchain_create_info = vkraw::VkSwapchainCreateInfoKHR {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
			pNext: ptr::null(),
			flags: vkraw::VkSwapchainCreateFlagBitsKHR::_EMPTY,
			surface: self.surface.surface,
			minImageCount: self.num_swapchain_images,
			imageFormat: self.colour_format,
			imageColorSpace: self.colour_space,
			imageExtent: vkraw::VkExtent2D{ width: self.width, height: self.height },
			imageArrayLayers: 1,
			imageUsage: vkraw::VkImageUsageFlags::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
			imageSharingMode: vkraw::VkSharingMode::VK_SHARING_MODE_EXCLUSIVE,
			queueFamilyIndexCount: 0,
			pQueueFamilyIndices: ptr::null(),
			preTransform: self.swapchain_transform,
			compositeAlpha: self.composite_alpha,
			presentMode: self.present_mode,
			clipped: vkraw::VK_TRUE,
			oldSwapchain: if old_swapchain.is_some() { old_swapchain.as_ref().unwrap().swapchain } else { vkraw::VK_NULL_HANDLE }
		};

		let mut swapchain: vkraw::VkSwapchainKHR = 0;
		let res;
		{
			assert!(self.device.instance.vk.CreateSwapchainKHR.is_some());
			res = self.device.instance.vk.CreateSwapchainKHR.unwrap()(self.device.device, &swapchain_create_info, ptr::null(), &mut swapchain);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Swapchain { device: &self.device, swapchain: swapchain })
		} else {
			Err(res)
		}
	}
}

impl<'a> Swapchain<'a> {
	pub fn get_swapchain_images(&self) -> Vec<Image<'a>> {

		let mut swapchain_image_count = 0;
		assert!(self.device.instance.vk.GetSwapchainImagesKHR.is_some());
		self.device.instance.vk.GetSwapchainImagesKHR.unwrap()(self.device.device, self.swapchain, &mut swapchain_image_count, ptr::null_mut());
		assert!(swapchain_image_count > 0);
		println!("Creating {} swapchain images", swapchain_image_count);
		let mut swapchain_images = Vec::<vkraw::VkImage>::with_capacity(swapchain_image_count as usize);
		unsafe {
			swapchain_images.set_len(swapchain_image_count as usize);
		}
		assert!(self.device.instance.vk.GetSwapchainImagesKHR.is_some());
		self.device.instance.vk.GetSwapchainImagesKHR.unwrap()(self.device.device, self.swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr());

		swapchain_images.iter().map(|x| Image { device: &self.device, image: *x, swapchain_image: true }).collect()
	}
}

pub struct ImageViewBuilder<'a> {
	pub image: &'a Image<'a>,
	pub view_type: vkraw::VkImageViewType,
	pub format: vkraw::VkFormat,
	pub components: vkraw::VkComponentMapping,
	pub subresource_range: vkraw::VkImageSubresourceRange
}

impl<'a> ImageViewBuilder<'a> {
	pub fn new(image: &'a Image, format: vkraw::VkFormat) -> Self {
		ImageViewBuilder {
			image: image,
			view_type: vkraw::VkImageViewType::VK_IMAGE_VIEW_TYPE_2D,
			format: format,
			components: vkraw::VkComponentMapping {
				r: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_R,
				g: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_G,
				b: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_B,
				a: vkraw::VkComponentSwizzle::VK_COMPONENT_SWIZZLE_A
			},
			subresource_range: vkraw::VkImageSubresourceRange {
				aspectMask: vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_COLOR_BIT,
				baseMipLevel: 0,
				levelCount: 1,
				baseArrayLayer: 0,
				layerCount: 1
			}
		}
	}

	pub fn depth_view<'y>(&'y mut self) -> &'y mut Self {
		self.subresource_range.aspectMask = vkraw::VkImageAspectFlags::VK_IMAGE_ASPECT_DEPTH_BIT;
		self
	}

	pub fn create(&self) -> Result<ImageView<'a>, vkraw::VkResult> {

		let mut image_view: vkraw::VkImageView;
		let img_create_info = vkraw::VkImageViewCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkImageViewCreateFlagBits::_EMPTY,
			image: self.image.image,
			viewType: self.view_type,
			format: self.format,
			components: self.components,
			subresourceRange: self.subresource_range
		};
		let res;
		unsafe{
			image_view = std::mem::uninitialized();
			println!("vkCreateImageView");
			res = vkraw::vkCreateImageView(self.image.device.device, &img_create_info, ptr::null(), &mut image_view);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(ImageView { image: &self.image, image_view: image_view })
		} else {
			Err(res)
		}
	}

}

impl<'a> Drop for Swapchain<'a> {
	fn drop(&mut self) {
		unsafe { vkraw::vkDeviceWaitIdle(self.device.device); }
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		println!("DestroySwapchainKHR");
		self.device.instance.vk.DestroySwapchainKHR.unwrap()(self.device.device, self.swapchain, ptr::null());
	}
}

impl<'a> Drop for Image<'a> {
	fn drop(&mut self) {

		// Don't destroy images from swapchains
		if !self.swapchain_image {
			assert!(self.device.device != vkraw::VK_NULL_HANDLE);
			println!("vkDestroyImage");
			unsafe {
				vkraw::vkDestroyImage(self.device.device, self.image, ptr::null());
			}
		}
	}
}

impl<'a> Drop for ImageView<'a> {
	fn drop(&mut self) {
		assert!(self.image.device.device != vkraw::VK_NULL_HANDLE);
		println!("vkDestroyImageView");
		unsafe {
			vkraw::vkDestroyImageView(self.image.device.device, self.image_view, ptr::null());
		}
	}
}

pub struct MemoryAllocator<'a> {
	device: &'a Device<'a>
}

pub fn staging_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a staging memory type")
}

pub fn gpu_only_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a gpu only memory type")
}

pub fn gpu_to_cpu_memory(memory_available: &(Vec<vkraw::VkMemoryType>, Vec<vkraw::VkMemoryHeap>)) -> usize {

	memory_available.0.iter().enumerate().filter_map(
		|(i, t)|
		{
			if t.propertyFlags.intersects(vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_CACHED_BIT | vkraw::VkMemoryPropertyFlags::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT) {
				Some(i)
			} else {
				None
			}
		}).next().expect("Couldn't find a gpu to cpu memory type")
}

impl<'a> MemoryAllocator<'a> {
	pub fn new(device: &'a Device) -> MemoryAllocator<'a> {
		MemoryAllocator {
			device: device
		}
	}
	pub fn get_buffer_memory_size_req(&self, buffer: &Buffer) -> u64 {

		let mut mem_reqs: vkraw::VkMemoryRequirements;
		unsafe {
			mem_reqs = std::mem::uninitialized();
			vkraw::vkGetBufferMemoryRequirements(self.device.device, buffer.buffer, &mut mem_reqs);
		}
		mem_reqs.size
	}
	pub fn allocate_buffer_memory(&self, buffer: &Buffer, memory_type_index: usize) -> Result<Mem, vkraw::VkResult> {

		let mem_alloc = vkraw::VkMemoryAllocateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
			pNext: ptr::null(),
			allocationSize: self.get_buffer_memory_size_req(buffer),
			memoryTypeIndex: memory_type_index as u32
		};
		let mut memory: vkraw::VkDeviceMemory = 0;
		let mut res;
		unsafe {
			res = vkraw::vkAllocateMemory(self.device.device, &mem_alloc, ptr::null(), &mut memory);
			assert!(res == vkraw::VkResult::VK_SUCCESS);

			if res != vkraw::VkResult::VK_SUCCESS {
				return Err(res)
			}

			// TODO: do this here?
			res = vkraw::vkBindBufferMemory(self.device.device, buffer.buffer, memory, 0);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Mem { memory_allocator: self, mem: memory, ptr: 0 })
		} else {
			Err(res)
		}
	}
	pub fn get_image_memory_size_req(&self, image: &Image) -> u64 {

		let mut mem_reqs: vkraw::VkMemoryRequirements;
		unsafe {
			mem_reqs = std::mem::uninitialized();
			vkraw::vkGetImageMemoryRequirements(self.device.device, image.image, &mut mem_reqs);
		}
		mem_reqs.size
	}
	pub fn allocate_image_memory(&self, image: &Image, memory_type_index: usize) -> Result<Mem, vkraw::VkResult> {

		let mem_alloc = vkraw::VkMemoryAllocateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
			pNext: ptr::null(),
			allocationSize: self.get_image_memory_size_req(image),
			memoryTypeIndex: memory_type_index as u32
		};
		let mut memory: vkraw::VkDeviceMemory = 0;
		let mut res;
		unsafe {
			res = vkraw::vkAllocateMemory(self.device.device, &mem_alloc, ptr::null(), &mut memory);
			assert!(res == vkraw::VkResult::VK_SUCCESS);

			if res != vkraw::VkResult::VK_SUCCESS {
				return Err(res)
			}

			// TODO: do this here?
			res = vkraw::vkBindImageMemory(self.device.device, image.image, memory, 0);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
		}

		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Mem { memory_allocator: self, mem: memory, ptr: 0 })
		} else {
			Err(res)
		}
	}
}

// The lifetime specifiers ('a) here say "Mem should live a shorter lifetime than Instance because we have a reference to it"
pub struct Mem<'a> {
	memory_allocator: &'a MemoryAllocator<'a>,
	mem: vkraw::VkDeviceMemory,
	ptr: u64
}
impl<'a> Mem<'a> {
	// &mut self means that we can only ever have one MappedMem instance (Cannot map() twice)
	pub fn map<T>(&mut self) -> MappedMem<T> {

		let mut data: *mut libc::c_void = ptr::null_mut();
		let res;
		unsafe {
			res = vkraw::vkMapMemory(self.memory_allocator.device.device, self.mem, 0, std::mem::size_of::<T>() as u64, 0, &mut data);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			assert!(data != ptr::null_mut());
		}
		self.ptr = data as u64;
		MappedMem { mem: self, ptr: &self.ptr, _phantom: std::marker::PhantomData }
	}
	pub fn map_raw(&mut self, size: usize) -> RawMappedMem {

		let mut data: *mut libc::c_void = ptr::null_mut();
		let res;
		unsafe {
			res = vkraw::vkMapMemory(self.memory_allocator.device.device, self.mem, 0, size as u64, 0, &mut data);
			assert!(res == vkraw::VkResult::VK_SUCCESS);
			assert!(data != ptr::null_mut());
		}
		self.ptr = data as u64;
		RawMappedMem { mem: self, ptr: &self.ptr }
	}
}
impl<'a> Drop for Mem<'a> {
	fn drop(&mut self) {
		unsafe {
			vkraw::vkFreeMemory(self.memory_allocator.device.device, self.mem, ptr::null());
		}
	}
}

pub struct MappedMem<'a, T> {
	mem: &'a Mem<'a>,
	ptr: &'a u64,
	_phantom: std::marker::PhantomData<T>
}
impl<'a, T> std::ops::Deref for MappedMem<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe { std::mem::transmute::<u64, &T>(*self.ptr) }
	}
}
impl<'a, T> std::ops::DerefMut for MappedMem<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { std::mem::transmute::<u64, &mut T>(*self.ptr) }
	}
}
impl<'a, T> MappedMem<'a, T> {
	pub fn get_ptr(&mut self) -> *mut T {
		unsafe { std::mem::transmute::<u64, *mut T>(*self.ptr) }
	}
}
impl<'a, T> Drop for MappedMem<'a, T> {
	fn drop(&mut self) {
		unsafe {
			vkraw::vkUnmapMemory(self.mem.memory_allocator.device.device, self.mem.mem);
		}
	}
}

pub struct RawMappedMem<'a> {
	mem: &'a Mem<'a>,
	ptr: &'a u64
}
impl<'a> RawMappedMem<'a> {
	pub fn get_ptr(&mut self) -> *mut u8 {
		unsafe { std::mem::transmute::<u64, *mut u8>(*self.ptr) }
	}
}
impl<'a> Drop for RawMappedMem<'a> {
	fn drop(&mut self) {
		unsafe {
			vkraw::vkUnmapMemory(self.mem.memory_allocator.device.device, self.mem.mem);
		}
	}
}

impl<'a> CommandPool<'a> {
	pub fn create_command_buffers(&self, num: usize) -> Result<Vec<CommandBuffer>, vkraw::VkResult> {
		// Create command buffers
		println!("Creating command buffers");
		let mut command_buffers = Vec::<vkraw::VkCommandBuffer>::with_capacity(num);
		let cmd_buf_create_info = vkraw::VkCommandBufferAllocateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
			pNext: ptr::null(),
			commandPool: self.command_pool,
			level: vkraw::VkCommandBufferLevel::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
			commandBufferCount: num as u32
		};
		let res;
		unsafe {
			command_buffers.set_len(num);
			res = vkraw::vkAllocateCommandBuffers(self.device.device, &cmd_buf_create_info, command_buffers.as_mut_ptr());
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(command_buffers.iter().map(|x| CommandBuffer{ command_pool: &self, command_buffer: *x }).collect())
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for CommandPool<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		println!("vkDestroyCommandPool");
		unsafe {
			vkraw::vkDestroyCommandPool(self.device.device, self.command_pool, ptr::null());
		}
	}
}

impl<'a> Drop for CommandBuffer<'a> {
	fn drop(&mut self) {
		assert!(self.command_pool.device.device != vkraw::VK_NULL_HANDLE);
		println!("vkFreeCommandBuffers");
		unsafe {
			vkraw::vkFreeCommandBuffers(self.command_pool.device.device, self.command_pool.command_pool, 1, &self.command_buffer);
		}
	}
}

pub struct RenderPassBuilder<'a> {
	pub device: &'a Device<'a>,
	pub subpasses: Vec<vkraw::VkSubpassDescription>,
	pub dependencies: Vec<vkraw::VkSubpassDependency>,
	pub attachments: Vec<vkraw::VkAttachmentDescription>,
	pub attachment_references: Vec<vkraw::VkAttachmentReference>
}

impl<'a> RenderPassBuilder<'a> {
	pub fn new(device: &'a Device) -> RenderPassBuilder<'a> {
		RenderPassBuilder {
			device: device,
			subpasses: Vec::<vkraw::VkSubpassDescription>::new(),
			dependencies: Vec::<vkraw::VkSubpassDependency>::new(),
			attachments: Vec::<vkraw::VkAttachmentDescription>::new(),
			attachment_references: Vec::<vkraw::VkAttachmentReference>::new()
		}
	}

	pub fn default_single_colour_depth<'y>(&'y mut self, colour_format: vkraw::VkFormat, depth_format: vkraw::VkFormat) -> &'y mut Self {

		self.attachments = vec![
			vkraw::VkAttachmentDescription {
				flags: vkraw::VkAttachmentDescriptionFlags::_EMPTY,
				format: colour_format,
				samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
				loadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
				storeOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
				stencilLoadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
				stencilStoreOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
				initialLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
				finalLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
			},
			vkraw::VkAttachmentDescription {
				flags: vkraw::VkAttachmentDescriptionFlags::_EMPTY,
				format: depth_format,
				samples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
				loadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_CLEAR,
				storeOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_STORE,
				stencilLoadOp: vkraw::VkAttachmentLoadOp::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
				stencilStoreOp: vkraw::VkAttachmentStoreOp::VK_ATTACHMENT_STORE_OP_DONT_CARE,
				initialLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_UNDEFINED,
				finalLayout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
			}
		];

		self.attachment_references = vec![
			vkraw::VkAttachmentReference {
				attachment: 0,
				layout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
			},
			vkraw::VkAttachmentReference {
				attachment: 1,
				layout: vkraw::VkImageLayout::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
			}
		];
		self.subpasses = vec![
			vkraw::VkSubpassDescription {
				flags: vkraw::VkSubpassDescriptionFlags::_EMPTY,
				pipelineBindPoint: vkraw::VkPipelineBindPoint::VK_PIPELINE_BIND_POINT_GRAPHICS,
				inputAttachmentCount: 0,
				pInputAttachments: ptr::null(),
				colorAttachmentCount: 1,
				pColorAttachments: &self.attachment_references[0],
				pResolveAttachments: ptr::null(),
				pDepthStencilAttachment: &self.attachment_references[1],
				preserveAttachmentCount: 0,
				pPreserveAttachments: ptr::null()
			}
		];

		self
	}

	pub fn create(&self) -> Result<RenderPass<'a>, vkraw::VkResult> {
		let render_pass_create_info = vkraw::VkRenderPassCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			attachmentCount: self.attachments.len() as u32,
			pAttachments: self.attachments.as_ptr(),
			subpassCount: self.subpasses.len() as u32,
			pSubpasses: self.subpasses.as_ptr(),
			dependencyCount: self.dependencies.len() as u32,
			pDependencies: self.dependencies.as_ptr()
		};

		let mut render_pass: vkraw::VkRenderPass = 0;
		let res;
		println!("vkCreateRenderPass");
		unsafe {
			res = vkraw::vkCreateRenderPass(self.device.device, &render_pass_create_info, ptr::null(), &mut render_pass);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(RenderPass { render_pass: render_pass, device: self.device })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for RenderPass<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyRenderPass");
			vkraw::vkDestroyRenderPass(self.device.device, self.render_pass, ptr::null());
		}
	}
}

pub struct FramebufferBuilder<'a> {
	pub device: &'a Device<'a>,
	pub image_view_attachments: Vec<&'a ImageView<'a>>,
	pub width: usize,
	pub height: usize,
	pub layers: usize,
	pub render_pass: Option<&'a RenderPass<'a>>
}

impl<'a> FramebufferBuilder<'a> {
	pub fn new(device: &'a Device<'a>) -> Self {
		FramebufferBuilder {
			device: device,
			image_view_attachments: Vec::<&'a ImageView<'a>>::new(),
			width: 0,
			height: 0,
			layers: 1,
			render_pass: None
		}
	}
	pub fn set_attachments<'y>(&'y mut self, attachments: Vec<&'a ImageView<'a>>) -> &'y mut Self {
		self.image_view_attachments = attachments;
		self
	}
	pub fn create(&self) -> Result<Framebuffer<'a>, vkraw::VkResult> {
		let attachments: Vec<vkraw::VkImageView> = self.image_view_attachments.iter().map(|x| x.image_view).collect();
		let fb_create_info = vkraw::VkFramebufferCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			renderPass: if self.render_pass.is_some() { self.render_pass.unwrap().render_pass } else { vkraw::VK_NULL_HANDLE },
			attachmentCount: attachments.len() as u32,
			pAttachments: attachments.as_ptr(),
			width: self.width as u32,
			height: self.height as u32,
			layers: self.layers as u32
		};
		let mut framebuffer: vkraw::VkFramebuffer;
		let res;
		println!("vkCreateFramebuffer");
		unsafe {
			framebuffer = std::mem::uninitialized();
			res = vkraw::vkCreateFramebuffer(self.device.device, &fb_create_info, ptr::null(), &mut framebuffer);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Framebuffer { device: &self.device, framebuffer: framebuffer })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Framebuffer<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyFramebuffer");
			vkraw::vkDestroyFramebuffer(self.device.device, self.framebuffer, ptr::null());
		}
	}
}

impl<'a> Drop for Semaphore<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroySemaphore");
			vkraw::vkDestroySemaphore(self.device.device, self.semaphore, ptr::null());
		}
	}
}

impl<'a> Drop for Fence<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyFence");
			vkraw::vkDestroyFence(self.device.device, self.fence, ptr::null());
		}
	}
}

pub struct DescriptorSetLayoutBuilder<'a> {
	pub device: &'a Device<'a>,
	pub bindings: Vec<vkraw::VkDescriptorSetLayoutBinding>
}

impl<'a> DescriptorSetLayoutBuilder<'a> {
	pub fn new(device: &'a Device<'a>) -> Self {
		DescriptorSetLayoutBuilder {
			device: device,
			bindings: Vec::<vkraw::VkDescriptorSetLayoutBinding>::new()
		}
	}
	pub fn create(&self) -> Result<DescriptorSetLayout<'a>, vkraw::VkResult> {

		let mut descriptor_set_layout: vkraw::VkDescriptorSetLayout = 0;
		let dsl_create_info = vkraw::VkDescriptorSetLayoutCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkDescriptorSetLayoutCreateFlags::_EMPTY,
			bindingCount: self.bindings.len() as u32,
			pBindings: self.bindings.as_ptr()
		};
		let res;
		unsafe {
			res = vkraw::vkCreateDescriptorSetLayout(self.device.device, &dsl_create_info, ptr::null(), &mut descriptor_set_layout);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(DescriptorSetLayout { device: self.device, dsl: descriptor_set_layout })
		} else {
			Err(res)
		}
	}
}

pub struct PipelineLayoutBuilder<'a> {
	pub device: &'a Device<'a>,
	pub dsls: Vec<&'a DescriptorSetLayout<'a>>
}

impl<'a> PipelineLayoutBuilder<'a> {
	pub fn new(device: &'a Device<'a>) -> Self {
		PipelineLayoutBuilder {
			device: device,
			dsls: Vec::<&'a DescriptorSetLayout>::new()
		}
	}
	pub fn create(self) -> Result<PipelineLayout<'a>, vkraw::VkResult> {

		let layouts: Vec<vkraw::VkDescriptorSetLayout> = self.dsls.iter().map(|x| x.dsl).collect();
		let pl_create_info = vkraw::VkPipelineLayoutCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			setLayoutCount: layouts.len() as u32,
			pSetLayouts: layouts.as_ptr(),
			pushConstantRangeCount: 0, // TODO 
			pPushConstantRanges: ptr::null() // TODO 
		};
		let mut pipeline_layout: vkraw::VkPipelineLayout = 0;
		let res;
		unsafe {
			res = vkraw::vkCreatePipelineLayout(self.device.device, &pl_create_info, ptr::null(), &mut pipeline_layout);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(PipelineLayout { device: self.device, pipeline_layout: pipeline_layout, dsls: self.dsls })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for DescriptorSetLayout<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyDescriptorSetLayout");
			vkraw::vkDestroyDescriptorSetLayout(self.device.device, self.dsl, ptr::null());
		}
	}
}

impl<'a> Drop for PipelineLayout<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyPipelineLayout");
			vkraw::vkDestroyPipelineLayout(self.device.device, self.pipeline_layout, ptr::null());
		}
	}
}

pub struct ShaderStage<'a> {
	pub module: ShaderModule<'a>,
	pub entry_point: String,
	pub stage: vkraw::VkShaderStageFlagBits
}

pub struct PipelineBuilder<'a> {
	pub device: &'a Device<'a>,
	pub layout: &'a PipelineLayout<'a>,
	pub render_pass: &'a RenderPass<'a>,
	pub shader_stages: Vec<ShaderStage<'a>>,
	pub vertex_input: Option<vkraw::VkPipelineVertexInputStateCreateInfo>,
	pub input_assembly: Option<vkraw::VkPipelineInputAssemblyStateCreateInfo>,
	pub viewport: Option<vkraw::VkPipelineViewportStateCreateInfo>,
	pub rasterisation: Option<vkraw::VkPipelineRasterizationStateCreateInfo>,
	pub multisample: Option<vkraw::VkPipelineMultisampleStateCreateInfo>,
	pub depth_stencil: Option<vkraw::VkPipelineDepthStencilStateCreateInfo>,
	pub colour_blend: Option<vkraw::VkPipelineColorBlendStateCreateInfo>,
	pub dynamic: Option<vkraw::VkPipelineDynamicStateCreateInfo>,
	pub subpass: usize,
	
	pub vertex_attributes: Vec<vkraw::VkVertexInputAttributeDescription>,
	pub vertex_bindings: Vec<vkraw::VkVertexInputBindingDescription>,
	pub scissors: Vec<vkraw::VkRect2D>,
	pub viewports: Vec<vkraw::VkViewport>,
	pub blend_attachments: Vec<vkraw::VkPipelineColorBlendAttachmentState>
}

impl<'a> PipelineBuilder<'a> {
	pub fn new(device: &'a Device<'a>, layout: &'a PipelineLayout<'a>, render_pass: &'a RenderPass<'a>) -> Self {
		PipelineBuilder {
			device: device,
			layout: layout,
			render_pass: render_pass,
			shader_stages: Vec::<ShaderStage>::new(),
			vertex_input: None,
			input_assembly: None,
			viewport: None,
			rasterisation: None,
			multisample: None,
			depth_stencil: None,
			colour_blend: None,
			dynamic: None,
			subpass: 0,
			vertex_attributes: Vec::<vkraw::VkVertexInputAttributeDescription>::new(),
			vertex_bindings: Vec::<vkraw::VkVertexInputBindingDescription>::new(),
			scissors: Vec::<vkraw::VkRect2D>::new(),
			viewports: Vec::<vkraw::VkViewport>::new(),
			blend_attachments: Vec::<vkraw::VkPipelineColorBlendAttachmentState>::new()
		}
	}
	pub fn default_graphics<'y>(&'y mut self, vertex_shader: ShaderModule<'a>, fragment_shader: ShaderModule<'a>, width: u32, height: u32) -> &'y mut Self {
		let vertex_size = std::mem::size_of::<f32>() * 6;
		self.vertex_bindings = vec![vkraw::VkVertexInputBindingDescription {
			binding: 0,
			stride: vertex_size as u32,
			inputRate: vkraw::VkVertexInputRate::VK_VERTEX_INPUT_RATE_VERTEX
		}];
		self.vertex_attributes = vec![vkraw::VkVertexInputAttributeDescription {
				location: 0,
				binding: 0,
				format: vkraw::VkFormat::VK_FORMAT_R32G32B32_SFLOAT,
				offset: 0
			},
			vkraw::VkVertexInputAttributeDescription {
				location: 1,
				binding: 0,
				format: vkraw::VkFormat::VK_FORMAT_R32G32B32_SFLOAT,
				offset: 12 // TODO get this from somewhere
			}];
		let vertex_input = vkraw::VkPipelineVertexInputStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			vertexBindingDescriptionCount: self.vertex_bindings.len() as u32,
			pVertexBindingDescriptions: self.vertex_bindings.as_ptr(),
			vertexAttributeDescriptionCount: self.vertex_attributes.len() as u32,
			pVertexAttributeDescriptions: self.vertex_attributes.as_ptr()
		};
		
		let input_assembly = vkraw::VkPipelineInputAssemblyStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			topology: vkraw::VkPrimitiveTopology::VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
			primitiveRestartEnable: vkraw::VK_FALSE
		};
		self.viewports = vec![vkraw::VkViewport {
			x: 0.0,
			y: 0.0,
			width: width as f32,
			height: height as f32,
			minDepth: 0.0,
			maxDepth: 1.0
		}];
		self.scissors = vec![vkraw::VkRect2D {
			offset: vkraw::VkOffset2D {
				x: 0,
				y: 0
			},
			extent: vkraw::VkExtent2D {
				width: width,
				height: height
			}
		}];
		let viewport = vkraw::VkPipelineViewportStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			viewportCount: self.viewports.len() as u32,
			pViewports: self.viewports.as_ptr(),
			scissorCount: self.scissors.len() as u32,
			pScissors: self.scissors.as_ptr()
		};
		let rasterisation = vkraw::VkPipelineRasterizationStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			depthClampEnable: vkraw::VK_FALSE,
			rasterizerDiscardEnable: vkraw::VK_FALSE,
			polygonMode: vkraw::VkPolygonMode::VK_POLYGON_MODE_FILL,
			cullMode: vkraw::VkCullModeFlags::VK_CULL_MODE_NONE,
			frontFace: vkraw::VkFrontFace::VK_FRONT_FACE_COUNTER_CLOCKWISE,
			depthBiasEnable: vkraw::VK_FALSE,
			depthBiasConstantFactor: 0.0,
			depthBiasClamp: 0.0,
			depthBiasSlopeFactor: 0.0,
			lineWidth: 1.0
		};
		let multisample = vkraw::VkPipelineMultisampleStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			rasterizationSamples: vkraw::VkSampleCountFlagBits::VK_SAMPLE_COUNT_1_BIT,
			sampleShadingEnable: vkraw::VK_FALSE,
			minSampleShading: 0.0,
			pSampleMask: ptr::null(),
			alphaToCoverageEnable: vkraw::VK_FALSE,
			alphaToOneEnable: vkraw::VK_FALSE
		};
		let depth_stencil = vkraw::VkPipelineDepthStencilStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			depthTestEnable: vkraw::VK_TRUE,
			depthWriteEnable: vkraw::VK_TRUE,
			depthCompareOp: vkraw::VkCompareOp::VK_COMPARE_OP_LESS_OR_EQUAL,
			depthBoundsTestEnable: vkraw::VK_FALSE,
			stencilTestEnable: vkraw::VK_FALSE,
			front: vkraw::VkStencilOpState {
				failOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				passOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				depthFailOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				compareOp: vkraw::VkCompareOp::VK_COMPARE_OP_ALWAYS,
				compareMask: 0,
				writeMask: 0,
				reference: 0
			},
			back: vkraw::VkStencilOpState {
				failOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				passOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				depthFailOp: vkraw::VkStencilOp::VK_STENCIL_OP_KEEP,
				compareOp: vkraw::VkCompareOp::VK_COMPARE_OP_ALWAYS,
				compareMask: 0,
				writeMask: 0,
				reference: 0
			},
			minDepthBounds: 0.0,
			maxDepthBounds: 1.0
		};
		self.blend_attachments = vec![vkraw::VkPipelineColorBlendAttachmentState {
			blendEnable: vkraw::VK_FALSE,
			srcColorBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			dstColorBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			colorBlendOp: vkraw::VkBlendOp::VK_BLEND_OP_ADD ,
			srcAlphaBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			dstAlphaBlendFactor: vkraw::VkBlendFactor::VK_BLEND_FACTOR_ZERO,
			alphaBlendOp: vkraw::VkBlendOp::VK_BLEND_OP_ADD ,
			colorWriteMask: vkraw::VkColorComponentFlags::all()
		}];
		let colour_blend = vkraw::VkPipelineColorBlendStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			logicOpEnable: vkraw::VK_FALSE,
			logicOp: vkraw::VkLogicOp::VK_LOGIC_OP_CLEAR,
			attachmentCount: self.blend_attachments.len() as u32,
			pAttachments: self.blend_attachments.as_ptr(),
			blendConstants: [0.0, 0.0, 0.0, 0.0]
		};
		let dynamic = vkraw::VkPipelineDynamicStateCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			dynamicStateCount: 0,
			pDynamicStates: ptr::null()
		};
		self.vertex_input = Some(vertex_input);
		self.input_assembly = Some(input_assembly);
		self.viewport = Some(viewport);
		self.rasterisation = Some(rasterisation);
		self.multisample = Some(multisample);
		self.depth_stencil = Some(depth_stencil);
		self.colour_blend = Some(colour_blend);
		self.dynamic = Some(dynamic);
		self.shader_stages = vec![
			ShaderStage {
				module: vertex_shader,
				entry_point: "main".to_string(),
				stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_VERTEX_BIT
			},
			ShaderStage {
				module: fragment_shader,
				entry_point: "main".to_string(),
				stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_FRAGMENT_BIT
			},
		];
		self
	}
	pub fn create(&self) -> Result<Pipeline<'a>, vkraw::VkResult> {

		let entry_points_cstring: Vec<CString> = self.shader_stages.iter().map(|x| std::ffi::CString::new(x.entry_point.clone()).unwrap()).collect();
		let modules: Vec<vkraw::VkPipelineShaderStageCreateInfo> = self.shader_stages.iter().enumerate().map(|x| vkraw::VkPipelineShaderStageCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			stage: x.1.stage,
			module: x.1.module.module,
			pName: entry_points_cstring[x.0].as_ptr() as *const u8,
			pSpecializationInfo: ptr::null()
		}).collect();

		let pipeline_create_info = vkraw::VkGraphicsPipelineCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkPipelineCreateFlags::_EMPTY,
			stageCount: modules.len() as u32,
			pStages: modules.as_ptr(),
			pVertexInputState: if self.vertex_input.is_some() { &self.vertex_input.unwrap() } else { ptr::null() },
			pInputAssemblyState: if self.input_assembly.is_some() { &self.input_assembly.unwrap() } else { ptr::null() },
			pTessellationState: ptr::null(), // TODO
			pViewportState: if self.viewport.is_some() { &self.viewport.unwrap() } else { ptr::null() },
			pRasterizationState: if self.rasterisation.is_some() { &self.rasterisation.unwrap() } else { ptr::null() },
			pMultisampleState: if self.multisample.is_some() { &self.multisample.unwrap() } else { ptr::null() },
			pDepthStencilState: if self.depth_stencil.is_some() { &self.depth_stencil.unwrap() } else { ptr::null() },
			pColorBlendState: if self.colour_blend.is_some() { &self.colour_blend.unwrap() } else { ptr::null() },
			pDynamicState: if self.dynamic.is_some() { &self.dynamic.unwrap() } else { ptr::null() },
			layout: self.layout.pipeline_layout,
			renderPass: self.render_pass.render_pass,
			subpass: self.subpass as u32,
			basePipelineHandle: vkraw::VK_NULL_HANDLE,
			basePipelineIndex: 0 as i32
		};
		let mut pipeline: vkraw::VkPipeline = 0;
		let res;
		unsafe {
			res = vkraw::vkCreateGraphicsPipelines(self.device.device, vkraw::VK_NULL_HANDLE, 1, &pipeline_create_info, ptr::null(), &mut pipeline);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(Pipeline { device: &self.device, pipeline: pipeline })
		} else {
			Err(res)
		}
	}
}

pub struct ComputePipelineBuilder<'a> {
	pub device: &'a Device<'a>,
	pub layout: &'a PipelineLayout<'a>,
	pub shader_stage: ShaderStage<'a>
}

impl<'a> ComputePipelineBuilder<'a> {
	pub fn new(device: &'a Device<'a>, layout: &'a PipelineLayout<'a>, compute_shader: ShaderModule<'a>) -> Self {
		ComputePipelineBuilder {
			device: device,
			layout: layout,
			shader_stage: ShaderStage {
				module: compute_shader,
				entry_point: "main".to_string(),
				stage: vkraw::VkShaderStageFlagBits::VK_SHADER_STAGE_COMPUTE_BIT
			}
		}
	}
	pub fn create(&self) -> Result<ComputePipeline<'a>, vkraw::VkResult> {

		let entry_point_cstring: CString = std::ffi::CString::new(self.shader_stage.entry_point.clone()).unwrap();
		let module: vkraw::VkPipelineShaderStageCreateInfo = vkraw::VkPipelineShaderStageCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
			pNext: ptr::null(),
			flags: 0,
			stage: self.shader_stage.stage,
			module: self.shader_stage.module.module,
			pName: entry_point_cstring.as_ptr() as *const u8,
			pSpecializationInfo: ptr::null()
		};

		let pipeline_create_info = vkraw::VkComputePipelineCreateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkPipelineCreateFlags::_EMPTY,
			stage: module,
			layout: self.layout.pipeline_layout,
			basePipelineHandle: vkraw::VK_NULL_HANDLE,
			basePipelineIndex: 0 as i32
		};
		let mut pipeline: vkraw::VkPipeline = 0;
		let res;
		unsafe {
			res = vkraw::vkCreateComputePipelines(self.device.device, vkraw::VK_NULL_HANDLE, 1, &pipeline_create_info, ptr::null(), &mut pipeline);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(ComputePipeline { device: &self.device, pipeline: pipeline })
		} else {
			Err(res)
		}
	}
}

impl<'a> Drop for Sampler<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroySampler");
			vkraw::vkDestroySampler(self.device.device, self.sampler, ptr::null());
		}
	}
}

impl<'a> Drop for Pipeline<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyPipeline");
			vkraw::vkDestroyPipeline(self.device.device, self.pipeline, ptr::null());
		}
	}
}

impl<'a> Drop for ShaderModule<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyShaderModule");
			vkraw::vkDestroyShaderModule(self.device.device, self.module, ptr::null());
		}
	}
}

impl<'a> Drop for DescriptorPool<'a> {
	fn drop(&mut self) {
		assert!(self.device.device != vkraw::VK_NULL_HANDLE);
		unsafe {
			println!("vkDestroyDescriptorPool");
			vkraw::vkDestroyDescriptorPool(self.device.device, self.descriptor_pool, ptr::null());
		}
	}
}

impl<'a> DescriptorPool<'a> {
	pub fn create_descriptor_sets(&self, layouts: Vec<&'a DescriptorSetLayout<'a>>) -> Result<Vec<DescriptorSet>, vkraw::VkResult> {
		let mut descriptor_sets = Vec::<vkraw::VkDescriptorSet>::with_capacity(layouts.len());
		let set_layouts: Vec<vkraw::VkDescriptorSetLayout> = layouts.iter().map(|x| x.dsl).collect();
		let ds_alloc = vkraw::VkDescriptorSetAllocateInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
			pNext: ptr::null(),
			descriptorPool: self.descriptor_pool,
			descriptorSetCount: set_layouts.len() as u32,
			pSetLayouts: set_layouts.as_ptr()
		};
		let res;
		unsafe {
			descriptor_sets.set_len(layouts.len());
			res = vkraw::vkAllocateDescriptorSets(self.device.device, &ds_alloc, descriptor_sets.as_mut_ptr());
		}
		let sets = descriptor_sets.iter().enumerate().map(|(i,x)| DescriptorSet { 
			descriptor_set: *x,
			descriptor_pool: &self,
			set_layouts: layouts[i]
		}).collect();
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(sets)
		} else {
			Err(res)
		}
	}
}

impl<'a> DescriptorSet<'a> {
	pub fn update_as_buffer(&self, buffer_info: vkraw::VkDescriptorBufferInfo, binding: usize, array_element: usize, dtype: vkraw::VkDescriptorType) {
		let write_ds = vkraw::VkWriteDescriptorSet {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
			pNext: ptr::null(),
			dstSet: self.descriptor_set,
			dstBinding: binding as u32,
			dstArrayElement: array_element as u32,
			descriptorCount: 1,
			descriptorType: dtype,
			pImageInfo: ptr::null(),
			pBufferInfo: &buffer_info,
			pTexelBufferView: ptr::null()
		};
		unsafe {
			vkraw::vkUpdateDescriptorSets(self.descriptor_pool.device.device, 1, &write_ds, 0, ptr::null());
		}
	}
	pub fn update_as_image(&self, image_info: vkraw::VkDescriptorImageInfo, binding: usize, array_element: usize, dtype: vkraw::VkDescriptorType) {
		let write_ds = vkraw::VkWriteDescriptorSet {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
			pNext: ptr::null(),
			dstSet: self.descriptor_set,
			dstBinding: binding as u32,
			dstArrayElement: array_element as u32,
			descriptorCount: 1,
			descriptorType: dtype,
			pImageInfo: &image_info,
			pBufferInfo: ptr::null(),
			pTexelBufferView: ptr::null()
		};
		unsafe {
			vkraw::vkUpdateDescriptorSets(self.descriptor_pool.device.device, 1, &write_ds, 0, ptr::null());
		}
	}
}

impl<'a> CommandBuffer<'a> {
	pub fn begin<'y>(&'y mut self) -> Result<&'y mut Self, vkraw::VkResult> {
	
		let begin_info = vkraw::VkCommandBufferBeginInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
			pNext: ptr::null(),
			flags: vkraw::VkCommandBufferUsageFlags::_EMPTY,
			pInheritanceInfo: ptr::null()
		};
		let res;
		unsafe {
			res = vkraw::vkBeginCommandBuffer(self.command_buffer, &begin_info);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(self)
		} else {
			Err(res)
		}
	}
	pub fn reset<'y>(&'y mut self) -> Result<&'y mut Self, vkraw::VkResult> {
		let res;
		unsafe {
			res = vkraw::vkResetCommandBuffer(self.command_buffer, vkraw::VkCommandBufferResetFlags::_EMPTY);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(self)
		} else {
			Err(res)
		}
	}
	pub fn begin_render_pass<'y>(&'y mut self, width: u32, height: u32, render_pass: &'y RenderPass<'y>, clear_values: Vec<ClearValue>, framebuffer: Option<&Framebuffer>) -> &'y mut Self {

		let mut raw_clear_values = Vec::<vkraw::VkClearValue>::new();
		for cv in clear_values {
			match cv {
				ClearValue::Colourf32(c) => { raw_clear_values.push(vkraw::VkClearValue { colour: vkraw::VkClearColorValue { float32: c } }) }
				ClearValue::Colouri32(c) => { raw_clear_values.push(vkraw::VkClearValue { colour: vkraw::VkClearColorValue { int32: c } }) }
				ClearValue::Colouru32(c) => { raw_clear_values.push(vkraw::VkClearValue { colour: vkraw::VkClearColorValue { uint32: c } }) }
				ClearValue::DepthStencil{ depth: depth, stencil: stencil } => { raw_clear_values.push(vkraw::VkClearValue { depthStencil: vkraw::VkClearDepthStencilValue { depth: depth, stencil: stencil } }) }
			}
		}

		let mut rp_begin_info = vkraw::VkRenderPassBeginInfo {
			sType: vkraw::VkStructureType::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
			pNext: ptr::null(),
			renderPass: render_pass.render_pass,
			framebuffer: vkraw::VK_NULL_HANDLE,
			renderArea: vkraw::VkRect2D {
				offset: vkraw::VkOffset2D {
					x: 0,
					y: 0
				},
				extent: vkraw::VkExtent2D {
					width: width,
					height: height
				}
			},
			clearValueCount: raw_clear_values.len() as u32,
			pClearValues: raw_clear_values.as_ptr()
		};
		if let Some(fb) = framebuffer {
			rp_begin_info.framebuffer = fb.framebuffer;
		}
		unsafe {
			vkraw::vkCmdBeginRenderPass(self.command_buffer, &rp_begin_info, vkraw::VkSubpassContents::VK_SUBPASS_CONTENTS_INLINE);
		}
		self
	}
	pub fn bind_descriptor_sets<'y>(&'y mut self, bind_point: vkraw::VkPipelineBindPoint, pipeline_layout: &'y PipelineLayout<'y>, first_set: u32, sets: Vec<&'y DescriptorSet<'y>>, offsets: Vec<u32>) -> &'y mut Self {

		let sets: Vec<vkraw::VkDescriptorSet> = sets.iter().map(|x| { x.descriptor_set }).collect();

		unsafe {
			vkraw::vkCmdBindDescriptorSets(self.command_buffer, bind_point, pipeline_layout.pipeline_layout, first_set, sets.len() as u32, sets.as_ptr(), offsets.len() as u32, offsets.as_ptr());
		}
		self
	}
	pub fn bind_pipeline<'y>(&'y mut self, bind_point: vkraw::VkPipelineBindPoint, pipeline: &'y Pipeline<'y>) -> &'y mut Self {
		unsafe {
			vkraw::vkCmdBindPipeline(self.command_buffer, bind_point, pipeline.pipeline);
		}
		self
	}
	pub fn bind_compute_pipeline<'y>(&'y mut self, bind_point: vkraw::VkPipelineBindPoint, pipeline: &'y ComputePipeline<'y>) -> &'y mut Self {
		unsafe {
			vkraw::vkCmdBindPipeline(self.command_buffer, bind_point, pipeline.pipeline);
		}
		self
	}
	pub fn bind_vertex_buffers<'y>(&'y mut self, first_binding: u32, buffers_offsets: Vec<(&'y Buffer<'y>, u64)>) -> &'y mut Self {

		let offsets: Vec<u64> = buffers_offsets.iter().map(|x| { x.1 }).collect();
		let buffers: Vec<vkraw::VkBuffer> = buffers_offsets.iter().map(|x| { x.0.buffer }).collect();

		unsafe {
			vkraw::vkCmdBindVertexBuffers(self.command_buffer, first_binding, buffers_offsets.len() as u32, buffers.as_ptr(), offsets.as_ptr());
		}
		self
	}
	pub fn bind_index_buffer<'y>(&'y mut self, buffer: &'y Buffer<'y>, offset: u64, index_type: vkraw::VkIndexType) -> &'y mut Self {
		unsafe {
			vkraw::vkCmdBindIndexBuffer(self.command_buffer, buffer.buffer, offset, index_type);
		}
		self
	}
	pub fn draw_indexed<'y>(&'y mut self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) -> &'y mut Self {
		unsafe {                                                                                                   
			vkraw::vkCmdDrawIndexed(self.command_buffer, index_count, instance_count, first_index, vertex_offset, first_instance);
		}
		self
	}
	pub fn end_render_pass<'y>(&'y mut self) -> &'y mut Self {
		unsafe {
			vkraw::vkCmdEndRenderPass(self.command_buffer);
		}
		self
	}
	pub fn pipeline_barrier<'y>(&'y mut self, src_stage_mask: vkraw::VkPipelineStageFlagBits, dst_stage_mask: vkraw::VkPipelineStageFlagBits, dependency_flags: vkraw::VkDependencyFlagBits, memory_barriers: Vec<vkraw::VkMemoryBarrier>, buffer_memory_barriers: Vec<vkraw::VkBufferMemoryBarrier>, image_memory_barriers: Vec<vkraw::VkImageMemoryBarrier>) -> &'y mut Self {

		unsafe {
			vkraw::vkCmdPipelineBarrier(self.command_buffer, src_stage_mask, dst_stage_mask, dependency_flags, memory_barriers.len() as u32, memory_barriers.as_ptr(), buffer_memory_barriers.len() as u32, buffer_memory_barriers.as_ptr(), image_memory_barriers.len() as u32, image_memory_barriers.as_ptr());
		}
		self
	}
	pub fn copy_buffer_to_image<'y>(&'y mut self, src_buffer: &'y Buffer<'y>, dst_image: &'y Image<'y>, dst_image_layout: vkraw::VkImageLayout, regions: Vec<vkraw::VkBufferImageCopy>) -> &'y mut Self {
	
		unsafe {
			vkraw::vkCmdCopyBufferToImage(self.command_buffer, src_buffer.buffer, dst_image.image, dst_image_layout, regions.len() as u32, regions.as_ptr());
		}
		self
	}
	pub fn end_command_buffer<'y>(&'y mut self) -> &'y mut Self {
		unsafe {
			vkraw::vkEndCommandBuffer(self.command_buffer);
		}
		self
	}
	pub fn dispatch<'y>(&'y mut self, x: u32, y: u32, z: u32) -> &'y mut Self {
		unsafe {
			vkraw::vkCmdDispatch(self.command_buffer, x, y, z);
		}
		self
	}
}

impl<'a> Fence<'a> {
	pub fn wait<'y>(&'y mut self, timeout: Option<u64>) -> Result<&'y mut Self, vkraw::VkResult> {
		let res;
		unsafe {
			res = vkraw::vkWaitForFences(self.device.device, 1, &self.fence, vkraw::VK_TRUE, timeout.unwrap_or(std::u64::MAX));
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(self)
		} else {
			Err(res)
		}
	}
	pub fn reset<'y>(&'y mut self) -> Result<&'y mut Self, vkraw::VkResult> {
		let res;
		unsafe {
			res = vkraw::vkResetFences(self.device.device, 1, &self.fence);
		}
		if res == vkraw::VkResult::VK_SUCCESS {
			Ok(self)
		} else {
			Err(res)
		}
	}
}


include!("ktx.rs");
