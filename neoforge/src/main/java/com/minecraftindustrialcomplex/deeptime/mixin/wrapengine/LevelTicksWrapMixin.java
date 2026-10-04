// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTicks;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.world.ticks.LevelTicks;
import net.minecraft.world.ticks.ScheduledTick;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Scheduled block and fluid ticks for band positions are kept at the stored position, so the tick
 * fires on the stored block (docs/design/wrap-engine.md §4.1); the queries ({@code hasScheduledTick},
 * {@code willTickThisTick}) fold the same way. The topology reaches the schedulers through a field set
 * at install. Applied only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(LevelTicks.class)
abstract class LevelTicksWrapMixin implements WrapTicks {

    @Unique
    private WrapTopology deeptime$wrapTopology;

    @Override
    public void deeptime$setWrap(WrapTopology topology) {
        deeptime$wrapTopology = topology;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    @Inject(method = "schedule", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldSchedule(ScheduledTick tick, CallbackInfo ci) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        BlockPos pos = tick.pos();
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        ((LevelTicks) (Object) this).schedule(new ScheduledTick(tick.type(), new BlockPos(f.x(), pos.getY(), f.z()),
                tick.triggerTick(), tick.priority(), tick.subTickOrder()));
        ci.cancel();
    }

    @ModifyVariable(method = "hasScheduledTick", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldHasScheduled(BlockPos pos) {
        return deeptime$fold(pos);
    }

    @ModifyVariable(method = "willTickThisTick", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldWillTick(BlockPos pos) {
        return deeptime$fold(pos);
    }

    @Unique
    private BlockPos deeptime$fold(BlockPos pos) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return pos;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        return f == null ? pos : new BlockPos(f.x(), pos.getY(), f.z());
    }
}
