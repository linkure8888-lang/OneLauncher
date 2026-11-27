package org.remixclient.companion;

import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.event.client.player.ClientPreAttackCallback;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import org.polyfrost.oneconfig.api.ui.v1.OneConfigUI;
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
    public static final String KEY_FULLBRIGHT = "key.remix_companion.fullbright";
    public static final String KEY_ZOOM = "key.remix_companion.zoom";
    public static final String KEY_EDITOR = "key.remix_companion.editor";

    private static RemixConfig config = new RemixConfig();
    private static KeyMapping openSettingsKey;
    private static KeyMapping fullBrightKey;
    private static KeyMapping zoomKey;
    private static KeyMapping editorKey;
    private static boolean hudRegistered;
    private static boolean ticksRegistered;
    private static boolean descriptionsApplied;

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

        // Hand the config page over to OneConfig: it builds the tree, applies its own
        // persisted store on top and lists the page in the GUI.
        try {
            config.preload();
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the OneConfig page; "
                    + "settings stay editable only through remix-companion.json.", t);
        }

        registerKeyBinding();
        registerHud();
        registerCombatEvents();
    }

    private void registerKeyBinding() {
        KeyMapping.Category category = null;
        try {
            category = KeyMapping.Category.register(
                    Identifier.fromNamespaceAndPath(MOD_ID, "controls"));
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
            if (category != null) {
                fullBrightKey = KeyMappingHelper.registerKeyMapping(new KeyMapping(
                        KEY_FULLBRIGHT,
                        InputConstants.Type.KEYBOARD,
                        InputConstants.KEY_B,
                        category));
            }
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the full brightness keybinding.", t);
            fullBrightKey = null;
        }

        try {
            if (category != null) {
                // G, not C: Zoomify (bundled in the content packs) already owns C, and
                // two zooms fighting over the FOV looks broken.
                zoomKey = KeyMappingHelper.registerKeyMapping(new KeyMapping(
                        KEY_ZOOM,
                        InputConstants.Type.KEYBOARD,
                        InputConstants.KEY_G,
                        category));
            }
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the zoom keybinding.", t);
            zoomKey = null;
        }

        try {
            if (category != null) {
                editorKey = KeyMappingHelper.registerKeyMapping(new KeyMapping(
                        KEY_EDITOR,
                        InputConstants.Type.KEYBOARD,
                        InputConstants.KEY_H,
                        category));
            }
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the HUD editor keybinding.", t);
            editorKey = null;
        }

        try {
            // Ticking is needed for the keybinding poll, the keystroke sampler and the
            // totem-popped detector, so it stays registered even when a keybinding failed.
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
            // OneConfig resolves the language late in the boot, so the description
            // rewrite retries until the tree is ready, then runs exactly once.
            descriptionsApplied = RemixDescriptions.apply(config);
            if (openSettingsKey != null && consume(openSettingsKey)) {
                openMenu(client);
            }
            if (editorKey != null && consume(editorKey)) {
                openEditor(client);
            }
            if (fullBrightKey != null && consume(fullBrightKey)) {
                RemixVisuals.toggle(client);
            } else if (config.fullBrightness != RemixVisuals.isFullBright()) {
                // The OneConfig switch was flipped; apply it before mirroring back.
                RemixVisuals.setFullBright(client, config.fullBrightness);
            }
            // Mirror the live state into the field the OneConfig switch reads.
            config.fullBrightness = RemixVisuals.isFullBright();
            // Zoom is hold-to-zoom, so it reads the live key state instead of consumeClick.
            RemixZoom.tick(client, zoomKey != null && zoomKey.isDown());
            RemixTrails.tick(client, config);
            if (config.showKeystrokes) {
                RemixKeystrokes.tick(client);
            }
            RemixJumpCircles.tick(client, config);
            RemixHud.tick(client);
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Tick handler failed.", t);
        }
    }

    /** Hit particles fire from the pre-attack event, which is client-side only. */
    private void registerCombatEvents() {
        try {
            ClientPreAttackCallback.EVENT.register((client, player, slot) -> {
                RemixHitEffects.onAttack(client, player, config);
                RemixHitsound.onAttack(client, player, config);
                // Never cancel: the swing itself must always go through.
                return false;
            });
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not register the attack callback; "
                    + "hit particles stay off.", t);
        }
    }

    /** Presses are drained whether or not they are acted on, so a queued tap never fires later. */
    private static boolean consume(KeyMapping mapping) {
        boolean pressed = false;
        while (mapping.consumeClick()) {
            pressed = true;
        }
        return pressed;
    }

    /** Opens the OneConfig GUI, where the Remix Client page lives. */
    public static void openMenu(Minecraft client) {
        if (client == null) {
            return;
        }
        try {
            OneConfigUI.INSTANCE.open();
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not open the OneConfig GUI.", t);
        }
    }

    /** Opens the HUD editor, unless another screen is already up. */
    public static void openEditor(Minecraft client) {
        if (client == null || client.gui.screen() != null) {
            return;
        }
        try {
            client.setScreenAndShow(new RemixHudEditorScreen(config));
        } catch (Throwable t) {
            LOGGER.error("[Remix Companion] Could not open the HUD editor.", t);
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

    public static boolean isFullBright() {
        return RemixVisuals.isFullBright();
    }
}
