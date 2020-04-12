fn main() {
    println!("cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=c++abi");
    println!("cargo:rustc-link-lib=compiler_rt");
    println!("cargo:rustc-link-lib=c");
    //println!("cargo:rustc-cdylib-link-arg=--allow-undefined");
    //println!("cargo:rustc-cdylib-link-arg=--verbose");
    //println!("cargo:rustc-cdylib-link-arg=-error-limit=0");
}
