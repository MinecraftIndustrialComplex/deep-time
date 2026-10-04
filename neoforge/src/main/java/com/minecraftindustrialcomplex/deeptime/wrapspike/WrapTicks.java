// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

/** Duck interface the spike's LevelTicks mixin adds: scheduled ticks fold to stored positions. */
public interface WrapTicks {
    void deeptime$setWrap(WrapTopology topology);
}
