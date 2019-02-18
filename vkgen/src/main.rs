
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::fmt::Write;
use std::str;

use std::io::prelude::*;
use std::collections::HashMap;

use inflector::cases::screamingsnakecase::to_screaming_snake_case;

use regex::Regex;

// c type mappings to rust style
fn guess_type_from_name(name: &String) -> String {
	match name.as_ref() {
		"VK_WHOLE_SIZE" => "u64".to_string(),
		"VK_LOD_CLAMP_NONE" => "f32".to_string(),
		"VK_TRUE" => "u32".to_string(),
		"VK_FALSE" => "u32".to_string(),
		_ => "usize".to_string()
	}
}

fn c_wsi_types_to_rust_types(type_name: &String) -> String {
	match type_name.as_ref() {
		"xcb_connection_t" => "pub type xcb_connection_t = xcb::ffi::xcb_connection_t;".to_string(),
//		"xcb_visualid_t " => "pub xcb_visualid_t = xcb::ffi::xcb_connection_t;".to_string(),
		"xcb_window_t" => "pub type xcb_window_t = u32;".to_string(),
		_ => format!("pub type {} = u64;", type_name.to_string())
	}
}

fn c_wsi_types_to_cfg_var(type_name: &String) -> String {
	match type_name.as_ref() {
		"xcb_connection_t" => "xcb".to_string(),
		"xcb_visualid_t " => "xcb".to_string(),
		"xcb_window_t" => "xcb".to_string(),
		_ => "".to_string()
	}
}

fn translate_types(original_type: String) -> String {
	match original_type.as_ref() {
		"void" => "c_void".to_string(),
		"uint32_t" => "u32".to_string(),
		"int32_t" => "i32".to_string(),
		"int" => "i32".to_string(),
		"uint64_t" => "u64".to_string(),
		"int64_t" => "i64".to_string(),
		"char" => "u8".to_string(),
		"uint8_t" => "u8".to_string(),
		"float" => "f32".to_string(),
		"VkBool32" => "VkBool32".to_string(),
		_ => original_type.to_string()
	}
}

fn translate_define(define: (String, String)) -> String {

	// TODO: this is the wrong way to use XML
	let re = Regex::new("[0-9]+$").unwrap();

	match define.0.as_ref() {
		"VK_HEADER_VERSION" => format!("\npub const VK_HEADER_VERSION: i32 = {};", re.captures(&define.1).unwrap().get(0).unwrap().as_str()).to_string(),
		"VK_NULL_HANDLE" => "\npub const VK_NULL_HANDLE: u64 = 0;".to_string(),
		"ANativeWindow" => "\npub type ANativeWindow = u64;".to_string(),
		"AHardwareBuffer" => "\npub type AHardwareBuffer = u64;".to_string(),
		_ => String::new()
	}
}

fn translate_member_name(original_name: String) -> String {
	match original_name.as_ref() {
		"type" => "_type".to_string(),
		_ => original_name.to_string()
	}
}

fn translate_parameter_name(original_name: String) -> String {
	match original_name.as_ref() {
		"type" => "_type".to_string(),
		_ => original_name.to_string()
	}
}

fn translate_values(original_value: String) -> String {
	match original_value.as_ref() {
		"1000.0f" => "1000.0".to_string(),
		"(~0U)" => "::std::usize::MAX".to_string(),
		"(~0ULL)" => "::std::u64::MAX".to_string(),
		"(~0U-1)" => "::std::usize::MAX - 1".to_string(),
		"(~0U-2)" => "::std::usize::MAX - 2".to_string(),
		_ => original_value.to_string()
	}
}

fn help() {

	println!("usage: ./main xml_input_filename (-o rs_output_filename)");
}

