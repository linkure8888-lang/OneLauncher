package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;

/**
 * WASD / space / mouse keystrokes panel with a clicks-per-second counter on the mouse keys.
 *
 * <p>Presses are sampled on the client tick, so a key that was tapped between two frames would
 * otherwise never look pressed; {@link #HOLD_MS} keeps it lit slightly longer than the tap.</p>
 */
final class RemixKeystrokes implements RemixModule {

    static final RemixKeystrokes INSTANCE = new RemixKeystrokes();

    private static final int BOX = 20;
    private static final int GAP = 2;
    private static final long HOLD_MS = 70L;
    private static final int SAMPLES = 16;

    private static final ClickCounter ATTACK = new ClickCounter();
    private static final ClickCounter USE = new ClickCounter();

    private static long upHeldUntil;
    private static long leftHeldUntil;
    private static long downHeldUntil;
    private static long rightHeldUntil;
    private static long jumpHeldUntil;

    private static boolean lastAttack;
    private static boolean lastUse;

    private RemixKeystrokes() {
    }

    @Override
    public String id() {
        return "keystrokes";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.keystrokes");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showKeystrokes;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showKeystrokes = enabled;
    }

    /** Samples the raw key states once per client tick. */
    static void tick(Minecraft client) {
        long now = System.currentTimeMillis();

        upHeldUntil = hold(client.options.keyUp.isDown(), upHeldUntil, now);
        leftHeldUntil = hold(client.options.keyLeft.isDown(), leftHeldUntil, now);
        downHeldUntil = hold(client.options.keyDown.isDown(), downHeldUntil, now);
        rightHeldUntil = hold(client.options.keyRight.isDown(), rightHeldUntil, now);
        jumpHeldUntil = hold(client.options.keyJump.isDown(), jumpHeldUntil, now);

        boolean attack = client.options.keyAttack.isDown();
        if (attack && !lastAttack) {
            ATTACK.press(now);
        }
        lastAttack = attack;

        boolean use = client.options.keyUse.isDown();
        if (use && !lastUse) {
            USE.press(now);
        }
        lastUse = use;
    }

    /** Forget previous state when the module is switched off, so it does not pop back lit. */
    static void reset() {
        upHeldUntil = 0L;
        leftHeldUntil = 0L;
        downHeldUntil = 0L;
        rightHeldUntil = 0L;
        jumpHeldUntil = 0L;
        lastAttack = false;
        lastUse = false;
        ATTACK.clear();
        USE.clear();
    }

    private static long hold(boolean down, long heldUntil, long now) {
        return down ? now + HOLD_MS : heldUntil;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        return BOX * 3 + GAP * 2;
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return BOX * 5 + GAP * 4;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();
        long now = System.currentTimeMillis();
        int rowWidth = width(client, config);

        // Row 1: W centred over the A/S/D row.
        box(context, font, BOX + GAP, 0, BOX, BOX,
                Component.translatable("remix_companion.keys.forward").getString(), null,
                now < upHeldUntil, backgroundAlpha);

        int second = BOX + GAP;
        box(context, font, 0, second, BOX, BOX,
                Component.translatable("remix_companion.keys.left").getString(), null,
                now < leftHeldUntil, backgroundAlpha);
        box(context, font, BOX + GAP, second, BOX, BOX,
                Component.translatable("remix_companion.keys.back").getString(), null,
                now < downHeldUntil, backgroundAlpha);
        box(context, font, (BOX + GAP) * 2, second, BOX, BOX,
                Component.translatable("remix_companion.keys.right").getString(), null,
                now < rightHeldUntil, backgroundAlpha);

        int third = second + BOX + GAP;
        box(context, font, 0, third, rowWidth, BOX,
                Component.translatable("remix_companion.keys.jump").getString(), null,
                now < jumpHeldUntil, backgroundAlpha);

        int fourth = third + BOX + GAP;
        int mouseWidth = (rowWidth - GAP) / 2;
        box(context, font, 0, fourth, mouseWidth, BOX,
                Component.translatable("remix_companion.keys.attack").getString(),
                Integer.toString(ATTACK.cps(now)), now < ATTACK.heldUntil, backgroundAlpha);
        box(context, font, mouseWidth + GAP, fourth, mouseWidth, BOX,
                Component.translatable("remix_companion.keys.use").getString(),
                Integer.toString(USE.cps(now)), now < USE.heldUntil, backgroundAlpha);
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int screenHeight = client.getWindow().getGuiScaledHeight();
        return new Origin(4, Math.max(4, (screenHeight - height(client, config)) / 2));
    }

    private static void box(GuiGraphicsExtractor context, Font font, int x, int y, int width,
                            int height, String label, String subLabel, boolean pressed,
                            int backgroundAlpha) {
        int fill = pressed
                ? RemixDraw.alpha(RemixDraw.ACCENT, Math.max(140, backgroundAlpha))
                : RemixDraw.panel(backgroundAlpha);
        context.fill(x, y, x + width, y + height, fill);
        if (pressed) {
            context.fill(x, y, x + width, y + 1, RemixDraw.TEXT);
        }

        int color = pressed ? 0xFFFFFFFF : RemixDraw.TEXT_DIM;
        int labelWidth = font.width(label);
        int labelY = subLabel == null ? y + (height - 8) / 2 : y + 2;
        context.text(font, label, x + (width - labelWidth) / 2, labelY, color, true);

        if (subLabel != null) {
            int subWidth = font.width(subLabel);
            context.text(font, subLabel, x + (width - subWidth) / 2, y + 11,
                    pressed ? 0xFFFFFFFF : RemixDraw.TEXT, true);
        }
    }

    /** Ring buffer of press timestamps; remembers the last press for the lit state. */
    private static final class ClickCounter {
        private final long[] times = new long[SAMPLES];
        private int cursor;
        private long heldUntil;

        void press(long now) {
            times[cursor] = now;
            cursor = (cursor + 1) % SAMPLES;
            heldUntil = now + HOLD_MS;
        }

        int cps(long now) {
            int clicks = 0;
            for (long time : times) {
                if (time != 0L && now - time >= 0L && now - time < 1000L) {
                    clicks++;
                }
            }
            return clicks;
        }

        void clear() {
            for (int i = 0; i < times.length; i++) {
                times[i] = 0L;
            }
            cursor = 0;
            heldUntil = 0L;
        }
    }
}
