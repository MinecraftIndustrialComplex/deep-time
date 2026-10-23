// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import net.minecraft.core.BlockPos;
import net.minecraft.core.HolderLookup;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.phys.Vec3;
import org.jetbrains.annotations.Nullable;

import java.util.List;

/**
 * Block-entity data in the band (docs/design/wrap-engine.md §12.8, E2). Band copies hold no block
 * entity on the server; what the client draws there (a chest, a sign's text, a banner) comes from the
 * stored block entity, addressed to the band position and turned:
 * <ul>
 *   <li>a band chunk's packet lists the stored block entities it shows ({@link #forEachShown});</li>
 *   <li>a change at a stored position is marked at its images too ({@link #markImages}), and a band
 *       holder sends the stored entity's data re-addressed ({@link #packetFor});</li>
 *   <li>range checks and the sign editor use the image nearest the player ({@link #nearestBandImage}).</li>
 * </ul>
 * Internal: for the engine's mixins and its GameTests, not adapters. Server thread only.
 */
public final class BandBlockEntities {

    private BandBlockEntities() {}

    /** A stored block entity a band chunk shows. */
    @FunctionalInterface
    public interface Shown {
        /**
         * @param bandPos      where the band shows it
         * @param stored       the block entity, at its stored position
         * @param storedToBand the motion from the stored position to {@code bandPos}
         */
        void accept(BlockPos bandPos, BlockEntity stored, CellIsometry storedToBand);
    }

    /**
     * Calls {@code out} for every block entity of the band chunk's loaded sources that the chunk shows:
     * its band position lies in the chunk and folds back to it (a split chunk shows part of each
     * source). Returns how many; 0 when the chunk is not a band chunk or the level has no topology.
     */
    public static int forEachShown(Level level, LevelChunk band, Shown out) {
        WrapTopology t = WrapEngine.topology(level);
        if (t == null || ((WrapChunk) band).deeptime$bandIso() == null)
            return 0;
        ChunkPos cp = band.getPos();
        int n = 0;
        for (WrapTopology.ChunkSource s : t.chunkSources(cp.x, cp.z)) {
            LevelChunk src = level.getChunkSource().getChunkNow(s.sourceX(), s.sourceZ());
            if (src == null)
                continue;
            CellIsometry toBand = s.iso().inverse();
            for (BlockEntity be : src.getBlockEntities().values()) {
                BlockPos p = be.getBlockPos();
                int bx = toBand.applyX(p.getX(), p.getZ()), bz = toBand.applyZ(p.getX(), p.getZ());
                if (bx >> 4 != cp.x || bz >> 4 != cp.z)
                    continue;
                WrapTopology.Fold f = t.fold(bx, bz);
                if (f == null || f.x() != p.getX() || f.z() != p.getZ())
                    continue;
                out.accept(new BlockPos(bx, p.getY(), bz), be, toBand);
                n++;
            }
        }
        return n;
    }

    /** The stored block entity's update tag as the band shows it: turned by {@link BandTags}. */
    public static CompoundTag bandTag(BlockEntity be, CellIsometry storedToBand, HolderLookup.Provider registries) {
        return turnTag(typeId(be.getType()), be.getUpdateTag(registries), storedToBand);
    }

    /** {@code tag} with its position-bearing keys ({@link BandTags#ROWS}) turned: a copy when a row applies, else {@code tag}. */
    public static CompoundTag turnTag(String type, CompoundTag tag, CellIsometry storedToBand) {
        List<BandTags.Row> rows = BandTags.rowsFor(type);
        if (rows.isEmpty())
            return tag;
        CompoundTag out = tag.copy();
        for (BandTags.Row r : rows) {
            if (!out.contains(r.key()))
                continue;
            switch (r.kind()) {
                case BLOCK_POS -> {
                    int[] a = out.getIntArray(r.key());
                    if (a.length == 3)
                        out.putIntArray(r.key(), BandTags.movePos(a, storedToBand));
                }
                case OFFSET -> {
                    int[] a = out.getIntArray(r.key());
                    if (a.length == 3)
                        out.putIntArray(r.key(), BandTags.turnOffset(a, storedToBand));
                }
                case DIRECTION -> out.putInt(r.key(), BandTags.turnDirection(out.getInt(r.key()), storedToBand));
            }
        }
        return out;
    }

