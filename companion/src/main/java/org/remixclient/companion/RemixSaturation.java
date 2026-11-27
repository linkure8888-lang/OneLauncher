package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.player.Inventory;

/**
 * Saturation HUD: ten hunger squares, the exact saturation value and the air supply when it
 * matters (underwater or falling), all in one flat row that matches the rest of the HUD.
 *
 * <p>Vanilla hides saturation completely; here the player finally sees the number PvP players
 * watch in other clients.</p>
 */
final class RemixSaturation implements RemixModule {

    static final RemixSaturation INSTANCE = new RemixSaturation();

    private static final int DRUMSTICKS = 10;
    private static final int SQUARE = 4;
    private static final int SQUARE_GAP = 1;
    private static final int PADDING = 2;
    private static final int ROWS = 2;
    private static final int ROW_HEIGHT = 10;
    private static final int MIN_WIDTH = 64;

    private RemixSaturation() {
    }

    @Override
    public String id() {
        return "saturation";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.saturation");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showSaturation;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showSaturation = enabled;
    }

    /** Food level (0-20), saturation (0-20) and air percent (0-100) for the current state. */
    private record FoodState(int food, float saturation, int airPercent, boolean showAir) {
        static FoodState of(LocalPlayer player, boolean preview) {
            if (player == null) {
                return new FoodState(20, 5f, 100, false);
            }
            var data = player.getFoodData();
            int air = player.getMaxAirSupply() <= 0 ? 100
                    : Math.round(player.getAirSupply() * 100f / player.getMaxAirSupply());
            // Air is only meaningful while it can run out, so the line stays hidden
            // on dry land and never wastes HUD space.
            boolean showAir = !preview && air < 100;
            return new FoodState(data.getFoodLevel(), data.getSaturationLevel(), air, showAir);
        }
    }

    private String text(FoodState state) {
        String food = Component.translatable("remix_companion.hud.food").getString();
        String sat = Component.translatable("remix_companion.hud.sat").getString();
        String line = food + " " + state.food() + "/20 · " + sat + " "
                + String.format("%.1f", state.saturation());
        if (state.showAir()) {
            String air = Component.translatable("remix_companion.hud.air").getString();
            line += " · " + air + " " + state.airPercent() + "%";
        }
        return line;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        Font font = client.font;
        String line = text(FoodState.of(client.player, false));
        int squares = DRUMSTICKS * (SQUARE + SQUARE_GAP) - SQUARE_GAP;
        return Math.max(MIN_WIDTH, Math.max(squares, font.width(line)) + PADDING * 2);
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return ROWS * ROW_HEIGHT;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        LocalPlayer player = client.player;
        FoodState state = FoodState.of(player, player == null);
        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();
        int squares = DRUMSTICKS * (SQUARE + SQUARE_GAP) - SQUARE_GAP;
        int width = Math.max(MIN_WIDTH, Math.max(squares, font.width(text(state))) + PADDING * 2);

        context.fill(0, 0, width, height(client, config), RemixDraw.panel(backgroundAlpha));

        // One square per drumstick: the filled part follows the real food value, so a 13/20
        // hunger shows 6.5 squares of green instead of lying in whole icons.
        int food = state.food();
        for (int i = 0; i < DRUMSTICKS; i++) {
            int x = PADDING + i * (SQUARE + SQUARE_GAP);
            float filled = Math.max(0f, Math.min(1f, food / 2f - i));
            int color;
            if (filled >= 1f) {
                color = RemixDraw.GREEN;
            } else if (filled > 0f) {
                color = RemixDraw.YELLOW;
            } else {
                color = RemixDraw.RED;
            }
            int height = Math.max(1, Math.round(SQUARE * filled));
            context.fill(x, PADDING + (SQUARE - height), x + SQUARE, PADDING + SQUARE,
                    RemixDraw.opaque(color));
        }

        String line = text(state);
        // Saturation reads best in gold: it is the resource competitive players actually watch.
        context.text(font, line, PADDING, PADDING + SQUARE + 2,
                state.saturation() > 0 ? RemixDraw.GOLD : RemixDraw.TEXT_DIM, true);
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int height = height(client, config);
        return new Origin(4, Math.max(4, client.getWindow().getGuiScaledHeight() - 58 - height - 10));
    }
}
