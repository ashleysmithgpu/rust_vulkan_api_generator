
extern crate quick_xml;
extern crate regex;

use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::fmt::Write;
use std::str;

use std::io::prelude::*;

// c type mappings to rust style
fn guess_type_from_name(name: &String) -> String {
	match name.as_ref() {
		"VK_WHOLE_SIZE" => "u64".to_string(),
		"VK_LOD_CLAMP_NONE" => "f32".to_string(),
		_ => "usize".to_string()
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
		"VkBool32" => "bool".to_string(),
		_ => original_type.to_string()
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
		"(~0U)" => "std::usize::MAX".to_string(),
		"(~0ULL)" => "std::u64::MAX".to_string(),
		"(~0U-1)" => "std::usize::MAX - 1".to_string(),
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

	println!("using xml file {}", xml_filename);

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

	let mut param_name = String::new();
	let mut param_type = String::new();
	let mut parameters = "".to_string();
	let mut param_ptr = false;
	let mut param_const = false;
	let mut param_array = false;
	let mut param_array_size = String::new();

	let mut struct_name = String::new();
	let mut struct_member_name = String::new();
	let mut struct_member_type = String::new();
	let mut struct_member_ptr = false;
	let mut struct_member_const = false;
	let mut struct_members = String::new();
	let mut struct_member_array = false;
	let mut struct_member_array_size = String::new();

	let mut enum_name = String::new();
	let mut enum_members = String::new();

	let mut matching_api_constants = false;

	let mut type_category = String::new();
	let mut type_name = String::new();


	// Used when writing output
	// name, value
	let mut api_constants = Vec::<(String, String)>::new();

	// name, parameters, return type
	let mut commands = Vec::<(String, String, String)>::new();

	// name
	let mut types = Vec::<String>::new();
	let mut bitmask_types = Vec::<String>::new();
	let mut handle_types = Vec::<String>::new();

	// name, members
	let mut enums = Vec::<(String, String)>::new();

	// name, members
	let mut bitflags = Vec::<(String, String)>::new();

	// name, members
	let mut structs = Vec::<(String, String)>::new();

	// Loop over each xml element
	loop {
		match reader.read_event(&mut buf) {
			Ok(Event::Start(ref e)) => {
				match e.name() {
					b"enums" => {
						if matching_what[0] == "registry" {
							let mut name = "";
							let mut etype = "";
							for att in e.attributes() {
								let tmp = att.unwrap();
								if str::from_utf8(tmp.key).unwrap() == "name" { name = str::from_utf8(tmp.value).unwrap(); }
								if str::from_utf8(tmp.key).unwrap() == "type" { etype = str::from_utf8(tmp.value).unwrap(); }
							}
							enum_name = name.to_string();
							if etype == "enum" {
								enum_type_bitmask = false;
							} else if etype == "bitmask" {
								enum_type_bitmask = true;
							} else if name == "API Constants" {
								matching_api_constants = true
							}
						}
					},
					b"member" => {
						if matching_what[0] == "type" {
							struct_member_const = false;
							struct_member_ptr = false;
							struct_member_array = false;
							struct_member_array_size.clear();
						}
					},
					b"type" => {
						if matching_what[0] == "types" {
							let mut name = "";
							let mut category = "";
							for att in e.attributes() {
								let tmp = att.unwrap();
								if str::from_utf8(tmp.key).unwrap() == "name" { name = str::from_utf8(tmp.value).unwrap(); }
								if str::from_utf8(tmp.key).unwrap() == "category" { category = str::from_utf8(tmp.value).unwrap(); }
							}
							type_category = category.to_string();

							if category == "struct" {
								struct_name = name.to_string();
							}
						}
					},
					_ => (),
				}
				let s = str::from_utf8(e.name()).unwrap();
				matching_what.push_front(s.to_string());
			},
			Ok(Event::Empty(ref e)) => {
				match e.name() {
					b"enum" => {
						if matching_what[0] == "enums" {
							let mut name = "";
							let mut value = "";
							let mut bitpos = "";
							for att in e.attributes() {
								let tmp = att.unwrap();
								if str::from_utf8(tmp.key).unwrap() == "name" { name = str::from_utf8(tmp.value).unwrap(); }
								if str::from_utf8(tmp.key).unwrap() == "value" { value = str::from_utf8(tmp.value).unwrap(); }
								if str::from_utf8(tmp.key).unwrap() == "bitpos" { bitpos = str::from_utf8(tmp.value).unwrap(); }
							}
							if matching_api_constants {
								api_constants.push((name.to_string(), value.to_string()));
							} else {
								if enum_type_bitmask {
									if !bitpos.is_empty() {
										let ibitpos = bitpos.parse::<i32>().unwrap();
										enum_members.write_fmt(format_args!("\t\tconst {} = 0b", name)).expect("Could not format string");
										for x in (0..32).rev() {
											if x == ibitpos {
												enum_members.write_fmt(format_args!("1")).expect("Could not format string");
											} else {
												enum_members.write_fmt(format_args!("0")).expect("Could not format string");
											}
										}
										enum_members.write_fmt(format_args!(";\n")).expect("Could not format string");
									} else {
										enum_members.write_fmt(format_args!("\t\tconst {} = {};\n", name, value)).expect("Could not format string");
									}
								} else {
									enum_members.write_fmt(format_args!("\t{} = {},\n", name, value)).expect("Could not format string");
								}
							}
						}
					},
					b"type" => {
						if matching_what[0] == "types" {
							let mut name = "";
							let mut requires = "";
							for att in e.attributes() {
								let tmp = att.unwrap();
								if str::from_utf8(tmp.key).unwrap() == "name" { name = str::from_utf8(tmp.value).unwrap(); }
								if str::from_utf8(tmp.key).unwrap() == "requires" { requires = str::from_utf8(tmp.value).unwrap(); }
							}
							if !requires.is_empty() {
								types.push(name.to_string());
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
					}

				// <member>blah</member>
				} else if matching_what[0] == "member" {

					struct_member_const |= text.find("const").is_some();
					struct_member_ptr |= text.find("*").is_some();
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
				}
			},
			Ok(Event::End(ref e)) => {
				match e.name() {
					b"enums" => {
						if matching_api_constants {
							matching_api_constants = false;
						} else {
							if !enum_members.is_empty() {
								if enum_type_bitmask {
									bitflags.push((enum_name.clone(), enum_members.clone()));
								} else {
									enums.push((enum_name.clone(), enum_members.clone()));
								}
								enum_members.clear();
							}
						}
					},
					b"member" => {
						if matching_what[0] == "member" {
							if struct_member_array {
								struct_members.write_fmt(format_args!("\t{}: [{}{} {}; {}],\n",
									struct_member_name,
									if struct_member_ptr { "*" } else { "" },
									if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
									struct_member_type,
									struct_member_array_size)).expect("Could not format string");
							} else {
								struct_members.write_fmt(format_args!("\t{}: {}{} {},\n",
									struct_member_name,
									if struct_member_ptr { "*" } else { "" },
									if struct_member_const { if struct_member_ptr { "const" } else { "" } } else { if struct_member_ptr { "mut" } else { "" } },
									struct_member_type)).expect("Could not format string");
							}
						}
					},
					b"command" => {
						if matching_what[0] == "command" {
							commands.push((function_name.clone(), parameters.clone(), return_value.clone()));
							parameters.clear();
						}
					},
					b"param" => {
						if matching_what[0] == "param" {
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
							param_ptr = false;
							param_const = false;
							param_array = false;
							param_array_size.clear();
						}
					},
					b"type" => {
						if matching_what[0] == "type" {
							if matching_what[1] == "types" {

								if type_category == "bitmask" {
									bitmask_types.push(type_name.clone());
								} else if type_category == "handle" {
									handle_types.push(type_name.clone());
								} else if type_category == "struct" && !struct_members.is_empty() {
									structs.push((struct_name.clone(), struct_members.clone()));
									struct_members.clear();
								}
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
	let fluff = r#"type VkDeviceSize = i64;
type VkSampleMask = i32;

type PFN_vkAllocationFunction = *const c_void;
type PFN_vkReallocationFunction = *const c_void;
type PFN_vkFreeFunction = *const c_void;
type PFN_vkInternalAllocationNotification = *const c_void;
type PFN_vkInternalFreeNotification = *const c_void;
type PFN_vkDebugReportCallbackEXT = *const c_void;
type PFN_vkVoidFunction = *const c_void;

struct VkClearColorValue {
	col: f32
}

struct VkClearValue {
	col: VkClearColorValue
}"#;
	{
		use std::io::Write;
		write!(output, "{}\n", fluff).expect("Failed to write");

		// Print constants
		for consts in api_constants {
			let tmp = &consts.0;
			write!(output, "const {}: {} = {};\n", consts.0, guess_type_from_name(tmp), translate_values(consts.1)).expect("Failed to write");
		}

		// Print typedefs
		for t in types {
			write!(output, "type {} = i64;\n", t).expect("Failed to write");
		}
		for t in bitmask_types {
			write!(output, "type {} = i32;\n", t).expect("Failed to write");
		}
		for t in handle_types {
			write!(output, "type {} = i64;\n", t).expect("Failed to write");
		}

		// Print enums
		for e in enums {
			write!(output, "#[repr(i32)]\nenum {} {{\n{}\n", e.0, e.1).expect("Failed to write");
			write!(output, "}}\n\n").expect("Failed to write");
		}

		// Print bitflags (bitmasks)
		for b in bitflags {
			write!(output, "bitflags! {{\n\tstruct {}: u32 {{\n{}\n", b.0, b.1).expect("Failed to write");
			write!(output, "\t}}\n}}\n\n").expect("Failed to write");
		}

		// Print structs
		for s in structs {
			write!(output, "#[repr(C)]\nstruct {} {{\n{}\n}}\n", s.0, s.1).expect("Failed to write");
		}

		// Print functions
		write!(output, "#[link(name = \"vulkan\")]\n").expect("Failed to write");
		write!(output, "extern {{\n").expect("Failed to write");

		for cmd in commands {

			write!(output, "\tfn {}({}) -> {};\n", cmd.0, cmd.1, cmd.2).expect("Failed to write");
		}
		write!(output, "}}\n").expect("Failed to write");
	}
}
