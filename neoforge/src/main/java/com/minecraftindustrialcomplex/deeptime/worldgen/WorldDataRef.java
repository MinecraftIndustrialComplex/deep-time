// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import java.util.Optional;

/**
 * Where a Deep Time level's data lives, as stored in its generator's codec (plan §7.1).
 *
 * @param folder              world-data folder, relative to the world folder ({@code deeptime})
 * @param circumferenceBlocks the world's east-west period in blocks: 4s on the cube world, C on a
 *                            Lambert-frame test world; Toroidal World's shape and the terrain source
 *                            know it before any world data exists
 * @param testTerrain         an analytic test terrain kind (e.g. {@code wrap-test}) used instead of
 *                            world data; GameTest dimensions and Phase 0 servers only
 * @param stagingId           the Create World screen's staging folder id, until the data is moved
 *                            into the world at server start (P1-11)
 * @param projection          the world's shape ({@code projection}: {@code cube}, the cube world of
 *                            format 0.7 and the only world the game plays, or {@code lambert}, the
 *                            simulation's frame, the default here so the synthetic test worlds and
 *                            test terrains need not say it); the world data's manifest must agree.
 *                            (The Petroff–Guyou world and the Lambert world's pole cap went with cube
 *                            plan §7.4, decisions log 2026-10-24.)
 */
public record WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain,
                           Optional<String> stagingId, String projection) {

    /** The simulation's Lambert frame: the synthetic test worlds and the analytic test terrains. */
    public static final String LAMBERT = "lambert";
    /** The cube world (format 0.7, docs/plans/cube-world-plan.md). */
    public static final String CUBE = "cube";

    /**
     * The cube world's face side is a multiple of this many blocks (ledger SCL-100, game choice; the
     * Rust {@code projection::FACE_QUANTUM}), so a cube world's circumference 4s is a multiple of 4 ×
     * this.
     */
    public static final int FACE_QUANTUM = CubeLayout.FACE_QUANTUM; // ledger: SCL-100 (game-choice)

    /**
     * The cube world's face side s for a planet {@code circumference} blocks round (the Rust
     * {@code projection::face_for}): the six faces cover the planet's area, rounded to the nearest
     * multiple of {@link #FACE_QUANTUM}. 4k: 1,024; 16k: 3,840; 32k: 7,424.
     */
    public static int faceFor(int circumference) {
        return CubeLayout.faceFor(circumference);
    }

    public static final Codec<WorldDataRef> CODEC = RecordCodecBuilder.<WorldDataRef>create(i -> i.group(
            Codec.STRING.optionalFieldOf("folder", "deeptime").forGetter(WorldDataRef::folder),
            Codec.intRange(1024, 1 << 20).fieldOf("circumference_blocks").forGetter(WorldDataRef::circumferenceBlocks),
            Codec.STRING.optionalFieldOf("test_terrain").forGetter(WorldDataRef::testTerrain),
            Codec.STRING.optionalFieldOf("staging_id").forGetter(WorldDataRef::stagingId),
            Codec.STRING.optionalFieldOf("projection", LAMBERT).forGetter(WorldDataRef::projection)
    ).apply(i, WorldDataRef::new));

    /** A Lambert-frame world (the synthetic worlds and test terrains). */
    public WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain, Optional<String> stagingId) {
        this(folder, circumferenceBlocks, testTerrain, stagingId, LAMBERT);
    }

    public WorldDataRef {
        if (circumferenceBlocks % 512 != 0)
            throw new IllegalArgumentException("circumference_blocks must be a multiple of 512 (a region file), got " + circumferenceBlocks);
        if (!LAMBERT.equals(projection) && !CUBE.equals(projection))
            throw new IllegalArgumentException("projection must be cube or lambert, got " + projection);
        if (CUBE.equals(projection) && circumferenceBlocks % (4 * FACE_QUANTUM) != 0)
            throw new IllegalArgumentException("a cube world's circumference_blocks is 4s, s a multiple of " + FACE_QUANTUM
                    + ", got " + circumferenceBlocks);
    }

    /** Whether the world is the cube world (format 0.7). */
    public boolean cube() {
        return CUBE.equals(projection);
    }

    /** s, the face side of a cube world (a quarter of the x period); 0 on a Lambert-frame world. */
    public int faceBlocks() {
        return cube() ? circumferenceBlocks / 4 : 0;
    }

    /** This reference with another world shape (the period and projection of an accepted run). */
    public WorldDataRef withShape(int circumference, String projection) {
        return new WorldDataRef(folder, circumference, testTerrain, stagingId, projection);
    }

    /** The test terrain kind in effect: the codec's, else {@code -Ddeeptime.testTerrain}. */
    public Optional<String> effectiveTestTerrain() {
        if (testTerrain.isPresent())
            return testTerrain;
        return Optional.ofNullable(System.getProperty("deeptime.testTerrain")).filter(s -> !s.isBlank());
    }
}
