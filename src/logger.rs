#[no_mangle]
unsafe extern "C" fn log_trace(message: *const u8, args: ...) {
    log::trace!("{:?} {:?}", message, args);
}

#[no_mangle]
unsafe extern "C" fn log_debug(message: *const u8, args: ...) {
    log::debug!("{:?} {:?}", message, args);
}
#[no_mangle]
unsafe extern "C" fn log_warn(message: *const u8, args: ...) {
    log::warn!("{:?} {:?}", message, args);
}
#[no_mangle]
unsafe extern "C" fn log_error(message: *const u8, args: ...) {
    log::error!("{:?} {:?}", message, args);
}
