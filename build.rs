extern crate cc;

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
            "colorchord/embeddedcommon/DFT32.c",
        ])
        .include("colorchord/colorchord2")
        .include("colorchord/colorchord2/rawdraw")
        .include("colorchord/embeddedcommon")
        .include("./include")
        .flag("-ffast-math")
        .flag("-O1")
        .compile("colorchord");

    println!(r"cargo:rustc-link-search=."); 
}