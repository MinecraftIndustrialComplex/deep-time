// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.LayerCatalog;
import com.minecraftindustrialcomplex.deeptime.screen.model.Milestones;
import com.minecraftindustrialcomplex.deeptime.screen.model.Phrase;
import com.minecraftindustrialcomplex.deeptime.screen.model.Problem;
import net.minecraft.locale.Language;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.MutableComponent;

/**
 * Translation keys of the planet editor, all under {@code deeptime.} in
 * {@code assets/deeptime/lang/en_us.json}. {@code LangKeysTest} finds every {@code tr("…")} in the
 * sources and checks the key exists, so a key is always written as a literal here.
 */
final class Lang {

    private Lang() {}

    /** {@code deeptime.<suffix>} with arguments. */
    static MutableComponent tr(String suffix, Object... args) {
        return Component.translatable("deeptime." + suffix, args);
    }

    /** A model problem as a component. */
    static MutableComponent problem(Problem p) {
        return Component.translatable(p.translationKey(), p.args().toArray());
    }

    /**
     * A key built at run time (a layer, an event kind, a preset family): its translation when the
     * language has one, else the humanised id.
     */
    static Component dynamic(String key, String id) {
        return Language.getInstance().has(key) ? Component.translatable(key) : Component.literal(Formats.humanize(id));
    }

    /** A layer's name: its own key, else the word for its last part (as the workbench names layers), else humanised. */
    static Component layer(String name) {
        Language lang = Language.getInstance();
        String key = LayerCatalog.key(name);
        if (lang.has(key))
            return Component.translatable(key);
        String word = LayerCatalog.wordKey(name);
        if (lang.has(word))
            return Component.translatable(word);
        return Component.literal(Formats.humanize(name.substring(name.lastIndexOf('/') + 1)));
    }

    /** A layer group's name (Planet, Climate, …). */
    static Component layerGroup(String group) {
        return dynamic(LayerCatalog.groupKey(group), group.isEmpty() ? "planet" : group);
    }

    /**
     * A model phrase ({@link Phrase}): its translation with the arguments (phrases among them
     * translated first), the literal text of a literal phrase, or, for a key the language lacks
     * (a stage or an event kind the game has no words for yet), its last part humanised.
     */
    static MutableComponent phrase(Phrase p) {
        if (p.isLiteral())
            return Component.literal(p.args().isEmpty() ? "" : String.valueOf(p.args().get(0)));
        Object[] args = p.args().stream().map(a -> a instanceof Phrase q ? (Object) phrase(q) : a).toArray();
        if (Language.getInstance().has(p.translationKey()))
            return Component.translatable(p.translationKey(), args);
        return Component.literal(Formats.humanize(p.key().substring(p.key().lastIndexOf('.') + 1)));
    }

    /** A History row: its words, else the event kind's name with the count. */
    static MutableComponent milestone(Milestones.Row r) {
        Phrase p = r.phrase();
        if (Language.getInstance().has(p.translationKey()))
            return phrase(p);
        MutableComponent name = eventKind(r.kind()).copy();
        return r.count() > 1 ? name.append(" ×" + r.count()) : name;
    }

    static Component eventKind(String kind) {
        return dynamic("deeptime.event." + kind.substring(kind.indexOf(':') + 1), kind);
    }

    static Component family(String family) {
        return dynamic("deeptime.editor.preset." + family, family);
    }
}