fn main() {

	// Parse arguments
	let args: Vec<String> = std::env::args().collect();

	let mut xml_filename = String::new();
	let mut rs_filename = String::new();

	match args.len() {
		2 => {
			xml_filename = args[1].to_string();
		},
		4 => {
			xml_filename = args[1].to_string();
			if args[2] == "-o" {
				rs_filename = args[3].to_string();
			} else {
				help();
			}
		},
		_ => {
			help();
		}
	}

	println!("Using input xml file \"{}\"", xml_filename);
	println!("Using output rs file \"{}\"", rs_filename);

	let mut f = std::fs::File::open(xml_filename).expect("Failed to open file");
	let mut contents = String::new();
	f.read_to_string(&mut contents).expect("Could not read file");

	let mut reader = Reader::from_str(&*contents);
	reader.trim_text(true);

	let mut buf = Vec::new();

	let array_regex = regex::Regex::new(r"\[([0-9+])\]").unwrap();

	// List of what we are matching against:
	// <a><b><c>
	// "c","b","a"
	let mut matching_what: std::collections::vec_deque::VecDeque<String> = std::collections::vec_deque::VecDeque::new();

	// State
	let mut function_name = String::new();
	let mut return_value = String::new();

	let mut enum_type_bitmask = false;

	let mut comments = Vec::<String>::new();

	let mut param_name = String::new();
	let mut param_type = String::new();
	let mut parameters = "".to_string();
	let mut param_ptr = false;
	let mut param_ptr_ptr = false;
	let mut param_const = false;
	let mut param_array = false;
	let mut param_array_size = String::new();

	let mut struct_name = String::new();
	let mut struct_member_name = String::new();
	let mut struct_member_type = String::new();
	let mut struct_member_ptr = false;
	let mut struct_member_ptr_ptr = false;
	let mut struct_member_const = false;
	let mut struct_members = String::new();
	let mut struct_contains_arrays = false;
	let mut struct_member_array = false;
	let mut struct_member_array_size = String::new();

	let mut enum_name = String::new();

	let mut matching_api_constants = false;

	let mut type_category = String::new();
	let mut type_name = String::new();
	let mut type_requires = String::new();

	let mut define_type_value = String::new();


	// Used when writing output
	// name, value
	let mut api_constants = Vec::<(String, String)>::new();

	// name, parameters, return type
	let mut commands = HashMap::new();

	// name
	let mut types = Vec::<String>::new();
	let mut bitmask_types = Vec::<(String, String)>::new();
	let mut handle_types = Vec::<String>::new();
	let mut define_types = Vec::<(String, String)>::new();

	// Enums
	struct Enum {
		name: String,
		values: Vec<(String, i32)>
	}
	let mut enums = Vec::<Enum>::new();

	enum BitflagsValueType {
		Bitpos(u32),
		Value(String)
	}
	struct Bitflags {
		name: String,
		values: Vec<(String, BitflagsValueType)>
	}
	let mut bitflags = Vec::<Bitflags>::new();

	// name, members, contains arrays
	let mut structs = Vec::<(String, String, bool)>::new();

	// Features
	enum FeatureContent {
		Command(String),
		Type(String),
		Enum(String)
	}

	struct FeatureBlock {
		comment: String,
		contents: Vec<FeatureContent>
	}

	let mut features = Vec::<FeatureBlock>::new();

	// Extensions
	enum ExtensionNewType {
		EnumExtension { name: String, offset: u32, extends: String, comment: String, dir: String },
		BitflagsExtension { name: String, bitpos: u32, extends: String, comment: String },
		Command(String),
		Type(String)
	}

	struct Extension {
		name: String,
		number: u32,
		extension_type: String,
		author: String,
		contact: String,
		supported: String,
		types: Vec<ExtensionNewType>
	}

	let mut extensions = Vec::<Extension>::new();

	// TODO: hack
	let mut require_feature = String::new();

	let mut attributes: HashMap<String, String> = HashMap::new();

	// Loop over each xml element
	loop {
		match reader.read_event(&mut buf) {
			Ok(Event::Start(ref e)) => {

				attributes.clear();
				for att in e.attributes() {
					let tmp = att.unwrap();
					attributes.insert(str::from_utf8(tmp.key).unwrap().to_string(), str::from_utf8(tmp.value).unwrap().to_string());
				}

				match e.name() {
					b"enums" => {
						if matching_what[0] == "registry" {
							let mut etype = "";
							if let Some(etypes) = attributes.get("type") {
								etype = etypes;
							}
							if let Some(name) = attributes.get("name") {
								enum_name = name.to_string();
							}
							if etype == "enum" {
								enum_type_bitmask = false;
								enums.push(Enum{
									name: enum_name.clone(),
									values: vec![]
								});

							} else if etype == "bitmask" {
								enum_type_bitmask = true;
								bitflags.push(Bitflags{
									name: enum_name.clone(),
									values: vec![]
								});

							} else if enum_name == "API Constants" {
								matching_api_constants = true
							}
						}
					},
					b"member" => {
						if matching_what[0] == "type" {
							struct_member_const = false;
							struct_member_ptr = false;
							struct_member_ptr_ptr = false;
							struct_member_array = false;
							struct_member_array_size.clear();
						}
					},
					b"type" => {
						if matching_what[0] == "types" {
							if let Some(category) = attributes.get("category") {
								type_category = category.to_string();
							}
							if let Some(requires) = attributes.get("requires") {
								type_requires = requires.to_string();
							} else {
								type_requires.clear();
							}

							if type_category == "struct" {
								if let Some(name) = attributes.get("name") {
									struct_name = name.to_string();
								}
							}
						}
					},
					b"require" => {
						if matching_what[0] == "feature" {
							if let Some(comment) = attributes.get("comment") {
								features.push(FeatureBlock{
									comment: comment.to_string(), contents: vec![]
								});
							}
						}
						if let Some(feature) = attributes.get("feature") {
							require_feature = feature.to_string();
						} else {
							require_feature = "".to_string();
						}
					},
					b"extension" => {
						if matching_what[0] == "extensions" {
							if let (Some(name), Some(number), Some(supported)) = (attributes.get("name"), attributes.get("number"), attributes.get("supported")) {

								extensions.push(Extension{
									name: name.to_string(),
									number: number.parse::<u32>().unwrap(),
									extension_type: if let Some(extension_type) = attributes.get("type") { extension_type.to_string() } else { "".to_string() },
									author: if let Some(author) = attributes.get("author") { author.to_string() } else { "".to_string() },
									contact: if let Some(contact) = attributes.get("contact") { contact.to_string() } else { "".to_string() },
									supported: supported.to_string(),
									types: vec![]
								});
							}
						}
					}
					_ => (),
				}
				let s = str::from_utf8(e.name()).unwrap();
				matching_what.push_front(s.to_string());
			},
			Ok(Event::Empty(ref e)) => {

				attributes.clear();
				for att in e.attributes() {
					let tmp = att.unwrap();
					attributes.insert(str::from_utf8(tmp.key).unwrap().to_string(), str::from_utf8(tmp.value).unwrap().to_string());
				}

				match e.name() {
					b"enum" => {


if require_feature == "" {


						if matching_what[0] == "enums" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							let value = if let Some(value) = attributes.get("value") { value.to_string() } else { if let Some(alias) = attributes.get("alias") { alias.to_string() } else { "".to_string() } };
							let bitpos = if let Some(bitpos) = attributes.get("bitpos") { bitpos.to_string() } else { "".to_string() };
							if matching_api_constants {
								api_constants.push((name, value));
							} else {
								if enum_type_bitmask {
									if !bitpos.is_empty() {
										bitflags.last_mut().unwrap().values.push((name, BitflagsValueType::Bitpos(bitpos.parse::<u32>().unwrap())));
									} else {
										bitflags.last_mut().unwrap().values.push((name, BitflagsValueType::Value(value)));
									}
								} else {
									enums.last_mut().unwrap().values.push((name, value.parse::<i32>().unwrap()));
								}
							}
						} else if matching_what[0] == "require" && matching_what[1] == "feature" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							features.last_mut().unwrap().contents.push(FeatureContent::Enum(name.to_string()));
						} else if matching_what[0] == "require" && matching_what[1] == "extension" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							let extends = if let Some(extends) = attributes.get("extends") { extends.to_string() } else { "".to_string() };
							let comment = if let Some(comment) = attributes.get("comment") { comment.to_string() } else { "".to_string() };

							// Extends a bitflags structure
							if let Some(bitpos) = attributes.get("bitpos") {
								extensions.last_mut().unwrap().types.push(ExtensionNewType::BitflagsExtension{
									name: name.to_string(),
									bitpos: bitpos.to_string().parse::<u32>().unwrap(),
									extends: extends,
									comment: comment,
								});
							// Extends an enum
							} else if let Some(offset) = attributes.get("offset") {
								let dir = if let Some(dir) = attributes.get("dir") { dir.to_string() } else { "".to_string() };
								extensions.last_mut().unwrap().types.push(ExtensionNewType::EnumExtension{
									name: name.to_string(),
									offset: offset.to_string().parse::<u32>().unwrap(),
									extends: extends,
									comment: comment,
									dir: dir,
								});
							}
						}

}


					},
					b"type" => {





if require_feature == "" {





						if matching_what[0] == "types" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							if attributes.contains_key("requires") {
								types.push(name.to_string());
							}
						} else if matching_what[0] == "require" && matching_what[1] == "feature" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							features.last_mut().unwrap().contents.push(FeatureContent::Type(name.to_string()));
						} else if matching_what[0] == "require" && matching_what[1] == "extension" {

							if let Some(name) = attributes.get("name") {
								extensions.last_mut().unwrap().types.push(ExtensionNewType::Type(name.to_string()));
							}
						}




}


					},
					b"command" => {


if require_feature == "" {



						if matching_what[0] == "require" && matching_what[1] == "feature" {
							let name = if let Some(name) = attributes.get("name") { name.to_string() } else { "".to_string() };
							features.last_mut().unwrap().contents.push(FeatureContent::Command(name.to_string()));
						} else if matching_what[0] == "require" && matching_what[1] == "extension" {

							if let Some(name) = attributes.get("name") {
								extensions.last_mut().unwrap().types.push(ExtensionNewType::Command(name.to_string()));
							}
						}



}

					},
					_ => (),
				}
			},
			Ok(Event::Text(ref e)) => {
				let text = e.unescape_and_decode(&reader).unwrap();

				// <name>blah</name>
				if matching_what[0] == "name" {

					if matching_what[1] == "proto" {
						function_name = text;
					} else if matching_what[1] == "param" {
						param_name = translate_parameter_name(text);
					} else if matching_what[1] == "member" {
						struct_member_name = translate_member_name(text);
					} else if matching_what[1] == "type" {
						type_name = text;
					}

				// <type>blah</type>
				} else if matching_what[0] == "type" {

					if matching_what[1] == "proto" {
						return_value = translate_types(text);
					} else if matching_what[1] == "param" {
						param_type = translate_types(text);
					} else if matching_what[1] == "member" {
						struct_member_type = translate_types(text);
					} else if matching_what[1] == "types" {
						define_type_value.push_str(&text);
					}

				// <member>blah</member>
				} else if matching_what[0] == "member" {

					struct_member_const |= text.find("const").is_some();
					struct_member_ptr |= text.find("*").is_some();
					struct_member_ptr_ptr |= text.find("*").is_some() && (text.find("*").unwrap() != text.rfind("*").unwrap());
					if text.find("[").is_some() {
						struct_member_array = true;

						for cap in array_regex.captures_iter(&text) {
							struct_member_array_size = cap[1].to_string();
						}
					}

				// <param>blah</param>
				} else if matching_what[0] == "param" {

					param_const |= text.find("const").is_some();
					param_ptr |= text.find("*").is_some();
					param_ptr_ptr |= text.find("*").is_some() && (text.find("*").unwrap() != text.rfind("*").unwrap());
					if text.find("[").is_some() {
						param_array = true;

						for cap in array_regex.captures_iter(&text) {
							param_array_size = cap[1].to_string();
						}
					}

				// <enum>blah</enum>
				} else if matching_what[0] == "enum" {

					if matching_what[1] == "member" {
						struct_member_array_size = text.to_string();
					} else if matching_what[1] == "param" {
						param_array_size = text.to_string();
					}

				// <comment>blah</comment>
				} else if matching_what[0] == "comment" {

					comments.push(text.to_string());
				}
			},
			Ok(Event::End(ref e)) => {
				match e.name() {
					b"enums" => {
						if matching_api_constants {
							matching_api_constants = false;
						}
					},
					b"member" => {
						if matching_what[0] == "member" {
							if struct_member_ptr_ptr {
								if struct_member_array {
									struct_members.write_fmt(format_args!("\tpub {}: [{}{}{}{} {}; {}],\n",
										struct_member_name,
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										struct_member_type,
										struct_member_array_size)).expect("Could not format string");
									struct_contains_arrays = true;
								} else {
									struct_members.write_fmt(format_args!("\tpub {}: {}{}{}{} {},\n",
										struct_member_name,
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										struct_member_type)).expect("Could not format string");
								}
							} else {
								if struct_member_array {
									struct_members.write_fmt(format_args!("\tpub {}: [{}{} {}; {}],\n",
										struct_member_name,
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										struct_member_type,
										struct_member_array_size)).expect("Could not format string");
									struct_contains_arrays = true;
								} else {
									struct_members.write_fmt(format_args!("\tpub {}: {}{} {},\n",
										struct_member_name,
										if struct_member_ptr { "*" } else { "" },
										if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
										struct_member_type)).expect("Could not format string");
								}
							}
						}
					},
					b"command" => {
						if matching_what[0] == "command" {
							commands.insert(function_name.clone(), (parameters.clone(), return_value.clone()));
							parameters.clear();
						}
					},
					b"param" => {
						if matching_what[0] == "param" && matching_what[1] == "command" {
							if param_ptr_ptr {
								if param_array {
									parameters.write_fmt(format_args!("{}: [{}{}{}{} {}; {}], ",
										param_name,
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										param_type,
										param_array_size)).expect("Could not format string");
								} else {
									parameters.write_fmt(format_args!("{}: {}{}{}{} {}, ",
										param_name,
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										param_type)).expect("Could not format string");
								}
							} else {
								if param_array {
									parameters.write_fmt(format_args!("{}: [{}{} {}; {}], ",
										param_name,
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										param_type,
										param_array_size)).expect("Could not format string");
								} else {
									parameters.write_fmt(format_args!("{}: {}{} {}, ",
										param_name,
										if param_ptr { "*" } else { "" },
										if param_const { if param_ptr { "const" } else { "" } } else { if param_ptr { "mut" } else { "" } },
										param_type)).expect("Could not format string");
								}
							}
							param_ptr = false;
							param_ptr_ptr = false;
							param_const = false;
							param_array = false;
							param_array_size.clear();
						}
					},
					b"type" => {
						if matching_what[0] == "type" {
							if matching_what[1] == "types" {

								if type_category == "bitmask" {
									bitmask_types.push((type_name.clone(), type_requires.clone()));
								} else if type_category == "handle" {
									handle_types.push(type_name.clone());
								} else if type_category == "struct" && !struct_members.is_empty() {
									structs.push((struct_name.clone(), struct_members.clone(), struct_contains_arrays));
								} else if type_category == "define" {
									define_types.push((type_name.clone(), define_type_value.clone()));
									define_type_value.clear();
								}
								struct_members.clear();
								struct_contains_arrays = false;
							}
						}
					},
					_ => (),
				}
				matching_what.pop_front();
			},
			Ok(Event::Eof) => break,
			Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
			_ => (),
		}

		buf.clear();
	}

	// Write output to file or stdout
	let mut output: std::io::BufWriter<Box<std::io::Write>> =
		std::io::BufWriter::new(if rs_filename.is_empty() {
			Box::new(std::io::stdout())
		} else {
			Box::new(std::fs::File::create(rs_filename).unwrap())
		});

	// TODO:
	let fluff1 = r#"
