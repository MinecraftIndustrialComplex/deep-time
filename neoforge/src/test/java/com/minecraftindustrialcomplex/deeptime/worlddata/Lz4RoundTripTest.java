// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.jpountz.lz4.LZ4Factory;
import net.jpountz.lz4.LZ4SafeDecompressor;
import org.junit.jupiter.api.Test;

import java.nio.ByteBuffer;
import java.util.Base64;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P0-09: lz4-java 1.8.0's pure-Java {@code LZ4Factory.safeInstance().safeDecompressor()} (the
 * game's own lz4-java) decodes {@code lz4_flex::block} output — the raw LZ4 block format both
 * sides share (plan §5.7.3).
 */
class Lz4RoundTripTest {

    private static final LZ4SafeDecompressor LZ4 = LZ4Factory.safeInstance().safeDecompressor();

    /** Vectors lz4_flex compressed (in the synth sample dump): tiny, text, noise, smooth, zeros. */
    @Test
    void decodesLz4FlexVectors() {
        JsonObject dump = Fixtures.samples("wrap1024");
        int n = 0;
        for (JsonElement v : dump.getAsJsonArray("lz4_vectors")) {
            JsonObject o = v.getAsJsonObject();
            byte[] input = Base64.getDecoder().decode(o.get("input").getAsString());
            byte[] block = Base64.getDecoder().decode(o.get("block").getAsString());
            byte[] out = new byte[input.length];
            int len = LZ4.decompress(block, 0, block.length, out, 0, out.length);
            assertEquals(input.length, len, o.get("name").getAsString());
            assertArrayEquals(input, out, o.get("name").getAsString());
            n++;
        }
        assertEquals(5, n);
    }

    /** Every LZ4 tile of every layer of the fixture decodes to exactly its tile's cell count. */
    @Test
    void decodesEveryFixtureTile() {
        WorldData wd = WorldData.open(Fixtures.world("wrap1024"));
        int tiles = 0;
        for (Manifest.LayerEntry e : wd.manifest().layers()) {
            TilePack pack = wd.layer(e.name()).pack();
            LayerDesc d = pack.desc();
            assertEquals(LayerDesc.Codec.LZ4_BLOCK, d.codec(), e.name());
            for (int t = 0; t < pack.tileCount(); t++) {
                ByteBuffer stored = pack.storedBytes(t);
                byte[] src = new byte[stored.remaining()];
                stored.get(src);
                int size = d.tileWidth(t % d.tilesX()) * d.tileHeight(t / d.tilesX()) * d.cellBytes();
                // One byte of slack: a decoder that overran would say so instead of stopping short.
                byte[] out = new byte[size + 1];
                assertEquals(size, LZ4.decompress(src, 0, src.length, out, 0, out.length), e.name() + " tile " + t);
                tiles++;
            }
        }
        assertTrue(tiles >= 8, "only " + tiles + " tiles");
    }
}
