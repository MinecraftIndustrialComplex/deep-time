// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.toroidal_world;

import com.minecraftindustrialcomplex.deeptime.compat.dh.TwDhGuard;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Coerce;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Deep Time's far-terrain generator passes Toroidal World's Distant Horizons guard (Ben,
 * 2026-10-02: "Mixin around TW's guard"; {@link TwDhGuard} has the reason and the rule).
 *
 * <p>Target: Toroidal World 0.19.3 (MIT) {@code public static void
 * com.toroidalworld.compat.distanthorizons.DhGenerators.requireOwn(IDhApiLevelWrapper,
 * IDhApiWorldGenerator)}, which TW's own {@code WorldGeneratorInjectorMixin} calls before DH binds
 * a generator and which throws for any generator outside DH's package on a level TW shapes. HEAD,
 * cancellable: when {@link TwDhGuard#letThrough} vouches for the generator (Deep Time's, made for
 * this level, whose TW loop is the world data's circumference), the method returns before TW's
 * check; for every other generator TW's check runs unchanged. The two parameters are DH API types
 * Deep Time does not compile against, hence {@code @Coerce Object}.
 *
 * <p>Optional: its own mixin config ({@code deeptime.tw_dh.mixins.json}, not required, no required
 * injections); {@code TwDhGuardMixinPlugin} applies it only when Toroidal World and Distant Horizons
 * are installed and the method exists with this signature, and logs whether it applied.
 */
@Pseudo
@Mixin(targets = "com.toroidalworld.compat.distanthorizons.DhGenerators", remap = false)
public abstract class DhGeneratorsMixin {

    @Inject(method = "requireOwn", at = @At("HEAD"), cancellable = true, require = 0)
    private static void deeptime$letFarTerrainThrough(@Coerce Object level, @Coerce Object generator, CallbackInfo ci) {
        if (TwDhGuard.letThrough(level, generator))
            ci.cancel();
    }
}
