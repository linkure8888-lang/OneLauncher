package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import org.polyfrost.oneconfig.api.config.v1.Property;
import org.polyfrost.oneconfig.api.config.v1.Tree;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Rewrites the OneConfig descriptions of our options into Russian when the launcher runs in
 * Russian, while the option titles stay English (a mixed audience understands "Hit particles"
 * better than a translated name, but full sentences read better in their own language).
 *
 * <p>OneConfig resolves description keys through the game's own language, which is chosen
 * in-game and not by the launcher; rewriting the description metadata directly keeps the
 * text under our control. Titles are left untouched.</p>
 */
final class RemixDescriptions {

    private static boolean applied;

    private RemixDescriptions() {
    }

    /** Field name -> Russian description, mirrors the annotations in {@link RemixConfig}. */
    private static final Map<String, String> RU = new LinkedHashMap<>();

    static {
        RU.put("showStatus", "Броня, оружие и тотемы в одной панели.");
        RU.put("showArmor", "Надетая броня с прочностью.");
        RU.put("showWeapon", "Предмет в руке и его количество.");
        RU.put("showTotems", "Счётчик тотемов и вспышка при использовании.");
        RU.put("showKeystrokes", "WASD, прыжок и кнопки мыши со счётчиком CPS.");
        RU.put("showKeybinds", "Индикаторы Shift, спринта и оффхенда.");
        RU.put("showTargetHud", "Здоровье и броня того, кто под прицелом.");
        RU.put("showPotions", "Активные эффекты и время их действия.");
        RU.put("showWatermark", "Название клиента, FPS и пинг.");
        RU.put("showCoords", "Текущие координаты XYZ.");
        RU.put("opacity", "Прозрачность фона всех панелей HUD.");
        RU.put("hitEffects", "Частицы при ударе по мобу.");
        RU.put("jumpCircles", "Кольцо частиц под ногами при прыжке.");
        RU.put("trails", "Мягкий след из частиц при ходьбе.");
        RU.put("hitParticleCount", "Сколько частиц создаёт один удар.");
        RU.put("showCrosshair", "Свой прицел вместо ванильного.");
        RU.put("crosshairDot", "Точка в центре прицела.");
        RU.put("crosshairOutline", "Тёмная обводка линий прицела.");
        RU.put("crosshairHitTint", "Прицел краснеет при попадании.");
        RU.put("crosshairGap", "Расстояние от центра до линий прицела.");
        RU.put("crosshairLength", "Длина каждой линии прицела.");
        RU.put("crosshairThickness", "Толщина линий прицела.");
        RU.put("fullBrightness", "Усиление гаммы, то же самое на клавишу B.");
    }

    /**
     * Runs once, after OneConfig has built our tree: clears every description key and puts
     * the Russian text in its place when the launcher language is Russian (or the pinned
     * value is {@code auto} and the game itself runs in Russian).
     *
     * @return true when the rewrite has happened (or is not needed), false to retry later
     */
    static boolean apply(RemixConfig config) {
        if (applied) {
            return true;
        }
        try {
            Tree tree = config.getTree();
            if (tree == null) {
                // OneConfig has not processed the init queue yet; try again next tick.
                return false;
            }
            if (!shouldUseRussian(config)) {
                applied = true;
                return true;
            }
            tree.onAllProps((id, prop) -> {
                String ru = RU.get(id);
                if (ru != null) {
                    // An empty key makes OneConfig fall back to the raw description text.
                    prop.addMetadata(Map.of("description", ru, "descriptionKey", ""));
                }
            });
            applied = true;
            RemixCompanionClient.LOGGER.info(
                    "[Remix Companion] Russian descriptions applied to the OneConfig page.");
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error(
                    "[Remix Companion] Could not rewrite option descriptions.", t);
            applied = true;
        }
        return true;
    }

    private static boolean shouldUseRussian(RemixConfig config) {
        String pinned = config.language;
        if ("ru".equals(pinned)) {
            return true;
        }
        if ("en".equals(pinned)) {
            return false;
        }
        try {
            // auto: follow the game language.
            Minecraft client = Minecraft.getInstance();
            return client != null
                    && client.getLanguageManager() != null
                    && client.getLanguageManager().getSelected().toLowerCase(java.util.Locale.ROOT)
                            .startsWith("ru");
        } catch (Throwable t) {
            return false;
        }
    }
}
