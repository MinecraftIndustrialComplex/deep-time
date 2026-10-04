// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

/** Duck interface the spike's Level mixin adds: the level's wrap topology (null for none). */
public interface WrapLevel {
    WrapTopology deeptime$wrap();

    void deeptime$setWrap(WrapTopology topology);
}
