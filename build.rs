
// TODO:
// Cargo does not support build scripts for lib targets, so this file is not used

use std::process::Command;
use std::path::Path;

fn main() {

	let exe_path = Path::new("target/debug/rust_vulkan_api_generator");
	let xml_path = Path::new("/home/ash/Projects/rustvk/vk.xml");
	let out_path = Path::new("src/lib.rs");

	assert!(exe_path.is_file());
	assert!(xml_path.is_file());

	Command::new(exe_path).args(&[xml_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()]).status().unwrap();

	// Link against the libvulkan.so from the loader
	// TODO: should we take over the loader functionality?
	println!("cargo:rustc-link-search=/home/ash/github/Vulkan-LoaderAndValidationLayers/dbuild/loader");
}
