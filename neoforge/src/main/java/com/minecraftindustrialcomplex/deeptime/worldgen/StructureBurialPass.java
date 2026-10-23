// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructurePiece;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer;
import net.minecraft.world.level.levelgen.structure.pools.JigsawJunction;
import net.neoforged.neoforge.common.world.PieceBeardifierModifier;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.LongAdder;
import java.util.function.Supplier;

/**
 * Applies {@link StructureBurial} to a freshly generated structure start in a Deep Time level, from
 * {@code mixin.StructureBurialMixin} (the return of {@code Structure.generate}, which world generation and
 * {@code /place structure} both go through). The start is moved before anything stores or reads it, so the
 * level only ever sees the buried one.
 *
 * <p><b>What a move has to keep consistent</b> (vanilla 1.21.1, read in the sources):
 * <ul>
 *   <li>each piece's bounding box and, for a jigsaw piece, its {@code position} (the template is placed there):
 *       {@link StructurePiece#move} does both;</li>
 *   <li>the jigsaw junctions: a {@code JigsawJunction} keeps an <i>absolute</i> {@code sourceGroundY}, which
 *       {@code Beardifier} reads to shape the terrain around the piece, and {@code move} leaves it alone, so it is
 *       rebuilt here (its {@code deltaY} is relative and stays);</li>
 *   <li>the {@code groundLevelDelta} of a jigsaw piece is relative, and the Beardifier's boxes are the pieces' own
 *       boxes (or a {@code PieceBeardifierModifier}'s, which is checked), so those follow;</li>
 *   <li>the start's cached bounding box (the pieces' union inflated by 12 for terrain adaptation) is filled lazily
 *       and {@code StructureStart} is final: a fresh start is made from the same pieces, so nothing stale is
 *       cached. Nothing has read the old one yet.</li>
 * </ul>
 *
 * <p>A move is deterministic (the start's geometry and the world data's ground decide it; no random number is
 * drawn), so chunks generated in any order, and again after a crash, give the same start.
 */
public final class StructureBurialPass {

    private StructureBurialPass() {}

    private static final Map<Structure, Boolean> VERDICTS = new ConcurrentHashMap<>();
    private static final Set<String> LOGGED = ConcurrentHashMap.newKeySet();
    private static final LongAdder MOVED = new LongAdder(), DROPPED = new LongAdder(), KEPT = new LongAdder(), MISMATCHED = new LongAdder();
    private static final ThreadLocal<Boolean> OFF = new ThreadLocal<>();

    /**
     * The start to store for {@code structure}: {@code start} itself when the rule leaves it alone,
     * a moved copy when it is buried, {@link StructureStart#INVALID_START} when it cannot be buried inside the
     * world (the structure sets then try the next candidate).
     */
    public static StructureStart apply(Structure structure, StructureStart start, RegistryAccess access,
                                       ChunkGenerator generator, LevelHeightAccessor heights) {
        if (!start.isValid() || OFF.get() != null || !StructureBurial.enabled()
                || !(generator instanceof GeologicChunkGenerator g) || !covers(structure, access))
            return start;
        List<StructurePiece> pieces = start.getPieces();
        Optional<BoundingBox> union = BoundingBox.encapsulatingBoxes(pieces.stream().map(StructurePiece::getBoundingBox).toList());
        if (union.isEmpty())
            return start;
        BoundingBox box = union.get();
        TerrainSource terrain = g.terrain();
        int ground = StructureBurial.leastGround(terrain::height, box.minX(), box.maxX(), box.minZ(), box.maxZ());
        int dy = StructureBurial.plan(box.minY(), box.maxY(), ground, heights.getMinBuildHeight());
        String id = idOf(structure, access);
        if (dy == 0) {
            KEPT.increment();
            return start;
        }
        if (dy == StructureBurial.DROP) {
            DROPPED.increment();
            DeepTime.LOGGER.debug("[deeptime] structure {} at chunk {} dropped: buried under the ground at y {} it would reach below the world floor (box y {}..{})",
                    id, start.getChunkPos(), ground, box.minY(), box.maxY());
            return StructureStart.INVALID_START;
        }
        move(pieces, dy);
        MOVED.increment();
        DeepTime.LOGGER.debug("[deeptime] structure {} at chunk {} moved down {} blocks to y {}..{} (least ground y {})",
                id, start.getChunkPos(), -dy, box.minY() + dy, box.maxY() + dy, ground);
        if (LOGGED.add(id))
            DeepTime.LOGGER.info("[deeptime] structure {} is buried under the ground of Deep Time levels when it would stand "
                    + "within {} blocks of it ({}=false turns this off)", id, StructureBurial.COVER, StructureBurial.PROPERTY);
        return new StructureStart(structure, start.getChunkPos(), start.getReferences(), new PiecesContainer(pieces));
    }

