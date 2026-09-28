// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * Base of the exceptions the native library throws (plan §6.3). Unchecked, because a native
 * method cannot declare the checked exceptions it throws through JNI. Every subclass keeps a
 * public one-{@code String} constructor: that is the one JNI's {@code ThrowNew} calls.
 */
public class DeepTimeNativeException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    public DeepTimeNativeException(String message) {
        super(message);
    }
}
