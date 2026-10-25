// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat;

import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SiteChunksTest {

    /** A cube world's net with faces 1024 blocks wide: C = 4096, face S at x in [-512, 512), z in [512, 1536). */
    private static final CubeLayout CUBE = CubeLayout.of(1024);
    private static final int C = 4096;

    /** An ore ellipsoid (shape 0) of semi-axes 20, 5, 20 centred on (x, 0, z). */
    private static Body ore(double x, double z) {
        return new Body(1, 3, 0, SiteChunks.ORE_ROLE, new double[]{x, 0, z}, new float[]{20, 5, 20}, 0f, 0f, Body.NO_CLIP,
                new int[]{(int) x - 20, -5, (int) z - 20}, new int[]{(int) x + 20, 5, (int) z + 20}, "granite", List.of());
    }

    /**
     * A body across S's west edge, a turning seam: the chunks past it are void (turned copies of the
     * partner face's, SCL-210, which place none of this body), so they count nothing, and the shares
     * of the real chunks are of the volume on the face.
     */
    @Test
    void aBodyPastATurningEdgeCountsOnlyItsChunksOnTheFaces() {
        int west = -512;
        Body b = ore(west + 4, 1000);
        var plain = SiteChunks.compute(List.of(b), C, null, 3);
        var cube = SiteChunks.compute(List.of(b), C, CUBE, 3);
        assertTrue(plain.byChunk().keySet().stream().anyMatch(k -> new ChunkPos(k).x < west >> 4),
                "the flat count reaches past the edge");
        for (long k : cube.byChunk().keySet()) {
            ChunkPos p = new ChunkPos(k);
            assertTrue(CUBE.faceOfBlock(p.getMinBlockX(), p.getMinBlockZ()) != null, "a void chunk counted: " + p);
        }
        assertTrue(cube.total() < plain.total(), "the void's volume left out: " + cube.total() + " of " + plain.total());
        double onFace = plain.byChunk().entrySet().stream()
                .filter(e -> new ChunkPos(e.getKey()).x >= west >> 4).mapToDouble(java.util.Map.Entry::getValue).sum();
        assertEquals(onFace, cube.total(), 1e-9, "the faces' chunks keep their counts");
        ChunkPos best = cube.best();
        assertNotNull(best);
        assertTrue(best.x >= west >> 4, "the best chunk is on the face: " + best);
    }

    /** A body wholly on a face counts the same with and without the cube. */
    @Test
    void aBodyOnAFaceIsCountedAsBefore() {
        Body b = ore(0, 1000);
        assertEquals(SiteChunks.compute(List.of(b), C, null, 3).byChunk(), SiteChunks.compute(List.of(b), C, CUBE, 3).byChunk());
    }
}
