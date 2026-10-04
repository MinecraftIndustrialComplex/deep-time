// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.lighting.LightEngine;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.tick.LevelTickEvent;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Predicate;

/**
 * The wrap-engine spike's runtime (dev-only; docs/design/wrap-engine.md §3 and §6). Inert unless the
 * JVM runs with {@code -Ddeeptime.wrapspike=true}: without it the spike's mixins are not applied at all
 * ({@code DeepTimeMixinPlugin}) and nothing here is registered, so the shipped mod is unchanged.
 *
 * <p>The design it tests, "one stored copy, folded access, replicated band":
 * <ul>
 *   <li><b>Folded access.</b> Every block position that crosses the {@link Level} API (reads, writes,
 *       neighbour updates, scheduled ticks) is folded to the one stored block it is, and block states
 *       are turned on the way in and out ({@code mixin.wrapspike.*}). Block logic therefore always runs
 *       on stored positions, and only its reads reach across the seam: fluids, redstone and shape
 *       updates see the far side as plain neighbours.</li>
 *   <li><b>Replicated band.</b> The real chunks beyond each edge, out to the band depth, hold turned
 *       copies of what they show, written straight into the chunk sections whenever the stored block
 *       changes ({@link #replicate}). Code that reads chunk storage directly — collisions, light,
 *       heightmaps, chunk packets to the client — then sees the far side with no change of its own.</li>
 *   <li><b>Entities</b> are carried across the seam line by the motion, turned
 *       ({@link #carryEntities}).</li>
 * </ul>
 */
public final class WrapEngine {

    /** The dev flag: {@code -Ddeeptime.wrapspike=true}. */
    public static final boolean ENABLED = Boolean.getBoolean("deeptime.wrapspike");

    /**
     * Entities the carry leaves where they are: those whose position another system owns (Create's
     * contraptions follow their anchor or their track; compat/create/wrap/CreateWrapCompat).
     */
    public static final List<Predicate<Entity>> CARRY_VETO = new CopyOnWriteArrayList<>();

    private WrapEngine() {}

    /** Wires the entity carrier; call once from the mod constructor. Does nothing without the flag. */
    public static void init() {
        if (!ENABLED)
            return;
        DeepTime.LOGGER.warn("[wrapspike] Deep Time wrap-engine SPIKE is on (-Ddeeptime.wrapspike=true): dev only");
        // Create's adapters touch Create's classes, so not during mod construction (which runs beside
        // Create's own, and loading its entity classes early breaks Registrate's registration).
        if (net.neoforged.fml.ModList.get().isLoaded("create"))
            NeoForge.EVENT_BUS.addListener(net.neoforged.neoforge.event.server.ServerAboutToStartEvent.class,
                    e -> com.minecraftindustrialcomplex.deeptime.compat.create.wrap.CreateWrapCompat.init());
        NeoForge.EVENT_BUS.addListener(LevelTickEvent.Post.class, e -> {
            if (e.getLevel() instanceof ServerLevel sl)
                carryEntities(sl);
        });
    }

    /** The level's topology, or {@code null} (always null without the flag: the mixin is not applied). */
    public static WrapTopology topology(Level level) {
        return ENABLED ? ((WrapLevel) level).deeptime$wrap() : null;
    }

    /** Installs (or with {@code null} removes) a topology on a server level and its tick schedulers. */
    public static void install(ServerLevel level, WrapTopology topology) {
        if (!ENABLED)
            throw new IllegalStateException("the wrap-engine spike needs -Ddeeptime.wrapspike=true");
        ((WrapLevel) level).deeptime$setWrap(topology);
        ((WrapTicks) level.getBlockTicks()).deeptime$setWrap(topology);
        ((WrapTicks) level.getFluidTicks()).deeptime$setWrap(topology);
    }

    /** Minecraft's rotation for a motion's turn. */
    public static Rotation rotation(CellIsometry iso) {
        return switch (iso.quarterTurns()) {
            case 1 -> Rotation.CLOCKWISE_90;
            case 2 -> Rotation.CLOCKWISE_180;
            case 3 -> Rotation.COUNTERCLOCKWISE_90;
            default -> Rotation.NONE;
        };
    }

    /** The block position {@code pos} moved by {@code iso} (y unchanged). */
    public static BlockPos apply(CellIsometry iso, BlockPos pos) {
        int[] p = iso.apply(pos.getX(), pos.getZ());
        return new BlockPos(p[0], pos.getY(), p[1]);
    }

