// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.gametest;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.Lifecycle;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistrationInfo;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTestServer;
import net.minecraft.gametest.framework.TestFunction;
import net.minecraft.server.WorldLoader;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * Lets Deep Time's test dimensions exist in the GameTest server (plan §9.4).
 *
 * <p>{@code GameTestServer.create} bakes the flat preset against an empty, frozen stem registry,
 * so datapack dimensions load as data but never become levels. This wraps that one
 * {@code WorldDimensions.bake} call and passes the datapack's {@code deeptime:*} stems instead —
 * only those, so a pack datapack that defines {@code minecraft:overworld} or another mod's
 * dimension cannot change the GameTest world (cold-read review 2, gap 9). Those of Deep Time's levels
 * that sit on vanilla's overworld biome list take such a datapack overworld's biome source, as a real
 * Deep Time world does ({@code worldgen.OverworldOverride}; Expanded Ecosphere). Applied only with
 * {@code -Ddeeptime.gametest=true} (DeepTimeMixinPlugin).
 *
 * <p>It also honours {@code -Ddeeptime.gametestOnly=<name>[,<name>…]} ({@code ./gametest.sh
 * -PgametestOnly=lakeHolds}): the server runs only the named tests (case-insensitive method names),
 * so one test can be run alone. Unset, every registered test runs.
 */
@Mixin(GameTestServer.class)
abstract class GameTestServerMixin {

    // The only WorldDimensions.bake call in GameTestServer is inside create()'s WorldLoader lambda,
    // whose one DataLoadContext argument is the lambda's own.
    @WrapOperation(method = "*", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/levelgen/WorldDimensions;bake(Lnet/minecraft/core/Registry;)Lnet/minecraft/world/level/levelgen/WorldDimensions$Complete;"))
    private static WorldDimensions.Complete deeptime$bakeTestStems(WorldDimensions dims, Registry<LevelStem> empty,
            Operation<WorldDimensions.Complete> original, @Local(argsOnly = true) WorldLoader.DataLoadContext ctx) {
        Registry<LevelStem> all = ctx.datapackDimensions().registryOrThrow(Registries.LEVEL_STEM);
        MappedRegistry<LevelStem> ours = new MappedRegistry<>(Registries.LEVEL_STEM, Lifecycle.stable());
        LevelStem datapackOverworld = all.get(LevelStem.OVERWORLD);
        com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport.datapackOverworld(datapackOverworld);
        all.entrySet().stream()
                .filter(e -> e.getKey().location().getNamespace().equals(DeepTime.MODID))
                .forEach(e -> ours.register(e.getKey(),
                        com.minecraftindustrialcomplex.deeptime.worldgen.OverworldOverride.adopt(e.getValue(), datapackOverworld),
                        RegistrationInfo.BUILT_IN));
        DeepTime.LOGGER.info("[gametest] GameTest server gets {} Deep Time test dimension(s): {}", ours.size(), ours.keySet());
        return original.call(dims, ours.freeze());
    }

    @ModifyVariable(method = "create", at = @At("HEAD"), argsOnly = true)
    private static Collection<TestFunction> deeptime$onlyNamedTests(Collection<TestFunction> tests) {
        String only = System.getProperty("deeptime.gametestOnly", "");
        if (only.isBlank())
            return tests;
        Set<String> names = Arrays.stream(only.split(",")).map(String::trim).filter(n -> !n.isEmpty())
                .map(n -> n.toLowerCase(Locale.ROOT)).collect(Collectors.toSet());
        List<TestFunction> kept = tests.stream().filter(t -> {
            String name = t.testName().toLowerCase(Locale.ROOT);
            return names.contains(name) || names.contains(name.substring(name.lastIndexOf('.') + 1));
        }).toList();
        DeepTime.LOGGER.info("[gametest] -Ddeeptime.gametestOnly={}: running {} of {} tests: {}", only, kept.size(), tests.size(),
                kept.stream().map(TestFunction::testName).toList());
        return kept;
    }
}
