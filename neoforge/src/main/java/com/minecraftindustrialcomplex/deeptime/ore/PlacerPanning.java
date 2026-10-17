// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.LevelReader;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.FallingBlock;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.storage.loot.LootTable;
import net.neoforged.neoforge.common.Tags;

/**
 * Placer gold as panning (Ben, 2026-10-16: "Could we use suspicious gravel w/ a loot table set to a
 * gold nugget?"; decisions log 2026-10-16). In a pack with no gold ore made for gravel or sand (no
 * Excavated Variants, no {@code #c:ores_in_ground/gravel} gold ore; ledger ORE-267 to ORE-269, a
 * game choice), the ore cell of a placer's
 * gravel or sand body is vanilla's {@code suspicious_gravel} or {@code suspicious_sand} whose block
 * entity carries Deep Time's loot table {@code deeptime:archaeology/placer_gold} (gold nuggets):
 * brushing the block is the panning. Deep Time ships no block of its own.
 *
 * <p>Only the material with a table ({@link #tableOf}: gold) is panned, only in a gravel or sand
 * cell ({@link #suspiciousFor}), and only where the block below holds it up
 * ({@link #supported}): suspicious gravel and sand fall like their plain blocks, so a cell over air,
 * water or a plant stays plain gravel or sand. The decision of whether a pack has a gold ore for
 * the host is {@link OreBlockResolver#panned}.
 */
public final class PlacerPanning {

    private PlacerPanning() {}

    /**
     * {@code deeptime:archaeology/placer_gold}: {@code data/deeptime/loot_table/archaeology/placer_gold.json},
     * one stack of 2 to 5 gold nuggets (ledger: ORE-268, game-choice).
     */
    public static final ResourceKey<LootTable> GOLD_TABLE = ResourceKey.create(Registries.LOOT_TABLE,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "archaeology/placer_gold"));

    private static final ResourceLocation GOLD = ResourceLocation.fromNamespaceAndPath("c", "gold");

    /** The loot table of a placer's panned block for {@code material}, or null when it has none (only gold has). */
    public static ResourceKey<LootTable> tableOf(ResourceLocation material) {
        return GOLD.equals(material) ? GOLD_TABLE : null;
    }

    /**
     * The suspicious block for a gravel ({@code #c:gravels}) or sand ({@code #c:sands}) block, or
     * null for any other block (soil, rock, …).
     */
    public static BlockState suspiciousFor(BlockState cell) {
        if (cell.is(Tags.Blocks.GRAVELS))
            return Blocks.SUSPICIOUS_GRAVEL.defaultBlockState();
        if (cell.is(Tags.Blocks.SANDS))
            return Blocks.SUSPICIOUS_SAND.defaultBlockState();
        return null;
    }

    /**
     * True when a suspicious block above {@code below} stays put: the gravity blocks fall when the
     * block under them is free (air, fire, a liquid or a replaceable plant;
     * {@link FallingBlock#isFree}, which {@code BrushableBlock} uses too).
     */
    public static boolean supported(BlockState below) {
        return !FallingBlock.isFree(below);
    }

    /**
     * Sets the loot table of the suspicious block just placed at {@code pos} (the way vanilla's
     * desert well does it; the block entity of a chunk still being generated is made from the
     * placeholder {@code setBlock} left). Returns false when there is no brushable block entity.
     */
    public static boolean setLoot(LevelReader level, BlockPos pos, ResourceKey<LootTable> table, long seed) {
        return level.getBlockEntity(pos, BlockEntityType.BRUSHABLE_BLOCK).map(be -> {
            be.setLootTable(table, seed);
            return true;
        }).orElse(false);
    }
}
