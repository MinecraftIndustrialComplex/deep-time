// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapEngine;
import com.simibubi.create.content.contraptions.AbstractContraptionEntity;
import net.minecraft.world.entity.Entity;

/**
 * Wires the wrap engine's Create adapters (docs/design/wrap-engine.md, "Create across turning seams").
 * Wired by {@link WrapEngine#init} (at server start) only when the engine is on and Create is installed; the rest of
 * the adapters are mixins under {@code mixin.wrapspike.compat.create}, which DeepTimeMixinPlugin
 * applies under the same two conditions.
 */
public final class CreateWrapCompat {

    private CreateWrapCompat() {}

    private static boolean done;

    /** Once, when the first server is about to start (after every registry is built). */
    public static synchronized void init() {
        if (done)
            return;
        done = true;
        // A contraption's position belongs to its anchor (a bearing, piston or gantry), its cart or its
        // train's graph, which reset it every tick: the engine's carry would only make it jitter, and
        // a carried structure would stay unturned. Contraptions do not cross seams yet; assembly
        // beyond a seam is refused (ContraptionSeamMixin).
        WrapEngine.CARRY_VETO.add(CreateWrapCompat::ownsItsPosition);
        DeepTime.LOGGER.warn("[wrapspike] Create adapters on: kinetics and fluids cross turning seams; belts, chain conveyors, "
                + "tracks and contraptions are refused at them");
    }

    static boolean ownsItsPosition(Entity e) {
        if (e instanceof AbstractContraptionEntity)
            return true;
        for (Entity p : e.getPassengers())
            if (p instanceof AbstractContraptionEntity)
                return true;
        return false;
    }
}
