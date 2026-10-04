// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import it.unimi.dsi.fastutil.longs.Long2IntOpenHashMap;
import it.unimi.dsi.fastutil.longs.LongIterator;
import it.unimi.dsi.fastutil.longs.LongLinkedOpenHashSet;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ChunkHolder;
import net.minecraft.server.level.ChunkLevel;
import net.minecraft.server.level.FullChunkStatus;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.level.TicketType;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.lighting.LightEngine;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.common.NeoForge;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * The replicated band of one server level (docs/design/wrap-engine.md §4.2): keeps every loaded band
 * chunk a turned copy of its source chunk, and carries entities across the edges.
 *
 * <ul>
 *   <li><b>Tickets.</b> A band chunk whose ticket level makes it loaded (≤ 33, FULL) holds a region
 *       ticket at the same level on its source chunk, so the source loads with it, ticks when it ticks
 *       and keeps entities when it does: the far side of a seam a player stands at is as alive as the
 *       near side. The level follows the band chunk's ({@code ChunkTicketLevelUpdatedEvent}), applied
 *       at the end of the tick (the event fires inside the distance manager's update).</li>
 *   <li><b>Copy on load.</b> A band chunk that loads waits in {@code pending} until its source is
 *       loaded too, then is rewritten from it block by block (only the blocks that differ), its
 *       generated block entities dropped and, when it was generated just now, its generated mobs
 *       removed (the source generated its own). Until then it shows its own generated terrain.</li>
 *   <li><b>Updates.</b> Every change of a stored block near an edge ({@code LevelChunk.setBlockState},
 *       the mixin) is written into each loaded band chunk that shows it ({@link #replicate}).</li>
 *   <li><b>Entities.</b> After each level tick, a root entity whose position is beyond an edge is
 *       carried to its stored position, turned ({@link #carry}).</li>
 * </ul>
 *
 * <p>Server thread only, apart from {@link #markDirty} (a write off the server thread queues its band
 * chunks for a fresh copy). Internal: the public methods are for the engine's mixins, not adapters.
 */
public final class BandManager {

    /** The ticket a band chunk holds on its source chunk; its value is the band chunk. */
    static final TicketType<ChunkPos> TICKET = TicketType.create("deeptime_wrap_band", Comparator.comparingLong(ChunkPos::toLong));
    private static final int FULL = ChunkLevel.byStatus(FullChunkStatus.FULL);
    /** Copy budget per tick: chunks, and nanoseconds. */
    private static final int MAX_COPIES_PER_TICK = 64;
    private static final long COPY_BUDGET_NS = 15_000_000L;

    final ServerLevel level;
    final WrapTopology topology;
    private final LongLinkedOpenHashSet pending = new LongLinkedOpenHashSet();
    private final LongOpenHashSet synced = new LongOpenHashSet();
    private final LongOpenHashSet fresh = new LongOpenHashSet();
    /** Band chunk → the level of the ticket it holds on its source (absent: none). */
    private final Long2IntOpenHashMap ticketLevel = new Long2IntOpenHashMap();
    /** Band chunk → its latest ticket level, from the event, not yet applied. */
    private final Long2IntOpenHashMap wantedLevel = new Long2IntOpenHashMap();
    private final ConcurrentLinkedQueue<Long> dirtyOffThread = new ConcurrentLinkedQueue<>();
    private long copies, copiedBlocks, replicas, carried;
    private boolean closed;

    BandManager(ServerLevel level, WrapTopology topology) {
        this.level = level;
        this.topology = topology;
        ticketLevel.defaultReturnValue(Integer.MAX_VALUE);
    }

    /** After install: every band chunk already loaded gets its ticket and a copy. */
    void start() {
        topology.forEachBandChunk(key -> {
            ChunkHolder h = level.getChunkSource().chunkMap.getVisibleChunkIfPresent(key);
            if (h != null) {
                wantedLevel.put(key, h.getTicketLevel());
                if (level.getChunkSource().getChunkNow(ChunkPos.getX(key), ChunkPos.getZ(key)) != null)
                    pending.add(key);
            }
        });
        DeepTime.LOGGER.info("[wrapengine] band manager started: {} band chunks loaded", pending.size());
    }

    /** On uninstall: release every ticket. */
    void close() {
        closed = true;
        for (Long2IntOpenHashMap.Entry e : ticketLevel.long2IntEntrySet())
            level.getChunkSource().removeRegionTicket(TICKET, source(e.getLongKey()), FULL - e.getIntValue(), new ChunkPos(e.getLongKey()));
        ticketLevel.clear();
    }

    boolean isSynced(long key) {
        return synced.contains(key);
    }

    String stats() {
        return "copies=" + copies + " blocks=" + copiedBlocks + " replicas=" + replicas + " carried=" + carried + " pending=" + pending.size()
                + " synced=" + synced.size() + " tickets=" + ticketLevel.size();
    }

    // --- Events ---------------------------------------------------------------------------------

    void onLevelChange(long key, int newLevel) {
        if (closed || !topology.mayFold(ChunkPos.getX(key) << 4, ChunkPos.getZ(key) << 4))
            return;
        if (topology.chunkFold(ChunkPos.getX(key), ChunkPos.getZ(key)) == null)
            return;
        wantedLevel.put(key, newLevel);
    }

    void onLoad(LevelChunk chunk, boolean isNew) {
        if (closed || ((WrapChunk) chunk).deeptime$bandIso() == null)
            return;
        long key = chunk.getPos().toLong();
        pending.add(key);
        synced.remove(key);
        if (isNew)
            fresh.add(key);
    }

    void onUnload(long key) {
        pending.remove(key);
        synced.remove(key);
        fresh.remove(key);
    }

    /** A stored chunk changed off the server thread: its band chunks are copied again. */
    void markDirty(long bandKey) {
        dirtyOffThread.add(bandKey);
    }

    // --- The tick -------------------------------------------------------------------------------

    void tick() {
        if (closed)
            return;
        applyTickets();
        Long d;
        while ((d = dirtyOffThread.poll()) != null)
            if (level.getChunkSource().getChunkNow(ChunkPos.getX(d), ChunkPos.getZ(d)) != null) {
                pending.add(d.longValue());
                synced.remove(d.longValue());
            }
        copyPending();
        carry();
    }

    private void applyTickets() {
        if (wantedLevel.isEmpty())
            return;
        for (Long2IntOpenHashMap.Entry e : wantedLevel.long2IntEntrySet()) {
            long key = e.getLongKey();
            int want = e.getIntValue() <= FULL ? e.getIntValue() : Integer.MAX_VALUE;
            int have = ticketLevel.get(key);
            if (want == have)
                continue;
            ChunkPos src = source(key);
            ChunkPos band = new ChunkPos(key);
            if (have != Integer.MAX_VALUE)
                level.getChunkSource().removeRegionTicket(TICKET, src, FULL - have, band);
            if (want != Integer.MAX_VALUE) {
                level.getChunkSource().addRegionTicket(TICKET, src, FULL - want, band);
                ticketLevel.put(key, want);
            } else {
                ticketLevel.remove(key);
            }
        }
        wantedLevel.clear();
    }

    private ChunkPos source(long bandKey) {
        WrapTopology.ChunkFold cf = topology.chunkFold(ChunkPos.getX(bandKey), ChunkPos.getZ(bandKey));
        return new ChunkPos(cf.sourceX(), cf.sourceZ());
    }

    private void copyPending() {
        if (pending.isEmpty())
            return;
        long t0 = System.nanoTime();
        int n = 0;
        LongIterator it = pending.iterator();
        while (it.hasNext() && n < MAX_COPIES_PER_TICK && System.nanoTime() - t0 < COPY_BUDGET_NS) {
            long key = it.nextLong();
            int bx = ChunkPos.getX(key), bz = ChunkPos.getZ(key);
            LevelChunk band = level.getChunkSource().getChunkNow(bx, bz);
            WrapTopology.ChunkFold cf = topology.chunkFold(bx, bz);
            if (band == null || cf == null) {
                if (cf == null)
                    it.remove();
                continue;
            }
            LevelChunk src = level.getChunkSource().getChunkNow(cf.sourceX(), cf.sourceZ());
            if (src == null)
                continue; // its ticket loads it; try again next tick
            copy(band, src, cf.iso());
            if (fresh.remove(key))
                dropGeneratedMobs(band);
            it.remove();
            synced.add(key);
            n++;
        }
    }

    /** Rewrites a band chunk from its source: every block that differs from the source's turned state. */
    private void copy(LevelChunk band, LevelChunk src, CellIsometry iso) {
        Rotation toBand = WrapEngine.rotation(iso.inverse());
        Map<BlockState, BlockState> turned = new IdentityHashMap<>();
        LevelChunkSection[] bs = band.getSections(), ss = src.getSections();
        int bx0 = band.getPos().getMinBlockX(), bz0 = band.getPos().getMinBlockZ();
        long written = 0;
        for (int si = 0; si < bs.length; si++) {
            LevelChunkSection b = bs[si], s = ss[si];
            if (b.hasOnlyAir() && s.hasOnlyAir())
                continue;
            int y0 = band.getSectionYFromSectionIndex(si) << 4;
            for (int lz = 0; lz < 16; lz++) {
                for (int lx = 0; lx < 16; lx++) {
                    int x = bx0 + lx, z = bz0 + lz;
                    int sx = iso.applyX(x, z) & 15, sz = iso.applyZ(x, z) & 15;
                    for (int ly = 0; ly < 16; ly++) {
                        BlockState want = turned.computeIfAbsent(s.getBlockState(sx, ly, sz), st -> st.rotate(toBand));
                        if (b.getBlockState(lx, ly, lz) != want) {
                            writeCopy(band, new BlockPos(x, y0 + ly, z), want);
                            written++;
                        }
                    }
                }
            }
        }
        // Copies carry no block entities (design §4.2): drop what the band chunk generated.
        for (BlockPos p : new ArrayList<>(band.getBlockEntities().keySet()))
            band.removeBlockEntity(p);
        band.setUnsaved(true);
        copies++;
        copiedBlocks += written;
    }

    private void dropGeneratedMobs(LevelChunk band) {
        ChunkPos p = band.getPos();
        AABB box = new AABB(p.getMinBlockX(), level.getMinBuildHeight(), p.getMinBlockZ(), p.getMaxBlockX() + 1, level.getMaxBuildHeight(),
                p.getMaxBlockZ() + 1);
        for (Entity e : level.getEntities((Entity) null, box, e -> !(e instanceof Player)))
            e.discard();
    }

    /**
     * After a stored block changed: its turned copy into every loaded band chunk that shows it. Unloaded
     * band chunks are copied when they load.
     */
    public void replicate(LevelChunk stored, BlockPos pos, CellIsometry[] images) {
        if (closed)
            return;
        boolean onThread = level.getServer().isSameThread();
        BlockState state = stored.getBlockState(pos);
        for (CellIsometry g : images) {
            int x = g.applyX(pos.getX(), pos.getZ()), z = g.applyZ(pos.getX(), pos.getZ());
            if (!onThread) {
                markDirty(ChunkPos.asLong(x >> 4, z >> 4));
                continue;
            }
            LevelChunk band = level.getChunkSource().getChunkNow(x >> 4, z >> 4);
            if (band == null)
                continue;
            writeCopy(band, new BlockPos(x, pos.getY(), z), state.rotate(WrapEngine.rotation(g)));
            replicas++;
        }
    }

    /**
     * The raw chunk-section write of one band copy: no onPlace, onRemove, neighbour updates or ticks
     * (the copy is a picture), then heightmaps, light and the client, as {@code LevelChunk.setBlockState}
     * does for a real change.
     */
    private void writeCopy(LevelChunk chunk, BlockPos at, BlockState state) {
        int y = at.getY();
        if (level.isOutsideBuildHeight(y))
            return;
        LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
        boolean wasEmpty = section.hasOnlyAir();
        int lx = at.getX() & 15, ly = y & 15, lz = at.getZ() & 15;
        BlockState old = section.setBlockState(lx, ly, lz, state, false);
        if (old == state)
            return;
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.MOTION_BLOCKING).update(lx, y, lz, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES).update(lx, y, lz, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.OCEAN_FLOOR).update(lx, y, lz, state);
        chunk.getOrCreateHeightmapUnprimed(Heightmap.Types.WORLD_SURFACE).update(lx, y, lz, state);
        boolean isEmpty = section.hasOnlyAir();
        if (wasEmpty != isEmpty)
            level.getChunkSource().getLightEngine().updateSectionStatus(at, isEmpty);
        if (LightEngine.hasDifferentLightProperties(chunk, at, old, state)) {
            chunk.getSkyLightSources().update(chunk, lx, y, lz);
            level.getChunkSource().getLightEngine().checkBlock(at);
        }
        level.getChunkSource().blockChanged(at);
        chunk.setUnsaved(true);
    }

    /**
     * A direct {@code LevelChunk.setBlockState} into a band chunk (a mod or tool writing chunk storage):
     * written to the stored block instead, turned, through the source chunk's own {@code setBlockState}
     * (so it replicates back into this band chunk). Returns what that returns, turned back.
     */
    public BlockState redirectChunkWrite(LevelChunk band, CellIsometry iso, BlockPos pos, BlockState state, boolean isMoving) {
        int sx = iso.applyX(pos.getX(), pos.getZ()), sz = iso.applyZ(pos.getX(), pos.getZ());
        LevelChunk src = level.getChunk(sx >> 4, sz >> 4);
        BlockState old = src.setBlockState(new BlockPos(sx, pos.getY(), sz), state.rotate(WrapEngine.rotation(iso)), isMoving);
        return old == null ? null : old.rotate(WrapEngine.rotation(iso.inverse()));
    }

    // --- Entities -------------------------------------------------------------------------------

    private record Crossing(Entity entity, WrapTopology.PointFold fold) {}

    /** Carries every root entity whose position is beyond an edge to where it is, turned. */
    private void carry() {
        List<Crossing> moving = null;
        for (Entity e : level.getAllEntities()) {
            if (e.isRemoved() || e.isPassenger() || !topology.mayFold(Mth.floor(e.getX()), Mth.floor(e.getZ())))
                continue;
            WrapTopology.PointFold f = topology.foldPoint(e.getX(), e.getZ());
            if (f == null)
                continue;
            if (moving == null)
                moving = new ArrayList<>();
            moving.add(new Crossing(e, f));
        }
        if (moving == null)
            return;
        for (Crossing c : moving)
            carry(c.entity(), c.fold());
    }

    private void carry(Entity en, WrapTopology.PointFold f) {
        Vec3 from = en.position();
        Vec3 to = new Vec3(f.x(), en.getY(), f.z());
        if (NeoForge.EVENT_BUS.post(new WrapCrossingEvent(level, en, f.iso(), from, to)).isCanceled())
            return;
        float dyaw = f.iso().yawDegrees();
        if (en instanceof ServerPlayer p) {
            p.connection.teleport(to.x, to.y, to.z, p.getYRot() + dyaw, p.getXRot());
            turnMotion(p, f.iso());
            p.hurtMarked = true;
        } else {
            en.moveTo(to.x, to.y, to.z, en.getYRot() + dyaw, en.getXRot());
            turnHeading(en, dyaw);
            turnMotion(en, f.iso());
            en.hasImpulse = true;
            if (en instanceof Mob m)
                m.getNavigation().stop();
        }
        for (Entity p : en.getIndirectPassengers()) {
            p.setYRot(p.getYRot() + dyaw);
            turnHeading(p, dyaw);
            turnMotion(p, f.iso());
        }
        positionRiders(en);
        carried++;
        DeepTime.LOGGER.debug("[wrapengine] carried {} across: ({}, {}) -> ({}, {}), yaw +{}", en.getType().toShortString(),
                String.format("%.2f", from.x), String.format("%.2f", from.z), String.format("%.2f", to.x), String.format("%.2f", to.z), dyaw);
    }

    private static void turnHeading(Entity en, float dyaw) {
        if (en instanceof LivingEntity le) {
            le.setYHeadRot(le.getYHeadRot() + dyaw);
            le.setYBodyRot(le.yBodyRot + dyaw);
        }
    }

    private static void turnMotion(Entity en, CellIsometry iso) {
        Vec3 v = en.getDeltaMovement();
        double[] rv = iso.applyVector(v.x, v.z);
        en.setDeltaMovement(rv[0], v.y, rv[1]);
    }

    private static void positionRiders(Entity vehicle) {
        for (Entity p : vehicle.getPassengers()) {
            vehicle.positionRider(p);
            positionRiders(p);
        }
    }
}
