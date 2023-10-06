#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]
#![feature(lint_reasons)]

#[cfg(not(feature = "no-log"))]
mod logger;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
