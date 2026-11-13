package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.resources.sounds.SimpleSoundInstance;
import net.minecraft.network.chat.Component;
import net.minecraft.sounds.SoundEvents;

import java.util.ArrayList;
import java.util.List;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import java.util.function.DoubleConsumer;
import java.util.function.DoubleSupplier;

/**
 * The Right Shift settings screen: one dark panel with the three HUD toggles, the scale and
 * opacity sliders and a reset button.
 *
 * <p>The widgets are drawn by hand instead of using vanilla {@code Button}/{@code AbstractSliderButton}
 * so the panel keeps a single flat look and there is no dependency on widget internals.</p>
 */
public class RemixSettingsScreen extends Screen {

    private static final int PANEL_WIDTH = 268;
    private static final int PANEL_HEIGHT = 222;

    private static final int COLOR_BACKDROP = 0x99000000;
    private static final int COLOR_PANEL = 0xF0131318;
    private static final int COLOR_BORDER = 0xFF2C2C36;
    private static final int COLOR_ROW = 0xFF191920;
    private static final int COLOR_ROW_HOVER = 0xFF22222B;
    private static final int COLOR_TRACK = 0xFF25252E;
    private static final int COLOR_ACCENT = 0xFF7C5CFF;
    private static final int COLOR_ACCENT_OFF = 0xFF33333D;
    private static final int COLOR_TEXT = 0xFFE8E8EF;
    private static final int COLOR_TEXT_DIM = 0xFF8B8B9A;
    private static final int COLOR_ON = 0xFF6BE08A;
    private static final int COLOR_OFF = 0xFF6A6A78;

    private final RemixConfig config;
    private final List<Toggle> toggles = new ArrayList<>();
    private final List<Slider> sliders = new ArrayList<>();

    private int panelX;
    private int panelY;
    private Rect resetButton;
    private Rect doneButton;
    private Slider dragging;

    public RemixSettingsScreen(RemixConfig config) {
        super(Component.translatable("remix_companion.screen.title"));
        this.config = config;
    }

    @Override
    protected void init() {
        // Clamp: on a small window or a large GUI scale the centred panel would
        // start at a negative coordinate and its buttons would be unreachable.
        panelX = Math.max(0, (width - PANEL_WIDTH) / 2);
        panelY = Math.max(0, (height - PANEL_HEIGHT) / 2);

        toggles.clear();
        sliders.clear();
        // A resize during a drag would otherwise leave the old slider captured.
        dragging = null;

        int rowX = panelX + 14;
        int rowWidth = PANEL_WIDTH - 28;

        int y = panelY + 46;
        toggles.add(new Toggle(new Rect(rowX, y, rowWidth, 20),
                Component.translatable("remix_companion.option.armor"),
                () -> config.showArmor, value -> config.showArmor = value));
        y += 24;
        toggles.add(new Toggle(new Rect(rowX, y, rowWidth, 20),
                Component.translatable("remix_companion.option.weapon"),
                () -> config.showWeapon, value -> config.showWeapon = value));
        y += 24;
        toggles.add(new Toggle(new Rect(rowX, y, rowWidth, 20),
                Component.translatable("remix_companion.option.totems"),
                () -> config.showTotems, value -> config.showTotems = value));

        sliders.add(new Slider(new Rect(rowX, panelY + 133, rowWidth, 6),
                Component.translatable("remix_companion.option.scale"),
                RemixConfig.MIN_SCALE, RemixConfig.MAX_SCALE, RemixConfig.SCALE_STEP,
                () -> config.scale, value -> config.scale = (float) value, false));
        sliders.add(new Slider(new Rect(rowX, panelY + 163, rowWidth, 6),
                Component.translatable("remix_companion.option.opacity"),
                RemixConfig.MIN_OPACITY, RemixConfig.MAX_OPACITY, 1.0,
                () -> config.opacity, value -> config.opacity = (int) Math.round(value), true));

        int buttonY = panelY + PANEL_HEIGHT - 34;
        resetButton = new Rect(rowX, buttonY, 104, 20);
        doneButton = new Rect(panelX + PANEL_WIDTH - 14 - 104, buttonY, 104, 20);
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor context, int mouseX, int mouseY, float delta) {
        Font font = this.font;

        context.fill(0, 0, width, height, COLOR_BACKDROP);

        context.fill(panelX - 1, panelY - 1, panelX + PANEL_WIDTH + 1, panelY + PANEL_HEIGHT + 1,
                COLOR_BORDER);
        context.fill(panelX, panelY, panelX + PANEL_WIDTH, panelY + PANEL_HEIGHT, COLOR_PANEL);
        context.fill(panelX, panelY, panelX + PANEL_WIDTH, panelY + 2, COLOR_ACCENT);

        context.text(font, getTitle(), panelX + 14, panelY + 14, COLOR_TEXT, false);
        context.text(font, Component.translatable("remix_companion.screen.subtitle"),
                panelX + 14, panelY + 26, COLOR_TEXT_DIM, false);

        for (Toggle toggle : toggles) {
            toggle.render(context, font, mouseX, mouseY);
        }
        for (Slider slider : sliders) {
            slider.render(context, font);
        }

        drawButton(context, font, resetButton,
                Component.translatable("remix_companion.button.reset"), mouseX, mouseY);
        drawButton(context, font, doneButton,
                Component.translatable("remix_companion.button.done"), mouseX, mouseY);
    }

