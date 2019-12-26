extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=static=docparse");
    println!("cargo:rustc-link-lib=static=spcomp2");
    println!("cargo:rustc-link-lib=static=smx");
    // println!("cargo:rustc-link-lib=static=stdc++");

    println!("cargo:rerun-if-changed=binding/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("binding/wrapper.h")

        .rust_target(bindgen::RustTarget::Nightly)
        
        .clang_arg("-x").clang_arg("c++")
        .clang_arg("-std=c++14")

        .parse_callbacks(Box::new(bindgen::CargoCallbacks))

        .generate()

        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}