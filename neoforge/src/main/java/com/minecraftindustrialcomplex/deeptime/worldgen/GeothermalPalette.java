// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import net.minecraft.commands.arguments.blocks.BlockStateParser;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.TagsUpdatedEvent;

import java.util.ArrayList;
import java.util.EnumMap;
import java.util.List;
import java.util.Map;

/**
 * Geothermal role → blocks (P2-14), resolved the way rock classes are ({@link PaletteRules}): the
 * blocks of {@code #deeptime:geothermal/<role>} if there are any (several are chosen among by a
 * hash of the position), else the first installed entry of {@code #deeptime:geothermal_fallback/<role>},
 * else the role's vanilla block. Deep Time ships no such tags and adds no block; a pack that has
 * a sulfur or cinnabar block names it in {@code geothermal/sulfur} or {@code geothermal/cinnabar}.
 * Travertine is the rock class of that name ({@code #deeptime:rock/travertine}, dripstone block in
 * vanilla), so a pack's travertine serves the terraces too.
 *
 * <table>
 *   <caption>The roles and their vanilla blocks</caption>
 *   <tr><td>TRAVERTINE</td><td>rock class {@code travertine}: {@code minecraft:dripstone_block}</td></tr>
 *   <tr><td>SINTER</td><td>white siliceous crust: {@code minecraft:calcite}</td></tr>
 *   <tr><td>ALTERED</td><td>bleached altered ground: {@code minecraft:white_terracotta}</td></tr>
 *   <tr><td>STAIN</td><td>yellow staining, mats at a pool's edge: {@code minecraft:yellow_terracotta}</td></tr>
 *   <tr><td>MAT</td><td>orange and red microbial mats: {@code minecraft:orange_terracotta}</td></tr>
 *   <tr><td>SULFUR</td><td>native sulfur around a vent: {@code minecraft:yellow_concrete}</td></tr>
 *   <tr><td>CINNABAR</td><td>mercury sulfide stains: {@code minecraft:red_terracotta}</td></tr>
 *   <tr><td>VENT</td><td>hot ground in a fumarole: {@code minecraft:magma_block}</td></tr>
 * </table>
 *
 * <p>Rebuilt whenever the server (re)loads its data, like {@link RockPalette}; chunk generation reads
 * {@link #get()}. Before the first load, and for the tests, it holds the vanilla blocks.
 */
public final class GeothermalPalette {

    public enum Role {
        TRAVERTINE("travertine", "minecraft:dripstone_block"), SINTER("sinter", "minecraft:calcite"),
        ALTERED("altered", "minecraft:white_terracotta"), STAIN("stain", "minecraft:yellow_terracotta"),
        MAT("mat", "minecraft:orange_terracotta"), SULFUR("sulfur", "minecraft:yellow_concrete"),
        CINNABAR("cinnabar", "minecraft:red_terracotta"), VENT("vent", "minecraft:magma_block");

        public final String id, vanilla;

        Role(String id, String vanilla) {
            this.id = id;
            this.vanilla = vanilla;
        }

        public Block vanillaBlock() {
            return BuiltInRegistries.BLOCK.get(ResourceLocation.parse(vanilla));
        }
    }

    public static final String TAG_MEMBERS = "geothermal/", TAG_FALLBACK = "geothermal_fallback/";

    private static volatile GeothermalPalette current;

    private final Map<Role, PaletteRules.Result<BlockState>> byRole;

    private GeothermalPalette(Map<Role, PaletteRules.Result<BlockState>> byRole) {
        this.byRole = byRole;
    }

    /** The palette of the vanilla blocks alone (before the first data load). */
    public static GeothermalPalette vanilla() {
        Map<Role, PaletteRules.Result<BlockState>> m = new EnumMap<>(Role.class);
        for (Role r : Role.values())
            m.put(r, new PaletteRules.Result<>(r.id, PaletteRules.Source.VANILLA_FALLBACK, List.of(r.vanillaBlock().defaultBlockState()),
                    List.of()));
        return new GeothermalPalette(m);
    }

    public static GeothermalPalette get() {
        GeothermalPalette p = current;
        if (p == null) {
            p = vanilla();
            current = p;
        }
        return p;
    }

    public static void init() {
        NeoForge.EVENT_BUS.addListener(TagsUpdatedEvent.class, e -> {
            if (e.getUpdateCause() != TagsUpdatedEvent.UpdateCause.SERVER_DATA_LOAD)
                return;
            GeothermalPalette p = build(e.getRegistryAccess());
            current = p;
            DeepTime.LOGGER.info("Geothermal palette: {}", p.toJson());
        });
    }

    /** Builds a palette from a server's registries (tags must be bound). */
    public static GeothermalPalette build(RegistryAccess access) {
        Map<Role, PaletteRules.Result<BlockState>> m = new EnumMap<>(Role.class);
        for (Role role : Role.values()) {
            PaletteRules.Kinds<BlockState> kinds = BlockState::isAir;
            var candidates = new PaletteRules.Candidates<>(tag(access, TAG_MEMBERS + role.id), List.of(),
                    tag(access, TAG_FALLBACK + role.id), List.of(), java.util.Optional.of(role.vanillaBlock().defaultBlockState()));
            m.put(role, PaletteRules.resolve(role.id, candidates, kinds, BlockStateParser::serialize));
        }
        return new GeothermalPalette(m);
    }

    private static List<BlockState> tag(RegistryAccess access, String path) {
        TagKey<Block> key = TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, path));
        List<BlockState> out = new ArrayList<>();
        access.registryOrThrow(Registries.BLOCK).getTag(key).ifPresent(set -> {
            for (Holder<Block> h : set)
                out.add(h.value().defaultBlockState());
        });
        return out;
    }

    /**
     * The block of {@code role} at a position: one of its members picked by {@code key}. Travertine
     * is the rock class of that name (the palette of {@link RockPalette}); where that resolves to
     * nothing, sinter.
     */
    public BlockState block(Role role, long key) {
        if (role == Role.TRAVERTINE) {
            BlockState rock = RockPalette.get().block("travertine", key);
            if (rock != null)
                return rock;
            return block(Role.SINTER, key);
        }
        BlockState s = byRole.get(role).pick(key);
        return s != null ? s : Blocks.STONE.defaultBlockState();
    }

    /** Every block {@code role} can place (the members; the one fallback or vanilla block otherwise). */
    public List<BlockState> blocks(Role role) {
        if (role == Role.TRAVERTINE) {
            var rock = RockPalette.get().result("travertine");
            if (rock.isPresent() && !rock.get().blocks().isEmpty())
                return rock.get().blocks();
            return blocks(Role.SINTER);
        }
        return byRole.get(role).blocks();
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        for (Role r : Role.values()) {
            var res = byRole.get(r);
            StringBuilder sb = new StringBuilder();
            res.blocks().forEach(b -> sb.append(sb.length() == 0 ? "" : ",").append(BlockStateParser.serialize(b)));
            if (r == Role.TRAVERTINE)
                RockPalette.get().result("travertine").ifPresent(t -> {
                    sb.setLength(0);
                    t.blocks().forEach(b -> sb.append(sb.length() == 0 ? "" : ",").append(BlockStateParser.serialize(b)));
                });
            o.addProperty(r.id, sb.toString() + " (" + res.source().id() + ")");
        }
        return o;
    }
}
