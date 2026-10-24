// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import java.util.Optional;

/**
 * Where a Deep Time level's data lives, as stored in its generator's codec (plan §7.1).
 *
 * @param folder              world-data folder, relative to the world folder ({@code deeptime})
 * @param circumferenceBlocks C, the world's east-west period in blocks; Toroidal World's shape and
 *                            the terrain source know it before any world data exists
 * @param testTerrain         an analytic test terrain kind (e.g. {@code wrap-test}) used instead of
 *                            world data; GameTest dimensions and Phase 0 servers only
 * @param stagingId           the Create World screen's staging folder id, until the data is moved
 *                            into the world at server start (P1-11)
 * @param poleCapDeg          where the pole soft cap starts (plan §5.1 {@code scale.pole_cap_deg},
 *                            §7.11; default 70°, Ben 2026-10-03)
 * @param poleStopDeg         where the soft cap becomes a hard stop ({@code pole_stop_deg}; default 80°)
 * @param projection          the world's projection ({@code projection}: {@code petroff_guyou}, the
 *                            seamless world of format 0.5, {@code cube}, the cube world of format 0.7,
 *                            or {@code lambert}, the default here so the synthetic test worlds need not
 *                            say it); the world data's manifest must agree. On a Petroff–Guyou world
 *                            {@code circumferenceBlocks} is the x period 2b and on a cube world the
 *                            period 4s; on both the pole cap gives way to the edge border
 */
public record WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain,
                           Optional<String> stagingId, double poleCapDeg, double poleStopDeg, String projection) {

    public static final String LAMBERT = "lambert";
    public static final String PETROFF_GUYOU = "petroff_guyou";
    /** The cube world (format 0.7, docs/plans/cube-world-plan.md; phase 2 opt-in level types). */
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
            Codec.doubleRange(1.0, 89.0).optionalFieldOf("pole_cap_deg", PoleCapCurve.DEFAULT_CAP_DEG).forGetter(WorldDataRef::poleCapDeg),
            Codec.doubleRange(1.0, 90.0).optionalFieldOf("pole_stop_deg", PoleCapCurve.DEFAULT_STOP_DEG).forGetter(WorldDataRef::poleStopDeg),
            Codec.STRING.optionalFieldOf("projection", LAMBERT).forGetter(WorldDataRef::projection)
    ).apply(i, WorldDataRef::new)).validate(r -> r.poleCapDeg() < r.poleStopDeg() ? DataResult.success(r)
            : DataResult.error(() -> "pole_cap_deg (" + r.poleCapDeg() + ") must be below pole_stop_deg (" + r.poleStopDeg() + ")"));

    /** A Lambert world (the synthetic worlds and test terrains). */
    public WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain, Optional<String> stagingId,
                        double poleCapDeg, double poleStopDeg) {
        this(folder, circumferenceBlocks, testTerrain, stagingId, poleCapDeg, poleStopDeg, LAMBERT);
    }

    public WorldDataRef {
        if (circumferenceBlocks % 512 != 0)
            throw new IllegalArgumentException("circumference_blocks must be a multiple of 512 (a region file), got " + circumferenceBlocks);
        if (!LAMBERT.equals(projection) && !PETROFF_GUYOU.equals(projection) && !CUBE.equals(projection))
            throw new IllegalArgumentException("projection must be lambert, petroff_guyou or cube, got " + projection);
        if (CUBE.equals(projection) && circumferenceBlocks % (4 * FACE_QUANTUM) != 0)
            throw new IllegalArgumentException("a cube world's circumference_blocks is 4s, s a multiple of " + FACE_QUANTUM
                    + ", got " + circumferenceBlocks);
    }

    /** Whether the world is the cube world (format 0.7). */
    public boolean cube() {
        return CUBE.equals(projection);
    }

    /** s, the face side of a cube world (a quarter of the x period); 0 on the polar worlds. */
    public int faceBlocks() {
        return cube() ? circumferenceBlocks / 4 : 0;
    }

    /** Whether the world is the seamless Petroff–Guyou world (format 0.5). */
    public boolean petroffGuyou() {
        return PETROFF_GUYOU.equals(projection);
    }

    /** b, the hemisphere side of a Petroff–Guyou world (half the x period); 0 on Lambert. */
    public int hemisphereBlocks() {
        return petroffGuyou() ? circumferenceBlocks / 2 : 0;
    }

    /** The pole cap of this level (plan §7.11); null on a Petroff–Guyou or cube world, whose poles are
     *  ordinary ground (their turning edges have the edge border instead, {@code wrap/EdgeBorder}). */
    public PoleCapCurve poleCap() {
        return petroffGuyou() || cube() ? null : new PoleCapCurve(circumferenceBlocks, poleCapDeg, poleStopDeg);
    }

    /** This reference with another world shape (the period and projection of an accepted run). */
    public WorldDataRef withShape(int circumference, String projection) {
        return new WorldDataRef(folder, circumference, testTerrain, stagingId, poleCapDeg, poleStopDeg, projection);
    }

    /** The test terrain kind in effect: the codec's, else {@code -Ddeeptime.testTerrain}. */
    public Optional<String> effectiveTestTerrain() {
        if (testTerrain.isPresent())
            return testTerrain;
        return Optional.ofNullable(System.getProperty("deeptime.testTerrain")).filter(s -> !s.isBlank());
    }
}
