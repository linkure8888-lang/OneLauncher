package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;

/**
 * One movable HUD module.
 *
 * <p>Every module draws itself in its own coordinate space with {@code (0, 0)} as the top-left
 * corner: the HUD applies the configured position and scale around that corner. Bounds are
 * reported in unscaled units, so the editor can hit-test and the layout stays stable across
 * GUI scales.</p>
 */
interface RemixModule {

    /** Stable id, used as the key in the config file. Never change it once released. */
    String id();

    /** Label shown on the frame in the HUD editor. */
    Component title();

    boolean enabled(RemixConfig config);

    void setEnabled(RemixConfig config, boolean enabled);

    /** Width in unscaled units. May depend on the current content. */
    int width(Minecraft client, RemixConfig config);

    /** Height in unscaled units. May depend on the current content. */
    int height(Minecraft client, RemixConfig config);

    /** Draws the module with its top-left corner at {@code (0, 0)}. */
    void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config);

    /** Where the module sits before the player moves it. */
    Origin defaultOrigin(Minecraft client, RemixConfig config);

    /** Crosshair-like modules are locked to their default spot. */
    default boolean movable() {
        return true;
    }

    /** GUI-scaled pixel position of a module's top-left corner. */
    record Origin(int x, int y) {
    }
}
