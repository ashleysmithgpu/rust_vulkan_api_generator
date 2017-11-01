fn main() {

	// Link against the libvulkan.so from the loader
	// TODO: should we take over the loader functionality?
	println!("cargo:rustc-link-search=/home/ash/github/Vulkan-LoaderAndValidationLayers/dbuild/loader");
}
