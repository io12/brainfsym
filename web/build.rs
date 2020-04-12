fn main() {
    println!(
        "cargo:rustc-link-search={}",
        dirs::home_dir()
            .expect("failed getting home dir")
            .join(".emscripten_cache")
            .join("wasm")
            .display()
    );
    println!("cargo:rustc-link-lib=static=c++abi-noexcept");
}
