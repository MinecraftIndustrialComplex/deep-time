// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.toroidalworld.api.v1.ToroidalShape;
import com.toroidalworld.api.v1.ToroidalWorldApi;
import net.minecraft.core.Direction;
import net.minecraft.server.level.ServerLevel;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Lets Deep Time's far-terrain generator through Toroidal World's Distant Horizons guard, and only
 * it (Ben, 2026-10-02: "Mixin around TW's guard"; decisions log).
 *
 * <p><b>TW's guard.</b> Toroidal World (MIT) 0.19.3 wraps DH's {@code WorldGeneratorInjector.bind}
 * and calls {@code DhGenerators.requireOwn(level, generator)}, which throws for every generator whose
 * class is outside {@code com.seibel.distanthorizons.} on a level TW shapes. The reason, read in its
 * code (decompiled on the test VM) and in its commit 4f7f03a "keep Distant Horizons' own generator
 * in shaped dimensions": TW makes the seam seamless by reshaping the level's own chunk generator
 * and by folding Distant Horizons' section keys into the canonical lap (its repository, retrieval
 * queue and LOD tree mixins). DH's own generator draws LODs from that chunk generator, so its LODs
 * meet at the seam the way the chunks do. A foreign generator does not know the seam, and its LODs
 * would not line up there (TW says the same of world types with a generator of their own: "the
 * terrain will not line up at the seam").
 *
 * <p><b>Why Deep Time's generator meets that.</b> It is asked only for the sections TW's fold leaves
 * (TW's retrieval mixin folds the position before DH calls any generator), and it samples the
 * same baked world data the level's chunk generator reads, folding every column's x into the
 * planet's lap: the world data wraps every C blocks, the period TW loops the level with
 * ({@code PlanetShape}). So a section on either side of the seam, or across it, is the terrain the
 * chunks there have. This class checks that premise on the level before letting the generator
 * through ({@link TwDhGuardRule#verdict}): the generator is Deep Time's and was made for this level, the level's
 * TW shape loops X with exactly the world data's circumference, and it does not loop Z (the world
 * data clamps at the poles). Anything else, and TW's guard runs as written.
 *
 * <p>Nothing of TW's seam handling is touched: its key folding, retrieval, seam ranking and LOD-tree
 * mixins keep working on Deep Time's sections exactly as on DH's own. The mixin is
 * {@code mixin/compat/toroidal_world/DhGeneratorsMixin}, gated by {@code TwDhGuardMixinPlugin}.
 */
public final class TwDhGuard {

    private static final Map<String, TwDhGuardRule.Verdict> SEEN = new ConcurrentHashMap<>();

    private TwDhGuard() {}

    /**
     * Called from TW's {@code DhGenerators.requireOwn} (head): true lets {@code generator} through
     * for {@code levelWrapper} (DH's {@code IDhApiLevelWrapper}); false leaves the decision to TW.
     * Never throws: on any error TW's guard runs.
     */
    public static boolean letThrough(Object levelWrapper, Object generator) {
        DhLodGenerator ours = DhLodGenerator.of(generator);
        if (ours == null)
            return false;   // not Deep Time's: TW decides
        TwDhGuardRule.Verdict v;
        String level = ours.levelName();
        try {
            v = check(levelWrapper, ours);
        } catch (ReflectiveOperationException | RuntimeException | LinkageError e) {
            v = new TwDhGuardRule.Verdict(false, true, "the check failed: " + e);
        }
        TwDhGuardRule.Verdict before = SEEN.put(level, v);
        if (!v.equals(before)) {
            if (!v.guarded())
                DeepTime.LOGGER.info("[deeptime] Toroidal World's Distant Horizons guard: not needed on {} ({})", level, v.reason());
            else if (v.pass())
                DeepTime.LOGGER.info("[deeptime] Toroidal World's Distant Horizons guard: Deep Time's far-terrain generator passes on {} ({})",
                        level, v.reason());
            else
                DeepTime.LOGGER.warn("[deeptime] Toroidal World's Distant Horizons guard: left to Toroidal World on {}: {}", level, v.reason());
        }
        return v.pass();
    }

    private static TwDhGuardRule.Verdict check(Object levelWrapper, DhLodGenerator ours) throws ReflectiveOperationException {
        DhApiBinding api = DhFarTerrain.api().orElse(null);
        if (api == null)
            return new TwDhGuardRule.Verdict(false, true, "Distant Horizons' API is not bound");
        if (!(api.wrappedLevel(levelWrapper) instanceof ServerLevel level))
            return new TwDhGuardRule.Verdict(false, true, "the level is not a server level");
        ToroidalShape shape = ToroidalWorldApi.shapeOf(level).orElse(null);
        int c = level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g.dataRef().circumferenceBlocks() : -1;
        return TwDhGuardRule.verdict(ours.levelName(), level.dimension().location().toString(), shape != null,
                shape != null && shape.loops(Direction.Axis.X), shape == null ? 0 : shape.widthBlocks(Direction.Axis.X),
                shape != null && shape.loops(Direction.Axis.Z), c);
    }

    /** What the guard decided for {@code level} ({@code minecraft:overworld}), or null if it never ran. */
    public static TwDhGuardRule.Verdict seen(String level) {
        return SEEN.get(level);
    }
}
