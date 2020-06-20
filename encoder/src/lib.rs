#[macro_use]
extern crate log;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use anyhow::{Result, Error};

struct Encoder {
    width: i32,
    height: i32,
}

#[no_mangle]
pub extern "C" fn new_encoder(width: i32, height: i32) -> *mut c_void {
    let e = Encoder{width, height};
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn free_encoder(encoder: *mut c_void) {
}

#[no_mangle]
pub extern "C" fn encode_frame(encoder: *mut c_void, frame: *const c_char) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn double(value: i32) -> i32 {
    value * 2
}



