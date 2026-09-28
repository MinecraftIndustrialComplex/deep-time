// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A minimal JSON reader for {@code natives.json}: objects become {@link Map}, arrays {@link List},
 * strings {@link String}, numbers {@link Long} (or {@link Double} with a fraction or exponent),
 * plus {@link Boolean} and {@code null}. {@link NativeLoader} must stay JDK-only because
 * {@code NativeSmoke} runs it on a bare JVM with only the mod jar on the classpath, so no Gson
 * (plan §6.5).
 */
final class MiniJson {

    private final String s;
    private int i;

    private MiniJson(String s) {
        this.s = s;
    }

    static Object parse(String text) {
        MiniJson p = new MiniJson(text);
        p.ws();
        Object v = p.value();
        p.ws();
        if (p.i != p.s.length())
            throw p.error("trailing characters");
        return v;
    }

    private IllegalArgumentException error(String what) {
        return new IllegalArgumentException("natives.json: " + what + " at offset " + i);
    }

    private void ws() {
        while (i < s.length() && Character.isWhitespace(s.charAt(i)))
            i++;
    }

    private char peek() {
        if (i >= s.length())
            throw error("unexpected end");
        return s.charAt(i);
    }

    private void expect(char c) {
        if (peek() != c)
            throw error("expected '" + c + "'");
        i++;
    }

    private Object value() {
        char c = peek();
        switch (c) {
            case '{':
                return object();
            case '[':
                return array();
            case '"':
                return string();
            case 't':
                return literal("true", Boolean.TRUE);
            case 'f':
                return literal("false", Boolean.FALSE);
            case 'n':
                return literal("null", null);
            default:
                if (c == '-' || (c >= '0' && c <= '9'))
                    return number();
                throw error("unexpected '" + c + "'");
        }
    }

    private Object literal(String word, Object v) {
        if (!s.startsWith(word, i))
            throw error("expected " + word);
        i += word.length();
        return v;
    }

    private Map<String, Object> object() {
        Map<String, Object> m = new LinkedHashMap<>();
        expect('{');
        ws();
        if (peek() == '}') {
            i++;
            return m;
        }
        while (true) {
            ws();
            String k = string();
            ws();
            expect(':');
            ws();
            m.put(k, value());
            ws();
            if (peek() == ',') {
                i++;
                continue;
            }
            expect('}');
            return m;
        }
    }

    private List<Object> array() {
        List<Object> l = new ArrayList<>();
        expect('[');
        ws();
        if (peek() == ']') {
            i++;
            return l;
        }
        while (true) {
            ws();
            l.add(value());
            ws();
            if (peek() == ',') {
                i++;
                continue;
            }
            expect(']');
            return l;
        }
    }

    private String string() {
        expect('"');
        StringBuilder b = new StringBuilder();
        while (true) {
            char c = peek();
            i++;
            if (c == '"')
                return b.toString();
            if (c != '\\') {
                b.append(c);
                continue;
            }
            char e = peek();
            i++;
            switch (e) {
                case '"', '\\', '/' -> b.append(e);
                case 'b' -> b.append('\b');
                case 'f' -> b.append('\f');
                case 'n' -> b.append('\n');
                case 'r' -> b.append('\r');
                case 't' -> b.append('\t');
                case 'u' -> {
                    if (i + 4 > s.length())
                        throw error("bad \\u escape");
                    b.append((char) Integer.parseInt(s.substring(i, i + 4), 16));
                    i += 4;
                }
                default -> throw error("bad escape");
            }
        }
    }

    private Object number() {
        int start = i;
        boolean fraction = false;
        while (i < s.length()) {
            char c = s.charAt(i);
            if (c == '.' || c == 'e' || c == 'E')
                fraction = true;
            else if (!(c == '-' || c == '+' || (c >= '0' && c <= '9')))
                break;
            i++;
        }
        String n = s.substring(start, i);
        try {
            return fraction ? (Object) Double.parseDouble(n) : (Object) Long.parseLong(n);
        } catch (NumberFormatException e) {
            throw error("bad number " + n);
        }
    }
}
