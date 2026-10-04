// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrapengine.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapChunk;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import it.unimi.dsi.fastutil.shorts.ShortList;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * The chunk-level half of the wrap engine (docs/design/wrap-engine.md §4.1 and §4.2):
 * <ul>
 *   <li><b>The write choke point.</b> {@code LevelChunk.setBlockState} on a band chunk is never a
 *       legitimate write (folded {@code Level.setBlock} never reaches a band chunk, and band copies are
 *       written into the sections directly), so it is a mod or tool writing chunk storage: it is
 *       redirected to the stored block, turned.</li>
 *   <li><b>Replication.</b> After {@code setBlockState} on a stored chunk near an edge, the block's
 *       turned copy goes into every loaded band chunk that shows it.</li>
 *   <li><b>No generation post-processing in band chunks</b>: it would write the band chunk's generated
 *       blocks through the fold into the stored world.</li>
 * </ul>
 * The chunk's place in the topology is cached per topology. Applied only with
 * {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(LevelChunk.class)
abstract class LevelChunkWrapMixin implements WrapChunk {

    @Unique
    private static final CellIsometry[] DEEPTIME$NONE = new CellIsometry[0];
    @Unique
    private static boolean deeptime$warnedOffThread;

    @Unique
    private WrapTopology deeptime$cacheFor;
    @Unique
    private CellIsometry deeptime$bandIso;
    @Unique
    private CellIsometry[] deeptime$images = DEEPTIME$NONE;

    @Shadow
    public abstract Level getLevel();

    /** Brings the cache up to the level's topology; the topology, or null when there is none. */
    @Unique
    private WrapTopology deeptime$refresh() {
        Level level = getLevel();
        WrapTopology t = level == null ? null : ((WrapLevel) level).deeptime$wrap();
        if (t != null && t != deeptime$cacheFor) {
            ChunkPos p = ((LevelChunk) (Object) this).getPos();
            WrapTopology.ChunkFold cf = t.chunkFold(p.x, p.z);
            deeptime$bandIso = cf == null ? null : cf.iso();
            deeptime$images = cf == null ? t.chunkImages(p.x, p.z) : DEEPTIME$NONE;
            deeptime$cacheFor = t;
        }
        return t;
    }

    @Override
    public CellIsometry deeptime$bandIso() {
        return deeptime$refresh() == null ? null : deeptime$bandIso;
    }

    @Override
    public CellIsometry[] deeptime$images() {
        return deeptime$refresh() == null ? DEEPTIME$NONE : deeptime$images;
    }

    @Inject(method = "setBlockState", at = @At("HEAD"), cancellable = true)
    private void deeptime$chokePoint(BlockPos pos, BlockState state, boolean isMoving, CallbackInfoReturnable<BlockState> cir) {
        if (!(getLevel() instanceof ServerLevel level) || deeptime$refresh() == null || deeptime$bandIso == null)
            return;
        if (!level.getServer().isSameThread()) {
            if (!deeptime$warnedOffThread) {
                deeptime$warnedOffThread = true;
                DeepTime.LOGGER.warn("[wrapengine] a band chunk was written off the server thread at {}; left as it is (it is re-copied when it reloads)", pos);
            }
            return;
        }
        var bands = ((WrapLevel) level).deeptime$bands();
        if (bands == null)
            return;
        cir.setReturnValue(bands.redirectChunkWrite((LevelChunk) (Object) this, deeptime$bandIso, pos, state, isMoving));
    }

    @Inject(method = "setBlockState", at = @At("RETURN"))
    private void deeptime$replicate(BlockPos pos, BlockState state, boolean isMoving, CallbackInfoReturnable<BlockState> cir) {
        if (!(getLevel() instanceof ServerLevel level) || deeptime$refresh() == null || deeptime$images.length == 0)
            return;
        var bands = ((WrapLevel) level).deeptime$bands();
        if (bands != null)
            bands.replicate((LevelChunk) (Object) this, pos, deeptime$images);
    }

    @Inject(method = "postProcessGeneration", at = @At("HEAD"), cancellable = true)
    private void deeptime$noBandPostProcessing(CallbackInfo ci) {
        if (deeptime$refresh() == null || deeptime$bandIso == null)
            return;
        for (ShortList l : ((LevelChunk) (Object) this).getPostProcessing())
            if (l != null)
                l.clear();
        ci.cancel();
    }
}
