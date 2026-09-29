// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import net.minecraft.resources.ResourceLocation;

import java.util.List;
import java.util.Map;

/**
 * One event of a world's shared event log (plan §5.5; {@code crates/deeptime-format/EVENTS.md}).
 *
 * @param id        {@code writer << 48 | sequence}
 * @param tStart    years relative to world creation (negative = the geologic past)
 * @param tEnd      = tStart for an instantaneous event
 * @param kind      e.g. {@code deeptime:deposit_formation}
 * @param source    the writing stage, e.g. {@code deeptime:deposits}
 * @param magnitude kind-defined, for ranking
 * @param place     where it happened
 * @param causes    the events that caused it (possibly of other writers; unknown ones are kept)
 * @param params    kind-defined values: {@code Long}, {@code Double}, {@code String}, or an event id
 *                  as {@link IdRef}
 */
public record GeoEvent(long id, double tStart, double tEnd, ResourceLocation kind, ResourceLocation source,
                       float magnitude, Place place, List<Long> causes, Map<String, Object> params) {

    /** An id-typed param value. */
    public record IdRef(long id) {}

    /**
     * Where an event happened: {@code type} is global, point, plate, region, site or path; a point
     * has {@code lat}/{@code lon} (degrees; longitude east of the seam), the others an id in
     * {@code ref}.
     */
    public record Place(String type, float lat, float lon, long ref) {}

    /** {@code <writer>:<sequence>}, the event's text form. */
    public String text() {
        return (id >>> 48) + ":" + (id & ((1L << 48) - 1));
    }
}
