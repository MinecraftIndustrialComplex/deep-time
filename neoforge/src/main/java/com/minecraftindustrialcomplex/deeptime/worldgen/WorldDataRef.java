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
 *                            §7.11; default 75°)
 * @param poleStopDeg         where the soft cap becomes a hard stop ({@code pole_stop_deg}; default 80°)
 */
public record WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain,
                           Optional<String> stagingId, double poleCapDeg, double poleStopDeg) {

    public static final Codec<WorldDataRef> CODEC = RecordCodecBuilder.<WorldDataRef>create(i -> i.group(
            Codec.STRING.optionalFieldOf("folder", "deeptime").forGetter(WorldDataRef::folder),
            Codec.intRange(1024, 1 << 20).fieldOf("circumference_blocks").forGetter(WorldDataRef::circumferenceBlocks),
            Codec.STRING.optionalFieldOf("test_terrain").forGetter(WorldDataRef::testTerrain),
            Codec.STRING.optionalFieldOf("staging_id").forGetter(WorldDataRef::stagingId),
            Codec.doubleRange(1.0, 89.0).optionalFieldOf("pole_cap_deg", PoleCapCurve.DEFAULT_CAP_DEG).forGetter(WorldDataRef::poleCapDeg),
            Codec.doubleRange(1.0, 90.0).optionalFieldOf("pole_stop_deg", PoleCapCurve.DEFAULT_STOP_DEG).forGetter(WorldDataRef::poleStopDeg)
    ).apply(i, WorldDataRef::new)).validate(r -> r.poleCapDeg() < r.poleStopDeg() ? DataResult.success(r)
            : DataResult.error(() -> "pole_cap_deg (" + r.poleCapDeg() + ") must be below pole_stop_deg (" + r.poleStopDeg() + ")"));

    public WorldDataRef {
        if (circumferenceBlocks % 512 != 0)
            throw new IllegalArgumentException("circumference_blocks must be a multiple of 512 (a region file), got " + circumferenceBlocks);
    }

    /** The pole cap of this level (plan §7.11). */
    public PoleCapCurve poleCap() {
        return new PoleCapCurve(circumferenceBlocks, poleCapDeg, poleStopDeg);
    }

    /** The test terrain kind in effect: the codec's, else {@code -Ddeeptime.testTerrain}. */
    public Optional<String> effectiveTestTerrain() {
        if (testTerrain.isPresent())
            return testTerrain;
        return Optional.ofNullable(System.getProperty("deeptime.testTerrain")).filter(s -> !s.isBlank());
    }
}
