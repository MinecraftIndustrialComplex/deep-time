// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;

/**
 * Deep Time's biome rules (Ben, decisions log 2026-10-04: "Build out support for both so if I install EE it
 * works or if I go w/ terralith it also works"): data files that place a pack's biomes from Deep Time's own
 * coordinates (the Köppen class, the climate in °C and mm, the wetland flag, the ice cover, playas, the
 * volcanic roles, the rock at the surface) rather than from where the pack's parameter list puts them. That
 * is what makes biomes reachable that the climate search alone never reaches (Expanded Ecosphere's ice cap
 * below vanilla's temperature range, its salt lakes and mud pools above it, its volcanoes at a
 * continentalness no land has). The game side is {@link BiomeRules}; this class is the file format and the
 * matching, plain Java so it can be tested without the game. FORMAT.md, "Biome rules", is the schema.
 *
 * <p>A file ({@code data/<ns>/deeptime/biome_rules/<name>.json}) is active when the level's biome source can
 * place at least one biome of its {@code requires} list (none: always). Its rules are tried in order (files in
 * the order of their ids); the first rule whose conditions hold for a column and that has a candidate in the
 * source decides: the source's own climate search is run over that rule's candidates alone, so the pack's own
 * parameter ranges still choose among them, and the nearest candidate answers. A rule with a {@code role} is
 * consulted only where a volcanic zone of that role holds the column ({@link VolcanicBiomes}), before the
 * role's tag. A rule marked {@code exclusive} keeps its biomes out of the climate search everywhere else.
 *
 * @see BiomeRules
 */
public final class BiomeRuleSet {

    private BiomeRuleSet() {}

    /** The resource folder of the rule files. */
    public static final String DIR = "deeptime/biome_rules";

    /** The wetland state of a column: none, wetland in a temperate or warm climate, cold (boreal) wetland. */
    public enum Wetland {
        NONE, WARM, COLD;

        /** Whether this state meets a rule's {@code wetland} condition ({@code any}, {@code warm}, {@code cold}, {@code none}). */
        static boolean meets(String want, Wetland have) {
            return switch (want) {
                case "any" -> have != NONE;
                case "warm" -> have == WARM;
                case "cold" -> have == COLD;
                case "none" -> have == NONE;
                default -> false;
            };
        }
    }

    /** A closed range, either end open when NaN. */
    public record Range(double min, double max) {
        public boolean has(double v) {
            return !Double.isNaN(v) && (Double.isNaN(min) || v >= min) && (Double.isNaN(max) || v <= max);
        }
    }

    /** What a rule asks about one column. Implementations compute lazily: a rule asks only what it names. */
    public interface Facts {
        /** The Köppen class code ("Af" ... "EF"; "sea" off land). */
        String koppen();

        /** Annual mean, warmest and coldest month temperatures at the column's surface, °C (NaN without a climate). */
        double tMean();

        double tWarm();

        double tCold();

        /** Annual precipitation, mm (NaN without a climate). */
        double pMm();

        /** Height of the surface above sea level, m. */
        double elevationM();

        Wetland wetland();

        /** Whether the ice cover lays ice on the column. */
        boolean ice();

        /** Whether the column is a playa (the hydrology's salt flat). */
        boolean playa();

        /** Whether any volcanic zone holds the column. */
        boolean volcanic();

        /** The rock class at the top of the column's strata ("" when the world has none). */
        String topRock();

        /** Whether the biome the plain climate search gives the column is in this list of ids and {@code #tags}. */
        boolean plainIn(List<String> ids);

        /**
         * The geology roles of the column's ground, a bit per {@link GeologyGround.Role} (shield province, granite,
         * coasts of carbonate, granite or basalt, dry sedimentary rock, humid carbonate); 0 when unknown.
         */
        default int ground() {
            return 0;
        }

        /**
         * Whether the biome the source's own climate search gives the column with nothing kept out (the slot its climate
         * point is in) is in this list of ids and {@code #tags}; false when unknown.
         */
        default boolean slotIn(List<String> ids) {
            return false;
        }
    }

