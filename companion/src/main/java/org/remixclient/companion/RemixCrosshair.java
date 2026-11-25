package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.LivingEntity;

/**
 * Replaces the vanilla crosshair with a configurable one: gap, arm length, thickness, optional
 * dot and outline, plus a tint while a living entity is under it.
 *
 * <p>The vanilla crosshair can only be removed from the HUD itself, so {@code HudMixin} cancels
 * it and calls {@link #markVanillaHidden()}. Nothing is drawn here until that has happened at
 * least once, which means a mixin that failed to apply leaves the game's own crosshair alone
 * instead of stacking two of them.</p>
 */
public final class RemixCrosshair implements RemixModule {

    static final RemixCrosshair INSTANCE = new RemixCrosshair();

    /** Largest possible extent, so the editor frame never shrinks below the real crosshair. */
    private static final int BOX = 2 * (RemixConfig.MAX_GAP + RemixConfig.MAX_LENGTH) + 3;

    private static volatile boolean vanillaHidden;

    private RemixCrosshair() {
    }

    /** Called from the mixin right after it cancelled the vanilla crosshair. */
    public static void markVanillaHidden() {
        vanillaHidden = true;
    }

    @Override
    public String id() {
        return "crosshair";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.crosshair");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showCrosshair;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showCrosshair = enabled;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        return BOX;
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return BOX;
    }

    /** The crosshair belongs to the crosshair position, so it cannot be dragged around. */
    @Override
    public boolean movable() {
        return false;
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();
        RemixConfig.Placement placement = config.placement(id());
        int size = Math.round(BOX * placement.scale);
        return new Origin((screenWidth - size) / 2, (screenHeight - size) / 2);
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        if (!config.showCrosshair || !vanillaHidden) {
            return;
        }

        int center = BOX / 2;
        int rgb = config.crosshairColor;
        if (config.crosshairHitTint && client.crosshairPickEntity instanceof LivingEntity) {
            rgb = config.crosshairHitColor;
        }
        int color = RemixDraw.opaque(rgb);

        // Two passes: a slightly fatter, darker copy for contrast on bright blocks, then
        // the real crosshair on top.
        if (config.crosshairOutline) {
            drawArms(context, center, config.crosshairGap, config.crosshairLength + 1,
                    config.crosshairThickness + 2, 0xB0000000);
        }

        // A gap of zero turns the four arms into one continuous cross, so they are
        // always drawn — only the dot is optional on top of them.
        drawArms(context, center, config.crosshairGap, config.crosshairLength,
                config.crosshairThickness, color);

        if (config.crosshairDot) {
            int radius = Math.max(1, config.crosshairThickness / 2 + 1);
            context.fill(center - radius, center - radius, center + radius, center + radius, color);
        }
    }

    private static void drawArms(GuiGraphicsExtractor context, int center, int gap, int length,
                                 int thickness, int color) {
        int half = thickness / 2;

        // Vertical arms.
        context.fill(center - half, center - gap - length, center - half + thickness,
                center - gap, color);
        context.fill(center - half, center + gap, center - half + thickness,
                center + gap + length, color);

        // Horizontal arms.
        context.fill(center - gap - length, center - half, center - gap,
                center - half + thickness, color);
        context.fill(center + gap, center - half, center + gap + length,
                center - half + thickness, color);
    }
}
