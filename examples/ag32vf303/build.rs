fn main() {
    println!("cargo:rustc-link-arg-bins=-Tmemory.x");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
