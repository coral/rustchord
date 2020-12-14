extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
    .shared_flag(true)
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
        .compile("colorchord");
        
        let bindings = bindgen::Builder::default()
        .header("colorchord/colorchord2/color.c")
        .generate()
        .expect("Unable to generate bindings");
        
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
        
        println!(r"cargo:rustc-link-search=."); 
    }