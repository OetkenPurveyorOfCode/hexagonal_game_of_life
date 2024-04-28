fn main() {
    cc::Build::new()
        .define("_CRT_SECURE_NO_WARNINGS", "1")
        .opt_level(2)
        .define("PLATFORM_DESKTOP","1")
        .include("./raylib/src/external/glfw/include")
        .file("./raylib/src/rcore.c")
        .file("./raylib/src/rshapes.c")
        .file("./raylib/src/rglfw.c")
        .file("./raylib/src/raudio.c")
        .file("./raylib/src/rmodels.c")
        .file("./raylib/src/rtext.c")
        .file("./raylib/src/rtextures.c")
        .file("./raylib/src/utils.c")
        .compile("raylib");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=gdi32");
}
