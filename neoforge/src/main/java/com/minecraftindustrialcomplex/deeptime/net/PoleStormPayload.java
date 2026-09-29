// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/**
 * The pole storm as the server sees it for one player (plan §7.11), sent whenever it changes
 * noticeably ({@code wrap/PoleCap}).
 *
 * @param storm the storm intensity 0…1 the client draws as fog and snow
 * @param push  the wind the client adds to its own movement every tick, blocks/tick², toward the
 *              equator: to the player, or to the vehicle the player steers. The server leaves it 0
 *              for players it does not push (creative, spectator, a passenger of a vehicle the
 *              server moves itself) and applies the hard stop on its own side.
 */
public record PoleStormPayload(float storm, float push) implements CustomPacketPayload {

    public static final Type<PoleStormPayload> TYPE =
            new Type<>(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "pole_storm"));

    public static final StreamCodec<FriendlyByteBuf, PoleStormPayload> STREAM_CODEC =
            StreamCodec.ofMember(PoleStormPayload::write, PoleStormPayload::read);

    public static final PoleStormPayload CALM = new PoleStormPayload(0f, 0f);

    /** Largest push a client accepts (well above {@code PoleCapCurve.PUSH_MAX}): a bad value cannot launch it. */
    public static final float PUSH_LIMIT = 0.5f;

    public PoleStormPayload {
        // A malformed or hostile packet must not break the client: NaN and out-of-range values are clamped.
        storm = Float.isFinite(storm) ? Math.max(0f, Math.min(1f, storm)) : 0f;
        push = Float.isFinite(push) ? Math.max(0f, Math.min(PUSH_LIMIT, push)) : 0f;
    }

    private void write(FriendlyByteBuf buf) {
        buf.writeFloat(storm);
        buf.writeFloat(push);
    }

    private static PoleStormPayload read(FriendlyByteBuf buf) {
        return new PoleStormPayload(buf.readFloat(), buf.readFloat());
    }

    @Override
    public Type<PoleStormPayload> type() {
        return TYPE;
    }
}
