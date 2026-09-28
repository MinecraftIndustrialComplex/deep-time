// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/** A native file operation failed; the message is {@code <path>: <error>} (plan §6.3). */
public class DeepTimeIoException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public DeepTimeIoException(String message) {
        super(message);
    }
}
