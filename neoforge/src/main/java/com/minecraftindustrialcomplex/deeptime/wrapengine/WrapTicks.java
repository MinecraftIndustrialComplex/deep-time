// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/** Duck interface the LevelTicks mixin adds: scheduled ticks fold to stored positions. */
public interface WrapTicks {
    void deeptime$setWrap(WrapTopology topology);
}
