// Dat memory.x (chay sau bootloader ak-mcu-base) vao duong tim cua linker.
// Duong tim nay lan sang moi bai phu thuoc ak-kit, nen cac bai khong can build.rs.
use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
