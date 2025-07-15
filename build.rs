use std::process::Command;
 
fn main() {
    println!("cargo:rerun-if-changed=src/exceptions.s");
 
    let status = Command::new("aarch64-linux-gnu-gcc") // 你可能需要安装 aarch64-linux-gnu-gcc
        .args(&["-c", "src/exceptions.s", "-o", "target/exceptions.o"])
        .status()
        .expect("Failed to execute aarch64-linux-gnu-gcc");
 
    if !status.success() {
        panic!("GCC failed to compile exceptions.s");
    }
 
    println!("cargo:rustc-link-arg=target/exceptions.o");
}
