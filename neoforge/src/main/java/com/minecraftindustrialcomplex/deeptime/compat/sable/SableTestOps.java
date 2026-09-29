// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.sable;

import dev.ryanhcode.sable.api.SubLevelAssemblyHelper;
import dev.ryanhcode.sable.api.physics.handle.RigidBodyHandle;
import dev.ryanhcode.sable.api.sublevel.ServerSubLevelContainer;
import dev.ryanhcode.sable.api.sublevel.SubLevelContainer;
import dev.ryanhcode.sable.companion.math.BoundingBox3i;
import dev.ryanhcode.sable.companion.math.BoundingBox3ic;
import dev.ryanhcode.sable.sublevel.ServerSubLevel;
import dev.ryanhcode.sable.sublevel.SubLevel;
import dev.ryanhcode.sable.sublevel.storage.SubLevelRemovalReason;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.phys.Vec3;
import org.joml.Vector3d;
import org.joml.Vector3dc;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * Every Sable call the planet-server tests and the {@code sable} probe make (P0-06).
 *
 * <p>Sable (2.0.5, PolyForm Shield) is compileOnly and absent from the GameTest server, so its
 * types must never appear in a GameTest holder's method signatures: NeoForge's registration
 * loads every declared method of an enabled holder (cold-read review 2, gap 2). This class is
 * the only place that names Sable types, and its public methods take and return vanilla and JDK
 * types only. Callers check {@code ModList.get().isLoaded("sable")} before the first call, which
 * is when this class (and Sable's) would load.
 */
public final class SableTestOps {

    public static final String MOD_ID = "sable";

    /** A sub-level as the tests and the probe see it. */
    public record SubLevelInfo(UUID id, int blocks, Vec3 position, boolean removed) {}

    private SableTestOps() {}

    private static ServerSubLevelContainer container(ServerLevel level) {
        return SubLevelContainer.getContainer(level);
    }

    private static ServerSubLevel find(ServerLevel level, UUID id) {
        ServerSubLevelContainer c = container(level);
        SubLevel s = c == null ? null : c.getSubLevel(id);
        return s instanceof ServerSubLevel server ? server : null;
    }

    /**
     * Moves the blocks at {@code blocks} (world positions, contiguous in raw coordinates, so a
     * hull across the seam is given with x past C/2 rather than folded) into a new sub-level whose
     * pose sits at {@code anchor} + 0.5, and returns its id. Server thread only.
     */
    public static UUID assemble(ServerLevel level, BlockPos anchor, List<BlockPos> blocks) {
        ServerSubLevel s = SubLevelAssemblyHelper.assembleBlocks(level, anchor, blocks, BoundingBox3i.from(blocks));
        return s.getUniqueId();
    }

    /** Adds a linear velocity (blocks per second) to the sub-level's rigid body. */
    public static boolean addVelocity(ServerLevel level, UUID id, Vec3 linear) {
        ServerSubLevel s = find(level, id);
        if (s == null)
            return false;
        RigidBodyHandle.of(s).addLinearAndAngularVelocity(new Vector3d(linear.x, linear.y, linear.z), new Vector3d());
        return true;
    }

    /** The rigid body's linear velocity (blocks per second), or null when the sub-level is gone. */
    public static Vec3 velocity(ServerLevel level, UUID id) {
        ServerSubLevel s = find(level, id);
        if (s == null)
            return null;
        Vector3dc v = RigidBodyHandle.of(s).getLinearVelocity();
        return new Vec3(v.x(), v.y(), v.z());
    }

    /** The sub-level's pose position (world space), or null when it is gone. */
    public static Vec3 position(ServerLevel level, UUID id) {
        ServerSubLevel s = find(level, id);
        if (s == null)
            return null;
        Vector3dc p = s.logicalPose().position();
        return new Vec3(p.x(), p.y(), p.z());
    }

    /** Non-air blocks in the sub-level's plot, or -1 when it is gone. */
    public static int blockCount(ServerLevel level, UUID id) {
        ServerSubLevel s = find(level, id);
        return s == null ? -1 : countBlocks(level, s);
    }

    private static int countBlocks(ServerLevel level, SubLevel s) {
        // The plot's box is in the level's own block space (Sable's plots lie outside the world's
        // span, and TW treats them as a foreign frame it never folds).
        BoundingBox3ic box = s.getPlot().getBoundingBox();
        int n = 0;
        BlockPos.MutableBlockPos p = new BlockPos.MutableBlockPos();
        for (int x = box.minX(); x <= box.maxX(); x++)
            for (int y = box.minY(); y <= box.maxY(); y++)
                for (int z = box.minZ(); z <= box.maxZ(); z++)
                    if (!level.getBlockState(p.set(x, y, z)).isAir())
                        n++;
        return n;
    }

    public static boolean exists(ServerLevel level, UUID id) {
        ServerSubLevel s = find(level, id);
        return s != null && !s.isRemoved();
    }

    /** Removes the sub-level the way Sable's {@code /sable sub_level <id> remove} does. */
    public static boolean remove(ServerLevel level, UUID id) {
        ServerSubLevelContainer c = container(level);
        ServerSubLevel s = find(level, id);
        if (c == null || s == null)
            return false;
        var plot = s.getPlot();
        var origin = c.getOrigin();
        c.removeSubLevel(plot.plotPos.x - origin.x, plot.plotPos.z - origin.y, SubLevelRemovalReason.REMOVED);
        return true;
    }

    /** Every loaded sub-level of the level. */
    public static List<SubLevelInfo> list(ServerLevel level) {
        ServerSubLevelContainer c = container(level);
        List<SubLevelInfo> out = new ArrayList<>();
        if (c == null)
            return out;
        for (ServerSubLevel s : c.getAllSubLevels()) {
            Vector3dc p = s.logicalPose().position();
            out.add(new SubLevelInfo(s.getUniqueId(), countBlocks(level, s), new Vec3(p.x(), p.y(), p.z()), s.isRemoved()));
        }
        return out;
    }
}
