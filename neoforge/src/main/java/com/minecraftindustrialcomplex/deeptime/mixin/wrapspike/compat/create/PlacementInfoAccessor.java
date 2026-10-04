// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.simibubi.create.content.trains.track.TrackPlacement;
import net.minecraft.core.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

/** Track placement's result fields (package-private in Create). */
@Mixin(value = TrackPlacement.PlacementInfo.class, remap = false)
public interface PlacementInfoAccessor {
    @Accessor("valid")
    void deeptime$setValid(boolean valid);

    @Accessor("pos1")
    BlockPos deeptime$pos1();

    @Accessor("pos2")
    BlockPos deeptime$pos2();
}
