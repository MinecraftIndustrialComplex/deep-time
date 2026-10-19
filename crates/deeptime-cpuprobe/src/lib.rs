// SPDX-License-Identifier: LGPL-3.0-or-later
//! The CPU probe the native loader runs before it picks a Deep Time native (decisions log
//! 2026-10-18: every x86-64 platform ships a baseline and an x86-64-v3 build of `deeptime_jni`,
//! and the loader takes v3 when the CPU has the whole level; plan §6.4, §6.5).
//!
//! It is a library of its own, with its own JNI class (`nativebridge.CpuProbe`), because the JVM
//! binds a class's native methods to the first loaded library that exports them: the baseline
//! `deeptime_jni` cannot answer the question without becoming the library every `DeepTimeNative`
//! call then goes to. It has no dependencies, not even `jni`: one exported function that returns
//! a number. It is only ever built for the baseline CPU of its target.
//!
//! The detection is the standard library's (`is_x86_feature_detected!`: CPUID, and for the AVX
//! family also the operating system's XCR0 state through XGETBV, so a CPU whose OS does not save
//! the YMM registers counts as lacking them), plus CPUID leaf 8000_0001h for LAHF/SAHF, which the
//! macro does not cover.

use std::ffi::c_void;

/// What the x86-64-v3 level of the x86-64 psABI requires beyond the x86-64 baseline (its v2
/// level, then v3), in bit order: bit `i` of [`features`] is `X86_64_V3[i]`. `rustc --print cfg
/// -C target-cpu=x86-64-v3` lists the same set less `lahfsahf` (which LLVM enables without a
/// cfg name). Java's `CpuProbe.FEATURES` lists the same names in the same order; the test
/// `java_lists_the_same_features` holds them to it.
pub const X86_64_V3: [&str; 16] = [
    "sse3",
    "ssse3",
    "sse4.1",
    "sse4.2",
    "popcnt",
    "cmpxchg16b",
    "lahfsahf",
    "avx",
    "avx2",
    "bmi1",
    "bmi2",
    "f16c",
    "fma",
    "lzcnt",
    "movbe",
    "xsave",
];

/// The meaning of [`probe`]'s bits; bumped when it changes. The loader refuses another value.
pub const ABI: u32 = 1;

/// The bit mask of the [`X86_64_V3`] features this CPU (and its operating system) has; 0 off
/// x86-64.
#[cfg(target_arch = "x86_64")]
pub fn features() -> u32 {
    let has = [
        is_x86_feature_detected!("sse3"),
        is_x86_feature_detected!("ssse3"),
        is_x86_feature_detected!("sse4.1"),
        is_x86_feature_detected!("sse4.2"),
        is_x86_feature_detected!("popcnt"),
        is_x86_feature_detected!("cmpxchg16b"),
        lahf_sahf(),
        is_x86_feature_detected!("avx"),
        is_x86_feature_detected!("avx2"),
        is_x86_feature_detected!("bmi1"),
        is_x86_feature_detected!("bmi2"),
        is_x86_feature_detected!("f16c"),
        is_x86_feature_detected!("fma"),
        is_x86_feature_detected!("lzcnt"),
        is_x86_feature_detected!("movbe"),
        is_x86_feature_detected!("xsave"),
    ];
    has.iter()
        .enumerate()
        .fold(0, |mask, (i, &h)| if h { mask | 1 << i } else { mask })
}

/// Off x86-64 there is no v3 build to choose.
#[cfg(not(target_arch = "x86_64"))]
pub fn features() -> u32 {
    0
}

/// LAHF and SAHF in 64-bit mode: CPUID leaf 8000_0001h, ECX bit 0 (Intel SDM vol. 2A, "CPUID";
/// AMD APM vol. 3, "CPUID Fn8000_0001_ECX"). Every CPU with AVX2 has it; checked so the probe
/// covers the whole level.
#[cfg(target_arch = "x86_64")]
fn lahf_sahf() -> bool {
    use std::arch::x86_64::__cpuid;
    // `__cpuid` needs no target feature on x86-64; it is `unsafe` on older toolchains only.
    #[allow(unused_unsafe)]
    let max_extended = unsafe { __cpuid(0x8000_0000) }.eax;
    #[allow(unused_unsafe)]
    let ecx = if max_extended >= 0x8000_0001 {
        unsafe { __cpuid(0x8000_0001) }.ecx
    } else {
        0
    };
    ecx & 1 == 1
}

/// [`ABI`] in the high 32 bits, [`features`] in the low 32.
pub fn probe() -> i64 {
    ((u64::from(ABI) << 32) | u64::from(features())) as i64
}

/// `static native long probe()` of `com.minecraftindustrialcomplex.deeptime.nativebridge.CpuProbe`.
/// The JNI types are plain pointers here (`JNIEnv *`, `jclass`), never read.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_CpuProbe_probe(
    _env: *mut c_void,
    _class: *mut c_void,
) -> i64 {
    probe()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_abi_is_in_the_high_bits() {
        let p = probe() as u64;
        assert_eq!(p >> 32, u64::from(ABI));
        assert_eq!(p as u32, features());
        assert_eq!(features() >> X86_64_V3.len(), 0, "no bit beyond the list");
    }

    /// Whatever this test binary was compiled to assume, the CPU running it has.
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn compiled_features_are_detected() {
        let compiled = [
            cfg!(target_feature = "sse3"),
            cfg!(target_feature = "ssse3"),
            cfg!(target_feature = "sse4.1"),
            cfg!(target_feature = "sse4.2"),
            cfg!(target_feature = "popcnt"),
            cfg!(target_feature = "cmpxchg16b"),
            false,
            cfg!(target_feature = "avx"),
            cfg!(target_feature = "avx2"),
            cfg!(target_feature = "bmi1"),
            cfg!(target_feature = "bmi2"),
            cfg!(target_feature = "f16c"),
            cfg!(target_feature = "fma"),
            cfg!(target_feature = "lzcnt"),
            cfg!(target_feature = "movbe"),
            cfg!(target_feature = "xsave"),
        ];
        let f = features();
        for (i, &c) in compiled.iter().enumerate() {
            assert!(
                !c || f & (1 << i) != 0,
                "{} compiled in but not detected",
                X86_64_V3[i]
            );
        }
        // Every x86-64 CPU since the first ones has SSE3 (and every CPU this runs on has LAHF/SAHF
        // in 64-bit mode in practice): a probe reading nothing would fail here.
        assert_ne!(f & 1, 0, "sse3 not detected");
    }

    #[test]
    fn java_lists_the_same_features() {
        let java = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/nativebridge/CpuProbe.java"
        ))
        .expect("CpuProbe.java");
        let start = java
            .find("FEATURES = {")
            .expect("FEATURES = { in CpuProbe.java");
        let body = &java[start..start + java[start..].find('}').expect("closing brace")];
        let names: Vec<&str> = body.split('"').skip(1).step_by(2).collect();
        assert_eq!(names, X86_64_V3, "CpuProbe.FEATURES differs from X86_64_V3");
        assert!(
            java.contains(&format!("static final int ABI = {ABI};")),
            "CpuProbe.ABI differs from {ABI}"
        );
    }
}