    private void drawButton(GuiGraphicsExtractor context, Font font, Rect rect, Component label,
                            int mouseX, int mouseY) {
        boolean hovered = rect.contains(mouseX, mouseY);
        context.fill(rect.x, rect.y, rect.x + rect.width, rect.y + rect.height,
                hovered ? COLOR_ROW_HOVER : COLOR_ROW);
        context.fill(rect.x, rect.y, rect.x + rect.width, rect.y + 1,
                hovered ? COLOR_ACCENT : COLOR_ACCENT_OFF);
        int textWidth = font.width(label);
        context.text(font, label.getVisualOrderText(),
                rect.x + (rect.width - textWidth) / 2, rect.y + (rect.height - 8) / 2,
                hovered ? COLOR_TEXT : COLOR_TEXT_DIM, false);
    }

    @Override
    public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
        double mouseX = event.x();
        double mouseY = event.y();
        if (event.button() != 0) {
            return super.mouseClicked(event, doubleClick);
        }

        for (Toggle toggle : toggles) {
            if (toggle.bounds.contains(mouseX, mouseY)) {
                toggle.setter.accept(!toggle.getter.getAsBoolean());
                playClick();
                config.save();
                return true;
            }
        }
        for (Slider slider : sliders) {
            if (slider.hit(mouseX, mouseY)) {
                dragging = slider;
                slider.apply(mouseX);
                playClick();
                return true;
            }
        }
        if (resetButton.contains(mouseX, mouseY)) {
            config.reset();
            playClick();
            config.save();
            return true;
        }
        if (doneButton.contains(mouseX, mouseY)) {
            playClick();
            onClose();
            return true;
        }
        return super.mouseClicked(event, doubleClick);
    }

    @Override
    public boolean mouseDragged(MouseButtonEvent event, double deltaX, double deltaY) {
        if (dragging != null) {
            dragging.apply(event.x());
            return true;
        }
        return super.mouseDragged(event, deltaX, deltaY);
    }

    @Override
    public boolean mouseReleased(MouseButtonEvent event) {
        if (dragging != null) {
            dragging.apply(event.x());
            dragging = null;
            config.save();
            return true;
        }
        return super.mouseReleased(event);
    }

    private void playClick() {
        try {
            Minecraft.getInstance().getSoundManager()
                    .play(SimpleSoundInstance.forUI(SoundEvents.UI_BUTTON_CLICK, 1.0F));
        } catch (Throwable ignored) {
            // A missing click sound is never worth interrupting the player for.
        }
    }

    /** Persist on the way out, whatever route was taken (Escape, Done button, ...). */
    @Override
    public void removed() {
        config.save();
        super.removed();
    }

    /** Keep the world running so the HUD behind the panel previews the changes live. */
    @Override
    public boolean isPauseScreen() {
        return false;
    }

    /** Plain integer rectangle in GUI-scaled coordinates. */
    private record Rect(int x, int y, int width, int height) {
        boolean contains(double mouseX, double mouseY) {
            return mouseX >= x && mouseX < x + width && mouseY >= y && mouseY < y + height;
        }
    }

    private static final class Toggle {
        private final Rect bounds;
        private final Component label;
        private final BooleanSupplier getter;
        private final Consumer<Boolean> setter;

        private Toggle(Rect bounds, Component label, BooleanSupplier getter, Consumer<Boolean> setter) {
            this.bounds = bounds;
            this.label = label;
            this.getter = getter;
            this.setter = setter;
        }

        private void render(GuiGraphicsExtractor context, Font font, int mouseX, int mouseY) {
            boolean enabled = getter.getAsBoolean();
            boolean hovered = bounds.contains(mouseX, mouseY);

            context.fill(bounds.x, bounds.y, bounds.x + bounds.width, bounds.y + bounds.height,
                    hovered ? COLOR_ROW_HOVER : COLOR_ROW);
            context.fill(bounds.x, bounds.y, bounds.x + 2, bounds.y + bounds.height,
                    enabled ? COLOR_ACCENT : COLOR_ACCENT_OFF);
            context.text(font, label, bounds.x + 10, bounds.y + (bounds.height - 8) / 2,
                    COLOR_TEXT, false);

            Component state = Component.translatable(enabled
                    ? "remix_companion.state.on"
                    : "remix_companion.state.off");
            int stateWidth = font.width(state);
            context.text(font, state,
                    bounds.x + bounds.width - 10 - stateWidth,
                    bounds.y + (bounds.height - 8) / 2,
                    enabled ? COLOR_ON : COLOR_OFF, false);
        }
    }

    private static final class Slider {
        private final Rect track;
        private final Component label;
        private final double min;
        private final double max;
        private final double step;
        private final DoubleSupplier getter;
        private final DoubleConsumer setter;
        private final boolean integer;

        private Slider(Rect track, Component label, double min, double max, double step,
                       DoubleSupplier getter, DoubleConsumer setter, boolean integer) {
            this.track = track;
            this.label = label;
            this.min = min;
            this.max = max;
            this.step = step;
            this.getter = getter;
            this.setter = setter;
            this.integer = integer;
        }

        private boolean hit(double mouseX, double mouseY) {
            return mouseX >= track.x - 4 && mouseX <= track.x + track.width + 4
                    && mouseY >= track.y - 8 && mouseY <= track.y + track.height + 8;
        }

        private void apply(double mouseX) {
            double ratio = (mouseX - track.x) / (double) track.width;
            ratio = Math.max(0.0, Math.min(1.0, ratio));
            double raw = min + ratio * (max - min);
            if (step > 0) {
                raw = Math.round(raw / step) * step;
            }
            setter.accept(Math.max(min, Math.min(max, raw)));
        }

        private void render(GuiGraphicsExtractor context, Font font) {
            double value = getter.getAsDouble();
            context.text(font, label, track.x, track.y - 12, COLOR_TEXT_DIM, false);

            String display = integer
                    ? Integer.toString((int) Math.round(value))
                    : String.format("%.2f", value) + "x";
            int displayWidth = font.width(display);
            context.text(font, display,
                    track.x + track.width - displayWidth, track.y - 12, COLOR_TEXT, false);

            context.fill(track.x, track.y, track.x + track.width, track.y + track.height, COLOR_TRACK);

            double ratio = (value - min) / (max - min);
            int filled = (int) Math.round(ratio * track.width);
            if (filled > 0) {
                context.fill(track.x, track.y, track.x + filled, track.y + track.height, COLOR_ACCENT);
            }

            int knobX = track.x + filled;
            context.fill(
                    Math.max(track.x, knobX - 2), track.y - 2,
                    Math.min(track.x + track.width, knobX + 2), track.y + track.height + 2,
                    0xFFFFFFFF);
        }
    }
}