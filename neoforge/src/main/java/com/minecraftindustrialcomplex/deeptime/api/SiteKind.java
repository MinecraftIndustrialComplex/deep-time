// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import java.util.Locale;

/** What a {@link Site} is (plan §5.6), in the on-disk order of FORMAT.md. */
public enum SiteKind {
    DEPOSIT, RESERVOIR, PLACER, SUPERGENE, FANTASY;

    /** The lower-case name used in JSON and commands. */
    public String id() {
        return name().toLowerCase(Locale.ROOT);
    }

    public static SiteKind ofCode(int code) {
        return values()[code];
    }
}
