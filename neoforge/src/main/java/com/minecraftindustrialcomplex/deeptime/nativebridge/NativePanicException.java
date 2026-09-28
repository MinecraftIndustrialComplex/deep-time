// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * Native code panicked (plan §6.3). The message is the panic message followed by
 * {@code (at <file>:<line>:<column>)} when the location is known. The session the call was on is
 * poisoned; other sessions and the JVM carry on.
 */
public class NativePanicException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public NativePanicException(String message) {
        super(message);
    }
}
