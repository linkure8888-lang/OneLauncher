package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import net.minecraft.world.effect.MobEffect;
import net.minecraft.world.effect.MobEffectCategory;
import net.minecraft.world.effect.MobEffectInstance;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Text list of every active status effect with its remaining time. Harmful effects come first so
 * a poison never hides below three buffs.
 */
final class RemixPotions implements RemixModule {

    static final RemixPotions INSTANCE = new RemixPotions();

    private static final int ROW_HEIGHT = 10;
    private static final int MAX_ROWS = 8;
    /** Minimum width so an empty list still has a grabbable frame in the HUD editor. */
    private static final int MIN_WIDTH = 64;

    private RemixPotions() {
    }

    @Override
    public String id() {
        return "potions";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.potions");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showPotions;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showPotions = enabled;
    }

    private static List<MobEffectInstance> effects(Minecraft client) {
        if (client.player == null) {
            return List.of();
        }
        List<MobEffectInstance> effects = new ArrayList<>(client.player.getActiveEffects());
        effects.sort(Comparator
                .comparingInt(RemixPotions::severity)
                .thenComparing(effect -> effect.getEffect().value().getDisplayName().getString()));
        return effects.size() > MAX_ROWS ? effects.subList(0, MAX_ROWS) : effects;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        Font font = client.font;
        int widest = 0;
        for (MobEffectInstance effect : effects(client)) {
            widest = Math.max(widest, font.width(text(effect)));
        }
        return Math.max(MIN_WIDTH, widest + 4);
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return Math.max(1, effects(client).size()) * ROW_HEIGHT;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        List<MobEffectInstance> effects = effects(client);
        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();

        for (int i = 0; i < effects.size(); i++) {
            MobEffectInstance effect = effects.get(i);
            MobEffect type = effect.getEffect().value();
            RemixDraw.line(context, font, text(effect), 2, i * ROW_HEIGHT,
                    colorFor(type, effect), backgroundAlpha);
        }
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();
        return new Origin(Math.max(4, screenWidth - 4 - width(client, config)),
                Math.max(4, screenHeight - 46 - height(client, config)));
    }

    private static String text(MobEffectInstance effect) {
        MobEffect type = effect.getEffect().value();
        return type.getDisplayName().getString()
                + amplifierSuffix(effect.getAmplifier())
                + "  " + duration(effect);
    }

    /** Harmful first (0), neutral (1), beneficial last (2). */
    private static int severity(MobEffectInstance effect) {
        MobEffectCategory category = effect.getEffect().value().getCategory();
        if (category == MobEffectCategory.HARMFUL) {
            return 0;
        }
        return category == MobEffectCategory.NEUTRAL ? 1 : 2;
    }

    private static int colorFor(MobEffect type, MobEffectInstance effect) {
        // Amber when the effect is about to run out — the same cue players already
        // watch for on the vanilla icon row.
        if (!effect.isInfiniteDuration() && effect.getDuration() < 200) {
            return RemixDraw.YELLOW;
        }
        return switch (type.getCategory()) {
            case HARMFUL -> RemixDraw.RED;
            case NEUTRAL -> RemixDraw.TEXT_DIM;
            default -> RemixDraw.GREEN;
        };
    }

    private static String amplifierSuffix(int amplifier) {
        return amplifier <= 0 ? "" : " " + roman(amplifier + 1);
    }

    private static String roman(int value) {
        return switch (value) {
            case 2 -> "II";
            case 3 -> "III";
            case 4 -> "IV";
            case 5 -> "V";
            case 6 -> "VI";
            case 7 -> "VII";
            case 8 -> "VIII";
            case 9 -> "IX";
            case 10 -> "X";
            default -> Integer.toString(value);
        };
    }

    private static String duration(MobEffectInstance effect) {
        if (effect.isInfiniteDuration()) {
            return "\u221E";
        }
        int seconds = Math.max(0, effect.getDuration() / 20);
        if (seconds >= 60) {
            return seconds / 60 + ":" + String.format("%02d", seconds % 60);
        }
        return seconds + "s";
    }
}
