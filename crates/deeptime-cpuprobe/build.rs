// SPDX-License-Identifier: LGPL-3.0-or-later
//! macOS: the probe's own relocatable install name, so it never claims `deeptime_jni`'s (dyld may
//! match a second image to an already loaded one by install name; plan §6.4).

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,@rpath/libdeeptime_cpuprobe.dylib");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
