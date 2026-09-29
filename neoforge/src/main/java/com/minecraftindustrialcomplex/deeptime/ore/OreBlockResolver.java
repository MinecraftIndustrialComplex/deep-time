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
 *   <li>Otherwise, or where EV has no variant for that host: a block of {@code #c:ores/<material>}
 *       that is also in {@code #c:ores_in_ground/deepslate} below y = 0, else in
 *       {@code #c:ores_in_ground/stone}.</li>
 *   <li>Otherwise any block of {@code #c:ores/<material>}.</li>
 * </ol>
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
        if (ev.isPresent()) {
            BlockState v = ev.get().variant(ores, host);
            if (v != null)
                return v;
        }
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

    /** True when Excavated Variants is installed. */
    public static boolean evLoaded() {
        return ModList.get() != null && ModList.get().isLoaded(EvBridge.MOD_ID);
    }
}
