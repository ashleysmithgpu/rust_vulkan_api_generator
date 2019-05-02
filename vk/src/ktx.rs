
use std::io;
use byteorder::ReadBytesExt;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const FILE_IDENTIFIER: [u8; 12] = [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];

// From glcorearb.h https://www.khronos.org/registry/OpenGL/api/GL/glcorearb.h
// and 2.7 glBaseInternalFormat
#[derive(Debug, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum GLBaseInternalFormat {
	GL_DEPTH_COMPONENT = 0x1902,
	GL_DEPTH_STENCIL = 0x84F9,
	GL_RED = 0x1903,
	GL_RG = 0x8227,
	GL_RGB = 0x1907,
	GL_RGBA = 0x1908,
	GL_STENCIL_INDEX = 0x1901
}

// 2.6 glInternalFormat
#[derive(Debug, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum GLSizedInternalFormat {
	GL_R8 = 0x8229,
	GL_R8_SNORM = 0x8F94,
	GL_R16 = 0x822A,
	GL_R16_SNORM = 0x8F98,
	GL_RG8 = 0x822B,
	GL_RG8_SNORM = 0x8F95,
	GL_RG16 = 0x822C,
	GL_RG16_SNORM = 0x8F99,
	GL_R3_G3_B2 = 0x2A10,
	GL_RGB4 = 0x804F,
	GL_RGB5 = 0x8050,
	GL_RGB565 = 0x8D62,
	GL_RGB8 = 0x8051,
	GL_RGB8_SNORM = 0x8F96,
	GL_RGB10 = 0x8052,
	GL_RGB12 = 0x8053,
	GL_RGB16 = 0x8054,
	GL_RGB16_SNORM = 0x8F9A,
	GL_RGBA2 = 0x8055,
	GL_RGBA4 = 0x8056,
	GL_RGB5_A1 = 0x8057,
	GL_RGBA8 = 0x8058,
	GL_RGBA8_SNORM = 0x8F97,
	GL_RGB10_A2 = 0x8059,
	GL_RGB10_A2UI = 0x906F,
	GL_RGBA12 = 0x805A,
	GL_RGBA16 = 0x805B,
	GL_RGBA16_SNORM = 0x8F9B,
	GL_SRGB8 = 0x8C41,
	GL_SRGB8_ALPHA8 = 0x8C43,
	GL_R16F = 0x822D,
	GL_RG16F = 0x822F,
	GL_RGB16F = 0x881B,
	GL_RGBA16F = 0x881A,
	GL_R32F = 0x822E,
	GL_RG32F = 0x8230,
	GL_RGB32F = 0x8815,
	GL_RGBA32F = 0x8814,
	GL_R11F_G11F_B10F = 0x8C3A,
	GL_RGB9_E5 = 0x8C3D,
	GL_R8I = 0x8231,
	GL_R8UI = 0x8232,
	GL_R16I = 0x8233,
	GL_R16UI = 0x8234,
	GL_R32I = 0x8235,
	GL_R32UI = 0x8236,
	GL_RG8I = 0x8237,
	GL_RG8UI = 0x8238,
	GL_RG16I = 0x8239,
	GL_RG16UI = 0x823A,
	GL_RG32I = 0x823B,
	GL_RG32UI = 0x823C,
	GL_RGB8I = 0x8D8F,
	GL_RGB8UI = 0x8D7D,
	GL_RGB16I = 0x8D89,
	GL_RGB16UI = 0x8D77,
	GL_RGB32I = 0x8D83,
	GL_RGB32UI = 0x8D71,
	GL_RGBA8I = 0x8D8E,
	GL_RGBA8UI = 0x8D7C,
	GL_RGBA16I = 0x8D88,
	GL_RGBA16UI = 0x8D76,
	GL_RGBA32I = 0x8D82,
	GL_RGBA32UI = 0x8D70,
	GL_DEPTH_COMPONENT16 = 0x81A5,
	GL_DEPTH_COMPONENT24 = 0x81A6,
	GL_DEPTH_COMPONENT32 = 0x81A7,
	GL_DEPTH_COMPONENT32F = 0x8CAC,
	GL_DEPTH24_STENCIL8 = 0x88F0,
	GL_DEPTH32F_STENCIL8 = 0x8CAD,
	GL_STENCIL_INDEX1 = 0x8D46,
	GL_STENCIL_INDEX4 = 0x8D47,
	GL_STENCIL_INDEX8 = 0x8D48,
	GL_STENCIL_INDEX16 = 0x8D49
}

// 2.5 glFormat
#[derive(Debug, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum GLFormat {
	GL_STENCIL_INDEX = 0x1901,
	GL_DEPTH_COMPONENT = 0x1902,
	GL_DEPTH_STENCIL = 0x84F9,
	GL_RED = 0x1903,
	GL_GREEN = 0x1904,
	GL_BLUE = 0x1905,
	GL_RG = 0x8227,
	GL_RGB = 0x1907,
	GL_RGBA = 0x1908,
	GL_BGR = 0x80E0,
	GL_BGRA = 0x80E1,
	GL_RED_INTEGER = 0x8D94,
	GL_GREEN_INTEGER = 0x8D95,
	GL_BLUE_INTEGER = 0x8D96,
	GL_RG_INTEGER = 0x8228, 
	GL_RGB_INTEGER = 0x8D98,
	GL_RGBA_INTEGER = 0x8D99,
	GL_BGR_INTEGER = 0x8D9A,
	GL_BGRA_INTEGER = 0x8D9B
}

