// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;

import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * The biome rules' part of {@link BiomeCensus} ({@link BiomeRules}; decisions log 2026-10-04): per land column
 * at the surface, what decided its biome ({@link VolcanicBiomes#lastDecision()}: a rule, a volcanic role or the
 * climate search), and three checks the GameTests read: a column a rule decided holds one of that rule's
 * candidates and lies in one of its Köppen classes (read by the census itself, from the layer the rules read:
 * {@code climate/koppen_l2} when the bake wrote it, else {@code climate/koppen}); an
 * exclusive candidate appears only where a rule put it; and the share of cold wetland that took a peatland
 * biome where the pack has one.
 */
public final class RuleCensus {

    private final BiomeRules.Set_ rules;
    private final Map<String, BiomeRules.Bound> byId = new HashMap<>();
    private final Map<String, Map<String, Integer>> decided = new TreeMap<>();
    private final Map<String, Integer> seen = new TreeMap<>();
    private int columns, notCandidate, wrongClass, exclusiveLeaked, coldWetland, coldWetlandPeat;
    private final List<String> examples = new java.util.ArrayList<>();
    /** Cold-wetland columns per decision and biome ("&lt;decision&gt; &lt;biome&gt;"), so a low peatland share names its cause. */
    private final Map<String, Integer> coldWetlandBy = new TreeMap<>();

    public RuleCensus(BiomeRules.Set_ rules) {
        this.rules = rules;
        for (BiomeRules.Bound b : rules.ordinary())
            byId.put("rule:" + b.file() + "/" + b.rule().id(), b);
        for (List<BiomeRules.Bound> l : rules.byRole().values())
            for (BiomeRules.Bound b : l)
                byId.put("rule:" + b.file() + "/" + b.rule().id(), b);
    }

    /**
     * One land column at the surface.
     *
     * @param decision what decided it ({@link VolcanicBiomes#lastDecision()}), null for the climate search
     * @param koppen   its Köppen code as the census read it
     * @param coldWet  whether it is cold wetland (the census's own reading of the bake's flag or rule)
     */
    public void add(Holder<Biome> b, String decision, String koppen, boolean coldWet) {
        columns++;
        String name = BiomeCensus.nameOf(b);
        String key = decision == null ? "climate search" : decision;
        decided.computeIfAbsent(key, k -> new TreeMap<>()).merge(name, 1, Integer::sum);
        BiomeRules.Bound r = decision == null ? null : byId.get(decision);
        if (r != null) {
            if (!r.candidates().contains(b)) {
                notCandidate++;
                example(name + " decided by " + decision + " is not one of its candidates");
            }
            if (!r.rule().koppen().isEmpty() && r.rule().koppen().stream().noneMatch(koppen::startsWith)) {
                wrongClass++;
                example(name + " decided by " + decision + " in class " + koppen);
            }
        } else if (rules.exclusive().contains(b) && (decision == null || !decision.startsWith("role:"))) {
            exclusiveLeaked++;
            example(name + " (exclusive) placed by " + key + " in class " + koppen);
        }
        for (BiomeRules.Bound x : byId.values())
            if (x.candidates().contains(b)) {
                seen.merge(name, 1, Integer::sum);
                break;
            }
        if (coldWet) {
            coldWetland++;
            if (b.is(PEATLAND))
                coldWetlandPeat++;
            coldWetlandBy.merge(key + " " + name, 1, Integer::sum);
        }
    }

    static final net.minecraft.tags.TagKey<Biome> PEATLAND = net.minecraft.tags.TagKey.create(net.minecraft.core.registries.Registries.BIOME,
            net.minecraft.resources.ResourceLocation.fromNamespaceAndPath(com.minecraftindustrialcomplex.deeptime.DeepTime.MODID, "peatland"));

    private void example(String s) {
        if (examples.size() < 8)
            examples.add(s);
    }

    public int columns() {
        return columns;
    }

    public int ruleColumns() {
        int n = 0;
        for (var e : decided.entrySet())
            if (e.getKey().startsWith("rule:"))
                n += e.getValue().values().stream().mapToInt(Integer::intValue).sum();
        return n;
    }

    public int notCandidate() {
        return notCandidate;
    }

    public int wrongClass() {
        return wrongClass;
    }

    public int exclusiveLeaked() {
        return exclusiveLeaked;
    }

    public int coldWetland() {
        return coldWetland;
    }

    public int coldWetlandPeat() {
        return coldWetlandPeat;
    }

    /** Cold-wetland columns per decision and biome, keyed "&lt;decision&gt; &lt;biome&gt;". */
    public Map<String, Integer> coldWetlandBy() {
        return coldWetlandBy;
    }

    /** Columns per biome of the rules' candidates (every rule's, role rules included). */
    public Map<String, Integer> seen() {
        return seen;
    }

    /** Columns per decision ("climate search", "role:&lt;id&gt;", "rule:&lt;file&gt;/&lt;id&gt;"). */
    public Map<String, Integer> decisions() {
        Map<String, Integer> out = new LinkedHashMap<>();
        decided.forEach((k, v) -> out.put(k, v.values().stream().mapToInt(Integer::intValue).sum()));
        return out;
    }

    public List<String> examples() {
        return examples;
    }

    /** Columns per biome of one decision. */
    public Map<String, Integer> biomes(String decision) {
        return decided.getOrDefault(decision, Map.of());
    }

    public JsonObject json() {
        JsonObject o = new JsonObject();
        JsonArray files = new JsonArray();
        rules.files().forEach(files::add);
        o.add("files", files);
        o.addProperty("land_columns", columns);
        o.addProperty("rule_columns", ruleColumns());
        JsonObject by = new JsonObject();
        for (var e : decided.entrySet()) {
            int total = e.getValue().values().stream().mapToInt(Integer::intValue).sum();
            JsonObject d = new JsonObject();
            d.addProperty("columns", total);
            JsonArray top = new JsonArray();
            e.getValue().entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue(Comparator.reverseOrder())).limit(6)
                    .forEach(x -> top.add(x.getKey() + " " + x.getValue()));
            d.add("top", top);
            by.add(e.getKey(), d);
        }
        o.add("by_decision", by);
        JsonObject s = new JsonObject();
        seen.forEach(s::addProperty);
        o.add("candidates_seen", s);
        JsonArray absent = new JsonArray();
        java.util.Set<String> all = new java.util.TreeSet<>();
        for (BiomeRules.Bound x : byId.values())
            for (Holder<Biome> b : x.candidates())
                all.add(BiomeCensus.nameOf(b));
        for (String n : all)
            if (!seen.containsKey(n))
                absent.add(n);
        o.add("candidates_absent", absent);
        o.addProperty("not_candidate", notCandidate);
        o.addProperty("wrong_class", wrongClass);
        o.addProperty("exclusive_leaked", exclusiveLeaked);
        o.addProperty("cold_wetland_columns", coldWetland);
        o.addProperty("cold_wetland_on_peatland", coldWetlandPeat);
        JsonObject cw = new JsonObject();
        coldWetlandBy.forEach(cw::addProperty);
        o.add("cold_wetland_by", cw);
        JsonArray ex = new JsonArray();
        examples.forEach(ex::add);
        o.add("examples", ex);
        return o;
    }
}
