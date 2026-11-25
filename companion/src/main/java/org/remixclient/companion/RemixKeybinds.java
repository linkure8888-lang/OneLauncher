package org.remixclient.companion;

import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;

/**
 * Second key panel for the keys that are not movement: sneak, sprint and offhand swap. Kept apart
 * from {@link RemixKeystrokes} so the WASD block stays compact.
 */
final class RemixKeybinds implements RemixModule {

    static final RemixKeybinds INSTANCE = new RemixKeybinds();

    private static final int WIDTH = 52;
    private static final int HEIGHT = 16;
    private static final int GAP = 2;
    private static final int ROWS = 3;

    private RemixKeybinds() {
    }

    @Override
    public String id() {
        return "keybinds";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.keybinds");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showKeybinds;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showKeybinds = enabled;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        return WIDTH;
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return HEIGHT * ROWS + GAP * (ROWS - 1);
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        int backgroundAlpha = config.backgroundAlpha();
        Font font = client.font;

        row(context, font, 0, 0, client.options.keyShift, "remix_companion.keybinds.sneak",
                backgroundAlpha);
        row(context, font, 0, HEIGHT + GAP, client.options.keySprint,
                "remix_companion.keybinds.sprint", backgroundAlpha);
        row(context, font, 0, (HEIGHT + GAP) * 2, client.options.keySwapOffhand,
                "remix_companion.keybinds.offhand", backgroundAlpha);
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int screenHeight = client.getWindow().getGuiScaledHeight();
        int keystrokesTop = Math.max(4, (screenHeight - RemixKeystrokes.INSTANCE.height(client, config)) / 2);
        return new Origin(4, keystrokesTop + RemixKeystrokes.INSTANCE.height(client, config) + 4);
    }

    private static void row(GuiGraphicsExtractor context, Font font, int x, int y, KeyMapping key,
                            String translationKey, int backgroundAlpha) {
        boolean pressed = key != null && key.isDown();
        context.fill(x, y, x + WIDTH, y + HEIGHT,
                pressed ? RemixDraw.alpha(RemixDraw.ACCENT, Math.max(140, backgroundAlpha))
                        : RemixDraw.panel(backgroundAlpha));
        context.fill(x, y, x + 2, y + HEIGHT, pressed ? RemixDraw.TEXT : RemixDraw.ACCENT);

        String label = Component.translatable(translationKey).getString();
        context.text(font, label, x + 6, y + (HEIGHT - 8) / 2,
                pressed ? 0xFFFFFFFF : RemixDraw.TEXT_DIM, true);
    }
}