// 2.3 glType
#[derive(Debug, FromPrimitive)]
#[allow(non_camel_case_types)]
pub enum GLType {
	GL_UNSIGNED_BYTE = 0x1401,
	GL_BYTE = 0x1400,
	GL_UNSIGNED_SHORT = 0x1403,
	GL_SHORT = 0x1402,
	GL_UNSIGNED_INT = 0x1405,
	GL_INT = 0x1404,
	GL_HALF_FLOAT = 0x140B,
	GL_FLOAT = 0x1406,
	GL_UNSIGNED_BYTE_3_3_2 = 0x8032,
	GL_UNSIGNED_BYTE_2_3_3_REV = 0x8362,
	GL_UNSIGNED_SHORT_5_6_5 = 0x8363,
	GL_UNSIGNED_SHORT_5_6_5_REV = 0x8364,
	GL_UNSIGNED_SHORT_4_4_4_4 = 0x8033,
	GL_UNSIGNED_SHORT_4_4_4_4_REV = 0x8365,
	GL_UNSIGNED_SHORT_5_5_5_1 = 0x8034,
	GL_UNSIGNED_SHORT_1_5_5_5_REV = 0x8366,
	GL_UNSIGNED_INT_8_8_8_8 = 0x8035,
	GL_UNSIGNED_INT_8_8_8_8_REV = 0x8367,
	GL_UNSIGNED_INT_10_10_10_2 = 0x8036,
	GL_UNSIGNED_INT_2_10_10_10_REV = 0x8368,
	GL_UNSIGNED_INT_24_8 = 0x84FA,
	GL_UNSIGNED_INT_10F_11F_11F_REV = 0x8C3B,
	GL_UNSIGNED_INT_5_9_9_9_REV = 0x8C3E,
	GL_FLOAT_32_UNSIGNED_INT_24_8_REV = 0x8DAD
}

pub struct FormatSize {
	pub flags: u32,
	pub	palette_size_in_bits: usize,
	pub	block_size_in_bits: usize,
	pub	block_width: usize,
	pub	block_height: usize,
	pub	block_depth: usize
}

