
use std::process::Command;
use std::path::Path;

fn main() {

	// Build the generator
	Command::new("cargo").current_dir("../vkgen").args(&["build"]).status().unwrap();

	// Run the generator
	let exe_path = Path::new("../vkgen/target/debug/vkgen");
	let xml_path = Path::new("../vkgen/vk.xml");
	let out_path = Path::new("src/lib.rs");

	assert!(exe_path.is_file());
	assert!(xml_path.is_file());

	Command::new(exe_path).args(&[xml_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()]).status().unwrap();

	// Link against the libvulkan.so from the loader
	// TODO: should we take over the loader functionality?
	// TODO: should we link dynamically?
	println!("cargo:rustc-link-search=/home/ash/github/Vulkan-LoaderAndValidationLayers/dbuild/loader");
}
