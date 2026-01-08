use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=FFMS_LIB_DIR");
    println!("cargo:rerun-if-env-changed=FFMS_INCLUDE_DIR");

    if cfg!(target_os = "windows") {
        let ffmpeg_found = vcpkg::Config::new()
            .emit_includes(true)
            .find_package("ffmpeg")
            .is_ok();

        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
        let is_static = target_features.split(',').any(|f| f == "crt-static");

        if ffmpeg_found && target_arch == "x86_64" && is_static {
            let sys_libs = [
                "bcrypt", "mfuuid", "strmiids", "advapi32", "crypt32", "user32", "ole32",
            ];
            for lib in sys_libs {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }

        if let Ok(lib_dir) = env::var("FFMS_LIB_DIR") {
            println!("cargo:rustc-link-search=native={}", lib_dir);
        }
        println!("cargo:rustc-link-lib=static=ffms2");
    }
}
