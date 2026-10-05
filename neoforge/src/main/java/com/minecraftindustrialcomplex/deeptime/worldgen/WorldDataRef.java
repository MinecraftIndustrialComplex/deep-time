// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
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
 *                            seamless world of format 0.5, or {@code lambert}, the default here so
 *                            the synthetic test worlds need not say it); the world data's manifest
 *                            must agree. On a Petroff–Guyou world {@code circumferenceBlocks} is the
 *                            x period 2b and the pole cap gives way to the edge border
 */
public record WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain,
                           Optional<String> stagingId, double poleCapDeg, double poleStopDeg, String projection) {

    public static final String LAMBERT = "lambert";
    public static final String PETROFF_GUYOU = "petroff_guyou";

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
        if (!LAMBERT.equals(projection) && !PETROFF_GUYOU.equals(projection))
            throw new IllegalArgumentException("projection must be lambert or petroff_guyou, got " + projection);
    }

    /** Whether the world is the seamless Petroff–Guyou world (format 0.5). */
    public boolean petroffGuyou() {
        return PETROFF_GUYOU.equals(projection);
    }

    /** b, the hemisphere side of a Petroff–Guyou world (half the x period); 0 on Lambert. */
    public int hemisphereBlocks() {
        return petroffGuyou() ? circumferenceBlocks / 2 : 0;
    }

    /** The pole cap of this level (plan §7.11); null on a Petroff–Guyou world, whose poles are ordinary
     *  ground (its north and south edges have the temporary edge border instead, {@code wrap/EdgeBorder}). */
    public PoleCapCurve poleCap() {
        return petroffGuyou() ? null : new PoleCapCurve(circumferenceBlocks, poleCapDeg, poleStopDeg);
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
