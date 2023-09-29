use core::ffi::CStr;
use heapless::Vec;
use log::Level;

#[repr(C)]
#[expect(unused)]
#[derive(Debug, Clone, Copy)]
enum LittleFsLogLevel {
    Trace = 0,
    Debug = 1,
    Warn = 2,
    Error = 3,
}

impl From<LittleFsLogLevel> for Level {
    fn from(log_level: LittleFsLogLevel) -> Self {
        match log_level {
            LittleFsLogLevel::Trace => Level::Trace,
            LittleFsLogLevel::Debug => Level::Debug,
            LittleFsLogLevel::Warn => Level::Warn,
            LittleFsLogLevel::Error => Level::Error,
        }
    }
}

#[no_mangle]
unsafe extern "C" fn log_msg(log_level: LittleFsLogLevel, message: *const u8, mut args: ...) {
    let message_cstr = CStr::from_ptr(message as *const i8)
        .to_str()
        .expect("Should be able to convert the C string to rust");
    log::log!(log_level.into(), "{}", message_cstr);

    let extracted_types = extract_types::<100>(message_cstr);
    let mut list = args.as_va_list();
    for (i, extracted_type) in extracted_types.iter().enumerate() {
        match extracted_type {
            ArgType::Integer => {
                let arg = list.arg::<i64>();
                log::log!(log_level.into(), "Argument {i}: {arg}");
            }
            ArgType::UnsignedInteger => {
                let arg = list.arg::<u64>();
                log::log!(log_level.into(), "Argument {i}: {arg}");
            }
            ArgType::UnsignedIntegerHex => {
                let arg = list.arg::<u64>();
                log::log!(log_level.into(), "Argument {i}: 0x{arg:X}");
            }
            ArgType::FloatingPoint => {
                let arg = list.arg::<f64>();
                log::log!(log_level.into(), "Argument {i}: {arg}");
            }
            _ => {
                log::log!(
                    log_level.into(),
                    "Unsupported argument type {i}: {extracted_type:?}"
                );
            }
        };
    }
}

#[derive(Debug, Clone)]
pub enum ArgType {
    Integer,
    UnsignedInteger,
    UnsignedIntegerHex,
    FloatingPoint,
    Character,
    String,
    Pointer,
    WriteBytes,
    Literal,
    Unknown,
}

fn extract_types<const Size: usize>(format_string: &str) -> Vec<ArgType, Size> {
    let mut types = Vec::<ArgType, Size>::new();

    let mut chars = format_string.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next_char) = chars.peek() {
                let arg_type = match next_char {
                    'd' | 'i' => ArgType::Integer,
                    'u' | 'o' => ArgType::UnsignedInteger,
                    'x' | 'X' => ArgType::UnsignedIntegerHex,
                    'f' | 'F' | 'e' | 'E' | 'g' | 'G' | 'a' | 'A' => ArgType::FloatingPoint,
                    'c' => ArgType::Character,
                    's' => ArgType::String,
                    'p' => ArgType::Pointer,
                    'n' => ArgType::WriteBytes,
                    '%' => ArgType::Literal,
                    _ => ArgType::Unknown,
                };
                types
                    .push(arg_type)
                    .expect("Should be able to push the argument type in the type list");
                chars.next();
            }
        }
    }
    types
}
