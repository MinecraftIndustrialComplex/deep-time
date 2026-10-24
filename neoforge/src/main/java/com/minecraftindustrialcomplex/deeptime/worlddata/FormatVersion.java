// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The world-data format versions this reader understands (plan §5.7.2). {@link #CURRENT} mirrors
 * {@code [workspace.metadata.deeptime] format_version} in the workspace {@code Cargo.toml}, the one
 * place the polar worlds' version is set; {@code FormatVersionTest} fails when they drift. A cube
 * world's manifest says {@link #CUBE} (FORMAT.md "Polar and cube worlds side by side"); every file
 * header of either keeps {@link #MINOR}. The twin of Rust's {@code FORMAT_VERSION},
 * {@code CUBE_FORMAT_VERSION} and {@code READS_VERSIONS}.
 */
public final class FormatVersion {

    /** The version string a polar world's {@code manifest.json} carries. */
    public static final String CURRENT = "0.6";

    /** The version string a cube world's {@code manifest.json} carries (cube plan phase 2: read side by side with 0.6). */
    public static final String CUBE = "0.7";

    /** What {@link #require} accepts, as its refusal names it (Rust {@code READS_VERSIONS}). */
    public static final String READS = CURRENT + " (polar worlds) and " + CUBE + " (cube worlds)";

    /** The minor part of {@link #CURRENT}, written into every file header (a cube world's too). */
    public static final int MINOR = 6;

    private FormatVersion() {}

    /**
     * Before 1.0 a reader accepts exactly the versions it writes: {@link #CURRENT} and {@link #CUBE}.
     * Which of the two a world must say is its projection's ({@link #forKind}), checked by
     * {@link Manifest#read} once the layout is parsed.
     *
     * @throws WorldDataException naming the versions otherwise
     */
    public static void require(String found) {
        if (!CURRENT.equals(found) && !CUBE.equals(found))
            throw new WorldDataException(refusal(found));
    }

    /** The version a world of projection {@code kind} carries (null: a Lambert world without a layout block). */
    public static String forKind(WorldProjection.Kind kind) {
        return kind == WorldProjection.Kind.CUBE ? CUBE : CURRENT;
    }

    /**
     * The refusal message: "world data 0.4, this Deep Time reads 0.6 (polar worlds) and 0.7 (cube worlds); before 1.0
     * worlds are not migrated".
     */
    public static String refusal(String found) {
        return "world data " + found + ", this Deep Time reads " + READS + "; before 1.0 worlds are not migrated";
    }
}
