#[repr(C)]
#[expect(unused)]
#[derive(Clone, Copy)]
enum LogLevel {
    Trace = 0,
    Debug = 1,
    Warn = 2,
    Error = 3,
}

#[expect(unused)]
unsafe extern "C" fn logger(log_level: LogLevel, message: *const u8, args: ...) {
    match log_level {
        LogLevel::Trace => log::trace!("{:?} {:?}", message, args),
        LogLevel::Debug => log::debug!("{:?} {:?}", message, args),
        LogLevel::Warn => log::warn!("{:?} {:?}", message, args),
        LogLevel::Error => log::error!("{:?} {:?}", message, args),
    }
}