#![allow(non_snake_case)]
#![feature(const_fn)]
#![feature(untagged_unions)]

#[macro_use]
extern crate bitflags;

use std::mem;
use libc::{c_void};

pub const VK_VERSION_1_0: u32 = 1;

pub const fn VK_MAKE_VERSION(major: u32, minor: u32, patch: u32) -> u32 {
	((major) << 22) | ((minor) << 12) | (patch)
}

pub const VK_API_VERSION_1_0: u32 = VK_MAKE_VERSION(1, 0, 0);

pub const fn VK_VERSION_MAJOR(version: u32) -> u32 {
	version >> 22
}

pub const fn VK_VERSION_MINOR(version: u32) -> u32 {
	(version >> 22) & 0x3ff
}

pub const fn VK_VERSION_PATCH(version: u32) -> u32 {
	version & 0x3ff
}
"#;

	let fluff2 = r#"

#[allow(non_camel_case_types)]
pub type VkDeviceSize = u64;
#[allow(non_camel_case_types)]
pub type VkSampleMask = u32;

#[allow(non_camel_case_types)]
pub type PFN_vkAllocationFunction = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkReallocationFunction = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkFreeFunction = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkInternalAllocationNotification = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkInternalFreeNotification = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkDebugReportCallbackEXT = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkDebugUtilsMessengerCallbackEXT = *const c_void;
#[allow(non_camel_case_types)]
pub type PFN_vkVoidFunction = *const c_void;

