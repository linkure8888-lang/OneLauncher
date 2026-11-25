package org.remixclient.companion;

import net.minecraft.client.Minecraft;

/**
 * Full brightness toggle. Raises the vanilla gamma option while it is on and puts the player's
 * own value back when it is switched off, so a session cannot permanently wash out the game.
 */
final class RemixVisuals {

    private static final double FULLBRIGHT_GAMMA = 10.0;
    private static final double DEFAULT_GAMMA = 0.5;

    private static boolean fullBright;
    private static double savedGamma = Double.NaN;

    private RemixVisuals() {
    }

    static void toggle(Minecraft client) {
        try {
            setFullBright(client, !fullBright);
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error("[Remix Companion] Full brightness toggle failed.", t);
        }
    }

    static void setFullBright(Minecraft client, boolean enabled) {
        if (enabled == fullBright) {
            return;
        }
        if (enabled) {
            savedGamma = client.options.gamma().get();
            client.options.gamma().set(FULLBRIGHT_GAMMA);
        } else {
            // A NaN can only appear if the option read threw on enable; falling back to the
            // vanilla default is better than writing a broken gamma into options.txt.
            double restore = Double.isNaN(savedGamma) ? DEFAULT_GAMMA : savedGamma;
            client.options.gamma().set(Math.max(0.0, Math.min(1.0, restore)));
            savedGamma = Double.NaN;
        }
        fullBright = enabled;
        RemixCompanionClient.LOGGER.info("[Remix Companion] Full brightness {}.",
                enabled ? "enabled" : "disabled");
    }

    static boolean isFullBright() {
        return fullBright;
    }
}
