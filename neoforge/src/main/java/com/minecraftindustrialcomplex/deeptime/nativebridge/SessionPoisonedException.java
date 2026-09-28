// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * A call on a session whose earlier call panicked (plan §6.3). Only {@code destroy} and
 * {@code drainLog} still work on a poisoned session.
 */
public class SessionPoisonedException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public SessionPoisonedException(String message) {
        super(message);
    }
}
