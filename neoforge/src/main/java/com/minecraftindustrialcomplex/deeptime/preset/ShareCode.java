// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.preset;

import java.io.ByteArrayOutputStream;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Base64;
import java.util.zip.DataFormatException;
import java.util.zip.Deflater;
import java.util.zip.Inflater;

/**
 * Preset share codes (plan §5.1): {@code DT1.} + base64url(deflate(canonical JSON)), encoded and
 * decoded with {@code java.util.zip} only, so a server decodes the code in
 * {@code config/deeptime/server.json} without natives. The Rust twin is
 * {@code deeptime_core::share_code}; {@code crates/deeptime-core/tests/share-codes.json} holds codes
 * each side wrote, and both test suites decode both.
 *
 * <ul>
 *   <li>Canonical JSON: {@link Presets#canonical} (compact, keys sorted, serde_json's number and
 *       string forms), so one preset has one code.</li>
 *   <li>deflate: raw DEFLATE (RFC 1951, no zlib header), level 9.</li>
 *   <li>base64url: RFC 4648 §5 without padding; decoding accepts padding and skips whitespace.</li>
 * </ul>
 */
public final class ShareCode {

    /** The prefix of a version-1 code. */
    public static final String PREFIX = "DT1.";

    /** The largest preset a code may inflate to (a guard against deflate bombs). */
    public static final int MAX_JSON_BYTES = 1 << 20;

    private ShareCode() {}

    /** True for anything shaped like a share code of any version ({@code DT<digits>.}). */
    public static boolean isShareCode(String s) {
        if (s == null)
            return false;
        String t = s.strip();
        if (!t.startsWith("DT"))
            return false;
        int i = 2;
        while (i < t.length() && Character.isDigit(t.charAt(i)) && t.charAt(i) < 128)
            i++;
        return i > 2 && i < t.length() && t.charAt(i) == '.';
    }

    /** Encodes a preset (any JSON text; canonicalised first). */
    public static String encode(String json) {
        byte[] canon = Presets.canonical(json).getBytes(StandardCharsets.UTF_8);
        Deflater d = new Deflater(Deflater.BEST_COMPRESSION, true);
        try {
            d.setInput(canon);
            d.finish();
            ByteArrayOutputStream out = new ByteArrayOutputStream(canon.length / 2 + 64);
            byte[] buf = new byte[8192];
            while (!d.finished()) {
                int n = d.deflate(buf);
                out.write(buf, 0, n);
            }
            return PREFIX + Base64.getUrlEncoder().withoutPadding().encodeToString(out.toByteArray());
        } finally {
            d.end();
        }
    }

    /** Decodes a code into the JSON text it carries (exactly as it was encoded). */
    public static String decode(String code) {
        String c = code == null ? "" : code.strip();
        if (!c.startsWith(PREFIX)) {
            if (isShareCode(c))
                throw error(c.substring(0, c.indexOf('.')) + " codes are not supported by this Deep Time (it reads "
                        + PREFIX.substring(0, PREFIX.length() - 1) + ")");
            throw error("a share code starts with " + PREFIX);
        }
        StringBuilder body = new StringBuilder(c.length());
        for (int i = PREFIX.length(); i < c.length(); i++) {
            char ch = c.charAt(i);
            if (!Character.isWhitespace(ch))
                body.append(ch);
        }
        byte[] deflated;
        try {
            deflated = Base64.getUrlDecoder().decode(body.toString());
        } catch (IllegalArgumentException e) {
            throw error("base64url: " + e.getMessage());
        }
        byte[] json = inflate(deflated);
        String text;
        try {
            text = StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(json)).toString();
        } catch (CharacterCodingException e) {
            throw error("does not hold UTF-8 text");
        }
        try {
            Presets.parseStrict(text);
        } catch (PresetException e) {
            throw error("does not hold JSON: " + e.getMessage());
        }
        return text;
    }

    private static byte[] inflate(byte[] deflated) {
        // zlib may want one byte past a raw stream (Inflater's javadoc on "nowrap").
        Inflater inf = new Inflater(true);
        try {
            inf.setInput(Arrays.copyOf(deflated, deflated.length + 1));
            ByteArrayOutputStream out = new ByteArrayOutputStream(Math.max(64, deflated.length * 4));
            byte[] buf = new byte[8192];
            int stalls = 0;
            while (!inf.finished()) {
                int n = inf.inflate(buf);
                if (n == 0) {
                    if (inf.needsInput() || inf.needsDictionary() || ++stalls > 2)
                        throw error("not a deflate stream: it ends early");
                    continue;
                }
                stalls = 0;
                out.write(buf, 0, n);
                if (out.size() > MAX_JSON_BYTES)
                    throw error("inflates to more than " + MAX_JSON_BYTES + " bytes");
            }
            return out.toByteArray();
        } catch (DataFormatException e) {
            throw error("not a deflate stream: " + e.getMessage());
        } finally {
            inf.end();
        }
    }

    private static PresetException error(String reason) {
        return new PresetException("share code: " + reason);
    }
}