// From https://github.com/KhronosGroup/KTX-Software/blob/dbfc3ed538cbe0839039fceb09d6c2be8aede67b/lib/gl_format.h
pub fn get_format_size(sized_internal_format: GLSizedInternalFormat) -> FormatSize {
	match sized_internal_format {
		GLSizedInternalFormat::GL_R8 | GLSizedInternalFormat::GL_R8_SNORM | GLSizedInternalFormat::GL_R8UI => FormatSize {
			flags: 0,
			palette_size_in_bits: 0,
			block_size_in_bits: 1 * 8,
			block_width: 1,
			block_height: 1,
			block_depth: 1
		},
		GLSizedInternalFormat::GL_R11F_G11F_B10F | GLSizedInternalFormat::GL_RGB9_E5 => FormatSize {
			flags: 0,
			palette_size_in_bits: 0,
			block_size_in_bits: 32,
			block_width: 1,
			block_height: 1,
			block_depth: 1
		},
		_ => panic!("Invalid size")
	
		// TODO
/*
		//
		// 8 bits per component
		//
		case GL_R8:												// 1-component, 8-bit unsigned normalized
		case GL_R8_SNORM:										// 1-component, 8-bit signed normalized
		case GL_R8UI:											// 1-component, 8-bit unsigned integer
		case GL_R8I:											// 1-component, 8-bit signed integer
		case GL_SR8:											// 1-component, 8-bit sRGB
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 1 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RG8:											// 2-component, 8-bit unsigned normalized
		case GL_RG8_SNORM:										// 2-component, 8-bit signed normalized
		case GL_RG8UI:											// 2-component, 8-bit unsigned integer
		case GL_RG8I:											// 2-component, 8-bit signed integer
		case GL_SRG8:											// 2-component, 8-bit sRGB
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 2 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB8:											// 3-component, 8-bit unsigned normalized
		case GL_RGB8_SNORM:										// 3-component, 8-bit signed normalized
		case GL_RGB8UI:											// 3-component, 8-bit unsigned integer
		case GL_RGB8I:											// 3-component, 8-bit signed integer
		case GL_SRGB8:											// 3-component, 8-bit sRGB
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 3 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA8:											// 4-component, 8-bit unsigned normalized
		case GL_RGBA8_SNORM:									// 4-component, 8-bit signed normalized
		case GL_RGBA8UI:										// 4-component, 8-bit unsigned integer
		case GL_RGBA8I:											// 4-component, 8-bit signed integer
		case GL_SRGB8_ALPHA8:									// 4-component, 8-bit sRGB
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 4 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		//
		// 16 bits per component
		//
		case GL_R16:											// 1-component, 16-bit unsigned normalized
		case GL_R16_SNORM:										// 1-component, 16-bit signed normalized
		case GL_R16UI:											// 1-component, 16-bit unsigned integer
		case GL_R16I:											// 1-component, 16-bit signed integer
		case GL_R16F:											// 1-component, 16-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 2 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RG16:											// 2-component, 16-bit unsigned normalized
		case GL_RG16_SNORM:										// 2-component, 16-bit signed normalized
		case GL_RG16UI:											// 2-component, 16-bit unsigned integer
		case GL_RG16I:											// 2-component, 16-bit signed integer
		case GL_RG16F:											// 2-component, 16-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 4 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB16:											// 3-component, 16-bit unsigned normalized
		case GL_RGB16_SNORM:									// 3-component, 16-bit signed normalized
		case GL_RGB16UI:										// 3-component, 16-bit unsigned integer
		case GL_RGB16I:											// 3-component, 16-bit signed integer
		case GL_RGB16F:											// 3-component, 16-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 6 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA16:											// 4-component, 16-bit unsigned normalized
		case GL_RGBA16_SNORM:									// 4-component, 16-bit signed normalized
		case GL_RGBA16UI:										// 4-component, 16-bit unsigned integer
		case GL_RGBA16I:										// 4-component, 16-bit signed integer
		case GL_RGBA16F:										// 4-component, 16-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		//
		// 32 bits per component
		//
		case GL_R32UI:											// 1-component, 32-bit unsigned integer
		case GL_R32I:											// 1-component, 32-bit signed integer
		case GL_R32F:											// 1-component, 32-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 4 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RG32UI:											// 2-component, 32-bit unsigned integer
		case GL_RG32I:											// 2-component, 32-bit signed integer
		case GL_RG32F:											// 2-component, 32-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB32UI:										// 3-component, 32-bit unsigned integer
		case GL_RGB32I:											// 3-component, 32-bit signed integer
		case GL_RGB32F:											// 3-component, 32-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 12 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA32UI:										// 4-component, 32-bit unsigned integer
		case GL_RGBA32I:										// 4-component, 32-bit signed integer
		case GL_RGBA32F:										// 4-component, 32-bit floating-point
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16 * 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		//
		// Packed
		//
		case GL_R3_G3_B2:										// 3-component 3:3:2, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB4:											// 3-component 4:4:4, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 12;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB5:											// 3-component 5:5:5, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB565:											// 3-component 5:6:5, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB10:											// 3-component 10:10:10, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB12:											// 3-component 12:12:12, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 36;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA2:											// 4-component 2:2:2:2, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA4:											// 4-component 4:4:4:4, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGBA12:											// 4-component 12:12:12:12, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 48;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB5_A1:										// 4-component 5:5:5:1, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB10_A2:										// 4-component 10:10:10:2, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_RGB10_A2UI:										// 4-component 10:10:10:2, unsigned integer
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_R11F_G11F_B10F:									// 3-component 11:11:10, floating-point
		case GL_RGB9_E5:										// 3-component/exp 9:9:9/5, floating-point
			pFormatSize->flags = GL_FORMAT_SIZE_PACKED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		//
		// S3TC/DXT/BC
		//
		case GL_COMPRESSED_RGB_S3TC_DXT1_EXT:					// line through 3D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_RGBA_S3TC_DXT1_EXT:					// line through 3D space plus 1-bit alpha, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_S3TC_DXT1_EXT:					// line through 3D space, 4x4 blocks, sRGB
		case GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT:			// line through 3D space plus 1-bit alpha, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_S3TC_DXT5_EXT:					// line through 3D space plus line through 1D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_RGBA_S3TC_DXT3_EXT:					// line through 3D space plus 4-bit alpha, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT:			// line through 3D space plus line through 1D space, 4x4 blocks, sRGB
		case GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT:			// line through 3D space plus 4-bit alpha, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		case GL_COMPRESSED_LUMINANCE_LATC1_EXT:					// line through 1D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_LUMINANCE_LATC1_EXT:			// line through 1D space, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_LUMINANCE_ALPHA_LATC2_EXT:			// two lines through 1D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_LUMINANCE_ALPHA_LATC2_EXT:	// two lines through 1D space, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		case GL_COMPRESSED_RED_RGTC1:							// line through 1D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_RED_RGTC1:					// line through 1D space, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RG_RGTC2:							// two lines through 1D space, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_RG_RGTC2:						// two lines through 1D space, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		case GL_COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT:				// 3-component, 4x4 blocks, unsigned floating-point
		case GL_COMPRESSED_RGB_BPTC_SIGNED_FLOAT:				// 3-component, 4x4 blocks, signed floating-point
		case GL_COMPRESSED_RGBA_BPTC_UNORM:						// 4-component, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_BPTC_UNORM:				// 4-component, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		//
		// ETC
		//
		case GL_ETC1_RGB8_OES:									// 3-component ETC1, 4x4 blocks, unsigned normalized" ),
		case GL_COMPRESSED_RGB8_ETC2:							// 3-component ETC2, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ETC2:							// 3-component ETC2, 4x4 blocks, sRGB
		case GL_COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2:		// 4-component ETC2 with 1-bit alpha, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2:		// 4-component ETC2 with 1-bit alpha, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA8_ETC2_EAC:						// 4-component ETC2, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ETC2_EAC:				// 4-component ETC2, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		case GL_COMPRESSED_R11_EAC:								// 1-component ETC, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_R11_EAC:						// 1-component ETC, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RG11_EAC:							// 2-component ETC, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SIGNED_RG11_EAC:						// 2-component ETC, 4x4 blocks, signed normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		//
		// PVRTC
		//
		case GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG:				// 3-component PVRTC, 16x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_PVRTC_2BPPV1_EXT:				// 3-component PVRTC, 16x8 blocks, sRGB
		case GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG:				// 4-component PVRTC, 16x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_PVRTC_2BPPV1_EXT:			// 4-component PVRTC, 16x8 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 16;
			pFormatSize->blockHeight = 8;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG:				// 3-component PVRTC, 8x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_PVRTC_4BPPV1_EXT:				// 3-component PVRTC, 8x8 blocks, sRGB
		case GL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG:				// 4-component PVRTC, 8x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_PVRTC_4BPPV1_EXT:			// 4-component PVRTC, 8x8 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 8;
			pFormatSize->blockHeight = 8;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_PVRTC_2BPPV2_IMG:				// 4-component PVRTC, 8x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_PVRTC_2BPPV2_IMG:			// 4-component PVRTC, 8x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 8;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_PVRTC_4BPPV2_IMG:				// 4-component PVRTC, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB_ALPHA_PVRTC_4BPPV2_IMG:			// 4-component PVRTC, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		//
		// ASTC
		//
		case GL_COMPRESSED_RGBA_ASTC_4x4_KHR:					// 4-component ASTC, 4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR:			// 4-component ASTC, 4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_5x4_KHR:					// 4-component ASTC, 5x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR:			// 4-component ASTC, 5x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 5;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_5x5_KHR:					// 4-component ASTC, 5x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR:			// 4-component ASTC, 5x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 5;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_6x5_KHR:					// 4-component ASTC, 6x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR:			// 4-component ASTC, 6x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 6;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_6x6_KHR:					// 4-component ASTC, 6x6 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR:			// 4-component ASTC, 6x6 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 6;
			pFormatSize->blockHeight = 6;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_8x5_KHR:					// 4-component ASTC, 8x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR:			// 4-component ASTC, 8x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 8;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_8x6_KHR:					// 4-component ASTC, 8x6 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR:			// 4-component ASTC, 8x6 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 8;
			pFormatSize->blockHeight = 6;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_8x8_KHR:					// 4-component ASTC, 8x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR:			// 4-component ASTC, 8x8 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 8;
			pFormatSize->blockHeight = 8;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_10x5_KHR:					// 4-component ASTC, 10x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR:			// 4-component ASTC, 10x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 10;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_10x6_KHR:					// 4-component ASTC, 10x6 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR:			// 4-component ASTC, 10x6 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 10;
			pFormatSize->blockHeight = 6;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_10x8_KHR:					// 4-component ASTC, 10x8 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR:			// 4-component ASTC, 10x8 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 10;
			pFormatSize->blockHeight = 8;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_10x10_KHR:					// 4-component ASTC, 10x10 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR:			// 4-component ASTC, 10x10 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 10;
			pFormatSize->blockHeight = 10;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_12x10_KHR:					// 4-component ASTC, 12x10 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR:			// 4-component ASTC, 12x10 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 12;
			pFormatSize->blockHeight = 10;
			pFormatSize->blockDepth = 1;
			break;
		case GL_COMPRESSED_RGBA_ASTC_12x12_KHR:					// 4-component ASTC, 12x12 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR:			// 4-component ASTC, 12x12 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 12;
			pFormatSize->blockHeight = 12;
			pFormatSize->blockDepth = 1;
			break;

		case GL_COMPRESSED_RGBA_ASTC_3x3x3_OES:					// 4-component ASTC, 3x3x3 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_3x3x3_OES:			// 4-component ASTC, 3x3x3 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 3;
			pFormatSize->blockHeight = 3;
			pFormatSize->blockDepth = 3;
			break;
		case GL_COMPRESSED_RGBA_ASTC_4x3x3_OES:					// 4-component ASTC, 4x3x3 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x3x3_OES:			// 4-component ASTC, 4x3x3 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 3;
			pFormatSize->blockDepth = 3;
			break;
		case GL_COMPRESSED_RGBA_ASTC_4x4x3_OES:					// 4-component ASTC, 4x4x3 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x4x3_OES:			// 4-component ASTC, 4x4x3 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 3;
			break;
		case GL_COMPRESSED_RGBA_ASTC_4x4x4_OES:					// 4-component ASTC, 4x4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_4x4x4_OES:			// 4-component ASTC, 4x4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 4;
			break;
		case GL_COMPRESSED_RGBA_ASTC_5x4x4_OES:					// 4-component ASTC, 5x4x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x4x4_OES:			// 4-component ASTC, 5x4x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 5;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 4;
			break;
		case GL_COMPRESSED_RGBA_ASTC_5x5x4_OES:					// 4-component ASTC, 5x5x4 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x5x4_OES:			// 4-component ASTC, 5x5x4 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 5;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 4;
			break;
		case GL_COMPRESSED_RGBA_ASTC_5x5x5_OES:					// 4-component ASTC, 5x5x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_5x5x5_OES:			// 4-component ASTC, 5x5x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 5;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 5;
			break;
		case GL_COMPRESSED_RGBA_ASTC_6x5x5_OES:					// 4-component ASTC, 6x5x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x5x5_OES:			// 4-component ASTC, 6x5x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 6;
			pFormatSize->blockHeight = 5;
			pFormatSize->blockDepth = 5;
			break;
		case GL_COMPRESSED_RGBA_ASTC_6x6x5_OES:					// 4-component ASTC, 6x6x5 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x6x5_OES:			// 4-component ASTC, 6x6x5 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 6;
			pFormatSize->blockHeight = 6;
			pFormatSize->blockDepth = 5;
			break;
		case GL_COMPRESSED_RGBA_ASTC_6x6x6_OES:					// 4-component ASTC, 6x6x6 blocks, unsigned normalized
		case GL_COMPRESSED_SRGB8_ALPHA8_ASTC_6x6x6_OES:			// 4-component ASTC, 6x6x6 blocks, sRGB
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 6;
			pFormatSize->blockHeight = 6;
			pFormatSize->blockDepth = 6;
			break;

		//
		// ATC
		//
		case GL_ATC_RGB_AMD:									// 3-component, 4x4 blocks, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;
		case GL_ATC_RGBA_EXPLICIT_ALPHA_AMD:					// 4-component, 4x4 blocks, unsigned normalized
		case GL_ATC_RGBA_INTERPOLATED_ALPHA_AMD:				// 4-component, 4x4 blocks, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_COMPRESSED_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 128;
			pFormatSize->blockWidth = 4;
			pFormatSize->blockHeight = 4;
			pFormatSize->blockDepth = 1;
			break;

		//
		// Palletized
		//
		case GL_PALETTE4_RGB8_OES:								// 3-component 8:8:8,   4-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 16 * 24;
			pFormatSize->blockSizeInBits = 4;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_PALETTE4_RGBA8_OES:								// 4-component 8:8:8:8, 4-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 16 * 32;
			pFormatSize->blockSizeInBits = 4;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_PALETTE4_R5_G6_B5_OES:							// 3-component 5:6:5,   4-bit palette, unsigned normalized
		case GL_PALETTE4_RGBA4_OES:								// 4-component 4:4:4:4, 4-bit palette, unsigned normalized
		case GL_PALETTE4_RGB5_A1_OES:							// 4-component 5:5:5:1, 4-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 16 * 16;
			pFormatSize->blockSizeInBits = 4;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_PALETTE8_RGB8_OES:								// 3-component 8:8:8,   8-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 256 * 24;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_PALETTE8_RGBA8_OES:								// 4-component 8:8:8:8, 8-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 256 * 32;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_PALETTE8_R5_G6_B5_OES:							// 3-component 5:6:5,   8-bit palette, unsigned normalized
		case GL_PALETTE8_RGBA4_OES:								// 4-component 4:4:4:4, 8-bit palette, unsigned normalized
		case GL_PALETTE8_RGB5_A1_OES:							// 4-component 5:5:5:1, 8-bit palette, unsigned normalized
			pFormatSize->flags = GL_FORMAT_SIZE_PALETTIZED_BIT;
			pFormatSize->paletteSizeInBits = 256 * 16;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		//
		// Depth/stencil
		//
		case GL_DEPTH_COMPONENT16:
			pFormatSize->flags = GL_FORMAT_SIZE_DEPTH_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_DEPTH_COMPONENT24:
		case GL_DEPTH_COMPONENT32:
		case GL_DEPTH_COMPONENT32F:
		case GL_DEPTH_COMPONENT32F_NV:
			pFormatSize->flags = GL_FORMAT_SIZE_DEPTH_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_STENCIL_INDEX1:
			pFormatSize->flags = GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 1;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_STENCIL_INDEX4:
			pFormatSize->flags = GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 4;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_STENCIL_INDEX8:
			pFormatSize->flags = GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_STENCIL_INDEX16:
			pFormatSize->flags = GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 16;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_DEPTH24_STENCIL8:
			pFormatSize->flags = GL_FORMAT_SIZE_DEPTH_BIT | GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 32;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
		case GL_DEPTH32F_STENCIL8:
		case GL_DEPTH32F_STENCIL8_NV:
			pFormatSize->flags = GL_FORMAT_SIZE_DEPTH_BIT | GL_FORMAT_SIZE_STENCIL_BIT;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 64;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;

		default:
			pFormatSize->flags = 0;
			pFormatSize->paletteSizeInBits = 0;
			pFormatSize->blockSizeInBits = 8;
			pFormatSize->blockWidth = 1;
			pFormatSize->blockHeight = 1;
			pFormatSize->blockDepth = 1;
			break;
			*/
	}
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtxHeader {
    pub gl_type: u32,
    pub gl_type_size: u32,
    pub gl_format: u32,
    pub gl_internal_format: u32,
    pub gl_base_internal_format: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub number_of_array_elements: u32,
    pub number_of_faces: u32,
    pub number_of_mipmap_levels: u32,
    pub bytes_of_key_value_data: u32,
}
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtxFaceData {
	pub data: Vec<u8>
}
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtxArrayElement {
	pub faces: Vec<KtxFaceData>
}
#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct KtxMipmapLevel {
	pub image_size: u32,
	pub array_elements: Vec<KtxArrayElement>,
	pub data: Vec<u8>
}
impl std::fmt::Debug for KtxMipmapLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "KtxMipmapLevel {{ image_size: {}, array_elements: {:?}, data: xxx len {} }}", self.image_size, self.array_elements, self.data.len())
    }
}
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtxFile {
	pub header: KtxHeader,
	pub key_value_pairs: std::collections::HashMap<String, Vec<u8>>,
	pub data: Option<Vec<KtxMipmapLevel>>
}

