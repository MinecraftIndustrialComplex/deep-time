// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.simibubi.create.content.kinetics.base.IRotate;
import com.simibubi.create.content.kinetics.base.KineticBlockEntity;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Create's rotation propagation across a turning seam (docs/design/wrap-engine.md, "Create across
 * turning seams"; mixin {@code RotationPropagatorSeamMixin}).
 *
 * <p>Create decides whether two kinetic block entities connect, and with what ratio, in
 * {@code RotationPropagator.getRotationSpeedModifier(from, to)}, from their block states and the
 * difference of their positions. Under folded access the propagator finds a neighbour across a seam
 * (it reads the band position, which answers with the stored block entity), but that block entity's
 * position is its stored position, up to the world's width away, and its state is in its own frame, so
 * Create sees no connection. The adapter runs Create's own method a second time with {@code to} shown
 * as the viewer sees it: at its band position next to {@code from}, its state turned back, and any
 * direction handed to {@code to} turned into its frame. The ratio that comes out is in {@code from}'s
 * frame; multiplying by the axis sign of the turn ({@link SeamTurns}) gives the ratio in {@code to}'s
 * stored frame, which is what Create stores as its speed.
 *
 * <p>Custom connections ({@code propagateRotationTo}: belts, chain conveyors, gantries) are refused
 * across a seam ({@link SeamRefusals.Kind#CUSTOM_KINETIC}): their partners are found through stored
 * absolute positions that a turn does not carry.
 */
public final class KineticSeams {

    /** The pair being evaluated through a seam, or null. Server thread only (Create propagates there). */
    private static View current;

    private KineticSeams() {}

    /** {@code to} as {@code from} sees it through the seam. */
    record View(KineticBlockEntity from, KineticBlockEntity to, SeamFrames.Link link, BlockState state) {}

    /**
     * Wraps {@code getRotationSpeedModifier}: Create's answer where it finds a connection; otherwise,
     * when {@code to} is a neighbour of {@code from} across a seam, Create's answer for {@code to} as
     * seen, times the axis sign of the turn.
     */
    public static float modifier(KineticBlockEntity from, KineticBlockEntity to, Operation<Float> original) {
        Level level = from.getLevel();
        if (current != null || !SeamFrames.active(level))
            return original.call(from, to);
        BlockPos a = from.getBlockPos(), b = to.getBlockPos();
        // Create's own rules assume the two are neighbours (an axis connection checks only that the
        // offset lies along one axis), so its answer for a pair reached through a seam, whose stored
        // positions can be far apart along one axis, is meaningless and must not be used.
        boolean adjacent = Math.max(Math.abs(a.getX() - b.getX()), Math.max(Math.abs(a.getY() - b.getY()), Math.abs(a.getZ() - b.getZ()))) <= 1;
        if (adjacent) {
            float direct = original.call(from, to);
            if (direct != 0)
                return direct;
        }
        SeamFrames.Link link = SeamFrames.link(level, a, b, 1);
        if (link == null)
            return adjacent ? 0 : original.call(from, to);
        BlockState seen = link.view(to.getBlockState());
        if (!(seen.getBlock() instanceof IRotate rotate))
            return 0;
        current = new View(from, to, link, seen);
        float m;
        try {
            m = original.call(from, to);
        } finally {
            current = null;
        }
        if (m == 0)
            return 0;
        return m * SeamTurns.axisSign(link.toStored(), rotate.getRotationAxis(seen).ordinal());
    }

    /** {@code be.getBlockState()} inside the modifier: {@code to}'s state as seen while a view is active. */
    public static BlockState state(KineticBlockEntity be, Operation<BlockState> original) {
        View v = current;
        return v != null && be == v.to() ? v.state() : original.call(be);
    }

    /** {@code be.getBlockPos()} inside the modifier: {@code to}'s band position while a view is active. */
    public static BlockPos pos(KineticBlockEntity be, Operation<BlockPos> original) {
        View v = current;
        return v != null && be == v.to() ? v.link().viewPos() : original.call(be);
    }

    /** {@code getAxisModifier(be, direction)}: a direction handed to {@code to} is turned into its frame. */
    public static float axisModifier(KineticBlockEntity be, Direction direction, Operation<Float> original) {
        View v = current;
        if (v != null && be == v.to())
            return original.call(be, v.link().intoTarget().rotate(direction));
        return original.call(be, direction);
    }

    /** {@code from.propagateRotationTo(to, ...)}: a custom connection across a seam is refused. */
    public static float custom(KineticBlockEntity self, KineticBlockEntity target, BlockState stateFrom, BlockState stateTo,
                               BlockPos diff, boolean axes, boolean cogs, Operation<Float> original) {
        float r = original.call(self, target, stateFrom, stateTo, diff, axes, cogs);
        View v = current;
        if (r != 0 && v != null && target == v.to()) {
            SeamRefusals.refuse(self.getLevel(), self.getBlockPos(), SeamRefusals.Kind.CUSTOM_KINETIC,
                    self.getBlockState().getBlock() + " to " + target.getBlockPos().toShortString());
            return 0;
        }
        return r;
    }

    /**
     * {@code DirectionalShaftHalvesBlockEntity.getSourceFacing()}: when the source is across a seam, the
     * face it is seen through, or null to let Create compute it.
     */
    public static Direction sourceFacing(KineticBlockEntity be) {
        BlockPos source = be.source;
        if (source == null)
            return null;
        BlockPos pos = be.getBlockPos();
        if (pos.distManhattan(source) <= 1)
            return null;
        SeamFrames.Link link = SeamFrames.link(be.getLevel(), pos, source, 1);
        if (link == null || link.viewPos().distManhattan(pos) != 1)
            return null;
        BlockPos d = link.viewPos().subtract(pos);
        return Direction.getNearest(d.getX(), d.getY(), d.getZ());
    }
}
