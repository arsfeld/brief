fn main() {
    tauri_build::build();

    // On macOS, screencapturekit's Swift bridge depends on libswift_Concurrency.dylib.
    // The screencapturekit crate adds this rpath via its own build script, but Cargo
    // doesn't propagate cargo:rustc-link-arg from transitive dependencies, so we must
    // add it here at the top-level package where link args are honoured.
    #[cfg(target_os = "macos")]
    {
        // /usr/lib/swift is in the macOS dyld shared cache on macOS 12+
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}
