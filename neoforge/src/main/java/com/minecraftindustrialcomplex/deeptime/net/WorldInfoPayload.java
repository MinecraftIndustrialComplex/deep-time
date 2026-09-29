// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.FormatVersion;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;

/**
 * What the client knows about the planet it is on (plan §7.15), sent when a player logs in, changes
 * dimension or respawns: the pole storm, the F3 latitude/longitude line and, later, maps read it.
 * A player in a level that is not a Deep Time planet gets {@code planet = false} (the Nether, the
 * End, a vanilla overworld), so the client never shows stale numbers.
 *
 * @param planet        the player's level is a Deep Time planet
 * @param dimension     that level's id, so the client can ignore info for a level it has left
 * @param formatVersion the world-data format version ({@link FormatVersion#CURRENT} for test terrains)
 * @param circumference C in blocks (0 when {@code planet} is false)
 * @param mPerBlockH    horizontal scale, metres per block (plan §5.1)
 * @param mPerBlockV    vertical scale, metres per block
 * @param poleCapDeg    where the pole storm starts
 * @param poleStopDeg   the hard stop
 * @param planetName    the preset or synthetic kind the world came from
 */
public record WorldInfoPayload(boolean planet, String dimension, String formatVersion, int circumference,
                               double mPerBlockH, double mPerBlockV, float poleCapDeg, float poleStopDeg,
                               String planetName) implements CustomPacketPayload {

    public static final Type<WorldInfoPayload> TYPE =
            new Type<>(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "world_info"));

    /** Longest string the codec carries; longer names are cut (a name is for display only). */
    public static final int MAX_STRING = 256;

    public static final StreamCodec<FriendlyByteBuf, WorldInfoPayload> STREAM_CODEC =
            StreamCodec.ofMember(WorldInfoPayload::write, WorldInfoPayload::read);

    public WorldInfoPayload {
        dimension = cut(dimension);
        formatVersion = cut(formatVersion);
        planetName = cut(planetName);
    }

    private static String cut(String s) {
        if (s == null)
            return "";
        return s.length() <= MAX_STRING ? s : s.substring(0, MAX_STRING);
    }

    private void write(FriendlyByteBuf buf) {
        buf.writeBoolean(planet);
        buf.writeUtf(dimension, MAX_STRING);
        buf.writeUtf(formatVersion, MAX_STRING);
        buf.writeVarInt(circumference);
        buf.writeDouble(mPerBlockH);
        buf.writeDouble(mPerBlockV);
        buf.writeFloat(poleCapDeg);
        buf.writeFloat(poleStopDeg);
        buf.writeUtf(planetName, MAX_STRING);
    }

    private static WorldInfoPayload read(FriendlyByteBuf buf) {
        return new WorldInfoPayload(buf.readBoolean(), buf.readUtf(MAX_STRING), buf.readUtf(MAX_STRING), buf.readVarInt(),
                buf.readDouble(), buf.readDouble(), buf.readFloat(), buf.readFloat(), buf.readUtf(MAX_STRING));
    }

    @Override
    public Type<WorldInfoPayload> type() {
        return TYPE;
    }

    /** The pole cap this info describes, or null when it is not a planet or the numbers are unusable. */
    public PoleCapCurve poleCap() {
        if (!planet || circumference <= 0 || !(poleCapDeg > 0 && poleCapDeg < poleStopDeg && poleStopDeg <= 90))
            return null;
        return new PoleCapCurve(circumference, poleCapDeg, poleStopDeg);
    }

    /** The info for a level that is not a planet. */
    public static WorldInfoPayload notAPlanet(String dimension) {
        return new WorldInfoPayload(false, dimension, FormatVersion.CURRENT, 0, 0, 0, 0, 0, "");
    }

    /** The info for {@code level}: its generator's world-data reference plus the bound manifest. */
    public static WorldInfoPayload forLevel(ServerLevel level) {
        String dim = level.dimension().location().toString();
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g))
            return notAPlanet(dim);
        WorldDataRef ref = g.dataRef();
        String format = FormatVersion.CURRENT;
        double h = Manifest.DEFAULT_M_PER_BLOCK_H, v = Manifest.DEFAULT_M_PER_BLOCK_V;
        String name;
        var testTerrain = ref.effectiveTestTerrain();
        if (testTerrain.isPresent()) {
            name = testTerrain.get() + " (test terrain)";
        } else if (ActiveWorldData.has(ref.folder())) {
            Manifest m = ActiveWorldData.get(ref.folder()).manifest();
            format = m.formatVersion();
            h = m.mPerBlockH();
            v = m.mPerBlockV();
            name = m.planetName();
        } else {
            name = ref.folder();
        }
        return new WorldInfoPayload(true, dim, format, ref.circumferenceBlocks(), h, v,
                (float) ref.poleCapDeg(), (float) ref.poleStopDeg(), name);
    }
}
