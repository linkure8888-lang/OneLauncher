package org.remixclient.companion;

import com.mojang.blaze3d.platform.InputConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.network.chat.Component;

/**
 * OneConfig-style HUD editor: every module gets a frame you can grab.
 *
 * <ul>
 *   <li>right-drag a frame — move the module;</li>
 *   <li>mouse wheel over a frame — scale it (the crosshair scales too, it just cannot be moved);</li>
 *   <li>left-click a frame — enable or disable the module;</li>
 *   <li>{@code R} — put everything back to the default layout;</li>
 *   <li>{@code Esc} — save and close.</li>
 * </ul>
 *
 * <p>The world keeps running behind the editor ({@link #isPauseScreen()} is false) and the modules
 * themselves are drawn by the HUD layer, so what you see is exactly what the game will show.</p>
 */
public class RemixHudEditorScreen extends Screen {

    private static final int COLOR_FRAME = 0xFF3A3A48;
    private static final int COLOR_FRAME_HOVER = 0xFF7C5CFF;
    private static final int COLOR_FRAME_DRAGGED = 0xFF6BE08A;
    private static final int COLOR_DISABLED = 0x99000000;
    private static final int COLOR_TEXT = 0xFFF2F2F7;
    private static final int COLOR_TEXT_DIM = 0xFF8B8B9A;
    private static final int COLOR_BACKDROP = 0x33000000;

    private final RemixConfig config;
    private RemixModule dragging;

    public RemixHudEditorScreen(RemixConfig config) {
        super(RemixLang.component("editor.title"));
        this.config = config;
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor context, int mouseX, int mouseY, float delta) {
        Font font = this.font;
        context.fill(0, 0, width, height, COLOR_BACKDROP);

        RemixModule hovered = RemixHud.under(Minecraft.getInstance(), config, mouseX, mouseY);

        for (RemixModule module : RemixHud.modules()) {
            RemixHud.Bounds bounds = RemixHud.boundsOf(module, Minecraft.getInstance(), config);
            boolean isHovered = module == hovered;
            boolean isDragged = module == dragging;
            int frameColor = isDragged ? COLOR_FRAME_DRAGGED
                    : isHovered ? COLOR_FRAME_HOVER : COLOR_FRAME;

            context.fill(bounds.x(), bounds.y(), bounds.x() + bounds.width(),
                    bounds.y() + bounds.height(), 0x14000000);
            drawFrame(context, bounds, frameColor);

            if (!module.enabled(config)) {
                context.fill(bounds.x(), bounds.y(), bounds.x() + bounds.width(),
                        bounds.y() + bounds.height(), COLOR_DISABLED);
            }

            Component label = module.enabled(config)
                    ? RemixLang.component("module." + module.id())
                    : RemixLang.off(RemixLang.component("module." + module.id()));
            int labelWidth = font.width(label);
            int labelX = Math.max(0, Math.min(width - labelWidth, bounds.x()));
            int labelY = bounds.y() > 10 ? bounds.y() - 10 : bounds.y() + bounds.height() + 1;
            context.text(font, label, labelX, labelY,
                    module.enabled(config) ? COLOR_TEXT : COLOR_TEXT_DIM, false);

            if (!module.movable() && isHovered) {
                context.text(font, RemixLang.component("editor.locked"),
                        labelX, labelY + 10, COLOR_TEXT_DIM, false);
            }
        }

        Component hint = RemixLang.component("editor.hint");
        context.centeredText(font, hint, width / 2, height - 14, COLOR_TEXT_DIM);
    }

    private void drawFrame(GuiGraphicsExtractor context, RemixHud.Bounds bounds, int color) {
        int right = bounds.x() + bounds.width();
        int bottom = bounds.y() + bounds.height();
        context.fill(bounds.x(), bounds.y(), right, bounds.y() + 1, color);
        context.fill(bounds.x(), bottom - 1, right, bottom, color);
        context.fill(bounds.x(), bounds.y(), bounds.x() + 1, bottom, color);
        context.fill(right - 1, bounds.y(), right, bottom, color);
    }

    @Override
    public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
        RemixModule module = RemixHud.under(Minecraft.getInstance(), config,
                event.x(), event.y());
        if (module == null) {
            dragging = null;
            return super.mouseClicked(event, doubleClick);
        }

        if (event.button() == 1) {
            // Right button grabs the frame; the crosshair has no position to change.
            dragging = module.movable() ? module : null;
            return true;
        }
        if (event.button() == 0) {
            module.setEnabled(config, !module.enabled(config));
            if (!module.enabled(config)) {
                RemixKeystrokes.reset();
            }
            config.save();
            return true;
        }
        return super.mouseClicked(event, doubleClick);
    }

    @Override
    public boolean mouseDragged(MouseButtonEvent event, double deltaX, double deltaY) {
        if (dragging == null) {
            return super.mouseDragged(event, deltaX, deltaY);
        }

        RemixConfig.Placement placement = config.placement(dragging.id());
        Minecraft client = Minecraft.getInstance();
        RemixHud.Bounds bounds = RemixHud.boundsOf(dragging, client, config);
        // Start from the resolved origin so the first drag on an untouched module does not
        // jump: its stored position is still unset until now.
        float startX = placement.custom() ? placement.x : bounds.x();
        float startY = placement.custom() ? placement.y : bounds.y();

        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();
        float maxX = Math.max(0, screenWidth - bounds.width());
        float maxY = Math.max(0, screenHeight - bounds.height());

        placement.x = clamp(startX + (float) deltaX, 0.0f, maxX);
        placement.y = clamp(startY + (float) deltaY, 0.0f, maxY);
        return true;
    }

    @Override
    public boolean mouseReleased(MouseButtonEvent event) {
        if (dragging != null) {
            dragging = null;
            config.save();
            return true;
        }
        return super.mouseReleased(event);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontal, double vertical) {
        RemixModule module = RemixHud.under(Minecraft.getInstance(), config, mouseX, mouseY);
        if (module == null || vertical == 0.0) {
            return super.mouseScrolled(mouseX, mouseY, horizontal, vertical);
        }
        RemixConfig.Placement placement = config.placement(module.id());
        placement.scale = clamp(
                placement.scale + (float) vertical * RemixConfig.MODULE_SCALE_STEP,
                RemixConfig.MIN_MODULE_SCALE, RemixConfig.MAX_MODULE_SCALE);
        return true;
    }

    @Override
    public boolean keyPressed(KeyEvent event) {
        if (event.key() == InputConstants.KEY_R) {
            config.resetLayout();
            dragging = null;
            config.save();
            return true;
        }
        return super.keyPressed(event);
    }

    /** Persist on the way out, whatever route was taken (Escape, keybind, ...). */
    @Override
    public void removed() {
        config.save();
        super.removed();
    }

    /** Keep the world running so the layout is edited against real gameplay. */
    @Override
    public boolean isPauseScreen() {
        return false;
    }

    private static float clamp(float value, float min, float max) {
        return Math.max(min, Math.min(max, value));
    }
}
