// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/**
 * Duck interface the LevelChunk mixin adds: the chunk's place in its level's topology, cached per
 * topology (docs/design/wrap-engine.md §4.2).
 */
public interface WrapChunk {
    /**
     * The motion from this band chunk's first block to its source, or {@code null} when it is not a
     * band chunk. A split chunk ({@link #deeptime$bandSplit}) folds other blocks by other motions.
     */
    CellIsometry deeptime$bandIso();

    /** True when this band chunk folds by more than one motion (on two bands' diagonal, design §12.2). */
    boolean deeptime$bandSplit();

    /** The motions from this stored chunk to each band chunk that shows some of it (empty when none does). */
    CellIsometry[] deeptime$images();
}
