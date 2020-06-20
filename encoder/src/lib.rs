#[macro_use]
extern crate log;
use std::ffi::{c_void, CStr};
use anyhow::{Result, Error};

#[no_mangle]
pub extern "C" fn rust_initialize_vad() -> i32 {
    0
}
