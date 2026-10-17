// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.registries.Registries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.FallingBlock;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.entity.BrushableBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.storage.loot.LootContext;
import net.minecraft.world.level.storage.loot.LootParams;
import net.minecraft.world.level.storage.loot.LootTable;
import net.minecraft.world.level.storage.loot.parameters.LootContextParamSets;
import net.minecraft.world.level.storage.loot.parameters.LootContextParams;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.common.Tags;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Placer gold as panning (Ben, 2026-10-16: "Could we use suspicious gravel w/ a loot table set to a
 * gold nugget?"; decisions log 2026-10-16). In a pack with no gold ore made for gravel or sand (no
 * Excavated Variants, no {@code #c:ores_in_ground/gravel} gold ore; ledger ORE-267 to ORE-269, a
 * game choice), the ore cell of a placer's
 * gravel or sand body is vanilla's {@code suspicious_gravel} or {@code suspicious_sand} holding the
 * gold nuggets Deep Time's loot table {@code deeptime:archaeology/placer_gold} gives for the block:
 * brushing the block is the panning. Deep Time ships no block of its own.
 *
 * <p>The table is rolled when the block is placed ({@link #fill}), not when it is first brushed: the
 * block entity holds the nuggets as its item and no loot table. A placer is a finite natural
 * deposit, one brushing per block for whoever gets there first (coordinator's call, decisions log
 * 2026-10-17), and a block entity with a loot table is what per-player loot mods take over: Lootr
 * (in MIC) replaces every {@code #lootr:convert/gravels} or {@code /sands} block whose brushable
 * block entity has a loot table, once its chunk and the 24 around it are loaded, with its own
 * per-player block, and skips one without a table. The roll is the one brushing would make (the
 * same table, seed and {@code CHEST} parameters as {@code BrushableBlockEntity.unpackLootTable}),
 * except that it is raw: NeoForge's global loot modifiers are not applied, since they would run on a
 * worldgen thread.
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
     * Fills the suspicious block just placed at {@code pos} with its gold: {@link #roll} of
     * {@code table} with {@code seed}, stored as the block entity's item with no loot table (the
     * block entity of a chunk still being generated is made from the placeholder {@code setBlock}
     * left, as for vanilla's desert well). Returns false when there is no brushable block entity or
     * the table gives nothing; the caller then puts the plain block back.
     */
    public static boolean fill(WorldGenLevel level, BlockPos pos, ResourceKey<LootTable> table, long seed) {
        ItemStack gold = roll(level.getLevel(), pos, table, seed);
        if (gold.isEmpty())
            return false;
        return level.getBlockEntity(pos, BlockEntityType.BRUSHABLE_BLOCK).map(be -> {
            setItem(be, gold, level.registryAccess());
            return true;
        }).orElse(false);
    }

    /**
     * The stack brushing the block at {@code pos} yields: the first stack {@code table} rolls with
     * {@code seed} and vanilla's brushing parameters ({@code CHEST}, the block's centre as origin;
     * {@code BrushableBlockEntity.unpackLootTable} keeps only the first stack too), or
     * {@link ItemStack#EMPTY}. Raw: no global loot modifier is applied (they may read the level, and
     * this runs on a worldgen thread). A pure function of the table and the seed.
     */
    @SuppressWarnings("deprecation") // getRandomItemsRaw: the global loot modifiers are left out on purpose
    public static ItemStack roll(ServerLevel level, BlockPos pos, ResourceKey<LootTable> table, long seed) {
        LootTable loot = level.getServer().reloadableRegistries().getLootTable(table);
        LootParams params = new LootParams.Builder(level).withParameter(LootContextParams.ORIGIN, Vec3.atCenterOf(pos))
                .create(LootContextParamSets.CHEST);
        // A seeded random of the block's own (never the level's: this is not the server thread),
        // the one LootTable.getRandomItems(params, seed) makes for a seed that is not 0.
        LootContext context = new LootContext.Builder(params).withOptionalRandomSource(RandomSource.create(seed)).create(Optional.empty());
        List<ItemStack> out = new ArrayList<>(1);
        loot.getRandomItemsRaw(context, out::add);
        return out.isEmpty() ? ItemStack.EMPTY : out.get(0);
    }

    /** Puts {@code stack} in a brushable block entity as its item (the form vanilla saves it in once its table is unpacked). */
    static void setItem(BrushableBlockEntity be, ItemStack stack, HolderLookup.Provider registries) {
        CompoundTag tag = new CompoundTag();
        tag.put("item", stack.save(registries));
        be.loadCustomOnly(tag, registries);
    }
}
