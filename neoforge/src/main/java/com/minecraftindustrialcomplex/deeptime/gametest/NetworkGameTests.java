// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.net.DeepTimeNetwork;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.net.PoleStormPayload;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.worlddata.FormatVersion;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import io.netty.buffer.Unpooled;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.Level;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;
import net.neoforged.neoforge.network.negotiation.NegotiableNetworkComponent;
import net.neoforged.neoforge.network.negotiation.NegotiationResult;
import net.neoforged.neoforge.network.negotiation.NetworkComponentNegotiator;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * Deep Time's network channels (plan §7.15, P1-18) on the GameTest server. The planet server
 * enables this namespace too, so {@code gametest-planet.sh --run worldinfochannelrequired@…}
 * checks the channel there as well; {@code --scenario refuse_client} then refuses real
 * connections over TCP.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class NetworkGameTests {

    private NetworkGameTests() {}

    private static <T> T roundTrip(StreamCodec<FriendlyByteBuf, T> codec, T value, GameTestHelper helper) {
        FriendlyByteBuf buf = new FriendlyByteBuf(Unpooled.buffer());
        try {
            codec.encode(buf, value);
            int bytes = buf.readableBytes();
            T back = codec.decode(buf);
            helper.assertTrue(buf.readableBytes() == 0, "codec left " + buf.readableBytes() + " of " + bytes + " bytes unread");
            return back;
        } finally {
            buf.release();
        }
    }

    /** WorldInfoPayload survives encode → decode, overlong strings are cut, not refused. */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void worldInfoRoundTrip(GameTestHelper helper) {
        List<WorldInfoPayload> cases = List.of(
                new WorldInfoPayload(true, "minecraft:overworld", "0.0", 32768, 300.0, 20.0, 75f, 80f, "earthlike_quick_32k"),
                new WorldInfoPayload(true, "deeptime:test_planet", "0.0", 1024, 123.456789, 7.25, 60.5f, 89.75f, "Ærde — 地球 ✓"),
                WorldInfoPayload.notAPlanet("minecraft:the_nether"));
        for (WorldInfoPayload p : cases) {
            WorldInfoPayload back = roundTrip(WorldInfoPayload.STREAM_CODEC, p, helper);
            helper.assertTrue(p.equals(back), "round trip changed " + p + " into " + back);
        }
        String longName = "x".repeat(WorldInfoPayload.MAX_STRING + 50);
        WorldInfoPayload cut = roundTrip(WorldInfoPayload.STREAM_CODEC,
                new WorldInfoPayload(true, "minecraft:overworld", "0.0", 16384, 300, 20, 75, 80, longName), helper);
        helper.assertTrue(cut.planetName().length() == WorldInfoPayload.MAX_STRING, "long name not cut: " + cut.planetName().length());
        helper.assertTrue(cut.poleCap() != null && cut.poleCap().stopZ() > 3640 && cut.poleCap().stopZ() < 3642,
                "poleCap() of a 16k planet: " + cut.poleCap());
        helper.assertTrue(WorldInfoPayload.notAPlanet("minecraft:the_end").poleCap() == null, "a non-planet has a cap");
        DeepTime.LOGGER.info("[gametest] worldInfoRoundTrip {} cases ok, id {}", cases.size() + 1, WorldInfoPayload.TYPE.id());
        helper.succeed();
    }

    /** PoleStormPayload survives encode → decode; NaN and out-of-range values are clamped on arrival. */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void poleStormRoundTrip(GameTestHelper helper) {
        for (PoleStormPayload p : List.of(PoleStormPayload.CALM, new PoleStormPayload(0.6156f, 0.0354f), new PoleStormPayload(1f, 0.1f))) {
            PoleStormPayload back = roundTrip(PoleStormPayload.STREAM_CODEC, p, helper);
            helper.assertTrue(p.equals(back), "round trip changed " + p + " into " + back);
        }
        // What a hostile or broken server could send: raw floats, decoded through the codec.
        FriendlyByteBuf buf = new FriendlyByteBuf(Unpooled.buffer());
        try {
            buf.writeFloat(Float.NaN);
            buf.writeFloat(1e9f);
            PoleStormPayload bad = PoleStormPayload.STREAM_CODEC.decode(buf);
            helper.assertTrue(bad.storm() == 0f && bad.push() == PoleStormPayload.PUSH_LIMIT, "not clamped: " + bad);
            buf.clear();
            buf.writeFloat(7f);
            buf.writeFloat(-3f);
            bad = PoleStormPayload.STREAM_CODEC.decode(buf);
            helper.assertTrue(bad.storm() == 1f && bad.push() == 0f, "not clamped: " + bad);
        } finally {
            buf.release();
        }
        DeepTime.LOGGER.info("[gametest] poleStormRoundTrip ok, id {}", PoleStormPayload.TYPE.id());
        helper.succeed();
    }

    /**
     * Deep Time's channels are registered as NeoForge holds them (read from NeoForge's registry):
     * play phase, clientbound, version {@value DeepTimeNetwork#VERSION}, and <b>not optional</b>.
     * Fed to NeoForge's own negotiator with the server's channel list: a client offering every
     * channel but Deep Time's is refused, naming exactly Deep Time's channels; a client offering
     * all of them is accepted; a client with none (vanilla, or NeoForge without the mods) is refused.
     */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void worldInfoChannelRequired(GameTestHelper helper) {
        List<DeepTimeNetwork.Channel> all = DeepTimeNetwork.registrations();
        Set<ResourceLocation> ours = DeepTimeNetwork.TYPES.stream().map(t -> t.id()).collect(Collectors.toSet());
        StringBuilder d = new StringBuilder();
        for (ResourceLocation id : ours) {
            DeepTimeNetwork.Channel c = all.stream().filter(ch -> ch.id().equals(id)).findFirst().orElse(null);
            helper.assertTrue(c != null, id + " is not registered");
            helper.assertTrue(c.protocol() == ConnectionProtocol.PLAY, id + " is registered for " + c.protocol());
            helper.assertTrue(!c.optional(), id + " is OPTIONAL: a client without Deep Time could join");
            helper.assertTrue(c.flow().equals(Optional.of(PacketFlow.CLIENTBOUND)), id + " flow " + c.flow());
            helper.assertTrue(DeepTimeNetwork.VERSION.equals(c.version()), id + " version " + c.version());
            d.append(' ').append(id).append("(required,").append(c.version()).append(')');
        }
        long deeptimeChannels = all.stream().filter(c -> DeepTimeNetwork.ours(c.id())).count();
        helper.assertTrue(deeptimeChannels == ours.size(), "unexpected Deep Time channels: " + all);

        for (ConnectionProtocol protocol : List.of(ConnectionProtocol.CONFIGURATION, ConnectionProtocol.PLAY)) {
            List<NegotiableNetworkComponent> server = new ArrayList<>();
            for (DeepTimeNetwork.Channel c : all)
                if (c.protocol() == protocol)
                    server.add(new NegotiableNetworkComponent(c.id(), c.version(), c.flow(), c.optional()));
            List<NegotiableNetworkComponent> withoutUs = server.stream().filter(c -> !DeepTimeNetwork.ours(c.id())).toList();

            NegotiationResult full = NetworkComponentNegotiator.negotiate(server, server);
            helper.assertTrue(full.success(), protocol + ": a client with every channel was refused: " + full.failureReasons());

            NegotiationResult lacking = NetworkComponentNegotiator.negotiate(server, withoutUs);
            Set<ResourceLocation> expected = protocol == ConnectionProtocol.PLAY ? ours : Set.of();
            helper.assertTrue(lacking.success() == expected.isEmpty(),
                    protocol + ": a client without Deep Time " + (lacking.success() ? "was accepted" : "was refused"));
            helper.assertTrue(lacking.failureReasons().keySet().equals(expected),
                    protocol + ": refusal names " + lacking.failureReasons().keySet() + ", expected " + expected);

            NegotiationResult none = NetworkComponentNegotiator.negotiate(server, List.of());
            if (protocol == ConnectionProtocol.PLAY)
                helper.assertTrue(!none.success() && none.failureReasons().keySet().containsAll(ours),
                        "PLAY: a client with no channels was not refused for Deep Time's: " + none.failureReasons().keySet());
            d.append(String.format(" %s: server=%d full=%s withoutDeepTime=%s%s none=%s", protocol, server.size(), full.success(),
                    lacking.success(), lacking.failureReasons().keySet(), none.success()));
        }
        DeepTime.LOGGER.info("[gametest] worldInfoChannelRequired channels={}{}", all.size(), d);
        helper.succeed();
    }

    /** The F3 line's text (the client adds it to the debug screen). */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void planetDebugLine(GameTestHelper helper) {
        WorldInfoPayload info = new WorldInfoPayload(true, "minecraft:overworld", "0.0", 16384, 300, 20, 75, 80, "wrap-test");
        String north = PlanetDebugLine.format(info, 4096, -3500, 0f);
        String south = PlanetDebugLine.format(info, -8192, 3600, 0.62f);
        helper.assertTrue("Deep Time: 76.904° N 90.000° E · wrap-test (C 16384, 300 m/block)".equals(north), "north: " + north);
        helper.assertTrue("Deep Time: 79.102° S 180.000° W · wrap-test (C 16384, 300 m/block) · polar storm 62%".equals(south), "south: " + south);
        helper.assertTrue(PlanetDebugLine.format(WorldInfoPayload.notAPlanet("minecraft:the_nether"), 0, 0, 0) == null, "a line outside a planet");
        DeepTime.LOGGER.info("[gametest] planetDebugLine '{}' / '{}'", north, south);
        helper.succeed();
    }

    /**
     * A player gets the world info on login and again on every dimension change (plan §7.15): a
     * mock player logs into the flat overworld (not a planet), is moved to the analytic test planet
     * (C = 16,384, the default 75°/80° cap), to the world-data test planet (C = 1,024, its manifest's
     * scales and name) and back. Its own batch: a logged-in player must not disturb other tests.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, batch = "network_player", timeoutTicks = 200)
    public static void worldInfoFollowsThePlayer(GameTestHelper helper) {
        ServerLevel overworld = helper.getLevel().getServer().overworld();
        ServerLevel planet = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET);
        ServerLevel data = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_DATA);
        helper.assertTrue(planet != null && data != null, "the test planets are missing");
        MockPlayers.Mock m = MockPlayers.create(overworld, GameType.SURVIVAL);
        try {
            WorldInfoPayload atLogin = m.last(WorldInfoPayload.class);
            helper.assertTrue(atLogin != null, "no world info at login");
            helper.assertTrue(!atLogin.planet() && atLogin.dimension().equals(Level.OVERWORLD.location().toString()),
                    "login into the flat overworld: " + atLogin);

            m.forgetSent();
            m.player.teleportTo(planet, 0.5, 200, 0.5, 0, 0);
            WorldInfoPayload onPlanet = m.last(WorldInfoPayload.class);
            helper.assertTrue(onPlanet != null, "no world info after moving to the test planet");
            helper.assertTrue(onPlanet.planet() && onPlanet.circumference() == 16384 && onPlanet.poleCapDeg() == 75f
                            && onPlanet.poleStopDeg() == 80f && onPlanet.dimension().equals(GameTestSupport.TEST_PLANET.location().toString())
                            && onPlanet.formatVersion().equals(FormatVersion.CURRENT) && onPlanet.planetName().startsWith("wrap-test"),
                    "test planet: " + onPlanet);

            m.forgetSent();
            m.player.teleportTo(data, 0.5, 200, 0.5, 0, 0);
            WorldInfoPayload onData = m.last(WorldInfoPayload.class);
            helper.assertTrue(onData != null && onData.planet() && onData.circumference() == 1024 && onData.mPerBlockH() == 300.0
                            && onData.mPerBlockV() == Manifest.DEFAULT_M_PER_BLOCK_V && Objects.equals(onData.planetName(), "wrap-test"),
                    "world-data planet: " + onData);

            m.forgetSent();
            m.player.teleportTo(overworld, 0.5, 100, 0.5, 0, 0);
            WorldInfoPayload back = m.last(WorldInfoPayload.class);
            helper.assertTrue(back != null && !back.planet(), "back in the overworld: " + back);
            DeepTime.LOGGER.info("[gametest] worldInfoFollowsThePlayer login={} planet={} data={} back={}", atLogin, onPlanet, onData, back);
        } finally {
            m.remove();
        }
        helper.succeed();
    }
}
