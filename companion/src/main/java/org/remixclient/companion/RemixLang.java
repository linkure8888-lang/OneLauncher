package org.remixclient.companion;

import net.minecraft.network.chat.Component;

import java.util.Map;

/**
 * Companion-screen translations.
 *
 * <p>The vanilla translation pipeline follows the game language, but the launcher owns its own
 * language setting and pins {@link RemixConfig#language} into the companion config, so a Russian
 * launcher must show Russian companion screens even if the game itself runs in English. With
 * {@code auto} the regular {@code Component.translatable} path (assets lang files) is used.</p>
 */
final class RemixLang {

    private static final Map<String, String> RU = Map.ofEntries(
            Map.entry("editor.title", "HUD-редактор"),
            Map.entry("editor.hint", "ПКМ: переместить · Колесо: размер · ЛКМ: вкл/выкл · R: сброс · Esc: готово"),
            Map.entry("editor.locked", "закреплён на месте прицела"),
            Map.entry("module.status", "Броня и тотемы"),
            Map.entry("module.keystrokes", "Клавиши WASD"),
            Map.entry("module.keybinds", "Клавиши действий"),
            Map.entry("module.target", "Цель в прицеле"),
            Map.entry("module.potions", "Эффекты"),
            Map.entry("module.watermark", "Ватермарка"),
            Map.entry("module.crosshair", "Прицел"));

    private static final Map<String, String> EN = Map.ofEntries(
            Map.entry("editor.title", "HUD editor"),
            Map.entry("editor.hint", "Right-drag: move · Wheel: size · Left-click: on/off · R: reset · Esc: done"),
            Map.entry("editor.locked", "fixed to the crosshair position"),
            Map.entry("module.status", "Armor & totems"),
            Map.entry("module.keystrokes", "Keystrokes"),
            Map.entry("module.keybinds", "Keybinds"),
            Map.entry("module.target", "Target HUD"),
            Map.entry("module.potions", "Potions"),
            Map.entry("module.watermark", "Watermark"),
            Map.entry("module.crosshair", "Crosshair"));

    private RemixLang() {
    }

    /** Translates a companion key honouring the pinned language, or the game language for auto. */
    static String tr(String key) {
        String pinned = RemixCompanionClient.getConfig().language;
        if ("ru".equals(pinned)) {
            String text = RU.get(key);
            if (text != null) {
                return text;
            }
        }
        if ("en".equals(pinned)) {
            String text = EN.get(key);
            if (text != null) {
                return text;
            }
        }
        return Component.translatable("remix_companion." + key).getString();
    }

    /** A translatable component when auto, or a literal one when the language is pinned. */
    static Component component(String key) {
        String pinned = RemixCompanionClient.getConfig().language;
        if ("ru".equals(pinned) || "en".equals(pinned)) {
            return Component.literal(tr(key));
        }
        return Component.translatable("remix_companion." + key);
    }

    /** The "%s · OFF" editor label, formatted with the module title already translated. */
    static Component off(Component moduleTitle) {
        String pinned = RemixCompanionClient.getConfig().language;
        String title = moduleTitle.getString();
        if ("ru".equals(pinned)) {
            return Component.literal(title + " · ВЫКЛ");
        }
        if ("en".equals(pinned)) {
            return Component.literal(title + " · OFF");
        }
        return Component.translatable("remix_companion.editor.off", moduleTitle);
    }
}
