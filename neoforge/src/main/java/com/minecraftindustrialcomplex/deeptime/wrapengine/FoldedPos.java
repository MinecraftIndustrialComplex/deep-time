// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Rotation;

/**
 * The stored position of a band position on the read path of {@code Level.getBlockState}, carrying the
 * turn that takes the stored state back into the band's frame (docs/design/wrap-engine.md §5).
 *
 * <p>The read path folds the position at the method's head (a {@code @ModifyVariable}) and turns the
 * result at its return (a {@code @ModifyReturnValue} that tests {@code instanceof FoldedPos}), so the
 * common path allocates nothing and no callback object is made. Only the fold of a band position
 * allocates this. It never leaves {@code getBlockState}: callers get a plain state back.
 */
public final class FoldedPos extends BlockPos {

    /** The turn from the stored frame back to the queried (band) frame. */
    public final Rotation back;

    FoldedPos(int x, int y, int z, Rotation back) {
        super(x, y, z);
        this.back = back;
    }

    /** {@code pos} folded for a read, or {@code pos} itself when it does not fold. */
    public static BlockPos forRead(WrapTopology t, BlockPos pos) {
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return pos;
        return new FoldedPos(f.x(), pos.getY(), f.z(), WrapEngine.rotation(f.iso().inverse()));
    }
}
