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
