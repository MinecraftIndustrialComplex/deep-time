// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.level.ChunkEvent;
import net.neoforged.neoforge.event.level.ChunkTicketLevelUpdatedEvent;
import net.neoforged.neoforge.event.level.LevelEvent;
import net.neoforged.neoforge.event.tick.LevelTickEvent;
import org.jetbrains.annotations.Nullable;

/**
 * Deep Time's wrap engine: half-turn and quarter-turn seams without portals (docs/design/wrap-engine.md).
 * This class is the engine's <b>stable API</b> (design doc §9): other Deep Time code and its adapters
 * (Create, Sable, Distant Horizons) call only what is here, in {@link WrapTopology}, {@link WrapLayouts},
 * {@link CellIsometry}, {@link WrapEdge} and {@link WrapCrossingEvent}.
 *
 * <p>The design, "one stored copy, folded access, replicated band":
 * <ul>
 *   <li><b>Folded access.</b> Every block position that crosses the {@code Level} API (reads, writes,
 *       fluids, block entities, neighbour updates, scheduled ticks) is folded to the one stored block
 *       it is, and block states are turned on the way in and out ({@code mixin.wrapengine.*}). Block
 *       logic always runs on stored positions; only its reads reach across a seam, so fluids,
 *       redstone and shape updates see the far side as plain neighbours. A direct
 *       {@code LevelChunk.setBlockState} into a band chunk is redirected to the stored block too.</li>
 *   <li><b>Replicated band.</b> The real chunks beyond each edge, out to the band depth, hold turned
 *       copies of what they show ({@link BandManager}): copied when a band chunk loads (it holds a
 *       ticket on its source chunk, which therefore loads too, at the band chunk's own level), and
 *       rewritten whenever a stored block changes. Code that reads chunk storage directly —
 *       collisions, light, heightmaps, chunk packets, so the vanilla client — sees the far side with no
 *       change of its own.</li>
 *   <li><b>Entities</b> that pass an edge are carried to where they are, heading and velocity turned
 *       ({@link WrapCrossingEvent} lets an adapter take a crossing over).</li>
 * </ul>
 *
 * <p><b>Off by default.</b> Nothing happens unless the JVM runs with {@code -Ddeeptime.wrapengine=true}
 * (the flag): without it the engine's mixins are never woven ({@code DeepTimeMixinPlugin}), nothing
 * here registers, {@link #topology} returns null and {@link #install} refuses. Even with the flag a
 * level is unchanged until a topology is installed on it.
 */
public final class WrapEngine {

    /** The dev flag's system property. */
    public static final String FLAG = "deeptime.wrapengine";
    /** True when the JVM runs with {@code -Ddeeptime.wrapengine=true}. */
    public static final boolean ENABLED = Boolean.getBoolean(FLAG);

    private static final Rotation[] ROTATIONS = {Rotation.NONE, Rotation.CLOCKWISE_90, Rotation.CLOCKWISE_180, Rotation.COUNTERCLOCKWISE_90};

    private WrapEngine() {}

    /** Wires the band manager's events; call once from the mod constructor. Does nothing without the flag. */
    public static void init() {
        if (!ENABLED)
            return;
        DeepTime.LOGGER.warn("[wrapengine] Deep Time wrap engine is on (-D{}=true): dev only", FLAG);
        NeoForge.EVENT_BUS.addListener(LevelTickEvent.Post.class, e -> {
            if (e.getLevel() instanceof ServerLevel sl) {
                BandManager b = bands(sl);
                if (b != null)
                    b.tick();
            }
        });
        NeoForge.EVENT_BUS.addListener(ChunkEvent.Load.class, e -> {
            if (e.getLevel() instanceof ServerLevel sl && e.getChunk() instanceof LevelChunk c) {
                BandManager b = bands(sl);
                if (b != null)
                    b.onLoad(c, e.isNewChunk());
            }
        });
        NeoForge.EVENT_BUS.addListener(ChunkEvent.Unload.class, e -> {
            if (e.getLevel() instanceof ServerLevel sl) {
                BandManager b = bands(sl);
                if (b != null)
                    b.onUnload(e.getChunk().getPos().toLong());
            }
        });
        NeoForge.EVENT_BUS.addListener(ChunkTicketLevelUpdatedEvent.class, e -> {
            BandManager b = bands(e.getLevel());
            if (b != null)
                b.onLevelChange(e.getChunkPos(), e.getNewTicketLevel());
        });
        NeoForge.EVENT_BUS.addListener(LevelEvent.Unload.class, e -> {
            if (e.getLevel() instanceof ServerLevel sl && bands(sl) != null)
                ((WrapLevel) sl).deeptime$setBands(null);
        });
    }

    /** True when the engine's flag is on (its mixins are woven). */
    public static boolean enabled() {
        return ENABLED;
    }

    /** The level's topology, or {@code null} (always null without the flag, and on the client). */
    @Nullable
    public static WrapTopology topology(Level level) {
        return ENABLED ? ((WrapLevel) level).deeptime$wrap() : null;
    }

