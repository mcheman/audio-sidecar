use std::env;
use std::path::PathBuf;

const VERSION: &str = "1.4.3";

// this static links
fn main() {
    let lib_dir = format!("flac-{}", VERSION);

    let mut config = cmake::Config::new(lib_dir.clone());

    config
        .define("WITH_OGG", "OFF")
        .define("BUILD_CXXLIBS", "OFF")
        .define("BUILD_DOCS", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .define("BUILD_PROGRAMS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_UTILS", "OFF");

    let out_dir = config.build();
    println!("cargo::metadata=OUT_DIR={}", out_dir.display());

    // ensure that the libFLAC.a file from the flac build directory can be found for linking
    println!("cargo:rustc-link-search={}/lib/", out_dir.display());

    println!("cargo::rustc-link-lib=static=FLAC");

    let out_dir_include_path = out_dir
        .join("include")
        .join("FLAC")
        .canonicalize()
        .expect("to be able to canonicalize path");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        // .header("wrapper.h")
        .header(
            out_dir_include_path
                .join("metadata.h")
                .to_str()
                .expect("path to be valid Unicode"),
        )
        .header(
            out_dir_include_path
                .join("stream_encoder.h")
                .to_str()
                .expect("path to be valid Unicode"),
        )
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
