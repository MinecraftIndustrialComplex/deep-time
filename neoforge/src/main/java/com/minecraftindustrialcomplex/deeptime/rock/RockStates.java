// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.util.StringRepresentable;

import java.util.List;
import java.util.Map;

/**
 * One file of the {@code deeptime:rock_states} datapack registry
 * ({@code data/<ns>/deeptime/rock_states/<name>.json}): block <em>states</em> for a rock class,
 * which block tags cannot carry. The format is the one Ben adopted on 2026-09-27 (the rock-tag
 * README, "Block-state entries"):
 *
 * <pre>{@code
 * {"class": "loess", "use": "member",
 *  "entries": [{"block": "projectatmosphere:dust", "state": {"layers": "8"}, "required": false}]}
 * }</pre>
 *
 * <p>{@code member} entries join {@code #deeptime:rock/<class>} after the tag's blocks;
 * {@code fallback} entries follow the entries of {@code #deeptime:rock_fallback/<class>}. A state
 * lists property names and values as strings (as in blockstate JSON); unlisted properties take
 * the block's default. {@code required} defaults to true; a {@code false} entry whose block is not
 * installed is skipped. Files for one class merge in the order of their ids.
 */
public record RockStates(String rockClass, Use use, List<Entry> entries) {

    public enum Use implements StringRepresentable {
        MEMBER("member"),
        FALLBACK("fallback");

        public static final Codec<Use> CODEC = StringRepresentable.fromEnum(Use::values);

        private final String id;

        Use(String id) {
            this.id = id;
        }

        @Override
        public String getSerializedName() {
            return id;
        }
    }

    public record Entry(ResourceLocation block, Map<String, String> state, boolean required) {
        public static final Codec<Entry> CODEC = RecordCodecBuilder.create(i -> i.group(
                ResourceLocation.CODEC.fieldOf("block").forGetter(Entry::block),
                Codec.unboundedMap(Codec.STRING, Codec.STRING).optionalFieldOf("state", Map.of()).forGetter(Entry::state),
                Codec.BOOL.optionalFieldOf("required", true).forGetter(Entry::required)
        ).apply(i, Entry::new));
    }

    public static final Codec<RockStates> CODEC = RecordCodecBuilder.create(i -> i.group(
            RockClass.NAME.fieldOf("class").forGetter(RockStates::rockClass),
            Use.CODEC.fieldOf("use").forGetter(RockStates::use),
            Entry.CODEC.listOf().fieldOf("entries").forGetter(RockStates::entries)
    ).apply(i, RockStates::new));
}
