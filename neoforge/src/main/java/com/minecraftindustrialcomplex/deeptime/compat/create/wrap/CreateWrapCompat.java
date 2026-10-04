// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapCrossingEvent;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine;
import com.simibubi.create.content.contraptions.AbstractContraptionEntity;
import net.minecraft.world.entity.Entity;
import net.neoforged.neoforge.common.NeoForge;

/**
 * Wires the wrap engine's Create adapters (docs/design/wrap-engine.md §11, "Create across turning
 * seams"). {@link WrapEngine#init} calls {@link #init} when the first server is about to start, and only
 * when the engine is on and Create is installed: not during mod construction, which runs beside
 * Create's own, where loading Create's classes early disturbs Registrate. The rest of the adapters are
 * mixins under {@code mixin.wrapengine.compat.create}, which DeepTimeMixinPlugin applies under the same
 * two conditions.
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
        // train's graph, which reset it every tick: the engine's carry would only make it jitter, and a
        // carried structure would stay unturned. Contraptions do not cross seams yet; assembly beyond a
        // seam is refused (ContraptionSeamMixin). Cancelling the crossing leaves the entity where it is.
        NeoForge.EVENT_BUS.addListener(WrapCrossingEvent.class, e -> {
            if (ownsItsPosition(e.entity()))
                e.setCanceled(true);
        });
        DeepTime.LOGGER.warn("[wrapengine] Create adapters on: kinetics and fluids cross turning seams; belts, chain conveyors, "
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
