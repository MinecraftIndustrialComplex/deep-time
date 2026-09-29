// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.ev;

import dev.lukebemish.excavatedvariants.api.RegistryKeys;
import dev.lukebemish.excavatedvariants.api.data.Ore;
import dev.lukebemish.excavatedvariants.api.data.Stone;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

/**
 * {@link EvBridge} over Excavated Variants 4.3.1's API (compiled against it as {@code compileOnly};
 * loaded only when EV is installed). EV registers its {@code excavated_variants:ore} and
 * {@code excavated_variants:stone} registries into {@code BuiltInRegistries.REGISTRY} and fills them
 * before blocks register; after that every {@link Ore}'s {@link Ore#getStones()} maps each stone to
 * the ore's block in it: the original ore blocks (copper ore in stone, deepslate copper ore in
 * deepslate) and the variants EV generated for its other stones. A host block is an EV stone when
 * some {@link Stone#block} is it. One block can be several stones: EV 4.3.1 ships
 * {@code excavated_variants:minecraft/smooth_basalt} (overworld ores) and
 * {@code excavated_variants:create/smooth_basalt} (nether ores) for {@code minecraft:smooth_basalt}, so
 * a host's stones are all kept, in id order, and the first one the ore has a block in wins (Q10: with
 * only the last stone kept, ore in smooth basalt never became a variant).
 */
final class EvBridgeImpl implements EvBridge {

    private final Map<Block, List<ResourceKey<Stone>>> stonesByBlock = new HashMap<>();
    private final Map<Block, Ore> oreByBlock = new HashMap<>();
    private final Map<Block, ResourceKey<Stone>> stoneOfOreBlock = new HashMap<>();
    private final Set<Block> variants = new HashSet<>();
    private final int ores, stones;

    @SuppressWarnings("unchecked")
    private EvBridgeImpl() {
        Registry<Ore> oreReg = (Registry<Ore>) BuiltInRegistries.REGISTRY.get(RegistryKeys.ORE.location());
        Registry<Stone> stoneReg = (Registry<Stone>) BuiltInRegistries.REGISTRY.get(RegistryKeys.STONE.location());
        if (oreReg == null || stoneReg == null)
            throw new IllegalStateException("Excavated Variants' ore or stone registry is missing");
        for (var e : stoneReg.entrySet())
            BuiltInRegistries.BLOCK.getOptional(e.getValue().block)
                    .ifPresent(b -> stonesByBlock.computeIfAbsent(b, k -> new ArrayList<>()).add(e.getKey()));
        Comparator<ResourceKey<Stone>> byId = Comparator.comparing(k -> k.location().toString());
        stonesByBlock.values().forEach(l -> l.sort(byId));
        for (var e : oreReg.entrySet()) {
            Ore ore = e.getValue();
            Set<ResourceKey<Block>> originals = ore.originalBlocks;
            for (var be : ore.getBlocks().entrySet()) {
                Block b = BuiltInRegistries.BLOCK.getOptional(be.getKey()).orElse(null);
                if (b == null)
                    continue;
                oreByBlock.putIfAbsent(b, ore);
                stoneOfOreBlock.put(b, be.getValue());
                if (!originals.contains(be.getKey()))
                    variants.add(b);
            }
        }
        this.ores = oreReg.size();
        this.stones = stoneReg.size();
    }

    /** Called reflectively by {@link EvBridge#load()}. */
    static EvBridge create() {
        return new EvBridgeImpl();
    }

    @Override
    public BlockState variant(List<BlockState> oreStates, BlockState host) {
        List<ResourceKey<Stone>> stones = stonesByBlock.get(host.getBlock());
        if (stones == null)
            return null;
        for (ResourceKey<Stone> stone : stones) {
            for (BlockState s : oreStates) {
                Ore ore = oreByBlock.get(s.getBlock());
                if (ore == null)
                    continue;
                ResourceKey<Block> in = ore.getStones().get(stone);
                if (in == null)
                    continue;
                Block b = BuiltInRegistries.BLOCK.getOptional(in).orElse(null);
                if (b != null)
                    return b.defaultBlockState();
            }
        }
        return null;
    }

    @Override
    public Optional<ResourceLocation> stoneOf(BlockState host) {
        List<ResourceKey<Stone>> stones = stonesByBlock.get(host.getBlock());
        return stones == null ? Optional.empty() : Optional.of(stones.getFirst().location());
    }

    @Override
    public boolean isVariant(BlockState state) {
        return variants.contains(state.getBlock());
    }

    @Override
    public Optional<ResourceLocation> stoneOfVariant(BlockState state) {
        return Optional.ofNullable(stoneOfOreBlock.get(state.getBlock())).map(ResourceKey::location);
    }

    @Override
    public String describe() {
        return ores + " ores, " + stones + " stones, " + variants.size() + " generated variants";
    }
}
