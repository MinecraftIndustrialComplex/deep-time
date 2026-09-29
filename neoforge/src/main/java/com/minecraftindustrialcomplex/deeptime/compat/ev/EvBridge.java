// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.ev;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.fml.ModList;

import java.util.List;
import java.util.Optional;

/**
 * Excavated Variants (LGPL-3.0-or-later; plan §7.6, decisions file "Ore blocks") as Deep Time sees
 * it: the ore-in-rock variant of an ore for a host block. Only {@link EvBridgeImpl} names EV's
 * classes; it is loaded reflectively and only when EV is installed, so Deep Time runs without EV.
 * Signatures here use vanilla types only.
 */
public interface EvBridge {

    /** Excavated Variants' mod id. */
    String MOD_ID = "excavated_variants";

    /**
     * The EV variant, in {@code host}'s EV stone, of the first of {@code ores} (the blocks of
     * {@code #c:ores/<material>}, tag order) that EV knows as an ore; null when the host is not an
     * EV stone or EV has no such variant.
     */
    BlockState variant(List<BlockState> ores, BlockState host);

    /** EV's stone id for a host block, if EV knows it. */
    Optional<ResourceLocation> stoneOf(BlockState host);

    /** True when {@code state} is one of EV's generated variant blocks. */
    boolean isVariant(BlockState state);

    /** The EV stone a variant block sits in, if {@code state} is a variant. */
    Optional<ResourceLocation> stoneOfVariant(BlockState state);

    /** A short summary for the log (stones, ores, variants known). */
    String describe();

    /** The bridge when EV is installed and its API answers; empty otherwise (logged). */
    static Optional<EvBridge> load() {
        if (ModList.get() == null || !ModList.get().isLoaded(MOD_ID))
            return Optional.empty();
        try {
            Class<?> impl = Class.forName("com.minecraftindustrialcomplex.deeptime.compat.ev.EvBridgeImpl");
            return Optional.of((EvBridge) impl.getDeclaredMethod("create").invoke(null));
        } catch (Throwable t) {
            DeepTime.LOGGER.error("Excavated Variants is installed but its API did not answer; using #c:ores blocks", t);
            return Optional.empty();
        }
    }
}
