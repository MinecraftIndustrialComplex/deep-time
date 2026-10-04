// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/**
 * Duck interface the LevelChunk mixin adds: the chunk's place in its level's topology, cached per
 * topology (docs/design/wrap-engine.md §4.2).
 */
public interface WrapChunk {
    /** The motion from this band chunk to its source chunk, or {@code null} when it is not a band chunk. */
    CellIsometry deeptime$bandIso();

    /** The motions from this stored chunk to each band chunk that shows it (empty when none does). */
    CellIsometry[] deeptime$images();
}
