use std::ffi::{CString, CStr, c_void};
use std::os::raw::c_char;
use rav1e::{Context, EncoderConfig, Config, config::SpeedSettings};

pub struct Service {
    width: usize,
    height: usize,
    ctx: Context<u16>,
}

impl Service {
    pub fn new(width: usize, height: usize, endpoint: &str) -> Self {
        let mut enc = EncoderConfig::default();
        enc.width = width;
        enc.height = height;
        enc.speed_settings = SpeedSettings::from_preset(9);
        let cfg = Config::new().with_encoder_config(enc);
        let ctx: Context<u16> = cfg.new_context().unwrap();
        Self {
            width,
            height,
            ctx,
        }
    }

    pub fn send_frame(&self, data: &[u8]) {}
}

#[no_mangle]
pub extern fn new_service(width: u32, height: u32, endpoint: *const c_char) -> *mut Service {
    let endpoint = unsafe { CStr::from_ptr(endpoint) }.to_str().unwrap();
    Box::into_raw(Box::new(Service::new(width as _, height as _, endpoint)))
}

#[no_mangle]
pub extern fn free_service(svc: *mut Service) {
    unsafe {
        Box::from_raw(svc);
    }
}

#[no_mangle]
pub extern fn send_frame(svc: *mut Service, data: *const u8) {
    let svc = unsafe { &mut *svc };
    let data = unsafe { std::slice::from_raw_parts(data, svc.width * svc.height * 3) };
    svc.send_frame(data);
}
