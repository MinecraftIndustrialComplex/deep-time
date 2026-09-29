// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.zip.CRC32;

/**
 * The shared event log of a world-data folder (plan §5.5; {@code crates/deeptime-format/EVENTS.md};
 * Rust {@code EventLog}): every segment listed in {@code events/index.json}, checked (size, CRC32,
 * registered writer, unique ids) and merged by (t_start, id). What "Why here?" reads. Plain Java,
 * immutable, thread-safe.
 */
public final class EventData {

    public static final String INDEX = "events/index.json";
    static final int RECORD_BYTES = 64;
    static final int PARAM_BYTES = 16;

    /** Place tags (EVENTS.md). */
    public static final List<String> PLACES = List.of("global", "point", "plate", "region", "site", "path");

    /** An event-id param value (distinct from an integer). */
    public record IdParam(long id) {}

    /**
     * One event. {@code placeA}/{@code placeB} are the raw place words: a point's latitude and
     * longitude bits, or the plate, mask, site or path id. Param values are {@code Long},
     * {@code Double}, {@code String} or {@link IdParam}.
     */
    public record EventRecord(long id, double tStart, double tEnd, String kind, String source, float magnitude,
                              int placeTag, int placeA, int placeB, long[] causes, Map<String, Object> params) {

        public int writer() {
            return (int) (id >>> 48);
        }

        public long seq() {
            return id & ((1L << 48) - 1);
        }

        /** {@code <writer>:<sequence>}. */
        public String text() {
            return writer() + ":" + seq();
        }

        public String placeType() {
            return placeTag < PLACES.size() ? PLACES.get(placeTag) : "unknown";
        }

        public float lat() {
            return Float.intBitsToFloat(placeA);
        }

        public float lon() {
            return Float.intBitsToFloat(placeB);
        }
    }

    private final List<EventRecord> events;
    private final Map<Long, EventRecord> byId;
    private final Map<Long, List<Long>> effects;

    private EventData(List<EventRecord> events) {
        this.events = events;
        Map<Long, EventRecord> ids = new HashMap<>();
        Map<Long, List<Long>> eff = new HashMap<>();
        for (EventRecord e : events) {
            ids.put(e.id(), e);
            for (long c : e.causes())
                eff.computeIfAbsent(c, k -> new ArrayList<>()).add(e.id());
        }
        this.byId = Map.copyOf(ids);
        eff.replaceAll((k, v) -> List.copyOf(v));
        this.effects = Map.copyOf(eff);
    }

    public static EventData empty() {
        return new EventData(List.of());
    }

    /** Reads {@code events/} of {@code folder}; an empty log without {@code events/index.json}. */
    public static EventData read(Path folder) {
        Path index = folder.resolve(INDEX);
        if (!Files.isRegularFile(index))
            return empty();
        JsonObject o;
        try {
            o = JsonParser.parseString(Files.readString(index, StandardCharsets.UTF_8)).getAsJsonObject();
        } catch (IOException | RuntimeException e) {
            throw new WorldDataException(index + ": " + e.getMessage(), e);
        }
        FormatVersion.require(o.get("format_version").getAsString());
        Set<Integer> writers = new HashSet<>();
        for (JsonElement w : o.getAsJsonArray("writers"))
            writers.add(w.getAsJsonObject().get("id").getAsInt());
        List<EventRecord> all = new ArrayList<>();
        Set<Long> seen = new HashSet<>();
        for (JsonElement se : o.getAsJsonArray("segments")) {
            JsonObject s = se.getAsJsonObject();
            Path path = folder.resolve(s.get("path").getAsString());
            int writer = s.get("writer").getAsInt();
            if (!writers.contains(writer))
                throw Sectioned.corrupt(index, "segment " + path.getFileName() + " of unregistered writer " + writer);
            byte[] bytes;
            try {
                bytes = Files.readAllBytes(path);
            } catch (IOException e) {
                throw new WorldDataException(path + ": " + e.getMessage(), e);
            }
            if (bytes.length != s.get("bytes").getAsLong())
                throw Sectioned.corrupt(path, bytes.length + " bytes, index says " + s.get("bytes").getAsLong());
            CRC32 crc = new CRC32();
            crc.update(bytes);
            if (crc.getValue() != s.get("crc32").getAsLong())
                throw new WorldDataException(String.format("%s: segment CRC32 %08x, index says %08x", path, crc.getValue(),
                        s.get("crc32").getAsLong()));
            for (EventRecord e : decodeSegment(path, bytes, writer, s.get("segment").getAsInt())) {
                if (!seen.add(e.id()))
                    throw Sectioned.corrupt(path, "event " + e.text() + " appears twice");
                all.add(e);
            }
        }
        all.sort((a, b) -> {
            int c = Double.compare(a.tStart(), b.tStart());
            return c != 0 ? c : Long.compareUnsigned(a.id(), b.id());
        });
        return new EventData(List.copyOf(all));
    }

