
use std::process::Command;
use std::path::Path;

fn main() {

	// Build the generator
	Command::new("cargo").current_dir("../vkgen").args(&["build"]).status().unwrap();

	// Run the generator
	let exe_path;
	if cfg!(unix) {
		exe_path = Path::new("../vkgen/target/debug/vkgen");
	} else if cfg!(windows) {
		exe_path = Path::new("../vkgen/target/debug/vkgen.exe");
	} else {
		panic!("Unknown system");
	}
	let xml_path = Path::new("../vkgen/vk.xml");
	let out_path = Path::new("src/lib.rs");

	assert!(exe_path.is_file());
	assert!(xml_path.is_file());

	Command::new(exe_path).args(&[xml_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()]).status().unwrap();

	// Link against the libvulkan.so from the loader
	// TODO: should we take over the loader functionality?
	// TODO: should we link dynamically?
	if cfg!(unix) {
		println!("cargo:rustc-link-search=/home/ash/github/Vulkan-Loader/build/loader");
	} else if cfg!(windows) {
		println!("cargo:rustc-link-search=C:\\Users\\ash\\Documents\\GitHub\\Vulkan-Loader\\build\\loader\\Debug");
	} else {
		panic!("Unknown system");
	}
}
