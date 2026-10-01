// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The world-data format version this reader understands (plan §5.7.2). It mirrors
 * {@code [workspace.metadata.deeptime] format_version} in the workspace {@code Cargo.toml}, the one
 * place the version is set; {@code FormatVersionTest} fails when they drift.
 */
public final class FormatVersion {

    /** The version string written in {@code manifest.json}. */
    public static final String CURRENT = "0.3";

    /** The minor part of {@link #CURRENT}, written into every file header. */
    public static final int MINOR = 3;

    private FormatVersion() {}

    /**
     * Before 1.0 a reader accepts exactly its own version.
     *
     * @throws WorldDataException naming both versions otherwise
     */
    public static void require(String found) {
        if (!CURRENT.equals(found))
            throw new WorldDataException(refusal(found));
    }

    /** The refusal message: "world data 0.3, this Deep Time reads 0.1; before 1.0 worlds are not migrated". */
    public static String refusal(String found) {
        return "world data " + found + ", this Deep Time reads " + CURRENT + "; before 1.0 worlds are not migrated";
    }
}
