// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import net.neoforged.fml.loading.LoadingModList;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;

import java.io.InputStream;
import java.util.List;
import java.util.Set;

/**
 * Gates {@code deeptime.tw_dh.mixins.json}: Deep Time's patch to Toroidal World's Distant Horizons
 * guard ({@code mixin/compat/toroidal_world/DhGeneratorsMixin}, {@code compat/dh/TwDhGuard}).
 *
 * <p>The mixin applies only when Toroidal World and Distant Horizons are both installed and TW's
 * {@code DhGenerators} has {@code public static void requireOwn(IDhApiLevelWrapper,
 * IDhApiWorldGenerator)} as in TW 0.19.3 (read from the class file before anything is applied).
 * A TW update that renames or reshapes it skips the patch with a warning instead of failing the
 * boot: far terrain then falls back to what TW decides (Distant Horizons' own LODs from real chunks
 * on a wrapped level). Every outcome is logged: skipped and why, or applied (checked in the
 * patched method's code).
 */
public class TwDhGuardMixinPlugin implements IMixinConfigPlugin {

    public static final String TARGET = "com.toroidalworld.compat.distanthorizons.DhGenerators";
    public static final String METHOD = "requireOwn";
    public static final String DESCRIPTOR = "(Lcom/seibel/distanthorizons/api/interfaces/world/IDhApiLevelWrapper;"
            + "Lcom/seibel/distanthorizons/api/interfaces/override/worldGenerator/IDhApiWorldGenerator;)V";
    /** The handler's name prefix, as Mixin merges it into the target. */
    private static final String HANDLER = "deeptime$letFarTerrainThrough";
    /** A system property holding the outcome, for the probe (game code cannot load a class of a mixin package). */
    public static final String STATE = "deeptime.twDhGuard";

    private static final Logger LOGGER = LoggerFactory.getLogger("deeptime");

    @Override
    public void onLoad(String mixinPackage) {}

    @Override
    public String getRefMapperConfig() {
        return null;
    }

    @Override
    public boolean shouldApplyMixin(String targetClassName, String mixinClassName) {
        String skip = whySkip();
        if (skip != null) {
            LOGGER.info("[deeptime] Toroidal World's Distant Horizons guard: not patched ({})", skip);
            System.setProperty(STATE, "skipped: " + skip);
            return false;
        }
        LOGGER.info("[deeptime] Toroidal World's Distant Horizons guard: patching {}.{} so Deep Time's far-terrain generator "
                + "can pass it on Deep Time levels", TARGET, METHOD);
        System.setProperty(STATE, "pending: applies when Toroidal World loads its guard");
        return true;
    }

    /** Null when the patch can apply, else why not. */
    static String whySkip() {
        LoadingModList mods = LoadingModList.get();
        if (mods != null) {
            if (mods.getModFileById("toroidal_world") == null)
                return "Toroidal World is not installed";
            if (mods.getModFileById("distanthorizons") == null)
                return "Distant Horizons is not installed";
        }
        // The class file as a resource, untransformed (ModLauncher's bytecode provider gives no
        // untransformed bytecode, and a transformed read here would run Mixin inside its own prepare).
        ClassNode node = new ClassNode();
        String resource = TARGET.replace('.', '/') + ".class";
        try (InputStream in = open(resource)) {
            if (in == null)
                return TARGET + " was not found in this Toroidal World; its guard may have moved";
            new ClassReader(in).accept(node, ClassReader.SKIP_CODE | ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
        } catch (Exception e) {
            return TARGET + " could not be read (" + e + ")";
        }
        for (MethodNode m : node.methods) {
            if (m.name.equals(METHOD) && m.desc.equals(DESCRIPTOR) && (m.access & Opcodes.ACC_STATIC) != 0)
                return null;
        }
        return "this Toroidal World has no static " + METHOD + DESCRIPTOR + " in " + TARGET
                + "; its guard changed, so Deep Time leaves it alone (far terrain is off on wrapped levels until the patch follows)";
    }

    /** A class file of Toroidal World's jar: from its mod file, else through this mod's loader or the context loader. */
    private static InputStream open(String resource) throws java.io.IOException {
        LoadingModList mods = LoadingModList.get();
        var tw = mods == null ? null : mods.getModFileById("toroidal_world");
        if (tw != null) {
            java.nio.file.Path p = tw.getFile().findResource(resource.split("/"));
            if (p != null && java.nio.file.Files.isRegularFile(p))
                return java.nio.file.Files.newInputStream(p);
        }
        InputStream in = TwDhGuardMixinPlugin.class.getClassLoader().getResourceAsStream(resource);
        if (in == null && Thread.currentThread().getContextClassLoader() != null)
            in = Thread.currentThread().getContextClassLoader().getResourceAsStream(resource);
        return in;
    }

    @Override
    public void acceptTargets(Set<String> myTargets, Set<String> otherTargets) {}

    @Override
    public List<String> getMixins() {
        return null;
    }

    @Override
    public void preApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {}

    @Override
    public void postApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {
        for (MethodNode m : targetClass.methods) {
            if (!m.name.equals(METHOD) || !m.desc.equals(DESCRIPTOR))
                continue;
            for (AbstractInsnNode insn = m.instructions.getFirst(); insn != null; insn = insn.getNext()) {
                if (insn instanceof MethodInsnNode call && call.name.contains(HANDLER)) {
                    LOGGER.info("[deeptime] Toroidal World's Distant Horizons guard: patched ({}.{} asks Deep Time first; "
                            + "TW's seam handling is unchanged)", TARGET, METHOD);
                    System.setProperty(STATE, "applied");
                    return;
                }
            }
        }
        LOGGER.warn("[deeptime] Toroidal World's Distant Horizons guard: the patch did not reach {}.{}; "
                + "Toroidal World will refuse Deep Time's far terrain on wrapped levels", TARGET, METHOD);
        System.setProperty(STATE, "failed: the handler is not in the patched method");
    }
}
