// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.FormatVersion;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;

/**
 * What the client knows about the planet it is on (plan §7.15), sent when a player logs in, changes
 * dimension or respawns: the F3 latitude/longitude line, Distant Horizons' curvature and, later, maps
 * read it.
 * A player in a level that is not a Deep Time planet gets {@code planet = false} (the Nether, the
 * End, a vanilla overworld), so the client never shows stale numbers.
 *
 * @param planet        the player's level is a Deep Time planet
 * @param dimension     that level's id, so the client can ignore info for a level it has left
 * @param formatVersion the world-data format version ({@link FormatVersion#CURRENT} for test terrains)
 * @param circumference C in blocks (0 when {@code planet} is false)
 * @param mPerBlockH    horizontal scale, metres per block (plan §5.1)
 * @param mPerBlockV    vertical scale, metres per block
 * @param planetName    the preset or synthetic kind the world came from
 * @param projection    the world's projection, {@code cube} (format 0.7; {@code circumference} is the
 *                      period 4s) or {@code lambert} (the Lambert-frame test worlds and test terrains)
 * @param cubeMap       a cube world's map table by name (its manifest's {@code map.name}); "" otherwise
 * @param cubeMapSha256 that table's sha256 (lowercase hex): the client draws latitude and longitude only
 *                      when its jar ships the same table ("" otherwise)
 * @param cubeTurn      a cube world's rotation planet to cube, its 9 numbers row by row; empty otherwise
 */
public record WorldInfoPayload(boolean planet, String dimension, String formatVersion, int circumference,
                               double mPerBlockH, double mPerBlockV, String planetName, String projection,
                               String cubeMap, String cubeMapSha256, java.util.List<Double> cubeTurn)
        implements CustomPacketPayload {

    /** A Lambert-frame planet's info (the tests and the analytic test terrains: no cube fields). */
    public WorldInfoPayload(boolean planet, String dimension, String formatVersion, int circumference, double mPerBlockH,
                            double mPerBlockV, String planetName) {
        this(planet, dimension, formatVersion, circumference, mPerBlockH, mPerBlockV, planetName, WorldDataRef.LAMBERT,
                "", "", java.util.List.of());
    }

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
        projection = WorldDataRef.CUBE.equals(projection) ? WorldDataRef.CUBE : WorldDataRef.LAMBERT;
        cubeMap = cut(cubeMap);
        cubeMapSha256 = cut(cubeMapSha256);
        cubeTurn = cubeTurn == null || cubeTurn.size() != 9 ? java.util.List.of() : java.util.List.copyOf(cubeTurn);
    }

    /** Whether the planet is a cube world (format 0.7). */
    public boolean cube() {
        return WorldDataRef.CUBE.equals(projection);
    }

    /**
     * The world projection this info describes (a new object: {@link ClientPlanetState} keeps one), or
     * null when it is not a planet.
     */
    public com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection worldProjection() {
        if (!planet || circumference <= 0)
            return null;
        if (cube())
            return cubeProjection();
        return new com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert(circumference);
    }

    /**
     * A cube world's projection from the cube fields, or null when they are missing, the turn is not a
     * rotation, or this jar does not ship the server's map table by name and sha256 (the F3 line then
     * says so rather than show wrong numbers).
     */
    private com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection cubeProjection() {
        if (cubeTurn.size() != 9 || cubeMap.isEmpty() || circumference % 4 != 0)
            return null;
        double[][] rows = new double[3][3];
        for (int i = 0; i < 9; i++)
            rows[i / 3][i % 3] = cubeTurn.get(i);
        try {
            var cube = com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.shared(circumference / 4);
            var map = com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Cube.mapNamed(cubeMap, cubeMapSha256);
            var turn = com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn.fromRows(rows);
            return new com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Cube(
                    new com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld(cube, map, turn, 1.0));
        } catch (com.minecraftindustrialcomplex.deeptime.worlddata.WorldDataException e) {
            return null;
        }
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
        buf.writeUtf(planetName, MAX_STRING);
        buf.writeUtf(projection, MAX_STRING);
        buf.writeUtf(cubeMap, MAX_STRING);
        buf.writeUtf(cubeMapSha256, MAX_STRING);
        buf.writeVarInt(cubeTurn.size());
        for (double d : cubeTurn)
            buf.writeDouble(d);
    }

    private static WorldInfoPayload read(FriendlyByteBuf buf) {
        boolean planet = buf.readBoolean();
        String dimension = buf.readUtf(MAX_STRING), format = buf.readUtf(MAX_STRING);
        int c = buf.readVarInt();
        double h = buf.readDouble(), v = buf.readDouble();
        String name = buf.readUtf(MAX_STRING), projection = buf.readUtf(MAX_STRING);
        String map = buf.readUtf(MAX_STRING), sha = buf.readUtf(MAX_STRING);
        int n = buf.readVarInt();
        if (n != 0 && n != 9)
            throw new IllegalArgumentException("world info: a cube turn of " + n + " numbers");
        java.util.List<Double> turn = new java.util.ArrayList<>(n);
        for (int i = 0; i < n; i++)
            turn.add(buf.readDouble());
        return new WorldInfoPayload(planet, dimension, format, c, h, v, name, projection, map, sha, turn);
    }

    @Override
    public Type<WorldInfoPayload> type() {
        return TYPE;
    }

    /** The info for a level that is not a planet. */
    public static WorldInfoPayload notAPlanet(String dimension) {
        return new WorldInfoPayload(false, dimension, FormatVersion.CURRENT, 0, 0, 0, "");
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
        String cubeMap = "", cubeSha = "";
        java.util.List<Double> cubeTurn = java.util.List.of();
        var testTerrain = ref.effectiveTestTerrain();
        if (testTerrain.isPresent()) {
            name = testTerrain.get() + " (test terrain)";
        } else if (ActiveWorldData.has(ref.folder())) {
            Manifest m = ActiveWorldData.get(ref.folder()).manifest();
            format = m.formatVersion();
            h = m.mPerBlockH();
            v = m.mPerBlockV();
            name = m.planetName();
            if (m.layout() != null && m.layout().cube() != null) {
                var spec = m.layout().cube();
                cubeMap = spec.mapName();
                cubeSha = spec.mapSha256();
                java.util.List<Double> t = new java.util.ArrayList<>(9);
                for (double[] row : spec.rotation())
                    for (double d : row)
                        t.add(d);
                cubeTurn = t;
            }
        } else {
            name = ref.folder();
        }
        return new WorldInfoPayload(true, dim, format, ref.circumferenceBlocks(), h, v, name, ref.projection(), cubeMap, cubeSha,
                cubeTurn);
    }
}
