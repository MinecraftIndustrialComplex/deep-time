// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.Columns;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Function;

/**
 * The geology gate's ground (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"; research note
 * biome-mods-survey.md, fix 3): which {@link Role}s a column's ground has, from the world data a Deep Time world
 * already knows: its geologic province ({@code meta/province}), the rock class at its surface
 * ({@code surface/class_top}, with the rock-class registry's groups), its Köppen class ({@code climate/koppen}) and
 * whether it is on a coast. A pack's rock-named biomes (Terralith's Shield, Yosemite, White, Granite and Basalt
 * Cliffs, its red-bed canyons and Stony Spires) are kept to the ground of their role ({@code climate.VolcanicBiomes},
 * the tags {@code #deeptime:geology/<role>}).
 *
 * <p>Every rule here is a game choice (ledger BIO-147 to BIO-153): what ground suits which named landform.
 */
public final class GeologyGround {

    /**
     * A kind of ground a rock-named biome belongs on. The tag of its biomes is {@code #deeptime:geology/<id>}; the
     * ground tests are {@link #roles}. Each is a game choice (ledger: BIO-147 to BIO-153).
     */
    public enum Role {
        /** A Precambrian shield province (Terralith's Shield, after the Canadian Shield). BIO-147. */
        SHIELD("shield"),
        /** Granite or granodiorite at the surface (Terralith's Yosemite). BIO-148. */
        GRANITE("granite"),
        /** A coast of carbonate rock: chalk, limestone, dolostone, marble (Terralith's White Cliffs). BIO-149. */
        CARBONATE_COAST("carbonate_coast"),
        /** A coast of granite or granodiorite (Terralith's Granite Cliffs). BIO-149. */
        GRANITE_COAST("granite_coast"),
        /** A coast of basalt and other mafic lava (Terralith's Basalt Cliffs). BIO-149. */
        BASALT_COAST("basalt_coast"),
        /** Consolidated sedimentary rock at the surface in a dry (Köppen B) climate: red beds, canyons. BIO-150. */
        DRY_SEDIMENTARY("dry_sedimentary"),
        /** Carbonate rock at the surface in a humid (neither B nor E) climate: tower karst. BIO-151. */
        HUMID_KARST("humid_karst");

        /** The tag path under {@code deeptime:geology/}. */
        public final String id;

        Role(String id) {
            this.id = id;
        }

        public int bit() {
            return 1 << ordinal();
        }

        /** The role of a tag-path id, or null. */
        public static Role byId(String id) {
            for (Role r : values())
                if (r.id.equals(id))
                    return r;
            return null;
        }

        /** Every role's id, in order. */
        public static List<String> ids() {
            return java.util.Arrays.stream(values()).map(r -> r.id).toList();
        }

        /** The role ids of a bit set, joined by {@code +} ("-" for none), for logs and censuses. */
        public static String names(int bits) {
            StringBuilder s = new StringBuilder();
            for (Role r : values())
                if ((bits & r.bit()) != 0)
                    s.append(s.isEmpty() ? "" : "+").append(r.id);
            return s.isEmpty() ? "-" : s.toString();
        }
    }

    /** The rock groups the roles read (the rock-class registry's {@code groups}). */
    public static final String SEDIMENTARY = "deeptime:rock/sedimentary", CARBONATE = "deeptime:rock/carbonate",
            INTRUSIVE = "deeptime:rock/intrusive", FELSIC = "deeptime:rock/felsic", VOLCANIC = "deeptime:rock/volcanic",
            MAFIC = "deeptime:rock/mafic", UNCONSOLIDATED = "deeptime:rock/unconsolidated";

    /** The province class of a shield ({@code notes.bake.meta.provinces.province[i].class}). */
    public static final String SHIELD_CLASS = "Shield";

    /** A column is on a coast when the sea lies within this many blocks of it (two biome-blending reaches). BIO-152. */
    public static final int COAST_BLOCKS = 32; // ledger: BIO-152 (game-choice)
    /** How far from a coastal column the coast's bedrock is looked for, when the column itself is loose sediment. */
    public static final int COAST_ROCK_BLOCKS = 48; // ledger: BIO-152 (game-choice)

