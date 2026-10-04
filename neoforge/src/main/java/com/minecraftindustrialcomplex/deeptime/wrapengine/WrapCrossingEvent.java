// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import net.neoforged.bus.api.Event;
import net.neoforged.bus.api.ICancellableEvent;

/**
 * Posted on {@code NeoForge.EVENT_BUS} on the server thread when the wrap engine is about to carry a
 * root entity (one that rides nothing) across a seam (docs/design/wrap-engine.md §4.5 and §9). Cancel
 * it to handle the crossing yourself (an adapter for an entity that holds its own blocks, such as a
 * Create contraption or a Sable body); the engine then leaves the entity where it is this tick and
 * asks again next tick while it is still beyond the edge.
 *
 * <p>Not cancelled, the engine moves the entity to {@link #to()}, turns its yaw, head and body by
 * {@code iso().yawDegrees()} and its velocity by the turn, does the same for its passengers, and
 * stops a mob's path. A player is sent a teleport.
 */
public final class WrapCrossingEvent extends Event implements ICancellableEvent {

    private final ServerLevel level;
    private final Entity entity;
    private final CellIsometry iso;
    private final Vec3 from;
    private final Vec3 to;

    public WrapCrossingEvent(ServerLevel level, Entity entity, CellIsometry iso, Vec3 from, Vec3 to) {
        this.level = level;
        this.entity = entity;
        this.iso = iso;
        this.from = from;
        this.to = to;
    }

    public ServerLevel level() {
        return level;
    }

    public Entity entity() {
        return entity;
    }

    /** The motion from where the entity is (beyond the edge) to where it is stored. */
    public CellIsometry iso() {
        return iso;
    }

    /** The entity's position beyond the edge. */
    public Vec3 from() {
        return from;
    }

    /** Where the engine will put it. */
    public Vec3 to() {
        return to;
    }
}