    /**
     * Moves every piece by {@code dy} blocks (negative: down) and rebuilds the junctions; checks afterwards that
     * each box moved exactly {@code dy} (two pieces sharing one box would move it twice) and counts a mismatch.
     */
    static void move(List<StructurePiece> pieces, int dy) {
        int[] minBefore = new int[pieces.size()], beardBefore = new int[pieces.size()];
        for (int i = 0; i < pieces.size(); i++) {
            StructurePiece p = pieces.get(i);
            minBefore[i] = p.getBoundingBox().minY();
            beardBefore[i] = p instanceof PieceBeardifierModifier m ? m.getBeardifierBox().minY() : 0;
        }
        for (StructurePiece p : pieces) {
            p.move(0, dy, 0);
            if (p instanceof PoolElementStructurePiece pool)
                pool.getJunctions().replaceAll(j -> new JigsawJunction(j.getSourceX(), j.getSourceGroundY() + dy, j.getSourceZ(),
                        j.getDeltaY(), j.getDestProjection()));
        }
        for (int i = 0; i < pieces.size(); i++) {
            StructurePiece p = pieces.get(i);
            boolean ok = p.getBoundingBox().minY() == minBefore[i] + dy
                    && (!(p instanceof PieceBeardifierModifier m) || m.getBeardifierBox().minY() == beardBefore[i] + dy);
            if (!ok) {
                MISMATCHED.increment();
                DeepTime.LOGGER.warn("[deeptime] a {} piece did not move by {} blocks with its structure (box y {}, expected {})",
                        p.getClass().getSimpleName(), dy, p.getBoundingBox().minY(), minBefore[i] + dy);
            }
        }
    }

    /** Whether the rule looks at this structure; decided once per structure on its encoded JSON. */
    static boolean covers(Structure structure, RegistryAccess access) {
        Boolean v = VERDICTS.get(structure);
        if (v == null) {
            v = decide(structure, access);
            VERDICTS.put(structure, v);
        }
        return v;
    }

    private static boolean decide(Structure structure, RegistryAccess access) {
        try {
            var ops = access.createSerializationContext(JsonOps.INSTANCE);
            return Structure.DIRECT_CODEC.encodeStart(ops, structure).result().map(StructureBurial::covers).orElse(false);
        } catch (RuntimeException e) {
            return false;   // a structure that cannot be written out is left alone
        }
    }

    private static String idOf(Structure structure, RegistryAccess access) {
        var key = access.registryOrThrow(Registries.STRUCTURE).getKey(structure);
        return key == null ? structure.getClass().getSimpleName() : key.toString();
    }

    /**
     * Runs {@code body} with the rule off on this thread only: a test that wants a start as vanilla's own
     * generation makes it, next to the buried one, without flipping the system property under the other threads.
     */
    public static <T> T withoutBurial(Supplier<T> body) {
        OFF.set(Boolean.TRUE);
        try {
            return body.get();
        } finally {
            OFF.remove();
        }
    }

    /** Whether the rule would look at {@code structure} (for tests and probes). */
    public static boolean looksAt(Structure structure, RegistryAccess access) {
        return covers(structure, access);
    }

    /** Starts moved, dropped, left alone though covered, and pieces that did not move with their structure, since the process began. */
    public static String stats() {
        return "moved " + MOVED.sum() + ", dropped " + DROPPED.sum() + ", kept " + KEPT.sum() + ", mismatched pieces " + MISMATCHED.sum();
    }

    public static long moved() {
        return MOVED.sum();
    }

    public static long dropped() {
        return DROPPED.sum();
    }

    public static long mismatched() {
        return MISMATCHED.sum();
    }
}
