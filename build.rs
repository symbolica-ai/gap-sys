extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=/usr/local/gap/lib");
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
    let obj_path = output_path.join("extern.o");
    let lib_path = output_path.join("libextern.a");

    let clang_output = std::process::Command::new("clang")
        .arg("-O")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(std::env::temp_dir().join("bindgen").join("extern.c"))
        .output()
        .unwrap();

    if !clang_output.status.success() {
        panic!(
            "Could not compile object file:\n{}",
            String::from_utf8_lossy(&clang_output.stderr)
        );
    }

    let lib_output = Command::new("ar")
        .arg("rcs")
        .arg(&lib_path)
        .arg(&obj_path)
        .output()
        .unwrap();

    if !lib_output.status.success() {
        panic!(
            "Could not emit library file:\n{}",
            String::from_utf8_lossy(&lib_output.stderr)
        );
    }

    println!("cargo:rustc-link-search={}", output_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=extern");

    println!(
        "cargo:warning={}",
        std::env::temp_dir()
            .join("bindgen")
            .join("extern.c")
            .to_str()
            .unwrap()
    );

    // print output path as a warning
    println!(
        "cargo:warning={}",
        output_path.to_str().unwrap()
    );

    bindings
        .write_to_file(output_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
