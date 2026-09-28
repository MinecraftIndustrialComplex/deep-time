// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * World data could not be read: missing, corrupt, or written by another format version. Unchecked,
 * because tiles decode lazily inside density functions, which cannot throw checked exceptions.
 */
public class WorldDataException extends RuntimeException {
    public WorldDataException(String message) {
        super(message);
    }

    public WorldDataException(String message, Throwable cause) {
        super(message, cause);
    }
}