const KTX_GL_UNPACK_ALIGNMENT: usize = 4;

pub fn ktx_padn_len(n: usize, nbytes: usize) -> usize {
	((n-1) - (nbytes + (n-1) & (n-1)))
}

pub fn ktx_pad_unpack_align_len(nbytes: usize) -> usize {
	ktx_padn_len(KTX_GL_UNPACK_ALIGNMENT, nbytes)
}

impl KtxFaceData {
	pub fn deserialize<Endianness: byteorder::ByteOrder, R: std::io::Read + std::io::Seek>(header: &KtxHeader, source: &mut R, data: &mut Vec<u8>, size_in_bytes: usize) -> Result<KtxFaceData, io::Error>
    {
		let ret_val = KtxFaceData {
			data: Vec::<u8>::new()//with_capacity(size_in_bytes / std::mem::size_of::<i32>())
		};

		/*for byte in 0..size_in_bytes / std::mem::size_of::<i32>() {
			ret_val.data.push(source.read_u32::<Endianness>()?);
		}*/
		for _byte in 0..size_in_bytes {
			data.push(source.read_u8()?);
		}

		let current_offset = size_in_bytes as i64;

		// 2.17 "For non-array cubemap textures (any texture where numberOfFaces is 6 and numberOfArrayElements is 0) cubePadding contains between 0 and 3 bytes of value 0x00 to ensure that the data in each face begins at a file offset that is a multiple of 4. In all other cases cubePadding is empty (0 bytes long)."
		let has_cube_padding = header.number_of_faces == 6 && header.number_of_array_elements == 0;

		if has_cube_padding && current_offset % 4 != 0 {
			// 2.17 cubePadding
			source.seek(std::io::SeekFrom::Current(current_offset % 4));
		}

		Ok(ret_val)
	}
}

