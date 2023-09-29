use std::{env, path::PathBuf};
use which::which;

use bindgen::Formatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find the toolchain include path from the CC environment variable
    let cc = cc::Build::new().get_compiler();
    let gcc = cc
        .path()
        .to_str()
        .expect("Should be able to extract the name of the compiler");
    let gcc_absolute_path =
        which(gcc).expect("Should be able to find the compiler using the which function");
    let include_path = gcc_absolute_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(gcc.trim_end_matches("-gcc"))
        .join("include");

    // Generate the rust FFI bindings
    let mut builder = cc::Build::new();
    let target = env::var("TARGET")?;
    let builder = builder
        .flag("-std=c11")
        .flag("-DLFS_CONFIG=../src/littlefs_patches/lfs_util_sys.h")
        .flag("-DLFS_NO_MALLOC")
        .file("littlefs/lfs.c")
        .file("littlefs/lfs_util.c")
        .file("src/littlefs_patches/string.c");

    #[cfg(not(feature = "assertions"))]
    let builder = builder.flag("-DLFS_NO_ASSERT");

    #[cfg(feature = "trace")]
    let builder = builder
        .flag("-DLFS_YES_TRACE")
        .flag("-DLFS_YES_DEBUG")
        .flag("-DLFS_YES_WARN")
        .flag("-DLFS_YES_ERROR");

    #[cfg(feature = "debug")]
    let builder = builder
        .flag("-DLFS_YES_DEBUG")
        .flag("-DLFS_YES_WARN")
        .flag("-DLFS_YES_ERROR");
    #[cfg(feature = "warn")]
    let builder = builder.flag("-DLFS_YES_WARN").flag("-DLFS_YES_ERROR");
    #[cfg(feature = "error")]
    let builder = builder.flag("-DLFS_YES_ERROR");

    builder.compile("lfs-sys");

    let bindings = bindgen::Builder::default()
        .header("littlefs/lfs.h")
        .header("src/littlefs_patches/lfs_util_sys.h")
        .clang_arg(format!(
            "-I{}",
            include_path
                .to_str()
                .expect("Should be able to extract the include path")
        ))
        .clang_arg(format!("--target={}", target))
        .use_core()
        .ctypes_prefix("cty")
        .formatter(Formatter::Rustfmt)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