    /** Köppen class indices (`koppen::CODES`): 4–7 the dry climates (B), 29–30 the polar ones (E), 31 the sea. */
    static boolean dry(int k) {
        return k >= 4 && k <= 7;
    }

    static boolean humid(int k) {
        return k >= 0 && k <= 28 && !dry(k);
    }

    /**
     * The roles of a column's ground (a bit per {@link Role}). {@code rock} is the groups of its surface rock (empty
     * when unknown), {@code coastRock} those of the coast's bedrock (the column's own unless it is loose sediment;
     * empty when not on a coast or unknown). Pure, for the tests.
     */
    public static int roles(String provinceClass, Set<String> rock, boolean coastal, Set<String> coastRock, int koppen) {
        int bits = 0;
        if (SHIELD_CLASS.equals(provinceClass))
            bits |= Role.SHIELD.bit();
        boolean loose = rock.contains(UNCONSOLIDATED);
        if (!loose && rock.contains(INTRUSIVE) && rock.contains(FELSIC))
            bits |= Role.GRANITE.bit();
        if (coastal) {
            if (coastRock.contains(CARBONATE))
                bits |= Role.CARBONATE_COAST.bit();
            if (coastRock.contains(INTRUSIVE) && coastRock.contains(FELSIC))
                bits |= Role.GRANITE_COAST.bit();
            if (coastRock.contains(VOLCANIC) && coastRock.contains(MAFIC))
                bits |= Role.BASALT_COAST.bit();
        }
        if (!loose && dry(koppen) && rock.contains(SEDIMENTARY))
            bits |= Role.DRY_SEDIMENTARY.bit();
        if (!loose && humid(koppen) && rock.contains(CARBONATE))
            bits |= Role.HUMID_KARST.bit();
        return bits;
    }

    private final TerrainSource terrain;
    private final WorldData data;
    private final int seaLevel;
    private final RasterSampler province, rock, koppen, strata;
    private final String[] provinceClass;
    private final List<String> rockNames;
    private final Function<String, Set<String>> groups;

    /**
     * @param groups the rock-class registry's groups of a class name (empty for an unknown class)
     */
    public GeologyGround(WorldDataTerrain terrain, int seaLevel, Function<String, Set<String>> groups) {
        this.terrain = terrain;
        this.data = terrain.data();
        this.seaLevel = seaLevel;
        this.groups = groups;
        this.province = data.has("meta/province") ? data.layer("meta/province") : null;
        this.rock = data.has("surface/class_top") ? data.layer("surface/class_top") : null;
        this.koppen = data.has("climate/koppen") ? data.layer("climate/koppen") : null;
        this.strata = data.has(WorldData.HORIZONS) ? data.layer(WorldData.HORIZONS) : null;
        this.provinceClass = provinceClasses(data.manifest().notes());
        this.rockNames = data.classes();
    }

    /** {@code notes.bake.meta.provinces.province[i].class} by id (empty when the bake wrote none). */
    static String[] provinceClasses(JsonObject notes) {
        JsonObject n = notes;
        for (String k : new String[]{"bake", "meta", "provinces"}) {
            if (n == null || !n.has(k) || !n.get(k).isJsonObject())
                return new String[0];
            n = n.getAsJsonObject(k);
        }
        if (!n.has("province") || !n.get("province").isJsonArray())
            return new String[0];
        JsonArray a = n.getAsJsonArray("province");
        String[] out = new String[a.size()];
        for (JsonElement e : a) {
            JsonObject p = e.getAsJsonObject();
            int id = p.get("id").getAsInt();
            if (id >= 0 && id < out.length)
                out[id] = p.get("class").getAsString();
        }
        return out;
    }

    /** The province class at the column, or "" when unknown. */
    public String provinceAt(int x, int z) {
        if (province == null)
            return "";
        double v = province.nearest(terrain.wrapX(x), z);
        if (Double.isNaN(v))
            return "";
        int id = (int) Math.round(v);
        return id >= 0 && id < provinceClass.length && provinceClass[id] != null ? provinceClass[id] : "";
    }

    /** The rock class name at the column's surface ({@code surface/class_top}), or "" when unknown. */
    public String surfaceRockAt(int x, int z) {
        if (rock == null)
            return "";
        double v = rock.nearest(terrain.wrapX(x), z);
        if (Double.isNaN(v))
            return "";
        int i = (int) Math.round(v);
        return i >= 0 && i < rockNames.size() ? rockNames.get(i) : "";
    }

