use std::ffi::c_void;
use rav1e::{Context, EncoderConfig, Config, config::SpeedSettings};

pub struct Service {
    ctx: Context<u16>,
}

#[no_mangle]
pub extern fn new_service(width: u32, height: u32, endpoint: *const u8) -> *mut c_void {
    let mut enc = EncoderConfig::default();
    enc.width = width as _;
    enc.height = height as _;
    enc.speed_settings = SpeedSettings::from_preset(9);
    let cfg = Config::new().with_encoder_config(enc);
    let ctx: Context<u16> = cfg.new_context().unwrap();
    Box::into_raw(Box::new(Service{ ctx })) as _
}

#[no_mangle]
pub extern fn free_service(svc: *mut c_void) {
    unsafe {
        Box::from_raw(svc as *mut Service);
    }
}

#[no_mangle]
pub extern fn send_frame(svc: *mut c_void, data: *const u8) {
    let svc = unsafe { &mut *(svc as *mut Service) };
}