    /**
     * One rule. Every condition is optional (null or an empty list: no condition); all named conditions must
     * hold.
     *
     * @param id        a name for logs and the census
     * @param biomes    the candidates: biome ids and {@code #tags}, in order (the search, not the order, picks one)
     * @param role      a volcanic role id ({@code cone}, {@code crater}, ...): the rule is consulted only inside
     *                  that role's zones; null: an ordinary rule
     * @param koppen    Köppen class codes or their prefixes ({@code "Cs"} is Csa, Csb and Csc; {@code "E"} is ET and EF)
     * @param ground    the geology roles of which the column's ground must have one, a bit per
     *                  {@link GeologyGround.Role}; 0: no condition
     * @param inSlot    true: the rule holds only where the source's own climate search, with nothing kept out, gives
     *                  one of {@code biomes} (with {@code exclusive}, a gate: the candidates keep their own slots, but
     *                  only where the other conditions hold)
     * @param exclusive the candidates go only where this rule (or another rule naming them) places them
     */
    public record Rule(String id, List<String> biomes, String role, List<String> koppen, Range tMean, Range tWarm,
                       Range tCold, Range pMm, Range elevationM, String wetland, Boolean ice, Boolean playa, Boolean volcanic,
                       List<String> rock, int ground, boolean inSlot, List<String> replaces, List<String> keeps, boolean exclusive) {

        public Rule {
            biomes = List.copyOf(biomes);
            koppen = koppen == null ? List.of() : List.copyOf(koppen);
            rock = rock == null ? List.of() : List.copyOf(rock);
            replaces = replaces == null ? List.of() : List.copyOf(replaces);
            keeps = keeps == null ? List.of() : List.copyOf(keeps);
        }

        /**
         * Whether every condition holds for the column (the cheap ones first). {@code replaces} and {@code keeps} come
         * before the climate: their plain search is the level's own answer when no rule holds, so it is made anyway,
         * while the climate is sampled only for the rules that read it (the snow line's rules replace a few biomes
         * on every surface column).
         */
        public boolean matches(Facts f) {
            if (!koppen.isEmpty()) {
                String k = f.koppen();
                boolean any = false;
                for (String p : koppen)
                    any |= k.startsWith(p);
                if (!any)
                    return false;
            }
            if (ice != null && ice != f.ice())
                return false;
            if (playa != null && playa != f.playa())
                return false;
            if (volcanic != null && volcanic != f.volcanic())
                return false;
            if (wetland != null && !Wetland.meets(wetland, f.wetland()))
                return false;
            if (elevationM != null && !elevationM.has(f.elevationM()))
                return false;
            if (!replaces.isEmpty() && !f.plainIn(replaces))
                return false;
            if (!keeps.isEmpty() && f.plainIn(keeps))
                return false;
            if (tMean != null && !tMean.has(f.tMean()))
                return false;
            if (tWarm != null && !tWarm.has(f.tWarm()))
                return false;
            if (tCold != null && !tCold.has(f.tCold()))
                return false;
            if (pMm != null && !pMm.has(f.pMm()))
                return false;
            if (!rock.isEmpty() && !rock.contains(f.topRock()))
                return false;
            if (inSlot && !f.slotIn(biomes))
                return false;
            return ground == 0 || (f.ground() & ground) != 0;
        }
    }

    /**
     * One file: its id ({@code ns:name}), the biomes that make it active, the biomes it keeps out of the climate search
     * without placing them anywhere ({@code keep_out}: a pack's biomes whose slots Deep Time's coordinates reach in the
     * wrong place, such as land islands at a continentalness Deep Time gives the deep sea), its rules in order.
     */
    public record File(String id, List<String> requires, List<String> keepOut, List<Rule> rules) {
        public File {
            requires = List.copyOf(requires);
            keepOut = List.copyOf(keepOut);
            rules = List.copyOf(rules);
        }
    }

    private static final Set<String> RULE_KEYS = Set.of("id", "biomes", "role", "koppen", "t_mean_c", "t_warm_c", "t_cold_c", "p_mm",
            "elevation_m", "wetland", "ice", "playa", "volcanic", "rock", "ground", "in_slot", "replaces", "keeps", "exclusive", "_comment");
    private static final Set<String> WETLAND = Set.of("any", "warm", "cold", "none");
    private static final Set<String> ROLES = Set.of("hot_spring_field", "caldera", "crater", "cone", "ash_field", "lava_field",
            "thermal_caves");