    /** A block entity type's registry id ({@code minecraft:chest}), or "" for an unregistered one. */
    public static String typeId(BlockEntityType<?> type) {
        ResourceLocation id = BlockEntityType.getKey(type);
        return id == null ? "" : id.toString();
    }

    /**
     * The data packet a holder sends for band position {@code bandPos}: the stored block entity's,
     * addressed to {@code bandPos}, its tag turned. Null when {@code bandPos} is not a band position,
     * holds no block entity, or the entity sends no data packet of vanilla's kind (then vanilla's way
     * stands).
     */
    @Nullable
    public static ClientboundBlockEntityDataPacket packetFor(Level level, BlockPos bandPos) {
        WrapEngine.Folded f = WrapEngine.fold(level, bandPos);
        if (f == null)
            return null;
        BlockEntity be = level.getBlockEntity(f.pos());
        if (be == null || !(be.getUpdatePacket() instanceof ClientboundBlockEntityDataPacket p))
            return null;
        return new ClientboundBlockEntityDataPacket(bandPos.immutable(), p.getType(), turnTag(typeId(p.getType()), p.getTag(), f.iso().inverse()));
    }

    /**
     * After a change at {@code pos} ({@code ServerChunkCache.blockChanged}): marks each band position
     * that shows it, so the band's holders send it too (a block entity's data change leaves the band's
     * copy of the state as it was, so nothing else would). Does nothing for a position no band shows.
     */
    public static void markImages(ServerLevel level, BlockPos pos) {
        LevelChunk c = level.getChunkSource().getChunkNow(pos.getX() >> 4, pos.getZ() >> 4);
        if (c == null)
            return;
        CellIsometry[] images = ((WrapChunk) c).deeptime$images();
        if (images.length == 0)
            return;
        WrapTopology t = WrapEngine.topology(level);
        // In the chunk's own frame (Toroidal World can hand over a position a period away).
        ChunkPos cp = c.getPos();
        int x = cp.getMinBlockX() + (pos.getX() & 15), z = cp.getMinBlockZ() + (pos.getZ() & 15);
        for (CellIsometry g : images) {
            int bx = g.applyX(x, z), bz = g.applyZ(x, z);
            WrapTopology.Fold back = t.fold(bx, bz);
            if (back == null || back.x() != x || back.z() != z)
                continue;
            level.getChunkSource().blockChanged(new BlockPos(bx, pos.getY(), bz));
        }
    }

    /** After a band chunk is copied: marks every block entity it shows, so its viewers get their data. */
    static void markShown(ServerLevel level, LevelChunk band) {
        forEachShown(level, band, (bandPos, be, g) -> level.getChunkSource().blockChanged(bandPos));
    }

    /**
     * A band image of a stored position: where it is and the motion from there to the stored position.
     */
    public record Image(BlockPos pos, CellIsometry toStored) {}

    /**
     * The band image of {@code stored} nearest {@code from}, or null when no band image is nearer than
     * the stored position itself (or the level has no topology). The period's own copies on a looped
     * level are Toroidal World's, not the engine's: they never count.
     */
    @Nullable
    public static Image nearestBandImage(Level level, BlockPos stored, Vec3 from) {
        if (WrapEngine.topology(level) == null)
            return null;
        BlockPos img = WrapEngine.nearestImage(level, stored, from);
        if (img.equals(stored))
            return null;
        WrapEngine.Folded f = WrapEngine.fold(level, img);
        return f == null ? null : new Image(img, f.iso());
    }

    /**
     * {@code at} as the block at {@code stored} sees it: moved through the band image nearest it, so a
     * player in front of a sign's image stands in front of the sign. Null when the stored position is
     * the nearest (nothing to change).
     */
    @Nullable
    public static Vec3 seenFrom(Level level, BlockPos stored, Vec3 at) {
        Image im = nearestBandImage(level, stored, at);
        return im == null ? null : WrapEngine.applyPoint(im.toStored(), at);
    }
}
