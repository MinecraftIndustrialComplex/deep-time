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
 * The blocks of the lava flows ({@link LavaFlows}), resolved as {@link MagmaPalette} resolves the mush's:
 * the blocks of {@code #deeptime:lava_flow/<paint>} if there are any (several are chosen among by a hash of
 * the column), else the first installed entry of {@code #deeptime:lava_flow_fallback/<paint>}, else the
 * paint's vanilla block. Deep Time ships no such tags and adds no block (ledger VOL-162, game-choice).
 *
 * <table>
 *   <caption>The paints and their vanilla blocks</caption>
 *   <tr><td>aa</td><td>{@code minecraft:blackstone}: black and rough, as aa's clinker is</td></tr>
 *   <tr><td>pahoehoe</td><td>{@code minecraft:smooth_basalt}: dark and smooth, as pahoehoe's ropy skin is</td></tr>
 *   <tr><td>blocky</td><td>{@code minecraft:blackstone}: a viscous flow's top is rubble of dark blocks too; a pack
 *       can tell it from aa with its own tag</td></tr>
 *   <tr><td>weathered</td><td>{@code minecraft:tuff}: grey, as a flow grows grey with lichen</td></tr>
 *   <tr><td>oxidized</td><td>{@code minecraft:coarse_dirt}: brown, as an old flow oxidises and takes its first
 *       soil (in patches; trees can root in it)</td></tr>
 * </table>
 *
 * <p>Rebuilt whenever the server (re)loads its data; chunk generation and far terrain read {@link #get()}.
 * Before the first load, and for the tests, it holds the vanilla blocks.
 */
public final class LavaFlowPalette {

    public static final String TAG_MEMBERS = "lava_flow/", TAG_FALLBACK = "lava_flow_fallback/";

    /** The vanilla block of each paint. */
    static String vanilla(LavaFlows.Paint p) {
        return switch (p) {
            case AA, BLOCKY -> "minecraft:blackstone";
            case PAHOEHOE -> "minecraft:smooth_basalt";
            case WEATHERED -> "minecraft:tuff";
            case OXIDIZED -> "minecraft:coarse_dirt";
        };
    }

    private static Block vanillaBlock(LavaFlows.Paint p) {
        return BuiltInRegistries.BLOCK.get(ResourceLocation.parse(vanilla(p)));
    }

    private static volatile LavaFlowPalette current;

    private final Map<LavaFlows.Paint, PaletteRules.Result<BlockState>> byPaint;

    private LavaFlowPalette(Map<LavaFlows.Paint, PaletteRules.Result<BlockState>> byPaint) {
        this.byPaint = byPaint;
    }

    /** The palette of the vanilla blocks alone (before the first data load). */
    public static LavaFlowPalette vanilla() {
        Map<LavaFlows.Paint, PaletteRules.Result<BlockState>> m = new EnumMap<>(LavaFlows.Paint.class);
        for (LavaFlows.Paint p : LavaFlows.Paint.values())
            m.put(p, new PaletteRules.Result<>(p.id, PaletteRules.Source.VANILLA_FALLBACK, List.of(vanillaBlock(p).defaultBlockState()),
                    List.of()));
        return new LavaFlowPalette(m);
    }

    public static LavaFlowPalette get() {
        LavaFlowPalette p = current;
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
            LavaFlowPalette p = build(e.getRegistryAccess());
            current = p;
            DeepTime.LOGGER.info("Lava flow palette: {}", p.toJson());
        });
    }

    /** Builds a palette from a server's registries (tags must be bound). */
    public static LavaFlowPalette build(RegistryAccess access) {
        Map<LavaFlows.Paint, PaletteRules.Result<BlockState>> m = new EnumMap<>(LavaFlows.Paint.class);
        for (LavaFlows.Paint p : LavaFlows.Paint.values()) {
            PaletteRules.Kinds<BlockState> kinds = BlockState::isAir;
            var candidates = new PaletteRules.Candidates<>(tag(access, TAG_MEMBERS + p.id), List.of(),
                    tag(access, TAG_FALLBACK + p.id), List.of(), java.util.Optional.of(vanillaBlock(p).defaultBlockState()));
            m.put(p, PaletteRules.resolve(p.id, candidates, kinds, BlockStateParser::serialize));
        }
        return new LavaFlowPalette(m);
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

    /** The key a column's block is picked by (x folded into the lap, so near and far pick the same). */
    public static long key(int wx, int z) {
        return PaletteRules.mix(((long) wx * 0x9E3779B97F4A7C15L) ^ ((long) z * 0xC2B2AE3D27D4EB4FL) ^ 0x4C61766146L);
    }

    /** The block of a flow's code ({@link LavaFlows#code}) at a column, or null for a code that paints nothing. */
    public BlockState block(int code, long key) {
        LavaFlows.Paint p = LavaFlows.Paint.of(code);
        if (p == null)
            return null;
        BlockState s = byPaint.get(p).pick(key);
        return s != null ? s : Blocks.BLACKSTONE.defaultBlockState();
    }

    /** Every block {@code paint} can place. */
    public List<BlockState> blocks(LavaFlows.Paint paint) {
        return byPaint.get(paint).blocks();
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        for (LavaFlows.Paint p : LavaFlows.Paint.values()) {
            var res = byPaint.get(p);
            StringBuilder sb = new StringBuilder();
            res.blocks().forEach(b -> sb.append(sb.length() == 0 ? "" : ",").append(BlockStateParser.serialize(b)));
            o.addProperty(p.id, sb + " (" + res.source().id() + ")");
        }
        return o;
    }
}