    /** Parses one file; throws {@link IllegalArgumentException} naming the file and the problem. */
    public static File parse(String id, JsonObject o) {
        for (String k : o.keySet())
            if (!k.startsWith("_") && !Set.of("requires", "keep_out", "rules").contains(k))
                throw new IllegalArgumentException(id + ": unknown key '" + k + "'");
        List<String> requires = strings(o, "requires", id);
        List<String> keepOut = strings(o, "keep_out", id);
        JsonElement rs = o.get("rules");
        if (rs == null || !rs.isJsonArray())
            throw new IllegalArgumentException(id + ": no 'rules' array");
        List<Rule> rules = new ArrayList<>();
        int n = 0;
        for (JsonElement e : rs.getAsJsonArray()) {
            n++;
            if (!e.isJsonObject())
                throw new IllegalArgumentException(id + ": rule " + n + " is not an object");
            rules.add(rule(id, n, e.getAsJsonObject()));
        }
        return new File(id, requires, keepOut, rules);
    }

    private static Rule rule(String file, int n, JsonObject o) {
        String where = file + " rule " + n;
        for (String k : o.keySet())
            if (!k.startsWith("_") && !RULE_KEYS.contains(k))
                throw new IllegalArgumentException(where + ": unknown key '" + k + "'");
        String id = o.has("id") ? o.get("id").getAsString() : "rule" + n;
        where = file + " rule '" + id + "'";
        List<String> biomes = strings(o, "biomes", where);
        if (biomes.isEmpty())
            throw new IllegalArgumentException(where + ": no 'biomes'");
        String role = o.has("role") ? o.get("role").getAsString() : null;
        if (role != null && !ROLES.contains(role))
            throw new IllegalArgumentException(where + ": unknown role '" + role + "' (one of " + ROLES + ")");
        String wetland = o.has("wetland") ? o.get("wetland").getAsString().toLowerCase(Locale.ROOT) : null;
        if (wetland != null && !WETLAND.contains(wetland))
            throw new IllegalArgumentException(where + ": wetland must be one of " + WETLAND);
        int ground = 0;
        for (String g : strings(o, "ground", where)) {
            GeologyGround.Role r = GeologyGround.Role.byId(g);
            if (r == null)
                throw new IllegalArgumentException(where + ": unknown ground '" + g + "' (one of " + GeologyGround.Role.ids() + ")");
            ground |= r.bit();
        }
        return new Rule(id, biomes, role, strings(o, "koppen", where), range(o, "t_mean_c", where), range(o, "t_warm_c", where),
                range(o, "t_cold_c", where), range(o, "p_mm", where), range(o, "elevation_m", where), wetland, bool(o, "ice"),
                bool(o, "playa"), bool(o, "volcanic"), strings(o, "rock", where), ground, o.has("in_slot") && o.get("in_slot").getAsBoolean(),
                strings(o, "replaces", where), strings(o, "keeps", where), o.has("exclusive") && o.get("exclusive").getAsBoolean());
    }

    private static Boolean bool(JsonObject o, String key) {
        return o.has(key) ? o.get(key).getAsBoolean() : null;
    }

    private static Range range(JsonObject o, String key, String where) {
        if (!o.has(key))
            return null;
        if (!o.get(key).isJsonObject())
            throw new IllegalArgumentException(where + ": '" + key + "' must be {\"min\": .., \"max\": ..}");
        JsonObject r = o.getAsJsonObject(key);
        double min = r.has("min") ? r.get("min").getAsDouble() : Double.NaN;
        double max = r.has("max") ? r.get("max").getAsDouble() : Double.NaN;
        if (Double.isNaN(min) && Double.isNaN(max))
            throw new IllegalArgumentException(where + ": '" + key + "' names neither min nor max");
        if (!Double.isNaN(min) && !Double.isNaN(max) && min > max)
            throw new IllegalArgumentException(where + ": '" + key + "' has min > max");
        return new Range(min, max);
    }

    private static List<String> strings(JsonObject o, String key, String where) {
        JsonElement e = o.get(key);
        if (e == null)
            return List.of();
        List<String> out = new ArrayList<>();
        if (e.isJsonPrimitive())
            out.add(e.getAsString());
        else if (e.isJsonArray())
            for (JsonElement x : (JsonArray) e)
                out.add(x.getAsString());
        else
            throw new IllegalArgumentException(where + ": '" + key + "' must be a string or a list of strings");
        return out;
    }
}
