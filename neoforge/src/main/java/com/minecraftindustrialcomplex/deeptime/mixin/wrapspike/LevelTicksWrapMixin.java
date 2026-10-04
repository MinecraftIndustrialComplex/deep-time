// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike;

import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapTicks;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.world.ticks.LevelTicks;
import net.minecraft.world.ticks.ScheduledTick;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Scheduled block and fluid ticks for band positions are kept at the stored position, so the tick
 * fires on the stored block (docs/design/wrap-engine.md §3.2). Dev only ({@code -Ddeeptime.wrapspike=true}).
 */
@Mixin(LevelTicks.class)
abstract class LevelTicksWrapMixin implements WrapTicks {

    @Unique
    private volatile WrapTopology deeptime$wrapTopology;

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

    @Inject(method = "hasScheduledTick", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldHasScheduled(BlockPos pos, Object type, CallbackInfoReturnable<Boolean> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        cir.setReturnValue(((LevelTicks<?>) (Object) this).hasScheduledTick(new BlockPos(f.x(), pos.getY(), f.z()), cast(type)));
    }

    @SuppressWarnings("unchecked")
    private static <T> T cast(Object o) {
        return (T) o;
    }
}
