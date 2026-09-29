// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.coe;

import com.tom.createores.Config;
import com.tom.createores.OreData;
import com.tom.createores.OreDataAttachment;
import com.tom.createores.OreVeinGenerator;
import com.tom.createores.recipe.VeinRecipe;
import com.tom.createores.util.ThreeState;
import net.minecraft.core.BlockPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.crafting.RecipeHolder;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.LevelChunk;

import java.util.Comparator;
import java.util.List;
import java.util.Optional;

/**
 * {@link CoeAccess} over Create Ore Excavation 1.6.8 (MIT; compiled against it as
 * {@code compileOnly}, loaded only when COE is installed). Signatures checked with javap on the MIC
 * pack's jar: {@code OreDataAttachment.getData(LevelChunk)}, {@code OreData.populate(LevelChunk)},
 * {@code OreData.getResourcesRemaining(VeinRecipe)}, {@code OreVeinGenerator.pick(LevelChunk)} and
 * {@code getPicker(ServerLevel)}, {@code RandomSpreadGenerator.locate(ResourceLocation, BlockPos,
 * ServerLevel, int)}, {@code VeinRecipe.getPlacement()}.
 */
final class CoeAccessImpl implements CoeAccess {

    CoeAccessImpl() {}

    @Override
    public ChunkState chunkState(LevelChunk chunk) {
        return state((ServerLevel) chunk.getLevel(), OreDataAttachment.getData(chunk));
    }

    @Override
    public ChunkState populate(LevelChunk chunk) {
        OreData d = new OreData();
        d.populate(chunk);
        return state((ServerLevel) chunk.getLevel(), d);
    }

    private static ChunkState state(ServerLevel level, OreData d) {
        RecipeHolder<VeinRecipe> r = d.getRecipe(level.getRecipeManager());
        if (r == null)
            return new ChunkState(Optional.ofNullable(d.getRecipeId()), d.getRandomMul(), 0, 0, false);
        VeinRecipe v = r.value();
        boolean infinite = v.isFinite() == ThreeState.NEVER || (v.isFinite() == ThreeState.DEFAULT && Config.defaultInfinite);
        // COE's own amount for a finite vein (OreData.getResourcesRemaining before any extraction).
        long amount = Math.round(((v.getMaxAmount() - v.getMinAmount()) * d.getRandomMul() + v.getMinAmount())
                * (double) Config.finiteAmountBase);
        return new ChunkState(Optional.of(r.id()), d.getRandomMul(), amount, d.getResourcesRemaining(v), infinite);
    }

    @Override
    public Optional<ResourceLocation> pick(LevelChunk chunk) {
        RecipeHolder<VeinRecipe> r = OreVeinGenerator.pick(chunk);
        return r == null ? Optional.empty() : Optional.of(r.id());
    }

    @Override
    public List<VeinInfo> veins(ServerLevel level) {
        return level.getRecipeManager().getRecipes().stream()
                .filter(h -> h.value() instanceof VeinRecipe)
                .map(h -> {
                    VeinRecipe v = (VeinRecipe) h.value();
                    return new VeinInfo(h.id(), v.getMinAmount(), v.getMaxAmount(), v.isFinite().getSerializedName(),
                            v.getPlacement().spacing());
                })
                .sorted(Comparator.comparing(VeinInfo::id))
                .toList();
    }

    @Override
    public Optional<ChunkPos> gridChunk(ServerLevel level, ResourceLocation id, int cx, int cz) {
        return level.getRecipeManager().byKey(id)
                .filter(h -> h.value() instanceof VeinRecipe)
                .map(h -> ((VeinRecipe) h.value()).getPlacement().getPotentialStructureChunk(level.getSeed(), cx, cz));
    }

    @Override
    public Optional<BlockPos> locate(ServerLevel level, ResourceLocation id, BlockPos from, int radius) {
        return Optional.ofNullable(OreVeinGenerator.getPicker(level).locate(id, from, level, radius));
    }

    @Override
    public String config() {
        return "finiteAmountBase=" + Config.finiteAmountBase + " defaultInfinite=" + Config.defaultInfinite;
    }
}
