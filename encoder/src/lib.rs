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
    Box::into_raw(Box::new(Encoder{width, height})) as _
}

#[no_mangle]
pub extern "C" fn free_encoder(encoder: *mut c_void) {
    unsafe {
        Box::from_raw(encoder as *mut Encoder);
    }
}

#[no_mangle]
pub extern "C" fn encode_frame(encoder: *mut c_void, frame: *const c_char) -> i32 {
    let encoder: &mut Encoder = unsafe { &mut *(encoder as *mut Encoder) };
    0
}

#[no_mangle]
pub extern "C" fn mul(x: i32, y: i32) -> i32 {
    x * y
}



