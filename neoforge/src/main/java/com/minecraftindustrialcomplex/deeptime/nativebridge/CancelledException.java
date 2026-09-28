// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * The session was cancelled (plan §6.3): thrown by calls that need a finished run
 * ({@code finalizeWrite}), and the exception a cancelled {@link SimSession#completion()} completes
 * with.
 */
public class CancelledException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public CancelledException(String message) {
        super(message);
    }
}
