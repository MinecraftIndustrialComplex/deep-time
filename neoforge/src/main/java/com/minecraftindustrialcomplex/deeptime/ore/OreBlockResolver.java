// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.ev.EvBridge;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.TagsUpdatedEvent;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The ore block for a material in a host block (plan §7.6; decisions file "Ore blocks": Excavated
 * Variants' ore-in-rock blocks when it is installed, otherwise normal stone or deepslate ores):
 *
 * <ol>
 *   <li><b>Excavated Variants</b> ({@link EvBridge}, only when EV is loaded): the variant of an EV
 *       ore whose base blocks are in {@code #c:ores/<material>} in the host's EV stone.</li>
 *   <li>Otherwise, or where EV has no variant for that host, and the host is neither stone nor
 *       deepslate: a block of {@code #c:ores/<material>} that is also in
 *       {@code #c:ores_in_ground/<the host block's path>} ({@code gravel}, {@code sand}, …), so a
 *       pack can give a placer its own gold-bearing gravel without Excavated Variants.</li>
 *   <li>Otherwise a block of {@code #c:ores/<material>} that is also in
 *       {@code #c:ores_in_ground/deepslate} below y = 0, else in {@code #c:ores_in_ground/stone}.</li>
 *   <li>Otherwise any block of {@code #c:ores/<material>}.</li>
 * </ol>
 *
 * A placer's gold in a gravel or sand cell is the exception to steps 3 and 4 (Ben, 2026-10-16):
 * where steps 1 and 2 find nothing, the placer puts vanilla's suspicious gravel or sand with a
 * gold-nugget loot table instead of the stone ore ({@link #panned}, {@link PlacerPanning}).
 *
 * A material written {@code ns:name} is the tag {@code ns:ores/name}. Answers are cached per
 * (material, host block, below 0) and rebuilt when the server's tags reload. Null means "no block":
 * the material has no ore block in this pack (molybdenum, crude oil, …) and nothing is placed.
 */
public final class OreBlockResolver {

    private static volatile OreBlockResolver current = new OreBlockResolver(null, Optional.empty());

    private final RegistryAccess access;
    private final Optional<EvBridge> ev;
    private final Map<String, Optional<BlockState>> cache = new ConcurrentHashMap<>();
    private final Map<ResourceLocation, List<BlockState>> byMaterial = new ConcurrentHashMap<>();

    private OreBlockResolver(RegistryAccess access, Optional<EvBridge> ev) {
        this.access = access;
        this.ev = ev;
    }

    public static void init() {
        NeoForge.EVENT_BUS.addListener(TagsUpdatedEvent.class, e -> {
            if (e.getUpdateCause() != TagsUpdatedEvent.UpdateCause.SERVER_DATA_LOAD)
                return;
            current = new OreBlockResolver(e.getRegistryAccess(), EvBridge.load());
            DeepTime.LOGGER.info("Ore blocks: {}", current.ev.map(b -> "Excavated Variants (" + b.describe() + "), then #c:ores")
                    .orElse("#c:ores (Excavated Variants not installed)"));
        });
    }

    public static OreBlockResolver get() {
        return current;
    }

    /** True when ore blocks come from Excavated Variants first. */
    public boolean usesEv() {
        return ev.isPresent();
    }

    public Optional<EvBridge> ev() {
        return ev;
    }

    /** The ore tag of a material: {@code ns:name} → {@code #ns:ores/name}. */
    public static TagKey<Block> oreTag(ResourceLocation material) {
        return TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(material.getNamespace(), "ores/" + material.getPath()));
    }

    static final TagKey<Block> IN_STONE = TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath("c", "ores_in_ground/stone"));
    static final TagKey<Block> IN_DEEPSLATE = TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath("c", "ores_in_ground/deepslate"));

    /** {@code #c:ores_in_ground/<path of the host block's id>}: the ores a pack made for that host. */
    public static TagKey<Block> inGround(BlockState host) {
        String path = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(host.getBlock()).getPath();
        return TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath("c", "ores_in_ground/" + path));
    }

    /** The blocks of {@code #c:ores/<material>} in tag order (empty when the tag is absent). */
    public List<BlockState> blocks(ResourceLocation material) {
        if (access == null)
            return List.of();
        return byMaterial.computeIfAbsent(material, m -> {
            List<BlockState> out = new ArrayList<>();
            access.registryOrThrow(Registries.BLOCK).getTag(oreTag(m)).ifPresent(set -> {
                for (Holder<Block> h : set)
                    out.add(h.value().defaultBlockState());
            });
            return List.copyOf(out);
        });
    }

    /** True when {@code state} is an ore block of {@code material} (in {@code #c:ores/<material>}). */
    public static boolean isOreOf(BlockState state, ResourceLocation material) {
        return state.is(oreTag(material));
    }

    /**
     * The ore block of {@code material} for {@code host} at height {@code y}, or null when the pack
     * has none.
     */
    public BlockState resolve(ResourceLocation material, BlockState host, int y) {
        boolean deep = y < 0;
        String key = material + "|" + net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(host.getBlock()) + "|" + deep;
        return cache.computeIfAbsent(key, k -> Optional.ofNullable(compute(material, host, deep))).orElse(null);
    }

    private BlockState compute(ResourceLocation material, BlockState host, boolean deep) {
        List<BlockState> ores = blocks(material);
        if (ores.isEmpty())
            return null;
        BlockState made = madeFor(ores, host);
        if (made != null)
            return made;
        TagKey<Block> ground = deep ? IN_DEEPSLATE : IN_STONE;
        for (BlockState b : ores)
            if (b.is(ground))
                return b;
        if (deep)
            for (BlockState b : ores)
                if (b.is(IN_STONE))
                    return b;
        return ores.get(0);
    }

    /**
     * The ore block made for {@code host} itself (steps 1 and 2 of the order above): Excavated
     * Variants' variant for the host, else a block of {@code ores} in
     * {@code #c:ores_in_ground/<host>} when the host is neither stone nor deepslate; null when there
     * is none.
     */
    private BlockState madeFor(List<BlockState> ores, BlockState host) {
        if (ev.isPresent()) {
            BlockState v = ev.get().variant(ores, host);
            if (v != null)
                return v;
        }
        if (!host.is(net.minecraft.world.level.block.Blocks.STONE) && !host.is(net.minecraft.world.level.block.Blocks.DEEPSLATE)) {
            TagKey<Block> own = inGround(host);
            for (BlockState b : ores)
                if (b.is(own))
                    return b;
        }
        return null;
    }

    // ---- placer gold as panning (Ben, 2026-10-16) -------------------------------------------------

    private final Map<String, Optional<BlockState>> panned = new ConcurrentHashMap<>();

    /**
     * The suspicious block ({@link PlacerPanning}) a placer puts in {@code cell} where the ore of
     * {@code material} is due, or null when the placer places its ore block ({@link #resolve})
     * there: the material has no panning table (only gold has), {@code cell} is not gravel or sand
     * (soil over the bar, a rock bench), or the pack has a gold ore made for {@code host}, the rock
     * the ore sits in (Excavated Variants' gravel and sand variants, or a block in
     * {@code #c:ores_in_ground/gravel} or {@code /sand}). With Excavated Variants nothing changes.
     * The caller sets the block's loot table ({@link PlacerPanning#setLoot}) and checks that the
     * block under it holds it up ({@link PlacerPanning#supported}).
     */
    public BlockState panned(ResourceLocation material, BlockState cell, BlockState host) {
        var registry = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        String key = material + "|" + registry.getKey(cell.getBlock()) + "|" + registry.getKey(host.getBlock());
        return panned.computeIfAbsent(key, k -> Optional.ofNullable(computePanned(material, cell, host))).orElse(null);
    }

    private BlockState computePanned(ResourceLocation material, BlockState cell, BlockState host) {
        if (PlacerPanning.tableOf(material) == null)
            return null;
        BlockState suspicious = PlacerPanning.suspiciousFor(cell);
        if (suspicious == null)
            return null;
        List<BlockState> ores = blocks(material);
        return !ores.isEmpty() && madeFor(ores, host) != null ? null : suspicious;
    }

    // ---- raw-ore blocks (vanilla's ore veins; Ben, 2026-10-02) ----------------------------------

    private final Map<ResourceLocation, Optional<BlockState>> raw = new ConcurrentHashMap<>();

    /** {@code ns:name} → {@code #ns:storage_blocks/raw_name} (NeoForge's common tag for raw-ore blocks). */
    public static TagKey<Block> rawTag(ResourceLocation material) {
        return TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(material.getNamespace(),
                "storage_blocks/raw_" + material.getPath()));
    }

    /** Vanilla's raw-ore blocks, the fallback when a pack's tags name none. */
    static final Map<String, Block> VANILLA_RAW = Map.of(
            "iron", net.minecraft.world.level.block.Blocks.RAW_IRON_BLOCK,
            "copper", net.minecraft.world.level.block.Blocks.RAW_COPPER_BLOCK,
            "gold", net.minecraft.world.level.block.Blocks.RAW_GOLD_BLOCK);

    /**
     * The raw-ore block of {@code material} that a rich deposit core places among its ore (as
     * vanilla's copper and iron ore veins carry {@code raw_copper_block} and {@code raw_iron_block}):
     * the first block of {@code #<ns>:storage_blocks/raw_<material>} (so a pack's own, Create's raw
     * zinc block, …), else vanilla's raw iron, copper or gold block, else null (no raw block: the
     * deposit places ore there). Rock tags work the same way: a tag first, vanilla's block after.
     */
    public BlockState rawBlock(ResourceLocation material) {
        return raw.computeIfAbsent(material, m -> {
            if (access != null) {
                var tag = access.registryOrThrow(Registries.BLOCK).getTag(rawTag(m));
                if (tag.isPresent())
                    for (Holder<Block> h : tag.get())
                        return Optional.of(h.value().defaultBlockState());
            }
            Block v = VANILLA_RAW.get(m.getPath());
            return Optional.ofNullable(v == null ? null : v.defaultBlockState());
        }).orElse(null);
    }

    /** True when {@code state} is a raw-ore block of {@code material} (its tag, or vanilla's block). */
    public static boolean isRawOf(BlockState state, ResourceLocation material) {
        if (state.is(rawTag(material)))
            return true;
        Block v = VANILLA_RAW.get(material.getPath());
        return v != null && state.is(v);
    }

    /** True when Excavated Variants is installed. */
    public static boolean evLoaded() {
        return ModList.get() != null && ModList.get().isLoaded(EvBridge.MOD_ID);
    }
}