impl KtxArrayElement {
	pub fn deserialize<Endianness: byteorder::ByteOrder, R: std::io::Read + std::io::Seek>(header: &KtxHeader, source: &mut R, data: &mut Vec<u8>, size_in_bytes: usize) -> Result<KtxArrayElement, io::Error>
    {
		let mut ret_val = KtxArrayElement {
			faces: Vec::<KtxFaceData>::new()
		};

		for _face in 0..header.number_of_faces {
			ret_val.faces.push(KtxFaceData::deserialize::<Endianness, R>(header, source, data, size_in_bytes)?);
		}

		Ok(ret_val)
	}
}

impl KtxMipmapLevel {
	pub fn deserialize<Endianness: byteorder::ByteOrder, R: std::io::Read + std::io::Seek>(header: &KtxHeader, source: &mut R, size_in_bytes: usize) -> Result<KtxMipmapLevel, io::Error>
    {
		let mut ret_val = KtxMipmapLevel {
			image_size: source.read_u32::<Endianness>()?,
			array_elements: Vec::<KtxArrayElement>::new(),
			
			data: Vec::<u8>::with_capacity(size_in_bytes * header.number_of_array_elements as usize * header.number_of_faces as usize)
		};
		
		// "Replace with 1 if this field is 0."
		let number_of_array_elements = std::cmp::max(header.number_of_array_elements, 1);
		
		for _array_element in 0..number_of_array_elements {
			ret_val.array_elements.push(KtxArrayElement::deserialize::<Endianness, R>(header, source, &mut ret_val.data, size_in_bytes)?);
		}
		
		if ret_val.image_size % 4 != 0 {
			// 2.18 mipPadding
			source.seek(std::io::SeekFrom::Current((ret_val.image_size % 4) as i64));
		}
		
		Ok(ret_val)
	}
}

