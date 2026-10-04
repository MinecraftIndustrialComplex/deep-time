// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import net.neoforged.fml.loading.LoadingModList;
import org.objectweb.asm.tree.ClassNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;

import java.util.List;
import java.util.Set;

/**
 * Gates Deep Time's mixins.
 *
 * <ul>
 *   <li>A mixin in the sub-package {@code mixin.compat.<modid>} applies only when that mod
 *       is installed (COE, CDG, TerraBlender, ...).</li>
 *   <li>A mixin in {@code mixin.gametest} applies only with {@code -Ddeeptime.gametest=true},
 *       so test-only fixes never reach a player's game.</li>
 *   <li>A mixin in {@code mixin.wrapengine} (the wrap engine, docs/design/wrap-engine.md) applies
 *       only with {@code -Ddeeptime.wrapengine=true}; one in
 *       {@code mixin.wrapengine.compat.<modid>} (the engine's adapters for a mod, such as Create) also
 *       needs that mod installed.</li>
 *   <li>Everything else always applies.</li>
 * </ul>
 *
 * <p>This runs during mixin bootstrap, before the mod list is built, so it touches no other
 * Deep Time class and checks mods through FML's loading mod list.
 */
public class DeepTimeMixinPlugin implements IMixinConfigPlugin {

    private static final String COMPAT = "com.minecraftindustrialcomplex.deeptime.mixin.compat.";
    private static final String GAMETEST = "com.minecraftindustrialcomplex.deeptime.mixin.gametest.";
    private static final String WRAPENGINE = "com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.";

    private static final Logger LOGGER = LoggerFactory.getLogger("deeptime");

    @Override
    public void onLoad(String mixinPackage) {}

    @Override
    public String getRefMapperConfig() {
        return null;
    }

    @Override
    public boolean shouldApplyMixin(String targetClassName, String mixinClassName) {
        if (mixinClassName.startsWith(WRAPENGINE)) {
            // The wrap engine (docs/design/wrap-engine.md): dev only until Ben switches it on. Mixin
            // bootstrap must not load WrapEngine, so the flag is read here again.
            boolean on = Boolean.getBoolean("deeptime.wrapengine");
            // mixin.wrapengine.compat.<modid>.*: the engine's adapters for a mod, which need that mod too
            // (and -Ddeeptime.wrapengine.compat not false: ./gametest.sh -PwrapNoCompat, for A/B checks).
            String modid = modAfter(mixinClassName, WRAPENGINE + "compat.");
            String simple = mixinClassName.substring(mixinClassName.lastIndexOf('.') + 1);
            boolean skipped = modid != null && java.util.Arrays.asList(System.getProperty("deeptime.wrapengine.compat.skip", "").split(",")).contains(simple);
            if (on && modid != null && (skipped || !installed(modid) || !Boolean.parseBoolean(System.getProperty("deeptime.wrapengine.compat", "true")))) {
                LOGGER.info("Skipping wrap-engine mixin {}: {} is not installed or the adapters are off", mixinClassName, modid);
                return false;
            }
            if (on)
                LOGGER.warn("Applying wrap-engine mixin {} (-Ddeeptime.wrapengine=true)", mixinClassName);
            return on;
        }
        if (mixinClassName.startsWith(GAMETEST)) {
            boolean on = Boolean.getBoolean("deeptime.gametest");
            if (!on)
                LOGGER.debug("Skipping gametest mixin {}", mixinClassName);
            return on;
        }
        String modid = modAfter(mixinClassName, COMPAT);
        if (modid == null || installed(modid))
            return true;
        LOGGER.info("Skipping mixin {}: {} is not installed", mixinClassName, modid);
        return false;
    }

    private static boolean installed(String modid) {
        LoadingModList mods = LoadingModList.get();
        return mods == null || mods.getModFileById(modid) != null;
    }

    /** The mod id of {@code <prefix><modid>.<Class>}, or null when the name does not start so. */
    private static String modAfter(String mixinClassName, String prefix) {
        if (!mixinClassName.startsWith(prefix))
            return null;
        String rest = mixinClassName.substring(prefix.length());
        int dot = rest.indexOf('.');
        return dot <= 0 ? null : rest.substring(0, dot);
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
    public void postApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {}
}
