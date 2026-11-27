package org.remixclient.companion;

import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import org.joml.Matrix3x2fStack;

import java.util.List;

/**
 * The Remix Companion HUD layer: owns the module list, applies each module's saved position and
 * scale, and hands the frames to the editor.
 *
 * <p>The whole renderer disables itself permanently after the first fatal failure, and each
 * module runs isolated, so one broken panel can never take the others (or the frame) down.</p>
 */
public final class RemixHud {

    private static final List<RemixModule> MODULES = List.of(
            RemixStatus.INSTANCE,
            RemixKeystrokes.INSTANCE,
            RemixKeybinds.INSTANCE,
            RemixTargetHud.INSTANCE,
            RemixPotions.INSTANCE,
            RemixSaturation.INSTANCE,
            RemixPickups.INSTANCE,
            RemixWatermark.INSTANCE,
            RemixCrosshair.INSTANCE);

    private static boolean broken;

    private RemixHud() {
    }

    /** Called once per client tick. */
    public static void tick(Minecraft client) {
        RemixStatus.INSTANCE.tick(client);
        RemixPickups.INSTANCE.tick(client);
    }

    static List<RemixModule> modules() {
        return MODULES;
    }

    /** HUD layer entry point; matches Fabric's {@code HudElement}. */
    public static void render(GuiGraphicsExtractor context, DeltaTracker delta) {
        if (broken) {
            return;
        }
        try {
            renderSafely(context);
        } catch (Throwable t) {
            broken = true;
            RemixCompanionClient.LOGGER.error("[Remix Companion] HUD rendering failed; "
                    + "the Remix HUD has been disabled for this session.", t);
        }
    }

    private static void renderSafely(GuiGraphicsExtractor context) {
        Minecraft client = Minecraft.getInstance();
        if (client.player == null || client.gui.hud.isHidden()) {
            return;
        }

        RemixConfig config = RemixCompanionClient.getConfig();
        // While the editor is open every module is drawn, disabled ones included, so they
        // can be switched back on and positioned.
        boolean editorOpen = client.gui.screen() instanceof RemixHudEditorScreen;

        runIsolated("totem flash", () -> RemixStatus.INSTANCE.renderFlash(context, client));

        for (RemixModule module : MODULES) {
            if (!module.enabled(config) && !editorOpen) {
                continue;
            }
            runIsolated(module.id(), () -> drawModule(context, client, config, module));
        }
    }

    private static void drawModule(GuiGraphicsExtractor context, Minecraft client,
                                   RemixConfig config, RemixModule module) {
        RemixConfig.Placement placement = config.placement(module.id());
        Bounds bounds = boundsOf(module, client, config);
        Matrix3x2fStack pose = context.pose();
        pose.pushMatrix();
        try {
            pose.translate(bounds.x(), bounds.y());
            pose.scale(placement.scale);
            module.render(context, client, config);
        } finally {
            // An exception mid-draw must not leave the transform applied to the rest
            // of the frame (chat and screens would render in the wrong place).
            pose.popMatrix();
        }
    }

    /** Resolved top-left corner in GUI-scaled pixels. */
    static RemixModule.Origin originOf(RemixModule module, Minecraft client, RemixConfig config) {
        RemixConfig.Placement placement = config.placement(module.id());
        if (module.movable() && placement.custom()) {
            return new RemixModule.Origin(Math.round(placement.x), Math.round(placement.y));
        }
        return module.defaultOrigin(client, config);
    }

    /** Resolved on-screen rectangle in GUI-scaled pixels, scale already applied. */
    static Bounds boundsOf(RemixModule module, Minecraft client, RemixConfig config) {
        RemixConfig.Placement placement = config.placement(module.id());
        RemixModule.Origin origin = originOf(module, client, config);
        int width = Math.max(1, Math.round(module.width(client, config) * placement.scale));
        int height = Math.max(1, Math.round(module.height(client, config) * placement.scale));
        return new Bounds(origin.x(), origin.y(), width, height);
    }

    /** Topmost module under the cursor, or {@code null}. */
    static RemixModule under(Minecraft client, RemixConfig config, double mouseX, double mouseY) {
        for (int i = MODULES.size() - 1; i >= 0; i--) {
            RemixModule module = MODULES.get(i);
            if (boundsOf(module, client, config).contains(mouseX, mouseY)) {
                return module;
            }
        }
        return null;
    }

    /** Runs one module, logging the first failure instead of taking the HUD down with it. */
    private static void runIsolated(String name, Runnable module) {
        try {
            module.run();
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error(
                    "[Remix Companion] The {} panel failed and was skipped this frame.", name, t);
        }
    }

    /** Integer rectangle in GUI-scaled coordinates. */
    record Bounds(int x, int y, int width, int height) {
        boolean contains(double mouseX, double mouseY) {
            return mouseX >= x && mouseX < x + width && mouseY >= y && mouseY < y + height;
        }
    }
}