impl KtxFile {
    pub fn deserialize<R: std::io::Read + std::io::Seek>(source: &mut R) -> Result<KtxFile, io::Error>
    {
        // Read identifier
        let mut buffer: [u8; 12] = [0; 12];
        source.read_exact(&mut buffer)?;
        if buffer != FILE_IDENTIFIER {
            return Err(io::Error::new(io::ErrorKind::Other, "File is not a KTX file."));
        }

        // Read endianness
        let mut buffer: [u8; 4] = [0; 4];
        source.read_exact(&mut buffer)?;
        let little_endian: bool = match buffer[0] {
            0x01 => true,
            0x04 => false,
            _ => return Err(io::Error::new(io::ErrorKind::Other, "Invalid KTX file.")),
        };

        if little_endian {
            KtxFile::deserialize_file::<byteorder::LittleEndian, R>(source)
        } else {
            KtxFile::deserialize_file::<byteorder::BigEndian, R>(source)
        }
    }
	pub fn deserialize_file<Endianness: byteorder::ByteOrder, R: std::io::Read + std::io::Seek>(source: &mut R) -> Result<KtxFile, io::Error>
    {
        let mut ret_val = KtxFile {
            header: KtxHeader::deserialize::<Endianness, R>(source)?,
			key_value_pairs: std::collections::HashMap::<String, Vec<u8>>::new(),
			data: None
        };
		if ret_val.header.gl_type_size > 1 {
			// Need to convert texture endianness (2.2)
			ret_val.key_value_pairs = KtxFile::deserialize_key_value_pairs::<Endianness, R>(&ret_val.header, source)?;

			ret_val.data = Some(Vec::<KtxMipmapLevel>::new());
			for mipmap_level in 0..ret_val.header.number_of_mipmap_levels {
				let size_in_bytes = ret_val.header.element_face_slice_size(mipmap_level as usize);
				ret_val.data.as_mut().unwrap().push(KtxMipmapLevel::deserialize::<byteorder::NativeEndian, R>(&ret_val.header, source, size_in_bytes)?);
			}
		} else {
			// No need to convert endianness
			ret_val.key_value_pairs = KtxFile::deserialize_key_value_pairs::<byteorder::NativeEndian, R>(&ret_val.header, source)?;

			ret_val.data = Some(Vec::<KtxMipmapLevel>::new());
			for mipmap_level in 0..ret_val.header.number_of_mipmap_levels {
				let size_in_bytes = ret_val.header.element_face_slice_size(mipmap_level as usize);
				ret_val.data.as_mut().unwrap().push(KtxMipmapLevel::deserialize::<byteorder::NativeEndian, R>(&ret_val.header, source, size_in_bytes)?);
			}
		}
		Ok(ret_val)
    }
	pub fn deserialize_key_value_pairs<Endianness: byteorder::ByteOrder, R: std::io::Read + std::io::Seek>(header: &KtxHeader, source: &mut R) -> Result<std::collections::HashMap<String, Vec<u8>>, io::Error> {
		let mut current_offset = 0;
		let mut ret_val = std::collections::HashMap::<String, Vec<u8>>::new();
		
		while current_offset < header.bytes_of_key_value_data as usize {
		
			let key_and_value_byte_size = source.read_u32::<Endianness>()?;
			current_offset += std::mem::size_of::<u32>();
			
			let mut data = vec![0; key_and_value_byte_size as usize];
			source.read_exact(&mut data);
			current_offset += data.len();

			// https://www.khronos.org/opengles/sdk/tools/KTX/file_format_spec/
			source.seek(std::io::SeekFrom::Current(3 - (((key_and_value_byte_size as i64) + 3) % 4)));
			current_offset += (3 - (((key_and_value_byte_size as i64) + 3) % 4)) as usize;
			
			let nul_pos = data.iter().position(|x| *x == 0);
			let key = std::str::from_utf8(&data[0..(nul_pos.unwrap_or(data.len()))]).unwrap().to_owned();
			
			if let Some(nul_pos2) = nul_pos {
				ret_val.insert(key, data[nul_pos2..data.len()].to_vec());
			} else {
				ret_val.insert(key, vec![0;0]);
			}
			
		}
		Ok(ret_val)
	}
}

