// use sdl3_sys::audio::SDL_AudioFormat;
//
// #[repr(C)]
// pub struct FLAC__StreamEncoder {
//     _opaque: [::core::primitive::u8; 0],
// }
//
// #[repr(C)]
// pub struct FLAC__StreamMetadata {
//     _opaque: [::core::primitive::u8; 0],
// }
//
// #[repr(transparent)]
// #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct FLAC__MetadataType(pub ::core::ffi::c_uint);
//
// // see libflac's format.h
// impl FLAC__MetadataType {
//     pub const FLAC__METADATA_TYPE_STREAMINFO: Self = Self(0x0000);
//     pub const FLAC__METADATA_TYPE_PADDING: Self = Self(0x0001);
//     pub const FLAC__METADATA_TYPE_APPLICATION: Self = Self(0x0002);
//     pub const FLAC__METADATA_TYPE_SEEKTABLE: Self = Self(0x0003);
//     pub const FLAC__METADATA_TYPE_VORBIS_COMMENT: Self = Self(0x0004);
//     pub const FLAC__METADATA_TYPE_CUESHEET: Self = Self(0x0005);
//     pub const FLAC__METADATA_TYPE_PICTURE: Self = Self(0x0006);
//     pub const FLAC__METADATA_TYPE_UNDEFINED: Self = Self(0x0007);
// }
//
// #[link(name = "FLAC")]
// // #[link(name = "FLAC", kind = "static")]
// extern {
//
//
//     //todo can I use rust types for ffi if they're supposed to be the same??? ie bool and u32?
//     pub fn FLAC__stream_encoder_new() -> *mut FLAC__StreamEncoder;
//     pub fn FLAC__stream_encoder_set_compression_level(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, level: u32) -> bool;
//     pub fn FLAC__stream_encoder_set_channels(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, level: u32) -> bool;
//     pub fn FLAC__stream_encoder_set_bits_per_sample(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, level: u32) -> bool;
//     pub fn FLAC__stream_encoder_set_sample_rate(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, level: u32) -> bool;
//
//     pub fn FLAC__metadata_object_new(r#type: FLAC__MetadataType) -> *mut FLAC__StreamMetadata;
//     pub fn FLAC__stream_encoder_set_metadata(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, metadata: *mut *mut FLAC__StreamMetadata, num_blocks: u32) -> bool;
//
//
//     pub fn FLAC__stream_encoder_init_file(FLAC__StreamEncoder: *mut FLAC__StreamEncoder, ) -> bool;
// }

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
// todo NOTE that this emits many warnings such as these:
// todo "warning: `extern` block uses type `u128`, which is not FFI-safe"
// todo this is not actually a problem anymore, but the warning remains for now, see:\
// todo https://github.com/rust-lang/lang-team/issues/255
// todo this attribute to hides these warnings (and possibly others which are not safe...)
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
