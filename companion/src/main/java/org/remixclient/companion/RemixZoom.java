package org.remixclient.companion;

import net.minecraft.client.Minecraft;

/**
 * Smooth hold-to-zoom, the OptiFine muscle memory without a render mixin.
 *
 * <p>The FOV option itself is animated: while the zoom key is held the saved value is scaled
 * down along an eased curve, and on release it eases back and the player's own FOV is written
 * back untouched. A session cannot end with the zoomed FOV saved into options.txt.</p>
 */
final class RemixZoom {

    /** Zoom target as a fraction of the player's own FOV. */
    private static final double TARGET_FACTOR = 0.25;
    /** Per-tick approach speed; ~0.35 lands the ease in a few frames without overshoot. */
    private static final double EASE = 0.35;
    private static final double EPSILON = 0.005;

    /** The FOV the player had before the first zoom tick of a hold; NaN while idle. */
    private static double originalFov = Double.NaN;
    private static double factor = 1.0;

    /** Last value written into the option, so the restore can skip redundant writes. */
    private static int writtenFov = Integer.MIN_VALUE;

    private RemixZoom() {
    }

    /** Called every client tick with the live state of the zoom key. */
    static void tick(Minecraft client, boolean held) {
        if (client == null || client.options == null) {
            return;
        }
        try {
            if (!held && factor >= 1.0 - EPSILON) {
                // Fully released and back at the player's own FOV: nothing left to do.
                if (!Double.isNaN(originalFov)) {
                    write(client, clampFov(originalFov));
                    originalFov = Double.NaN;
                }
                return;
            }

            if (Double.isNaN(originalFov)) {
                originalFov = client.options.fov().get();
            }

            double target = held ? TARGET_FACTOR : 1.0;
            factor += (target - factor) * EASE;
            if (Math.abs(target - factor) < EPSILON) {
                factor = target;
            }

            double animated = originalFov * factor;
            // While easing back out, land exactly on the saved value so options.txt
            // never sees a rounding artefact.
            if (!held && factor == 1.0) {
                animated = originalFov;
            }
            write(client, clampFov(animated));
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error("[Remix Companion] Zoom tick failed.", t);
            originalFov = Double.NaN;
            factor = 1.0;
        }
    }

    /** Forget an in-flight zoom, e.g. when the game is closing. */
    static void reset() {
        originalFov = Double.NaN;
        factor = 1.0;
    }

    /** The FOV option is an integer in 26.3, so animated steps land on whole degrees. */
    private static int clampFov(double fov) {
        return (int) Math.round(Math.max(30.0, Math.min(110.0, fov)));
    }

    private static void write(Minecraft client, int fov) {
        if (fov != writtenFov) {
            client.options.fov().set(fov);
            writtenFov = fov;
        }
    }
}
