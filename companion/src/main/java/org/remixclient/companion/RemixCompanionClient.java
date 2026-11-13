package org.remixclient.companion;

import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Client entry point for Remix Companion.
 *
 * <p>Everything is registered defensively: if any single piece fails (missing Fabric API module,
 * broken keybinding, ...) the failure is logged and the rest of the game keeps running untouched.</p>
 */
public class RemixCompanionClient implements ClientModInitializer {

    public static final String MOD_ID = "remix_companion";
    public static final Logger LOGGER = LoggerFactory.getLogger("Remix Companion");

    /** Translation key of the keybinding category shown in Options → Controls. */
    public static final String KEY_CATEGORY = "key.category.remix_companion.controls";
    public static final String KEY_SETTINGS = "key.remix_companion.settings";

    private static RemixConfig config = new RemixConfig();
    private static KeyMapping openSettingsKey;
    private static boolean hudRegistered;
    private static boolean ticksRegistered;

    @Override
    public void onInitializeClient() {
        try {
            config = RemixConfig.load();
            LOGGER.info("[Remix Companion] Config ready at {}", RemixConfig.file());
        } catch (Throwable t) {
            // Should never happen (load() catches everything), but never let it kill startup.
            LOGGER.error("[Remix Companion] Config init failed, using in-memory defaults.", t);
            config = new RemixConfig();
        }

        registerKeyBinding();
        registerHud();
    }

    private void registerKeyBinding() {
        try {
            KeyMapping.Category category =
                    KeyMapping.Category.register(Identifier.fromNamespaceAndPath(MOD_ID, "controls"));
            openSettingsKey = KeyMappingHelper.registerKeyMapping(new KeyMapping(
                    KEY_SETTINGS,
                    InputConstants.Type.KEYBOARD,
                    InputConstants.KEY_RALT,
                    category));
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the Right Alt keybinding; "
                    + "open settings from the mod list instead.", t);
            openSettingsKey = null;
        }

        try {
            // Ticking is needed for the keybinding poll and for the totem-popped detector,
            // so it stays registered even when the keybinding itself failed.
            ClientTickEvents.END_CLIENT_TICK.register(RemixCompanionClient::onEndTick);
            ticksRegistered = true;
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the client tick callback.", t);
        }
    }

    private void registerHud() {
        try {
            // 26.3 registers HUD layers instead of the removed HudRenderCallback.
            HudElementRegistry.addLast(
                    Identifier.fromNamespaceAndPath(MOD_ID, "hud"),
                    RemixHud::render);
            hudRegistered = true;
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the HUD renderer.", t);
        }
    }

    private static void onEndTick(Minecraft client) {
        try {
            if (openSettingsKey != null) {
                boolean pressed = false;
                // Always drain the queue, otherwise a press swallowed by an open screen
                // would fire the moment that screen closes.
                while (openSettingsKey.consumeClick()) {
                    pressed = true;
                }
                if (pressed) {
                    openSettings(client);
                }
            }
            RemixHud.tick(client);
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Tick handler failed.", t);
        }
    }

    /** Opens the settings screen, unless another screen is already up. */
    public static void openSettings(Minecraft client) {
        if (client == null || client.gui.screen() != null) {
            return;
        }
        try {
            client.setScreenAndShow(new RemixSettingsScreen(config));
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not open the settings screen.", t);
        }
    }

    /** Never null: falls back to in-memory defaults before {@link #onInitializeClient()} runs. */
    public static RemixConfig getConfig() {
        return config;
    }

    public static boolean isHudRegistered() {
        return hudRegistered;
    }

    public static boolean areTicksRegistered() {
        return ticksRegistered;
    }
}