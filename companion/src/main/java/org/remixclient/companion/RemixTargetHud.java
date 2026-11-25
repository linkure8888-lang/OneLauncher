package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;

import java.util.Locale;

/**
 * Compact card for whatever the crosshair is pointing at: name, health bar, armour and distance.
 */
final class RemixTargetHud implements RemixModule {

    static final RemixTargetHud INSTANCE = new RemixTargetHud();

    private static final int MIN_WIDTH = 96;
    private static final int HEIGHT = 30;
    private static final int MAX_NAME_CHARS = 18;

    private RemixTargetHud() {
    }

    @Override
    public String id() {
        return "target";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.target");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showTargetHud;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showTargetHud = enabled;
    }

    /** The living entity under the crosshair, or {@code null}. */
    private static LivingEntity target(Minecraft client) {
        Entity picked = client.crosshairPickEntity;
        return picked instanceof LivingEntity living ? living : null;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        LivingEntity target = target(client);
        if (target == null) {
            return MIN_WIDTH;
        }
        Font font = client.font;
        LivingEntity probe = target;
        String meta = metaText(probe, client);
        int contentWidth = Math.max(font.width(shortened(probe.getDisplayName().getString())) + 6,
                Math.max(font.width(healthText(probe)), font.width(meta)));
        return Math.max(MIN_WIDTH, contentWidth + 12);
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return HEIGHT;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        LivingEntity target = target(client);
        if (target == null) {
            return;
        }

        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();
        int width = width(client, config);

        float maxHealth = Math.max(1.0f, target.getMaxHealth());
        float health = Math.max(0.0f, Math.min(maxHealth, target.getHealth()));
        float ratio = health / maxHealth;
        String name = shortened(target.getDisplayName().getString());
        String healthText = healthText(target);

        context.fill(0, 0, width, HEIGHT, RemixDraw.panel(backgroundAlpha));
        context.fill(0, 0, width, 1, RemixDraw.healthColor(ratio));

        context.text(font, name, 6, 5, RemixDraw.TEXT, true);
        context.text(font, healthText, width - 6 - font.width(healthText), 5,
                RemixDraw.healthColor(ratio), true);

        // Health bar: a dark groove with the filled part on top.
        int barLeft = 6;
        int barRight = width - 6;
        int barTop = 17;
        context.fill(barLeft, barTop, barRight, barTop + 4, 0xC0101014);
        int filled = Math.round((barRight - barLeft) * ratio);
        if (filled > 0) {
            context.fill(barLeft, barTop, barLeft + filled, barTop + 4,
                    RemixDraw.alpha(RemixDraw.healthColor(ratio), backgroundAlpha));
        }

        context.text(font, metaText(target, client), 6, 24, RemixDraw.TEXT_DIM, true);
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();
        int width = width(client, config);
        return new Origin(Math.max(4, (screenWidth - width) / 2), Math.max(4, screenHeight - 66 - HEIGHT));
    }

    private static String metaText(LivingEntity target, Minecraft client) {
        float distance = client.player == null ? 0.0f : client.player.distanceTo(target);
        return Component.translatable("remix_companion.target.armor").getString()
                + " " + target.getArmorValue()
                + "  " + Component.translatable("remix_companion.target.distance").getString()
                + " " + String.format(Locale.ROOT, "%.1f", distance);
    }

    private static String healthText(LivingEntity target) {
        float maxHealth = Math.max(1.0f, target.getMaxHealth());
        float health = Math.max(0.0f, Math.min(maxHealth, target.getHealth()));
        return String.format(Locale.ROOT, "%.1f / %.1f", health, maxHealth);
    }

    private static String shortened(String name) {
        if (name.length() <= MAX_NAME_CHARS) {
            return name;
        }
        int end = MAX_NAME_CHARS - 3;
        if (end > 0 && Character.isHighSurrogate(name.charAt(end - 1))) {
            end--;
        }
        return name.substring(0, end) + "...";
    }
}
