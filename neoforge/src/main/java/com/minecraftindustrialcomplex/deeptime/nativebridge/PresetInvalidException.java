// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * The preset or content bundle is malformed (plan §6.3). The message is
 * {@code <field path>: <reason>}, for example {@code $.dummy.steps: missing}.
 */
public class PresetInvalidException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public PresetInvalidException(String message) {
        super(message);
    }

    /** The JSON path of the offending field ({@code $} for the whole document). */
    public String fieldPath() {
        String m = getMessage();
        int i = m == null ? -1 : m.indexOf(": ");
        return i < 0 ? "$" : m.substring(0, i);
    }
}
