use core::{ffi::CStr, fmt};
use heapless::{String, Vec};
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
unsafe extern "C" fn log_msg(log_level: LittleFsLogLevel, message: *const i8, mut args: ...) {
    let message_cstr = CStr::from_ptr(message)
        .to_str()
        .expect("Should be able to convert the C string to rust");

    let mut message_str = String::<500>::new();

    let (extracted_types, extracted_strings) = extract_types::<100, 25>(message_cstr);
    let mut list = args.as_va_list();
    for (i, extracted_type) in extracted_types.iter().enumerate() {
        let _ = message_str.push_str(extracted_strings[i].as_str()); //FIXME: Check overflow ?
        match extracted_type {
            ArgType::Integer => {
                let arg = list.arg::<isize>();
                let _ = fmt::write(&mut message_str, format_args!("{arg}"));
            }
            ArgType::UnsignedInteger => {
                let arg = list.arg::<usize>();
                let _ = fmt::write(&mut message_str, format_args!("{arg}"));
            }
            ArgType::UnsignedIntegerHex => {
                let arg = list.arg::<usize>();
                let _ = fmt::write(&mut message_str, format_args!("0x{arg:X}"));
            }
            ArgType::FloatingPoint => {
                let arg = list.arg::<f64>();
                log::log!(log_level.into(), "{arg}");
            }
            ArgType::String => {
                if let Ok(arg) = CStr::from_ptr(list.arg::<*const i8>()).to_str() {
                    let _ = fmt::write(&mut message_str, format_args!("{arg}"));
                }
            }
            ArgType::Pointer => {
                let arg = list.arg::<*const usize>();
                let _ = fmt::write(&mut message_str, format_args!("{arg:?}"));
            }
            _ => {
                let _ = fmt::write(
                    &mut message_str,
                    format_args!("Unsupported argument type: {extracted_type:?}"),
                );
            }
        };
    }

    // If the extracted string list is bigger than the extracted types list, it means we have a remaining message after the last parameter.
    // It should always be 1 however.
    if extracted_strings.len() > extracted_types.len() {
        if let Some(last) = extracted_strings.last() {
            let _ = message_str.push_str(last.as_str()); //FIXME: Check overflow ?
        }
    }

    log::log!(log_level.into(), "{message_str}");
}

#[derive(Debug, Clone)]
struct DoublePeekIterator<I: Iterator> {
    iter: I,
    peeked1: Option<I::Item>,
    peeked2: Option<I::Item>,
}

impl<I: Iterator> DoublePeekIterator<I> {
    fn new(iter: I) -> Self {
        DoublePeekIterator {
            iter,
            peeked1: None,
            peeked2: None,
        }
    }

    fn peek(&mut self) -> Option<&I::Item> {
        if self.peeked1.is_none() {
            self.peeked1 = self.iter.next();
        }
        self.peeked1.as_ref()
    }

    fn peek2(&mut self) -> Option<&I::Item> {
        let _ = self.peek();
        if self.peeked2.is_none() {
            self.peeked2 = self.iter.next();
        }
        self.peeked2.as_ref()
    }
}

impl<I: Iterator> Iterator for DoublePeekIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.peeked1.take();
        self.peeked1 = self.peeked2.take();
        self.peeked2 = None;
        item.or_else(|| self.iter.next())
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

fn extract_types<const VecSize: usize, const StringSize: usize>(
    format_string: &str,
) -> (Vec<ArgType, VecSize>, Vec<String<StringSize>, VecSize>) {
    let mut types = Vec::<ArgType, VecSize>::new();
    let mut strings = Vec::<String<StringSize>, VecSize>::new();
    let mut pre_type_string = String::<StringSize>::new();

    let mut chars = DoublePeekIterator::new(format_string.chars());
    while let Some(c) = chars.next() {
        if chars.clone().count() < 2 {
            break;
        }
        if c == '%' {
            if let Some(&next_char) = chars.peek() {
                let arg_type = match next_char {
                    'l' => {
                        if let Some(&next_char) = chars.peek2() {
                            match next_char {
                                'd' | 'i' => ArgType::Integer,
                                'u' | 'o' => ArgType::UnsignedInteger,
                                _ => ArgType::Unknown,
                            }
                        } else {
                            ArgType::Unknown
                        }
                    }
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
                let _ = types.push(arg_type); //FIXME: Check overflow ?
                chars.next();
                let _ = strings.push(pre_type_string); //FIXME: Check overflow ?
                pre_type_string = String::<StringSize>::new();
            }
        } else {
            let _ = pre_type_string.push(c); //FIXME: Check overflow ?
        }
    }
    if !pre_type_string.is_empty() {
        let _ = strings.push(pre_type_string); //FIXME: Check overflow ?
    }
    (types, strings)
}
