// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * A stage would exceed the session's native memory cap (plan §5.7.5, §6.3). The message is
 * {@code needed <n> bytes, cap <m> bytes}.
 */
public class MemoryCapException extends DeepTimeNativeException {

    private static final long serialVersionUID = 1L;

    public MemoryCapException(String message) {
        super(message);
    }
}