    /**
     * Installs a topology on a server level (or with {@code null} removes it): folded access starts at
     * once, the band manager copies every loaded band chunk from its source within a few ticks.
     * Server thread only. The topology must be chunk-aligned ({@link WrapTopology#isChunkAligned}).
     */
    public static void install(ServerLevel level, @Nullable WrapTopology topology) {
        if (!ENABLED)
            throw new IllegalStateException("the wrap engine needs -D" + FLAG + "=true");
        if (!level.getServer().isSameThread())
            throw new IllegalStateException("WrapEngine.install off the server thread");
        if (topology != null && !topology.isChunkAligned())
            throw new IllegalArgumentException(topology + " is not chunk-aligned: every edge, span, depth and turn centre must lie on the chunk grid");
        BandManager old = bands(level);
        if (old != null)
            old.close();
        WrapLevel wl = (WrapLevel) level;
        wl.deeptime$setWrap(topology);
        ((WrapTicks) level.getBlockTicks()).deeptime$setWrap(topology);
        ((WrapTicks) level.getFluidTicks()).deeptime$setWrap(topology);
        BandManager bands = topology == null ? null : new BandManager(level, topology);
        wl.deeptime$setBands(bands);
        if (bands != null)
            bands.start();
        DeepTime.LOGGER.info("[wrapengine] {} on {}", topology == null ? "topology removed" : "installed " + topology, level.dimension().location());
    }

    /**
     * Benchmarks only: sets the topology the folded read path of {@code level} sees, without the tick
     * schedulers or a band manager, and returns the one it had. The GameTest benchmark swaps a topology
     * in and back within one call, so the functional tests' band state is not disturbed.
     */
    public static WrapTopology swapReadTopologyForBenchmark(ServerLevel level, @Nullable WrapTopology topology) {
        if (!ENABLED)
            throw new IllegalStateException("the wrap engine needs -D" + FLAG + "=true");
        WrapLevel wl = (WrapLevel) level;
        WrapTopology old = wl.deeptime$wrap();
        wl.deeptime$setWrap(topology);
        return old;
    }

    /** The level's band manager, or {@code null}. */
    @Nullable
    static BandManager bands(ServerLevel level) {
        return ENABLED ? ((WrapLevel) level).deeptime$bands() : null;
    }

    // --- Folding (the stable API) ---------------------------------------------------------------

    /**
     * A band position folded: the stored block it is and the motion from the band position to it.
     *
     * @param pos the stored position (same y)
     * @param iso band → stored
     */
    public record Folded(BlockPos pos, CellIsometry iso) {
        /** The turn a state takes from the band's frame into the stored frame (a write). */
        public Rotation toStored() {
            return rotation(iso);
        }

        /** The turn a stored state takes into the band's frame (a read). */
        public Rotation toBand() {
            return rotation(iso.inverse());
        }
    }

    /** {@code pos} folded, or {@code null} when it is not a band position (or the level has no topology). */
    @Nullable
    public static Folded fold(Level level, BlockPos pos) {
        WrapTopology t = topology(level);
        if (t == null)
            return null;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        return f == null ? null : new Folded(new BlockPos(f.x(), pos.getY(), f.z()), f.iso());
    }

    /**
     * The stored position of {@code pos}: its fold, or {@code pos} itself. Two positions are the same block
     * when their canonical positions are equal; compare these, not raw positions, in multiblock and
     * network code (a block entity's {@code getBlockPos()} is always canonical).
     */
    public static BlockPos canonical(Level level, BlockPos pos) {
        Folded f = fold(level, pos);
        return f == null ? pos : f.pos();
    }

    /** True when the chunk is a band chunk of the level's topology (its storage holds turned copies, read-only). */
    public static boolean isBandChunk(Level level, int chunkX, int chunkZ) {
        WrapTopology t = topology(level);
        return t != null && t.chunkFold(chunkX, chunkZ) != null;
    }

    /** True when the band chunk is loaded and its copy is up to date with its source (it was copied since it loaded). */
    public static boolean isBandSynced(ServerLevel level, int chunkX, int chunkZ) {
        BandManager b = bands(level);
        return b != null && b.isSynced(ChunkPos.asLong(chunkX, chunkZ));
    }

    /** The band manager's counters for logs and tests ({@code copies=… blocks=… replicas=… carried=…}). */
    public static String stats(ServerLevel level) {
        BandManager b = bands(level);
        return b == null ? "no topology" : b.stats();
    }

    /** The raw state in chunk storage at a position, bypassing the fold (what collisions and the client see). */
    public static BlockState rawState(Level level, BlockPos pos) {
        return level.getChunkAt(pos).getBlockState(pos);
    }

    // --- Motions on Minecraft types --------------------------------------------------------------

    /** Minecraft's rotation for a motion's turn. */
    public static Rotation rotation(CellIsometry iso) {
        return ROTATIONS[iso.quarterTurns()];
    }

    /** The block position {@code pos} moved by {@code iso} (y unchanged). */
    public static BlockPos apply(CellIsometry iso, BlockPos pos) {
        return new BlockPos(iso.applyX(pos.getX(), pos.getZ()), pos.getY(), iso.applyZ(pos.getX(), pos.getZ()));
    }

    /** A direction turned by {@code iso} (up and down unchanged). */
    public static Direction turn(CellIsometry iso, Direction d) {
        return rotation(iso).rotate(d);
    }

    /** A point (an entity's position) moved by {@code iso}. */
    public static Vec3 applyPoint(CellIsometry iso, Vec3 p) {
        double[] q = iso.applyPoint(p.x, p.z);
        return new Vec3(q[0], p.y, q[1]);
    }

    /** A vector (a velocity, a look direction) turned by {@code iso}. */
    public static Vec3 applyVector(CellIsometry iso, Vec3 v) {
        double[] q = iso.applyVector(v.x, v.z);
        return new Vec3(q[0], v.y, q[1]);
    }
}
