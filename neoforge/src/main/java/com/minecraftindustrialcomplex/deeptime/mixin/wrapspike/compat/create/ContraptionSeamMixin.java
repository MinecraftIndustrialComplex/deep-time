// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamRefusals;
import com.simibubi.create.content.contraptions.AssemblyException;
import com.simibubi.create.content.contraptions.Contraption;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.network.chat.Component;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate.StructureBlockInfo;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Map;

/**
 * A moving structure (bearing, piston, pulley, gantry, minecart or train contraption) whose blocks reach
 * beyond a turning seam is refused at assembly with Create's own assembly message, shown on the
 * controller's goggle tooltip (docs/design/wrap-engine.md, "Create across turning seams"): captured
 * block-entity data, multiblock controllers and glue would not be turned. Applied only with
 * {@code -Ddeeptime.wrapspike=true} and Create installed.
 */
@Mixin(value = Contraption.class, remap = false)
abstract class ContraptionSeamMixin {

    @Shadow
    public BlockPos anchor;

    @Shadow
    protected Map<BlockPos, StructureBlockInfo> blocks;

    @Inject(method = "searchMovedStructure", at = @At("RETURN"))
    private void deeptime$refuseBeyondSeam(Level world, BlockPos pos, Direction forcedDirection, CallbackInfoReturnable<Boolean> cir)
            throws AssemblyException {
        if (!cir.getReturnValueZ() || !SeamFrames.active(world) || anchor == null)
            return;
        for (BlockPos local : blocks.keySet()) {
            BlockPos at = local.offset(anchor);
            if (SeamFrames.fold(world, at) != null) {
                SeamRefusals.refuse(world, pos, SeamRefusals.Kind.CONTRAPTION, "block at " + at.toShortString() + " beyond a seam");
                throw new AssemblyException(Component.translatable(SeamRefusals.Kind.CONTRAPTION.key));
            }
        }
    }
}
