// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.ChunkHolderAccessor;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.Kind;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.Site;
import io.netty.channel.embedded.EmbeddedChannel;
import it.unimi.dsi.fastutil.shorts.ShortSet;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtUtils;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.BundlePacket;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket;
import net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.FilteredText;
import net.minecraft.util.Mth;
import net.minecraft.world.inventory.ChestMenu;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SignBlock;
import net.minecraft.world.level.block.StandingSignBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.entity.ChestBlockEntity;
import net.minecraft.world.level.block.entity.SignBlockEntity;
import net.minecraft.world.level.block.entity.SignText;
import net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;
import org.jetbrains.annotations.Nullable;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import static com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.check;
import static com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.log;
import static com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.open;
import static com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.site;
import static com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.whenSynced;

/**
 * E2 of the wrap engine (docs/design/wrap-engine.md §12.8): block-entity data in the band. A chest,
 * signs, an end gateway and a suspicious sand block are placed through the band (so they are stored on
 * the far side, turned), and then:
 * <ol>
 *   <li>the band chunk's packet lists each of them once, at its band position, with its position-bearing
 *       tags turned (the gateway's exit, the sand's hit direction), and the band chunk itself holds
 *       none;</li>
 *   <li>a band holder's data packet is the stored entity's, re-addressed and turned;</li>
 *   <li>a change to a stored entity's data is marked at its band position too;</li>
 *   <li>a player at the band image can use the chest (vanilla's {@code stillValid} keeps the menu open
 *       across ticks) while a chest out of reach stays out of reach;</li>
 *   <li>the sign editor opens on the image, the edit stays allowed across the sign's ticks, and the edit
 *       addressed to the image reaches the stored sign;</li>
 *   <li>which side of a sign a player edits is judged as the image shows it.</li>
 * </ol>
 * Run on a plain half-turn seam, a plain quarter-turn corner and a quarter-turn seam of the cube. The
 * expectations come from the band frame (where the player sees the block), never from pinned
 * coordinates. Namespace {@code deeptime_wrapengine}, enabled only by {@code ./gametest.sh -PwrapEngine}.
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class WrapBlockEntityGameTests {

    private static final String NS = WrapEngineGameTests.NS;
    private static final String TEMPLATE = WrapEngineGameTests.TEMPLATE;
    private static final int H = WrapEngineGameTests.CUBE_H;

    /**
     * F3-top, 32 to 80 blocks from the concave corner (−h, −h) where F3, F0 and N meet: the band shows
     * N's west strip, a quarter turn ({@link WrapEngineGameTests#CUBE_QUARTER} mirrored).
     */
    static final Site CUBE_BLOCK_ENTITIES = WrapEngineGameTests.cubeSite("cube-block-entities", Kind.QUARTER, -H - 80, -H, 0, 48,
            new int[]{-H - 16, -H + 32, -H - 96, -H - 16});

    /** Where along the edge each block goes (band positions at v = −2); signs face 0, 4, 8, 12 in the band. */
    private static final int SAND = 2, GATEWAY = 6, CHEST = 10;
    private static final int[] SIGNS = {16, 20, 24, 28};
    private static final String TEXT = "deep time", EDITED = "edited through the band";

    private WrapBlockEntityGameTests() {}

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void blockEntitiesAcrossHalfTurn(GameTestHelper helper) {
        blockEntities(helper, site(Kind.HALF, WrapEngineGameTests.BLOCK_ENTITIES));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void blockEntitiesAcrossQuarterTurn(GameTestHelper helper) {
        blockEntities(helper, site(Kind.QUARTER, WrapEngineGameTests.BLOCK_ENTITIES));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void cubeBlockEntitiesAcrossQuarterTurn(GameTestHelper helper) {
        blockEntities(helper, CUBE_BLOCK_ENTITIES);
    }

    /** One block entity in a band chunk's packet. */
    record Entry(BlockEntityType<?> type, @Nullable CompoundTag tag) {}

    @SuppressWarnings("removal")
    static void blockEntities(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        BlockPos sand = s.at(SAND, 0, -2), gateway = s.at(GATEWAY, 0, -2), chest = s.at(CHEST, 0, -2);
        BlockPos control = s.at(CHEST, 0, 15);
        BlockPos[] signs = new BlockPos[SIGNS.length];
        List<BlockPos> bands = new ArrayList<>(List.of(sand, gateway, chest));
        for (int i = 0; i < SIGNS.length; i++)
            bands.add(signs[i] = s.at(SIGNS[i], 0, -2));
        for (BlockPos b : bands) {
            BlockPos st = s.stored(b);
            check(!st.equals(b) && s.inBoxes(st.getX(), st.getZ()), "%s: band %s is stored at %s, inside the site's boxes", s.name(), b.toShortString(),
                    st.toShortString());
        }

        whenSynced(helper, s, () -> {
            // Placed through the band: each lands on its stored block, turned, with its block entity there.
            level.setBlock(chest, Blocks.CHEST.defaultBlockState(), Block.UPDATE_ALL);
            for (int i = 0; i < SIGNS.length; i++)
                level.setBlock(signs[i], Blocks.OAK_SIGN.defaultBlockState().setValue(StandingSignBlock.ROTATION, 4 * i), Block.UPDATE_ALL);
            level.setBlock(gateway, Blocks.END_GATEWAY.defaultBlockState(), Block.UPDATE_ALL);
            level.setBlock(sand, Blocks.SUSPICIOUS_SAND.defaultBlockState(), Block.UPDATE_ALL);
            level.setBlock(control, Blocks.CHEST.defaultBlockState(), Block.UPDATE_ALL);
            for (BlockPos b : bands) {
                BlockEntity stored = level.getBlockEntity(s.stored(b));
                check(stored != null && level.getBlockEntity(b) == stored, "%s: the folded read at band %s finds the stored block entity %s", s.name(),
                        b.toShortString(), stored);
                check(!level.getChunkAt(b).getBlockEntities().containsKey(b), "%s: the band chunk holds no block entity of its own at %s", s.name(),
                        b.toShortString());
            }
            ((ChestBlockEntity) level.getBlockEntity(chest)).setItem(0, new ItemStack(Items.DIAMOND, 3));
            ((SignBlockEntity) level.getBlockEntity(signs[0])).setText(new SignText().setMessage(0, Component.literal(TEXT)), true);
            BlockPos exitStored = s.stored(gateway).relative(Direction.EAST, 3).above();
            ((TheEndGatewayBlockEntity) level.getBlockEntity(gateway)).setExitPosition(exitStored, false);

            helper.runAfterDelay(3, () -> {
                // The sand's hit direction is cleared by its own reset tick, so it is set and read in one go.
                Direction hitStored = WrapEngine.turn(s.iso(sand), Direction.NORTH);
                check(hitStored != Direction.NORTH, "%s: north in the band is %s where the sand is stored", s.name(), hitStored);
                CompoundTag hit = new CompoundTag();
                hit.putInt("hit_direction", hitStored.ordinal());
                level.getBlockEntity(s.stored(sand)).loadWithComponents(hit, level.registryAccess());
                // The gateway's exit, three blocks east of the stored gateway, as the band shows it (by the block-rotation path).
                Direction east = WrapEngine.rotation(s.iso(gateway).inverse()).rotate(Direction.EAST);
                BlockPos exitBand = gateway.relative(east, 3).above();

                // 1. The band chunks' packets.
                Map<BlockPos, List<Entry>> data = chunkData(level, bands);
                one(s, data, chest, BlockEntityType.CHEST);
                for (BlockPos b : signs)
                    one(s, data, b, BlockEntityType.SIGN);
                Entry sign0 = one(s, data, signs[0], BlockEntityType.SIGN);
                check(sign0.tag() != null && sign0.tag().toString().contains(TEXT), "%s: the band sign's packet carries its text: %s", s.name(), sign0.tag());
                Entry gw = one(s, data, gateway, BlockEntityType.END_GATEWAY);
                BlockPos exitSent = gw.tag() == null ? null : NbtUtils.readBlockPos(gw.tag(), "exit_portal").orElse(null);
                check(exitBand.equals(exitSent), "%s: the band gateway's exit is %s (stored %s turned), sent %s", s.name(), exitBand.toShortString(),
                        exitStored.toShortString(), exitSent == null ? null : exitSent.toShortString());
                Entry sd = one(s, data, sand, BlockEntityType.BRUSHABLE_BLOCK);
                int hitSent = sd.tag() == null || !sd.tag().contains("hit_direction") ? -1 : sd.tag().getInt("hit_direction");
                check(hitSent == Direction.NORTH.ordinal(), "%s: the band sand's hit direction is north (stored %s turned), sent ordinal %d", s.name(),
                        hitStored, hitSent);

                // 2. A band holder's data packet: the stored entity's, re-addressed and turned.
                ClientboundBlockEntityDataPacket gp = BandBlockEntities.packetFor(level, gateway);
                check(gp != null && gp.getPos().equals(gateway) && exitBand.equals(NbtUtils.readBlockPos(gp.getTag(), "exit_portal").orElse(null)),
                        "%s: the gateway's data packet for its band position is addressed there with the exit turned: %s", s.name(),
                        gp == null ? null : gp.getPos().toShortString() + " " + gp.getTag());
                ClientboundBlockEntityDataPacket sp = BandBlockEntities.packetFor(level, sand);
                check(sp != null && sp.getPos().equals(sand) && sp.getTag().getInt("hit_direction") == Direction.NORTH.ordinal(),
                        "%s: the sand's data packet for its band position turns its hit direction: %s", s.name(), sp == null ? null : sp.getTag());
                check(BandBlockEntities.packetFor(level, s.stored(gateway)) == null, "%s: no re-addressed packet for a stored position", s.name());

                // 3. A change to the stored sign's data is marked at its band position (so the band's viewers get it).
                ChunkHolderAccessor holder8 = holder(level, signs[2]);
                int sec = level.getSectionIndex(signs[2].getY());
                short rel = SectionPos.sectionRelativePos(signs[2]);
                ShortSet before = holder8.deeptime$changedBlocksPerSection()[sec];
                check(before == null || !before.contains(rel), "%s: the band sign at %s is not marked before the change", s.name(), signs[2].toShortString());
                ((SignBlockEntity) level.getBlockEntity(s.stored(signs[2]))).setText(new SignText().setMessage(0, Component.literal("marked")), true);
                ShortSet after = holder8.deeptime$changedBlocksPerSection()[sec];
                check(after != null && after.contains(rel), "%s: setting the stored sign's text marks its band position %s", s.name(), signs[2].toShortString());

                // The band holder sends the stored sign's data to the band position.
                ServerPlayer p = helper.makeMockServerPlayerInLevel();
                sent(p);
                holder(level, signs[0]).deeptime$broadcastBlockEntity(List.of(p), level, signs[0]);
                List<Packet<?>> got = sent(p);
                check(got.stream().anyMatch(q -> q instanceof ClientboundBlockEntityDataPacket d && d.getPos().equals(signs[0]) && d.getTag().toString().contains(TEXT)),
                        "%s: the band holder sent the sign's data addressed to %s: %s", s.name(), signs[0].toShortString(), got);
                check(got.stream().noneMatch(q -> q instanceof ClientboundBlockEntityDataPacket d && d.getPos().equals(s.stored(signs[0]))),
                        "%s: and nothing addressed to the stored sign", s.name());

                // 4. Reach: at the band image of the chest, the far stored chest is usable; a chest out of reach is not.
                stand(p, s, CHEST, 1.3);
                BlockPos chestStored = s.stored(chest);
                Vec3 eye = p.getEyePosition();
                double r = p.blockInteractionRange() + 4.0;
                check(new AABB(chestStored).distanceToSqr(eye) > r * r, "%s: the stored chest %s is out of vanilla's reach (%.1f) from %s", s.name(),
                        chestStored.toShortString(), Math.sqrt(new AABB(chestStored).distanceToSqr(eye)), eye);
                check(p.canInteractWithBlock(chestStored, 4.0), "%s: the stored chest is in reach through its band image %s", s.name(), chest.toShortString());
                check(!p.canInteractWithBlock(control, 4.0), "%s: the control chest %s, out of reach and with no near image, is not", s.name(),
                        control.toShortString());
                p.openMenu((ChestBlockEntity) level.getBlockEntity(chestStored));
                check(p.containerMenu instanceof ChestMenu m && m.getContainer().getItem(0).is(Items.DIAMOND) && p.containerMenu.stillValid(p),
                        "%s: the chest's menu opened with its diamonds and is valid", s.name());

                helper.runAfterDelay(2, () -> {
                    check(p.containerMenu instanceof ChestMenu, "%s: the chest's menu is still open after the player's ticks (vanilla's stillValid)", s.name());
                    p.closeContainer();

                    // 5. The sign editor opens on the image the player stands at.
                    stand(p, s, SIGNS[0], 1.3);
                    SignBlockEntity sign = (SignBlockEntity) level.getBlockEntity(s.stored(signs[0]));
                    sent(p);
                    ((SignBlock) sign.getBlockState().getBlock()).openTextEdit(p, sign, true);
                    List<Packet<?>> ed = sent(p);
                    check(ed.stream().anyMatch(q -> q instanceof ClientboundOpenSignEditorPacket o && o.getPos().equals(signs[0]) && o.isFrontText()),
                            "%s: the sign editor opened at the band image %s: %s", s.name(), signs[0].toShortString(), ed);
                    check(ed.stream().anyMatch(q -> q instanceof ClientboundBlockUpdatePacket u && u.getPos().equals(signs[0])),
                            "%s: with a block update for the image", s.name());
                    check(ed.stream().noneMatch(q -> q instanceof ClientboundOpenSignEditorPacket o && o.getPos().equals(sign.getBlockPos())),
                            "%s: and not at the stored sign", s.name());

                    helper.runAfterDelay(3, () -> {
                        check(p.getUUID().equals(sign.getPlayerWhoMayEdit()), "%s: the sign's ticks keep the player as its editor (in reach through the image)",
                                s.name());
                        BlockEntity viaBand = level.getBlockEntity(signs[0]);
                        check(viaBand == sign, "%s: the edit addressed to the image finds the stored sign", s.name());
                        sign.updateSignText(p, true, List.of(FilteredText.passThrough(EDITED)));
                        String now = sign.getFrontText().getMessage(0, false).getString();
                        check(EDITED.equals(now), "%s: the stored sign reads \"%s\"", s.name(), now);

                        // 6. Front or back as the image shows it, from two places in front of each band sign.
                        int differs = 0;
                        double[] offsets = {1.3, -0.3};
                        boolean[][] want = {{true, false, false, true}, {true, true, false, false}};
                        for (int k = 0; k < offsets.length; k++)
                            for (int i = 0; i < SIGNS.length; i++) {
                                stand(p, s, SIGNS[i], offsets[k]);
                                BlockPos st = s.stored(signs[i]);
                                boolean expected = facesFront(WrapEngine.rawState(level, signs[i]), signs[i], p.position());
                                boolean raw = facesFront(level.getBlockState(st), st, p.position());
                                boolean front = ((SignBlockEntity) level.getBlockEntity(st)).isFacingFrontText(p);
                                check(expected == want[k][i], "%s: band sign %d (rotation %d) from offset %.1f: the band frame says front %s", s.name(), i, 4 * i,
                                        offsets[k], expected);
                                check(front == expected, "%s: band sign %d from offset %.1f: front %s, as the image shows (vanilla's own answer %s)", s.name(), i,
                                        offsets[k], front, raw);
                                if (raw != expected)
                                    differs++;
                            }
                        check(differs > 0, "%s: vanilla's own answer differs in %d of 8 cases, so the test tells the two apart", s.name(), differs);
                        level.getServer().getPlayerList().remove(p);
                        log("%s: block entities in the band hold (%d facing cases vanilla gets wrong)", s.name(), differs);
                        helper.succeed();
                    });
                });
            });
        });
    }

    /** Every block entity the band chunks of {@code at} send, by position. */
    private static Map<BlockPos, List<Entry>> chunkData(ServerLevel level, List<BlockPos> at) {
        Map<BlockPos, List<Entry>> out = new HashMap<>();
        Set<ChunkPos> chunks = new LinkedHashSet<>();
        for (BlockPos b : at)
            chunks.add(new ChunkPos(b));
        for (ChunkPos c : chunks) {
            ClientboundLevelChunkPacketData d = new ClientboundLevelChunkPacketData(level.getChunk(c.x, c.z));
            d.getBlockEntitiesTagsConsumer(c.x, c.z).accept((pos, type, tag) -> out.computeIfAbsent(pos.immutable(), k -> new ArrayList<>()).add(new Entry(type, tag)));
        }
        return out;
    }

    /** The one entry at {@code pos}, of {@code type}. */
    private static Entry one(Site s, Map<BlockPos, List<Entry>> data, BlockPos pos, BlockEntityType<?> type) {
        List<Entry> es = data.getOrDefault(pos, List.of());
        check(es.size() == 1 && es.get(0).type() == type, "%s: the band chunk lists one %s at %s: %s", s.name(), BandBlockEntities.typeId(type),
                pos.toShortString(), es);
        return es.get(0);
    }

    private static ChunkHolderAccessor holder(ServerLevel level, BlockPos pos) {
        return (ChunkHolderAccessor) level.getChunkSource().chunkMap.getVisibleChunkIfPresent(ChunkPos.asLong(pos));
    }

    /** Puts the player on the stored side half a block from the edge, beside column {@code u} of the band. */
    @SuppressWarnings("removal")
    private static void stand(ServerPlayer p, Site s, int u, double dx) {
        p.teleportTo(p.serverLevel(), s.ox() + u + dx, WrapEngineGameTests.Y0, s.oz() + 0.5, 0f, 0f);
    }

    /** What the mock player was sent since the last call, bundles opened. */
    private static List<Packet<?>> sent(ServerPlayer p) {
        EmbeddedChannel ch = (EmbeddedChannel) p.connection.getConnection().channel();
        ch.flushOutbound();
        List<Packet<?>> out = new ArrayList<>();
        for (Object m; (m = ch.readOutbound()) != null; )
            if (m instanceof Packet<?> q)
                unbundle(q, out);
        return out;
    }

    private static void unbundle(Packet<?> q, List<Packet<?>> out) {
        if (q instanceof BundlePacket<?> b) {
            for (Packet<?> r : b.subPackets())
                unbundle(r, out);
        } else
            out.add(q);
    }

    /** Vanilla's front-or-back test ({@code SignBlockEntity.isFacingFrontText}) for a sign in {@code state} at {@code pos}, from {@code at}. */
    private static boolean facesFront(BlockState state, BlockPos pos, Vec3 at) {
        SignBlock sign = (SignBlock) state.getBlock();
        Vec3 c = sign.getSignHitboxCenterPosition(state);
        double dx = at.x - (pos.getX() + c.x), dz = at.z - (pos.getZ() + c.z);
        float toPlayer = (float) (Mth.atan2(dz, dx) * 180.0F / (float) Math.PI) - 90.0F;
        return Mth.degreesDifferenceAbs(sign.getYRotationDegrees(state), toPlayer) <= 90.0F;
    }
}