    /**
     * After a stored block changed: write its turned copy into every band block that shows it. Straight
     * into the chunk section (no onPlace, no neighbour updates, no ticks: the copy is a picture), then
     * heightmaps, light and the client, as {@code LevelChunk.setBlockState} does for a real change.
     */
    public static void replicate(ServerLevel level, WrapTopology t, BlockPos stored) {
        List<int[]> images = t.images(stored.getX(), stored.getZ());
        if (images.isEmpty())
            return;
        BlockState state = level.getBlockState(stored);
        for (int[] q : images) {
            WrapTopology.Fold f = t.fold(q[0], q[1]);
            BlockPos at = new BlockPos(q[0], stored.getY(), q[1]);
            writeCopy(level, at, state.rotate(rotation(f.iso().inverse())));
        }
    }

    /** Rebuilds every band copy in a box from the stored blocks (after a topology is installed). */
    public static int resyncBand(ServerLevel level, WrapTopology t, int minX, int maxX, int minY, int maxY, int minZ, int maxZ) {
        int n = 0;
        BlockPos.MutableBlockPos m = new BlockPos.MutableBlockPos();
        for (int x = minX; x <= maxX; x++)
            for (int z = minZ; z <= maxZ; z++) {
                WrapTopology.Fold f = t.fold(x, z);
                if (f == null)
                    continue;
                for (int y = minY; y <= maxY; y++) {
                    BlockState s = level.getBlockState(m.set(f.x(), y, f.z()));
                    writeCopy(level, new BlockPos(x, y, z), s.rotate(rotation(f.iso().inverse())));
                    n++;
                }
            }
        return n;
    }

    /** The raw chunk-section write of one band copy. */
    static void writeCopy(ServerLevel level, BlockPos at, BlockState state) {
        LevelChunk chunk = level.getChunkAt(at);
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

    /** The raw stored state at a position, bypassing the fold (tests: what the client and collisions see). */
    public static BlockState rawState(ServerLevel level, BlockPos pos) {
        return level.getChunkAt(pos).getBlockState(pos);
    }

    /**
     * Carries every entity whose position has crossed an edge to where it is, turned: position by the
     * motion, heading and velocity by its turn. Runs after each level tick. Players get a teleport (the
     * client sees the same scene, turned, because the band shows the far side); other entities move.
     */
    static void carryEntities(ServerLevel level) {
        WrapTopology t = topology(level);
        if (t == null)
            return;
        List<Entity> moving = new ArrayList<>();
        for (WrapEdge e : t.edges()) {
            AABB box = bandBox(level, e);
            if (box != null)
                level.getEntities((Entity) null, box, en -> en.getVehicle() == null && CARRY_VETO.stream().noneMatch(v -> v.test(en)))
                        .forEach(moving::add);
        }
        for (Entity en : moving) {
            WrapTopology.PointFold f = t.foldPoint(en.getX(), en.getZ());
            if (f == null)
                continue;
            float yaw = en.getYRot() + f.iso().yawDegrees();
            Vec3 v = en.getDeltaMovement();
            double[] rv = f.iso().applyVector(v.x, v.z);
            if (en instanceof ServerPlayer p) {
                p.connection.teleport(f.x(), p.getY(), f.z(), yaw, p.getXRot());
            } else {
                en.moveTo(f.x(), en.getY(), f.z(), yaw, en.getXRot());
                if (en instanceof LivingEntity le) {
                    le.setYHeadRot(le.getYHeadRot() + f.iso().yawDegrees());
                    le.setYBodyRot(le.yBodyRot + f.iso().yawDegrees());
                }
            }
            en.setDeltaMovement(rv[0], v.y, rv[1]);
            en.hasImpulse = true;
            DeepTime.LOGGER.info("[wrapspike] carried {} across: ({}, {}) -> ({}, {}), yaw +{}", en.getType().toShortString(),
                    String.format("%.2f", en.xo), String.format("%.2f", en.zo), String.format("%.2f", f.x()), String.format("%.2f", f.z()),
                    f.iso().yawDegrees());
        }
    }

    private static AABB bandBox(ServerLevel level, WrapEdge e) {
        // Spike: only bounded seams (the test sites). A whole-world edge needs a per-chunk index of
        // entities near the edge, not one level-wide box (design §6.3).
        if (e.depth() == Integer.MAX_VALUE || e.spanLo() == Integer.MIN_VALUE || e.spanHi() == Integer.MAX_VALUE)
            return null;
        double lo = e.spanLo();
        double hi = e.spanHi();
        double a = e.minSide() ? (double) e.coord() - e.depth() : e.coord();
        double b = e.minSide() ? e.coord() : (double) e.coord() + e.depth();
        double y0 = level.getMinBuildHeight(), y1 = level.getMaxBuildHeight();
        return e.normalX() ? new AABB(a, y0, lo, b, y1, hi) : new AABB(lo, y0, a, hi, y1, b);
    }
}