impl KtxHeader {
    pub fn deserialize<Endianness: byteorder::ByteOrder, R: std::io::Read>(source: &mut R) -> Result<KtxHeader, io::Error>
    {
        let ret_val = KtxHeader {
            gl_type: source.read_u32::<Endianness>()?,
            gl_type_size: source.read_u32::<Endianness>()?,
            gl_format: source.read_u32::<Endianness>()?,
            gl_internal_format: source.read_u32::<Endianness>()?,
            gl_base_internal_format: source.read_u32::<Endianness>()?,
            pixel_width: source.read_u32::<Endianness>()?,
            pixel_height: source.read_u32::<Endianness>()?,
            pixel_depth: source.read_u32::<Endianness>()?,
            number_of_array_elements: source.read_u32::<Endianness>()?,
            number_of_faces: source.read_u32::<Endianness>()?,
            number_of_mipmap_levels: source.read_u32::<Endianness>()?,
            bytes_of_key_value_data: source.read_u32::<Endianness>()?,
        };

		assert!((ret_val.gl_type > 0) == (ret_val.gl_format > 0)); // For compressed textures, glType must equal 0,  For compressed textures, glFormat must equal 0
		assert!(ret_val.number_of_faces == 1 || ret_val.number_of_faces == 6); // 2.10 numberOfFaces specifies the number of cubemap faces. For cubemaps and cubemap arrays this should be 6. For non cubemaps this should be 1.
		assert!(ret_val.number_of_mipmap_levels != 0); // TODO: generate mip levels: 2.11: If numberOfMipmapLevels equals 0, it indicates that a full mipmap pyramid should be generated from level 0 at load time (this is usually not allowed for compressed formats).
		
		assert!(ret_val.gl_type == 0 || GLType::from_u32(ret_val.gl_type).is_some());
		assert!(ret_val.gl_format == 0 || GLFormat::from_u32(ret_val.gl_format).is_some());
		assert!(GLBaseInternalFormat::from_u32(ret_val.gl_base_internal_format).is_some());
		assert!(GLSizedInternalFormat::from_u32(ret_val.gl_internal_format).is_some());

		println!("gl_type {:?}", GLType::from_u32(ret_val.gl_type));
		println!("gl_format {:?}", GLFormat::from_u32(ret_val.gl_format));
		println!("gl_base_internal_format {:?}", GLBaseInternalFormat::from_u32(ret_val.gl_base_internal_format));
		println!("gl_internal_format {:?}", GLSizedInternalFormat::from_u32(ret_val.gl_internal_format));

		Ok(ret_val)
    }