    /**
     * The column's bedrock: its surface rock, or, where that is loose cover (saprolite, laterite, till, sand: the
     * weathering mantle holds most land), the first consolidated horizon of its stack within {@link #COVER_BLOCKS} of
     * the surface; "" when there is none that shallow.
     */
    public String rockAt(int x, int z) {
        String top = surfaceRockAt(x, z);
        if (top.isEmpty() || !groups.apply(top).contains(UNCONSOLIDATED) || strata == null)
            return top;
        Columns.Stack s = STACK.get();
        Columns.stack(data, strata, terrain.wrapX(x), z, s);
        int topY = (int) Math.floor(terrain.height(x, z));
        int h = Math.max(0, s.horizonAt(topY));
        for (; h < s.count; h++) {
            int hTop = Math.min(s.tops[h], topY);
            if (hTop < topY - COVER_BLOCKS)
                break;
            int ci = s.classes[h];
            String name = ci >= 0 && ci < rockNames.size() ? rockNames.get(ci) : "";
            if (!name.isEmpty() && !groups.apply(name).contains(UNCONSOLIDATED))
                return name;
        }
        return top;
    }

    /** Loose cover thinner than this many blocks still shows its bedrock's landform (at 35 m a block, about 280 m). */
    public static final int COVER_BLOCKS = 8; // ledger: BIO-153 (game-choice)

    private static final ThreadLocal<Columns.Stack> STACK = ThreadLocal.withInitial(() -> new Columns.Stack(32));

    /** The Köppen class index at the column (31, the sea, when unknown). */
    public int koppenAt(int x, int z) {
        if (koppen == null)
            return 31;
        double v = koppen.nearest(terrain.wrapX(x), z);
        return Double.isNaN(v) ? 31 : (int) Math.max(0, Math.min(31, Math.round(v)));
    }

    private static final int[][] DIRS = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}, {1, 1}, {1, -1}, {-1, 1}, {-1, -1}};

    /**
     * The sea within {@link #COAST_BLOCKS} of the column (eight directions, at half and full reach). Ground below sea
     * level in a closed basin is not sea (HYD-100; ledger: HYD-104).
     */
    public boolean coastal(int x, int z) {
        if (sea(x, z))
            return true;
        for (int[] d : DIRS)
            for (int r : new int[]{COAST_BLOCKS / 2, COAST_BLOCKS})
                if (sea(x + d[0] * r, z + d[1] * r))
                    return true;
        return false;
    }

    /** The coast's sea test: its own threshold, not {@code Water.seaCovers} (HYD-109). */
    private boolean sea(int x, int z) {
        return terrain.height(x, z) < seaLevel && !terrain.closedBasin(x, z);
    }

    /** The groups of the coast's bedrock: the column's rock, or the first consolidated rock round it. */
    private Set<String> coastRock(int x, int z, Set<String> own) {
        if (!own.isEmpty() && !own.contains(UNCONSOLIDATED))
            return own;
        for (int r : new int[]{COAST_ROCK_BLOCKS / 3, 2 * COAST_ROCK_BLOCKS / 3, COAST_ROCK_BLOCKS})
            for (int[] d : DIRS) {
                Set<String> g = groups.apply(rockAt(x + d[0] * r, z + d[1] * r));
                if (!g.isEmpty() && !g.contains(UNCONSOLIDATED))
                    return g;
            }
        return Set.of();
    }

    /** The roles of the column's ground (a bit per {@link Role}). */
    public int rolesAt(int x, int z) {
        Set<String> own = groups.apply(rockAt(x, z));
        boolean coastal = coastal(x, z);
        return roles(provinceAt(x, z), own, coastal, coastal ? coastRock(x, z, own) : Set.of(), koppenAt(x, z));
    }

    /** The groups of each rock class, from the rock-class registry's {@code groups}. */
    public static Function<String, Set<String>> groupsOf(Map<String, ? extends java.util.Collection<String>> byName) {
        Map<String, Set<String>> sets = new java.util.HashMap<>();
        byName.forEach((k, v) -> sets.put(k, Set.copyOf(v)));
        return name -> sets.getOrDefault(name, Set.of());
    }
}