// Rust assumes bool is u8, vulkan assumes it is u32
pub type VkBool32 = u32;

// TODO: how to do unions in rust?
#[derive(Copy, Clone)]
#[repr(C)]
pub union VkClearColorValue {
	pub float32: [f32; 4],
	pub int32: [i32; 4],
	pub uint32: [u32; 4]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union VkClearValue {
	pub colour: VkClearColorValue,
	pub depthStencil: VkClearDepthStencilValue
}

impl std::fmt::Debug for VkClearColorValue {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		unsafe { write!(f, "VkClearColorValue {{float32: {:?}; or int32: {:?}; or uint32: {:?}}}", self.float32, self.int32, self.uint32) }
	}
}

impl std::fmt::Debug for VkClearValue {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		unsafe { write!(f, "VkClearValue {{colour: {:?}; or depthStencil: {:?}}}", self.colour, self.depthStencil) }
	}
}

// TODO
// Cannot implement Debug for [u8; x > 32]
impl std::fmt::Debug for VkPhysicalDeviceProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "not implemented")
	}
}

impl std::fmt::Debug for VkLayerProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "not implemented")
	}
}

impl std::fmt::Debug for VkExtensionProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "not implemented")
	}
}

impl std::fmt::Debug for VkPhysicalDeviceMemoryProperties {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "not implemented")
	}
}"#;
	{
		use std::io::Write;






		write!(output, "/*\n{}\n*/\n", comments[0]).expect("Failed to write");
		write!(output, "{}\n", fluff1).expect("Failed to write");

		for define in define_types {
			write!(output, "{}", translate_define(define)).expect("Failed to write");
		}

		write!(output, "{}\n", fluff2).expect("Failed to write");

		// Print constants
		for consts in api_constants {
			let tmp = &consts.0;
			write!(output, "pub const {}: {} = {};\n", consts.0, guess_type_from_name(tmp), translate_values(consts.1)).expect("Failed to write");
		}

		// Print typedefs
		for t in types {
			let cfg = c_wsi_types_to_cfg_var(&t);
			if cfg.is_empty() {
				write!(output, "#[allow(non_camel_case_types)]\n{}\n", c_wsi_types_to_rust_types(&t)).expect("Failed to write");
			} else {
				write!(output, "#[allow(non_camel_case_types)]\n#[cfg(feature=\"{}\")]\n{}\n", cfg, c_wsi_types_to_rust_types(&t)).expect("Failed to write");
			}
		}
		for t in bitmask_types {
			if t.1.is_empty() {
				write!(output, "#[allow(non_camel_case_types)]\npub type {} = u32;\n", t.0).expect("Failed to write");
			} else {
				write!(output, "#[allow(non_camel_case_types)]\npub type {} = {};\n", t.0, t.1).expect("Failed to write");
			}
		}
		for t in handle_types {
			write!(output, "#[allow(non_camel_case_types)]\npub type {} = u64;\n", t).expect("Failed to write");
		}

		// See https://github.com/KhronosGroup/Vulkan-LoaderAndValidationLayers/blob/master/scripts/generator.py
		let extension_base_number = 1000000000;
		let extension_block_size = 1000;

		// Print enums
		for e in enums {
			write!(output, "#[allow(non_camel_case_types)]\n#[derive(Copy, Clone, PartialEq, Debug)]\n#[repr(C)]\npub enum {} {{\n", &e.name).expect("Failed to write");
			for v in e.values {
				write!(output, "\t{} = {},\n", v.0, v.1).expect("Failed to write");
			}

			// TODO: probably should use a hash map here
			for ext in &extensions {

				if ext.supported != "disabled" {

					let mut once = true;
					for ext_enum in &ext.types {

						match *ext_enum {
							ExtensionNewType::EnumExtension { ref name, ref offset, ref extends, ref comment, ref dir } => {
								if *extends == e.name {
									if once {
										write!(output, "\n\t// {}\n", ext.name).expect("Failed to write");
										write!(output, "\n#[cfg(feature = \"{}\")]\n", ext.name).expect("Failed to write");
										once = false;
									}
									write!(output, "\t{} = {}{},\n", name, dir, extension_base_number + (ext.number - 1) * extension_block_size + offset).expect("Failed to write");
								}
							},
							_ => ()
						}
					}
				}
			}

			write!(output, "\t{}_MAX_ENUM = 0x7FFFFFFF\n}}\n\n", to_screaming_snake_case(&e.name)).expect("Failed to write");
		}

		// Print bitflags (bitmasks)
		for b in bitflags {

			if b.values.len() > 0 {
				write!(output, "bitflags! {{\n#[repr(C)]\n\tpub struct {}: u32 {{\n", b.name).expect("Failed to write");
				write!(output, "\t\tconst _EMPTY = 0;\n").expect("Failed to write");

				for v in b.values {

					match v.1 {

						BitflagsValueType::Bitpos(bitpos) => {
							write!(output, "\t\tconst {} = 0b", v.0).expect("Could not format string");
							for x in (0..32).rev() {
								if x == bitpos {
									write!(output, "1").expect("Could not format string");
								} else {
									write!(output, "0").expect("Could not format string");
								}
							}
							write!(output, ";\n").expect("Could not format string");
						},
						BitflagsValueType::Value(value) => {
							write!(output, "\t\tconst {} = {};\n", v.0, value).expect("Could not format string");
						}
					}
				}
				write!(output, "\t}}\n}}\n\n").expect("Failed to write");
			} else {
				write!(output, "bitflags! {{\n#[repr(C)]\n\tpub struct {}: u32 {{\n\t\tconst _EMPTY = 0;\n\t}}\n}}\n\n", b.name).expect("Failed to write");
			}
		}

		// Print structs
		for s in structs {
			
			let mut was_ext = false;
			
			// TODO: probably should use a hash map here
			for ext in &extensions {

				if ext.supported != "disabled" {

					let mut once = true;
					for ext_enum in &ext.types {

						match *ext_enum {
							ExtensionNewType::Type(ref name) => {
								if *name == s.0 {
									if once {
										write!(output, "\n// {}\n", ext.name).expect("Failed to write");
										write!(output, "#[cfg(feature = \"{}\")]\n", ext.name).expect("Failed to write");
										once = false;
									}
									// Can't use automatic Debug since rust disallows this for arrays > 32
									if s.2 {
										write!(output, "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct {} {{\n{}\n}}\n", s.0, s.1).expect("Failed to write");
									} else {
										write!(output, "#[derive(Debug, Copy, Clone)]\n#[repr(C)]\npub struct {} {{\n{}\n}}\n", s.0, s.1).expect("Failed to write");
									}
									was_ext = true;
								}
							},
							_ => ()
						}
					}
				}
			}
			
			if !was_ext {
				// Can't use automatic Debug since rust disallows this for arrays > 32
				if s.2 {
					write!(output, "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct {} {{\n{}\n}}\n", s.0, s.1).expect("Failed to write");
				} else {
					write!(output, "#[derive(Debug, Copy, Clone)]\n#[repr(C)]\npub struct {} {{\n{}\n}}\n", s.0, s.1).expect("Failed to write");
				}
			}
		}

		// Print functions
		write!(output, "#[link(name = \"vulkan\")]\n").expect("Failed to write");
		write!(output, "extern {{\n").expect("Failed to write");

		for feature_block in features {

			let mut once = true;
			for feature_content in feature_block.contents {

				match feature_content {
					FeatureContent::Command(name) => {
						if let Some(cmd) = commands.get(&name) {
							if once {
								write!(output, "\n\t// {}\n", feature_block.comment).expect("Failed to write");
								once = false;
							}
							write!(output, "\tpub fn {}({}) -> {};\n", name, cmd.0, cmd.1).expect("Failed to write");
						}
					},
					_ => ()
				}
			}
		}

		write!(output, "}}\n").expect("Failed to write");

		// Print extension functions
		write!(output, "\n\t// Extensions\n").expect("Failed to write");

		write!(output, "\tpub struct VulkanFunctionPointers {{\n").expect("Failed to write");

		for ext in &extensions {

			if ext.supported != "disabled" {

				let mut once = true;
				for ext_cmd in &ext.types {

					match *ext_cmd {
						ExtensionNewType::Command(ref name) => {
							if let Some(cmd) = commands.get(name) {
								if once {
									write!(output, "\n\t\t// {}\n", ext.name).expect("Failed to write");
									once = false;
								}
								write!(output, "\n#[cfg(feature = \"{}\")]\n", ext.name).expect("Failed to write");
								write!(output, "\t\tpub {}: Option<extern \"C\" fn({}) -> {}>,\n", name.replace("vk", ""), cmd.0, cmd.1).expect("Failed to write");
							}
						},
						_ => ()
					}
				}
			}
		}



		write!(output, "\t}}\n\n\timpl VulkanFunctionPointers {{\n\t\tpub fn new(instance: VkInstance) -> VulkanFunctionPointers {{\n\t\t\tassert!(instance != VK_NULL_HANDLE);\n\t\t\tVulkanFunctionPointers {{\n").expect("Failed to write");

		for ext in &extensions {

			if ext.supported != "disabled" {

				let mut once = true;
				for ext_cmd in &ext.types {

					match *ext_cmd {
						ExtensionNewType::Command(ref name) => {
							if let Some(cmd) = commands.get(name) {
								if once {
									write!(output, "\n\t\t\t// {}\n", ext.name).expect("Failed to write");
									once = false;
								}
								write!(output, "\n#[cfg(feature = \"{}\")]\n", ext.name).expect("Failed to write");
								write!(output, "\t\t\t{}: unsafe {{ mem::transmute::<*const c_void,Option<extern \"C\" fn({}) -> {}>>(vkGetInstanceProcAddr(instance, \"{}\\0\".as_ptr())) }},\n", name.replace("vk", ""), cmd.0, cmd.1, name).expect("Failed to write");
							}
						},
						_ => ()
					}
				}
			}
		}

		write!(output, "}}\n\t\t}}\n\t}}\n").expect("Failed to write");
	}
}
