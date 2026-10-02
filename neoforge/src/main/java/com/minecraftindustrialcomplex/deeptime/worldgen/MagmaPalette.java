// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
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
 * The blocks of the body of altered hot rock around a volcano's crust-deep magma column (the
 * "mush"; Ben, decisions log 2026-10-02), resolved the way {@link GeothermalPalette} and the rock
 * classes are ({@link PaletteRules}): the blocks of {@code #deeptime:magma/<role>} if there are
 * any (several are chosen among by a hash of the position), else the first installed entry of
 * {@code #deeptime:magma_fallback/<role>}, else the role's vanilla block. Deep Time ships no such
 * tags and adds no block.
 *
 * <table>
 *   <caption>The roles and their vanilla blocks</caption>
 *   <tr><td>MUSH</td><td>the crystal-rich rock that holds the lenses: {@code minecraft:basalt}</td></tr>
 *   <tr><td>HOT</td><td>glowing pockets in it, about one block in 25: {@code minecraft:magma_block}</td></tr>
 * </table>
 *
 * <p>Why basalt: the shell that walls the lava in is smooth basalt, which is also what the rock
 * classes {@code basalt} and {@code gabbro} resolve to in the MIC pack, so the mush needs a block of
 * its own that still reads as dark, hot, volcanic rock; the pillar basalt is mined and smelted like
 * stone, is not an ore host and not a building block people confuse with the shell. Magma blocks glow
 * and burn feet, which is the right warning for ground this close to lava, and are sparse enough not
 * to make the mush impassable.
 *
 * <p>Rebuilt whenever the server (re)loads its data; chunk generation reads {@link #get()}. Before
 * the first load, and for the tests, it holds the vanilla blocks.
 */
public final class MagmaPalette {

    public enum Role {
        MUSH("mush", "minecraft:basalt"), HOT("hot", "minecraft:magma_block");

        public final String id, vanilla;

        Role(String id, String vanilla) {
            this.id = id;
            this.vanilla = vanilla;
        }

        public Block vanillaBlock() {
            return BuiltInRegistries.BLOCK.get(ResourceLocation.parse(vanilla));
        }
    }

    public static final String TAG_MEMBERS = "magma/", TAG_FALLBACK = "magma_fallback/";

    /** The share of mush blocks that are {@link Role#HOT}: 1 in this many. */
    public static final int HOT_ONE_IN = 25; // ledger: MAG-056

    private static volatile MagmaPalette current;

    private final Map<Role, PaletteRules.Result<BlockState>> byRole;

    private MagmaPalette(Map<Role, PaletteRules.Result<BlockState>> byRole) {
        this.byRole = byRole;
    }

    /** The palette of the vanilla blocks alone (before the first data load). */
    public static MagmaPalette vanilla() {
        Map<Role, PaletteRules.Result<BlockState>> m = new EnumMap<>(Role.class);
        for (Role r : Role.values())
            m.put(r, new PaletteRules.Result<>(r.id, PaletteRules.Source.VANILLA_FALLBACK, List.of(r.vanillaBlock().defaultBlockState()),
                    List.of()));
        return new MagmaPalette(m);
    }

    public static MagmaPalette get() {
        MagmaPalette p = current;
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
            MagmaPalette p = build(e.getRegistryAccess());
            current = p;
            DeepTime.LOGGER.info("Magma palette: {}", p.toJson());
        });
    }

    /** Builds a palette from a server's registries (tags must be bound). */
    public static MagmaPalette build(RegistryAccess access) {
        Map<Role, PaletteRules.Result<BlockState>> m = new EnumMap<>(Role.class);
        for (Role role : Role.values()) {
            PaletteRules.Kinds<BlockState> kinds = BlockState::isAir;
            var candidates = new PaletteRules.Candidates<>(tag(access, TAG_MEMBERS + role.id), List.of(),
                    tag(access, TAG_FALLBACK + role.id), List.of(), java.util.Optional.of(role.vanillaBlock().defaultBlockState()));
            m.put(role, PaletteRules.resolve(role.id, candidates, kinds, BlockStateParser::serialize));
        }
        return new MagmaPalette(m);
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

    /** The block of {@code role} at a position: one of its members picked by {@code key}. */
    public BlockState block(Role role, long key) {
        BlockState s = byRole.get(role).pick(key);
        return s != null ? s : Blocks.BASALT.defaultBlockState();
    }

    /** The mush block at a position: usually {@link Role#MUSH}, one in {@link #HOT_ONE_IN} {@link Role#HOT}. */
    public BlockState mush(int x, int y, int z) {
        long key = PaletteRules.mix(((long) x * 0x9E3779B97F4A7C15L) ^ ((long) z * 0xC2B2AE3D27D4EB4FL) ^ ((long) y << 40) ^ 0x4D75736821L);
        return Math.floorMod(key, (long) HOT_ONE_IN) == 0 ? block(Role.HOT, key >>> 7) : block(Role.MUSH, key >>> 7);
    }

    /** Every block {@code role} can place. */
    public List<BlockState> blocks(Role role) {
        return byRole.get(role).blocks();
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        for (Role r : Role.values()) {
            var res = byRole.get(r);
            StringBuilder sb = new StringBuilder();
            res.blocks().forEach(b -> sb.append(sb.length() == 0 ? "" : ",").append(BlockStateParser.serialize(b)));
            o.addProperty(r.id, sb + " (" + res.source().id() + ")");
        }
        return o;
    }
}
