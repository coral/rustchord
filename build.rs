extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {

    cc::Build::new()
    .shared_flag(true)
    .warnings(false)
    .extra_warnings(false)
    .files(&[
        "colorchord/colorchord2/color.c",
        "colorchord/colorchord2/notefinder.c",
        "colorchord/colorchord2/dft.c",
        "colorchord/colorchord2/filter.c", 
        "colorchord/colorchord2/decompose.c",
        "colorchord/colorchord2/parameters.c", 
        "colorchord/colorchord2/hook.c", 
        "colorchord/colorchord2/configs.c",
        "colorchord/colorchord2/chash.c",
        "colorchord/colorchord2/util.c",
        "colorchord/embeddedcommon/DFT32.c",
        ])
        .include("colorchord/colorchord2")
        .include("colorchord/colorchord2/rawdraw")
        .include("colorchord/embeddedcommon")
        .include("./include")
        .flag("-ffast-math")
        .flag("-O1")
        .flag("-Wno-everything")
        .compile("colorchord");
        println!(r"cargo:rustc-link-search=.");
        
        let m  = [
        "color.h",
        "configs.h",
        "decompose.h",
        "dft.h",
        "filter.h",
        "hook.h",
        "notefinder.h",
        "outdrivers.h",
        "parameters.h"];

        let mut bld = bindgen::Builder::default();
        for header in m.iter() {
            let hdr: String = "colorchord/colorchord2/".to_string() + &header.to_string();
            println!("cargo:rerun-if-changed={:?}", hdr);
            bld = bld.header(hdr);
        }
        let bindings = bld.clang_arg("-Icolorchord/colorchord2/rawdraw")
        .generate()
        .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
        
    }