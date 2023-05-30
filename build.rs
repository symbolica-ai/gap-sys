extern crate bindgen;

use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-lib=gap");
    println!("cargo:rerun-if-changed=wrapper.h");

    let input_path = std::env::current_dir().unwrap().join("wrapper.h");

    let bindings = bindgen::Builder::default()
        .header(input_path.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .wrap_static_fns(true)
        .generate()
        .expect("Unable to generate bindings");

    let output_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    cc::Build::new()
        .file(std::env::temp_dir().join("bindgen").join("extern.c"))
        .opt_level(3)
        .out_dir(&output_path)
        .compile("extern");

    println!("cargo:rustc-link-search={}", output_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=extern");

    bindings
        .write_to_file(output_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
