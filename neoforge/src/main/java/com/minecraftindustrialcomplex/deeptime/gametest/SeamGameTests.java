// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The seam in a real Toroidal World-shaped Deep Time overworld (P0-06). Planet namespace: these
 * run only on the dev planet server (./gametest-planet.sh), whose overworld is the planet; the
 * GameTest server's flat overworld has no seam. The harness positions them next to x = C/2.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class SeamGameTests {

    private SeamGameTests() {}

    static int circumference(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            helper.fail("the test level is not a Deep Time planet: " + level.getChunkSource().getGenerator().getClass().getName());
            throw new IllegalStateException();
        }
        return g.dataRef().circumferenceBlocks();
    }

    /**
     * Across the seam the world continues: column x = C/2 + k is the same column as x = −C/2 + k
     * (TW folds it), and the step from x = C/2 − 1 to x = C/2 is no larger than the terrain's own
     * steps next to it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void columnsContinuous(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        int folded = 0, jumps = 0;
        StringBuilder detail = new StringBuilder();
        // Rows inside the strip the harness forceloads around the seam (x = C/2 ± 32, z ± 32), so no
        // far chunk is generated synchronously on the server thread.
        for (int i = 0; i < 16; i++) {
            int z = (i - 8) * 4 + 1;
            for (int k = 0; k < 4; k++) {
                int beyond = level.getHeight(Heightmap.Types.WORLD_SURFACE, c / 2 + k, z);
                int canonical = level.getHeight(Heightmap.Types.WORLD_SURFACE, -c / 2 + k, z);
                if (beyond != canonical) {
                    folded++;
                    detail.append(" fold(z=").append(z).append(",k=").append(k).append("):").append(beyond).append("!=").append(canonical);
                }
            }
            int a = level.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 2, z);
            int b = level.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 1, z);
            int across = level.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2, z);
            int after = level.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 + 1, z);
            if (Math.abs(across - b) > Math.max(Math.max(Math.abs(b - a), Math.abs(after - across)), 2)) {
                jumps++;
                detail.append(" jump(z=").append(z).append("):").append(b).append("->").append(across);
            }
        }
        DeepTime.LOGGER.info("[gametest] columnsContinuous C={} foldMismatches={} seamJumps={}{}", c, folded, jumps, detail);
        helper.assertTrue(folded == 0 && jumps == 0, "seam discontinuities:" + detail);
        helper.succeed();
    }

    /** An entity moved across the seam is still one entity, now inside the canonical span. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void entityCrossesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        ArmorStand stand = EntityType.ARMOR_STAND.create(level);
        if (stand == null) {
            helper.fail("could not create an armor stand");
            return;
        }
        double y = level.getHeight(Heightmap.Types.MOTION_BLOCKING, c / 2 - 2, 0) + 1;
        stand.setNoGravity(true);
        stand.moveTo(c / 2.0 - 1.5, y, 0.5, 0, 0);
        level.addFreshEntity(stand);
        var id = stand.getUUID();
        // Walk it east across x = C/2 through Entity.move, a quarter block per tick: TW moves an
        // entity that steps over the border to the other side in the same tick. (A bare setPos
        // is a teleport and is not folded.)
        for (int t = 0; t < 16; t++)
            helper.runAfterDelay(5 + t, () -> stand.move(net.minecraft.world.entity.MoverType.SELF,
                    new net.minecraft.world.phys.Vec3(0.25, 0, 0)));
        helper.runAfterDelay(40, () -> {
            var found = level.getEntity(id);
            double x = found == null ? Double.NaN : found.getX();
            boolean inSpan = found != null && x >= -c / 2.0 && x < c / 2.0;
            DeepTime.LOGGER.info("[gametest] entityCrossesSeam C={} found={} x={} inCanonicalSpan={}", c, found != null, x, inSpan);
            if (found != null)
                found.discard();
            helper.assertTrue(found != null, "the entity vanished at the seam");
            helper.assertTrue(inSpan, "entity at x=" + x + " is outside the canonical span [" + (-c / 2) + ", " + (c / 2) + ")");
            helper.succeed();
        });
    }
}
