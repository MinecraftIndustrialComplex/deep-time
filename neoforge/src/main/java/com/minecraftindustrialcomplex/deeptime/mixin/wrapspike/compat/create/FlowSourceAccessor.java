// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.simibubi.create.content.fluids.FlowSource;
import net.createmod.catnip.math.BlockFace;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

/** The face a pipe's flow source sits on (package-private in Create). */
@Mixin(value = FlowSource.class, remap = false)
public interface FlowSourceAccessor {
    @Accessor("location")
    BlockFace deeptime$location();
}
