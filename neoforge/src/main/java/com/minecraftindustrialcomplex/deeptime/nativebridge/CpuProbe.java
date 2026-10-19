// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.util.ArrayList;
import java.util.List;

/**
 * The JNI side of the CPU probe, {@code crates/deeptime-cpuprobe} (decisions log 2026-10-18; plan
 * §6.5): which x86-64-v3 features this CPU and its operating system have, so {@link NativeLoader}
 * can pick the x86-64-v3 build of the native library or the baseline one.
 *
 * <p>The probe is a library of its own with this class of its own because the JVM binds a class's
 * native methods to the first loaded library that exports them: asking the baseline
 * {@code deeptime_jni} would make it the library every {@link DeepTimeNative} call goes to.
 *
 * <p>{@code FEATURES} and {@code ABI} mirror {@code deeptime_cpuprobe::X86_64_V3} and
 * {@code deeptime_cpuprobe::ABI}; the Rust test {@code java_lists_the_same_features} reads this file
 * and fails on any drift.
 */
final class CpuProbe {

    /** The meaning of {@link #probe}'s bits; the loader ignores a probe that reports another. */
    static final int ABI = 1;

    /** The x86-64-v3 level less the baseline, in bit order: bit {@code i} is {@code FEATURES[i]}. */
    static final String[] FEATURES = {"sse3", "ssse3", "sse4.1", "sse4.2", "popcnt", "cmpxchg16b", "lahfsahf",
            "avx", "avx2", "bmi1", "bmi2", "f16c", "fma", "lzcnt", "movbe", "xsave"};

    /** The mask of a CPU with the whole level. */
    static final int ALL = (1 << FEATURES.length) - 1;

    private CpuProbe() {}

    /** {@link #ABI} in the high 32 bits, the mask of the features present in the low 32. */
    static native long probe();

    /** The names of the x86-64-v3 features {@code mask} lacks, in {@link #FEATURES} order. */
    static List<String> missing(int mask) {
        List<String> out = new ArrayList<>();
        for (int i = 0; i < FEATURES.length; i++)
            if ((mask & (1 << i)) == 0)
                out.add(FEATURES[i]);
        return out;
    }
}
