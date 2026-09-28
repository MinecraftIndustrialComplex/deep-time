// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.mojang.serialization.Codec;
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
 */
public record WorldDataRef(String folder, int circumferenceBlocks, Optional<String> testTerrain,
                           Optional<String> stagingId) {

    public static final Codec<WorldDataRef> CODEC = RecordCodecBuilder.create(i -> i.group(
            Codec.STRING.optionalFieldOf("folder", "deeptime").forGetter(WorldDataRef::folder),
            Codec.intRange(1024, 1 << 20).fieldOf("circumference_blocks").forGetter(WorldDataRef::circumferenceBlocks),
            Codec.STRING.optionalFieldOf("test_terrain").forGetter(WorldDataRef::testTerrain),
            Codec.STRING.optionalFieldOf("staging_id").forGetter(WorldDataRef::stagingId)
    ).apply(i, WorldDataRef::new));

    public WorldDataRef {
        if (circumferenceBlocks % 512 != 0)
            throw new IllegalArgumentException("circumference_blocks must be a multiple of 512 (a region file), got " + circumferenceBlocks);
    }

    /** The test terrain kind in effect: the codec's, else {@code -Ddeeptime.testTerrain}. */
    public Optional<String> effectiveTestTerrain() {
        if (testTerrain.isPresent())
            return testTerrain;
        return Optional.ofNullable(System.getProperty("deeptime.testTerrain")).filter(s -> !s.isBlank());
    }
}
