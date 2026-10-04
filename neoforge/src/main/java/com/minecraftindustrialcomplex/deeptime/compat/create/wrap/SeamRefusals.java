// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.Level;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Connections a turning seam refuses until the wrap engine can carry them (docs/design/wrap-engine.md,
 * "Create across turning seams"): the connection is not made, the server logs one line per place and
 * kind, and players within {@value #TELL_RADIUS} blocks see the reason on their action bar. Nothing
 * throws. The kinds are lang keys under {@code deeptime.wrap.create.refused.}.
 */
public final class SeamRefusals {

    /** Kinds of refused connection; each is a lang key suffix. */
    public enum Kind {
        /** A belt, chain conveyor or gantry joined to its partner through a seam. */
        CUSTOM_KINETIC("custom_kinetic"),
        /** A belt placed so that it would cross or touch a seam. */
        BELT("belt"),
        /** A chain conveyor connection that would cross a seam. */
        CHAIN_CONVEYOR("chain_conveyor"),
        /** A train track (straight run or curve) that would cross a seam. */
        TRACK("track"),
        /** A moving structure whose blocks reach beyond a seam. */
        CONTRAPTION("contraption");

        public final String key;

        Kind(String id) {
            this.key = "deeptime.wrap.create.refused." + id;
        }
    }

    static final int TELL_RADIUS = 16;
    /** Ticks before the same place and kind is reported again. */
    static final int REPEAT_TICKS = 200;

    private static final Map<String, Long> LAST = new HashMap<>();
    private static final List<String> RECORD = Collections.synchronizedList(new ArrayList<>());

    private SeamRefusals() {}

    /** Records a refusal at {@code pos}: one log line and an action-bar message per place and kind. */
    public static void refuse(Level level, BlockPos pos, Kind kind, String detail) {
        String key = kind.name() + "@" + pos.toShortString();
        RECORD.add(key);
        if (!(level instanceof ServerLevel sl))
            return;
        long now = sl.getGameTime();
        synchronized (LAST) {
            Long last = LAST.get(key);
            if (last != null && now - last < REPEAT_TICKS)
                return;
            LAST.put(key, now);
        }
        DeepTime.LOGGER.warn("[wrap/create] refused across a turning seam: {} at {} ({})", kind.name().toLowerCase(), pos.toShortString(), detail);
        Component msg = Component.translatable(kind.key);
        for (ServerPlayer p : sl.players())
            if (p.blockPosition().closerThan(pos, TELL_RADIUS))
                p.displayClientMessage(msg, true);
    }

    /** How many refusals of a kind were recorded so far (tests). */
    public static int count(Kind kind) {
        synchronized (RECORD) {
            return (int) RECORD.stream().filter(k -> k.startsWith(kind.name() + "@")).count();
        }
    }
}
