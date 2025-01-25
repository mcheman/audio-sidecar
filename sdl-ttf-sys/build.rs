use std::env;
use std::path::PathBuf;

const VERSION: &str = "main";

// this static links
fn main() {
    let lib_dir = format!("sdl-ttf-{}", VERSION);

    let mut config = cmake::Config::new(lib_dir.clone());

    config.register_dep("sdl3");
    config
        // .define("SDL3_DIR", "OFF") // todo
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("SDLTTF_HARFBUZZ", "ON")
        // .define("SDLTTF_INSTALL", "OFF")
        // .define("SDLTTF_INSTALL_CPACK", "OFF")
        .define("SDLTTF_SAMPLES", "OFF")
        .define("SDLTTF_VENDORED", "OFF"); // todo unclear if these libraries (harfbuzz/freetype) are statically linked if vendored is off

    let out_dir = config.build();
    println!("cargo::metadata=OUT_DIR={}", out_dir.display());

    // ensure that jetbrains IDE can find bindings.rs
    println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());
    // ensure that the libFLAC.a file from the flac build directory can be found for linking
    println!("cargo:rustc-link-search={}/lib/", env::var("OUT_DIR").unwrap());

    println!("cargo::rustc-link-lib=static=SDL_TTF");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
