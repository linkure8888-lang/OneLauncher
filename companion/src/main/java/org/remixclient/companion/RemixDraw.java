package org.remixclient.companion;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;

/**
 * Shared colours and tiny drawing helpers for every Remix module.
 *
 * <p>Modules draw with plain {@code fill}/{@code text} calls so they all share one flat look and
 * none of them depends on vanilla widget or texture internals.</p>
 */
final class RemixDraw {

    static final int ACCENT = 0xFF7C5CFF;
    static final int TEXT = 0xFFF2F2F7;
    static final int TEXT_DIM = 0xFF9A9AAC;
    static final int GREEN = 0xFF55E06B;
    static final int YELLOW = 0xFFE8C33A;
    static final int RED = 0xFFE85C5C;
    static final int GOLD = 0xFFE0B84A;

    /** Full-screen flash colour used when a totem is consumed. */
    static final int TOTEM_FLASH = 0xFFC24A;

    /** Near-black panel fill, without alpha so callers can pick their own. */
    private static final int PANEL_RGB = 0x0A0A0E;

    private RemixDraw() {
    }

    /** Panel background with the configured opacity (0-255). */
    static int panel(int alpha) {
        return (Math.max(0, Math.min(255, alpha)) << 24) | PANEL_RGB;
    }

    /** Replaces the alpha channel of an opaque colour. */
    static int alpha(int rgb, int alpha) {
        int a = Math.max(0, Math.min(255, alpha));
        return (a << 24) | (rgb & 0xFFFFFF);
    }

    /** Full-opacity ARGB for a packed {@code 0xRRGGBB} colour. */
    static int opaque(int rgb) {
        return 0xFF000000 | (rgb & 0xFFFFFF);
    }

    /** Colours a 0..1 health ratio green → yellow → red. */
    static int healthColor(float ratio) {
        if (ratio > 0.6f) {
            return GREEN;
        }
        if (ratio >= 0.3f) {
            return YELLOW;
        }
        return RED;
    }

    /** One line of text on a panel, with the panel exactly as wide as the text. */
    static void line(GuiGraphicsExtractor context, Font font, String text, int x, int y,
                     int color, int backgroundAlpha) {
        int width = font.width(text);
        context.fill(x - 2, y - 1, x + width + 2, y + 9, panel(backgroundAlpha));
        context.text(font, text, x, y, color, true);
    }

    /** Right-aligned variant of {@link #line}, {@code right} being the right edge. */
    static void lineRight(GuiGraphicsExtractor context, Font font, String text, int right, int y,
                          int color, int backgroundAlpha) {
        line(context, font, text, right - font.width(text), y, color, backgroundAlpha);
    }
}
