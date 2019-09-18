extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let lib = pkg_config::find_library("lilv-0").expect("Unable to find lilv");

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .whitelist_function("lilv_.*")
        .whitelist_type("Lilv.*")
        .whitelist_var("LV2_CORE_.*")
        .clang_args(lib.include_paths.iter()
            .map(|path| String::from("-I") + path.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}