	pub fn is_compressed(&self) -> bool {
		self.gl_type != 0 // For compressed textures, glType must equal 0
	}

	pub fn is_array_texture(&self) -> bool {
		self.number_of_array_elements != 0 // 2.9 If the texture is not an array texture, numberOfArrayElements must equal 0.
	}

	pub fn is_1d_texture(&self) -> bool {
		self.pixel_height == 0 && self.pixel_depth == 0 // 2.8 For 1D textures pixelHeight and pixelDepth must be 0
	}

	pub fn is_2d_or_cube_texture(&self) -> bool {
		self.pixel_depth == 0 // 2.8 For 2D and cube textures pixelDepth must be 0.
	}
	
	pub fn element_face_slice_size(&self, mip_level: usize) -> usize {
		assert!(mip_level < self.number_of_mipmap_levels as usize);

		let block_count_x = std::cmp::max(1, self.pixel_width as usize) >> mip_level;
		let block_count_y = std::cmp::max(1, self.pixel_height as usize) >> mip_level;

		let block_size_in_bytes = get_format_size(GLSizedInternalFormat::from_u32(self.gl_internal_format).unwrap()).block_size_in_bits / 8;
		
		if self.is_compressed() {
			block_count_x * block_count_y * block_size_in_bytes
		} else {
			let mut row_bytes = block_count_x * block_size_in_bytes;
			row_bytes += ktx_pad_unpack_align_len(row_bytes);
			row_bytes * block_count_y
		}
	}

	pub fn image_offset(&self, mip_level: usize, array_element: usize, face: usize) -> usize {
		assert!(mip_level < self.number_of_mipmap_levels as usize);
		assert!(array_element < self.number_of_array_elements as usize);
		assert!(face < self.number_of_faces as usize);
		
		let mut offset = 0;
		for mip in 0..mip_level {
			let block_count_z = std::cmp::max(1, get_format_size(GLSizedInternalFormat::from_u32(self.gl_internal_format).unwrap()).block_depth >> mip);

			offset += self.element_face_slice_size(mip) * self.number_of_faces as usize * block_count_z * self.number_of_array_elements as usize;
		}

		offset += self.element_face_slice_size(mip_level) * array_element;
		offset += self.element_face_slice_size(mip_level) * face;
		
		offset
	}

	pub fn image_size_max(&self) -> usize {
		
		let mut offset = 0;
		for mip in 0..self.number_of_mipmap_levels as usize {
			let block_count_z = std::cmp::max(1, get_format_size(GLSizedInternalFormat::from_u32(self.gl_internal_format).unwrap()).block_depth >> mip);

			offset += self.element_face_slice_size(mip) * self.number_of_faces as usize * block_count_z * self.number_of_array_elements as usize;
		}

		offset += self.element_face_slice_size(self.number_of_mipmap_levels as usize - 1) * self.number_of_array_elements as usize;
		offset += self.element_face_slice_size(self.number_of_mipmap_levels as usize - 1) * self.number_of_faces as usize;
		
		offset
	}
	
	pub fn get_vk_format(&self) -> Option<vkraw::VkFormat> {
		// TODO
		match GLSizedInternalFormat::from_u32(self.gl_internal_format).unwrap() {
			GLSizedInternalFormat::GL_R11F_G11F_B10F => Some(vkraw::VkFormat::VK_FORMAT_B10G11R11_UFLOAT_PACK32),
			_ => None
		}
	}
}