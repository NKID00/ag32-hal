fn main() {
    println!("cargo::rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo::rustc-link-arg-bins=-Tdevice.x");
    println!("cargo::rustc-link-arg-bins=-Tmemory.x");
}
