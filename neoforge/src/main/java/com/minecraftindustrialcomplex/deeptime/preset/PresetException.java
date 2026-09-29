// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.preset;

/**
 * A preset, share code or preset argument that cannot be used, with a message a server owner or
 * player can act on. Unchecked: callers turn it into a log block or a screen message.
 */
public final class PresetException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    public PresetException(String message) {
        super(message);
    }

    public PresetException(String message, Throwable cause) {
        super(message, cause);
    }
}