    static List<EventRecord> decodeSegment(Path path, byte[] bytes, int writer, int segment) {
        Sectioned s = Sectioned.parse(path, bytes, "DTEV", 32, f -> 5);
        ByteBuffer f = s.fixed();
        int w = f.getShort(0) & 0xFFFF, seg = f.getInt(4), n = f.getInt(8), ns = f.getInt(12);
        if (w != writer || seg != segment)
            throw Sectioned.corrupt(path, "holds segment " + w + "-" + seg + ", index says " + writer + "-" + segment);
        ByteBuffer recs = s.section(1), causes = s.section(2), params = s.section(3);
        if (recs.capacity() != n * RECORD_BYTES)
            throw Sectioned.corrupt(path, recs.capacity() + " record bytes for " + n + " events");
        if (causes.capacity() % 8 != 0 || params.capacity() % PARAM_BYTES != 0)
            throw Sectioned.corrupt(path, "cause or params section not a whole number of entries");
        List<String> strings = s.strings(0, ns);
        List<EventRecord> out = new ArrayList<>(n);
        for (int r = 0; r < n; r++) {
            int a = r * RECORD_BYTES;
            long id = recs.getLong(a);
            int tag = recs.get(a + 36) & 0xFF;
            if (tag >= PLACES.size())
                throw Sectioned.corrupt(path, "event " + id + ": place tag " + tag);
            int cf = recs.getInt(a + 48), cn = recs.getInt(a + 52);
            if (cf < 0 || cn < 0 || (long) (cf + cn) * 8 > causes.capacity())
                throw Sectioned.corrupt(path, "event " + id + ": causes past the section");
            long[] cs = new long[cn];
            for (int i = 0; i < cn; i++)
                cs[i] = causes.getLong((cf + i) * 8);
            int pf = recs.getInt(a + 56), pn = recs.getInt(a + 60);
            if (pf < 0 || pn < 0 || (long) (pf + pn) * PARAM_BYTES > params.capacity())
                throw Sectioned.corrupt(path, "event " + id + ": params past the section");
            Map<String, Object> ps = new LinkedHashMap<>();
            for (int i = pf; i < pf + pn; i++) {
                int e = i * PARAM_BYTES;
                String key = s.string(strings, params.getInt(e), "param key");
                long raw = params.getLong(e + 8);
                Object value = switch (params.get(e + 4) & 0xFF) {
                    case 0 -> raw;
                    case 1 -> Double.longBitsToDouble(raw);
                    case 2 -> s.string(strings, (int) raw, "param value");
                    case 3 -> new IdParam(raw);
                    default -> throw Sectioned.corrupt(path, "event " + id + ": param type " + (params.get(e + 4) & 0xFF));
                };
                ps.put(key, value);
            }
            EventRecord ev = new EventRecord(id, recs.getDouble(a + 8), recs.getDouble(a + 16),
                    s.string(strings, recs.getInt(a + 24), "kind"), s.string(strings, recs.getInt(a + 28), "source"),
                    recs.getFloat(a + 32), tag, recs.getInt(a + 40), recs.getInt(a + 44), cs,
                    Collections.unmodifiableMap(ps));
            if ((int) (id >>> 48) != writer)
                throw Sectioned.corrupt(path, "event " + ev.text() + " in a segment of writer " + writer);
            out.add(ev);
        }
        return out;
    }

    /** Every event, merged by (t_start, id). */
    public List<EventRecord> events() {
        return events;
    }

    public int size() {
        return events.size();
    }

    /** The event with {@code id}, or null. */
    public EventRecord get(long id) {
        return byId.get(id);
    }

    /** The events {@code id} caused (reverse links). */
    public List<Long> effectsOf(long id) {
        return effects.getOrDefault(id, List.of());
    }

    /**
     * The causes of {@code id}, breadth first, up to {@code maxDepth} links away, each once; unknown
     * ids are skipped (Rust {@code EventLog::cause_chain}).
     */
    public List<EventRecord> causeChain(long id, int maxDepth) {
        List<EventRecord> out = new ArrayList<>();
        Set<Long> seen = new HashSet<>();
        seen.add(id);
        List<Long> frontier = List.of(id);
        for (int depth = 0; depth < maxDepth; depth++) {
            List<Long> next = new ArrayList<>();
            for (long f : frontier) {
                EventRecord e = byId.get(f);
                if (e == null)
                    continue;
                for (long c : e.causes()) {
                    if (seen.add(c)) {
                        EventRecord ce = byId.get(c);
                        if (ce != null) {
                            out.add(ce);
                            next.add(c);
                        }
                    }
                }
            }
            if (next.isEmpty())
                break;
            frontier = next;
        }
        return out;
    }
}
