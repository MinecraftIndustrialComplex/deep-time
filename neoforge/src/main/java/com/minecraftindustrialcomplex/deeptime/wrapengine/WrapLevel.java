// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/** Duck interface the Level mixin adds: the level's wrap topology (null for none) and its band manager. */
public interface WrapLevel {
    WrapTopology deeptime$wrap();

    void deeptime$setWrap(WrapTopology topology);

    BandManager deeptime$bands();

    void deeptime$setBands(BandManager bands);
}
