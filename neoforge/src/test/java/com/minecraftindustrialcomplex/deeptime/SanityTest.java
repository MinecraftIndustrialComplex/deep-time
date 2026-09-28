// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class SanityTest {
    @Test
    void modIdIsDeeptime() {
        assertEquals("deeptime", DeepTime.MODID);
    }
}
