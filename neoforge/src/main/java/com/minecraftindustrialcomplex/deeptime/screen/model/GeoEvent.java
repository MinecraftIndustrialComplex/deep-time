// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.List;

/**
 * One event of the shared log, as the timelapse ticker needs it ({@code EVENTS.md}, "JSON";
 * plan §5.5).
 *
 * @param tStart years relative to world creation (negative = the past)
 * @param kind   namespaced kind ({@code deeptime:rift_onset})
 * @param place  {@code global}, {@code point}, {@code plate}, {@code region}, {@code site} or {@code path}
 * @param lat    degrees for a point place, else NaN
 * @param lon    degrees east of the seam for a point place, else NaN
 * @param site   the site id of a site place, else −1
 */
public record GeoEvent(long id, double tStart, double tEnd, String kind, String place, double lat, double lon, long site,
                       double magnitude) {

    /** Parses a JSON array of events ({@code eventsSince}); skips entries it cannot read. */
    public static List<GeoEvent> parseArray(String json) {
        List<GeoEvent> out = new ArrayList<>();
        JsonArray a;
        try {
            a = JsonParser.parseString(json).getAsJsonArray();
        } catch (RuntimeException e) {
            return out;
        }
        for (JsonElement e : a) {
            try {
                out.add(of(e.getAsJsonObject()));
            } catch (RuntimeException ignored) {
                // a malformed event does not stop the ticker
            }
        }
        return out;
    }

    public static GeoEvent of(JsonObject o) {
        JsonObject p = o.has("place") && o.get("place").isJsonObject() ? o.getAsJsonObject("place") : new JsonObject();
        String type = p.has("type") ? p.get("type").getAsString() : "global";
        double lat = Double.NaN, lon = Double.NaN;
        long site = -1;
        if ("point".equals(type)) {
            lat = num(p, "lat");
            lon = num(p, "lon");
        } else if ("site".equals(type) && p.has("site")) {
            site = p.get("site").getAsLong();
        }
        double ts = num(o, "t_start");
        double te = o.has("t_end") ? num(o, "t_end") : ts;
        return new GeoEvent(o.get("id").getAsLong(), ts, te, o.get("kind").getAsString(), type, lat, lon, site,
                num(o, "magnitude"));
    }

    private static double num(JsonObject o, String k) {
        JsonElement e = o.get(k);
        return e == null || e.isJsonNull() ? Double.NaN : e.getAsDouble();
    }

    /** True when the event has a point on the map. */
    public boolean hasPoint() {
        return !Double.isNaN(lat) && !Double.isNaN(lon);
    }

    /** The kind's path ({@code rift_onset}). */
    public String kindPath() {
        return kind.substring(kind.indexOf(':') + 1);
    }
}
