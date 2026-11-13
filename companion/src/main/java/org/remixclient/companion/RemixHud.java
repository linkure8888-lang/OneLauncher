package org.remixclient.companion;

import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.tags.ItemTags;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.ShieldItem;
import org.joml.Matrix3x2fStack;

import java.util.ArrayList;
import java.util.List;

/**
 * The Remix Companion HUD: armor durability, held weapon durability and the totem counter,
 * plus a short flash whenever a totem pops.
 *
 * <p>The whole renderer is wrapped in a try/catch that disables itself permanently after the
 * first failure, so a broken HUD can never spam the log or crash the render loop.</p>
 */
public final class RemixHud {

    private static final List<EquipmentSlot> ARMOR_SLOTS = List.of(
            EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET);

    private static final int COLOR_GREEN = 0xFF55E06B;
    private static final int COLOR_YELLOW = 0xFFE8C33A;
    private static final int COLOR_RED = 0xFFE85C5C;
    private static final int COLOR_GOLD = 0xFFE0B84A;
    private static final int COLOR_BACKGROUND = 0x0A0A0E;
    private static final int COLOR_FLASH = 0xFFC24A;

    private static final long FLASH_DURATION_MS = 900L;
    private static final float ROW_HEIGHT = 10f;
    private static final int MAX_NAME_CHARS = 16;

    private static int lastTotemCount = -1;
    private static long flashStartedAt = -1L;
    private static boolean broken;

    private RemixHud() {
    }

    /** Called once per client tick: tracks the totem count so a drop can trigger the flash. */
    public static void tick(Minecraft client) {
        try {
            LocalPlayer player = client.player;
            if (player == null) {
                lastTotemCount = -1;
                return;
            }
            int count = countTotems(player);
            if (lastTotemCount >= 0 && count < lastTotemCount) {
                flashStartedAt = System.currentTimeMillis();
            }
            lastTotemCount = count;
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error("[Remix Companion] Totem tracker failed.", t);
            lastTotemCount = -1;
        }
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
        LocalPlayer player = client.player;
        if (player == null || client.gui.hud.isHidden()) {
            return;
        }

        RemixConfig config = RemixCompanionClient.getConfig();
        // While the settings screen is open every section is drawn, so the sliders and
        // toggles have a live preview behind the panel.
        boolean preview = client.gui.screen() instanceof RemixSettingsScreen;

        Font font = client.font;
        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();
        float scale = config.scale;

        List<Line> lines = new ArrayList<>(6);
        collectLines(player, config, preview, lines);

        if (!lines.isEmpty()) {
            drawLines(context, font, lines, config.backgroundAlpha(), scale, screenHeight);
        }

        drawFlash(context, font, screenWidth, screenHeight);

        // Keep the tick counter honest even on the very first frame after a world join.
        if (lastTotemCount < 0) {
            lastTotemCount = countTotems(player);
        }
    }

    private static void collectLines(LocalPlayer player, RemixConfig config, boolean preview,
                                     List<Line> lines) {
        if (preview || config.showArmor) {
            for (EquipmentSlot slot : ARMOR_SLOTS) {
                ItemStack stack = player.getItemBySlot(slot);
                if (stack.isEmpty() || !stack.isDamageableItem()) {
                    continue;
                }
                int percent = durabilityPercent(stack);
                lines.add(new Line(labelFor(slot) + " " + percent + "%", colorFor(percent)));
            }
        }

        if (preview || config.showWeapon) {
            ItemStack held = player.getMainHandItem();
            if (isTrackedWeapon(held) && held.isDamageableItem()) {
                int percent = durabilityPercent(held);
                lines.add(new Line(shortName(held) + " " + percent + "%", colorFor(percent)));
            }
        }

        if (preview || config.showTotems) {
            int totems = countTotems(player);
            String label = Component.translatable("remix_companion.hud.totems").getString();
            lines.add(new Line(label + " " + totems, totems > 0 ? COLOR_GOLD : COLOR_RED));
        }
    }

    private static void drawLines(GuiGraphicsExtractor context, Font font, List<Line> lines,
                                  int backgroundAlpha, float scale, int screenHeight) {
        Matrix3x2fStack pose = context.pose();
        pose.pushMatrix();
        try {
            pose.scale(scale);
    
            int left = 4;
            float topLogical = (screenHeight - 58f) / scale - lines.size() * ROW_HEIGHT;
    
            for (int i = 0; i < lines.size(); i++) {
                Line line = lines.get(i);
                int y = Math.round(topLogical + i * ROW_HEIGHT);
                int width = font.width(line.text);
                // Panel only — the text itself stays fully opaque so it survives bright biomes.
                context.fill(left - 2, y - 1, left + width + 2, y + 9,
                        (backgroundAlpha << 24) | COLOR_BACKGROUND);
                context.text(font, line.text, left, y, line.color, true);
            }
    
        } finally {
            // An exception mid-draw must not leave the scale applied to the rest
            // of the frame (chat and screens would render in the wrong scale).
            pose.popMatrix();
        }
    }

    private static void drawFlash(GuiGraphicsExtractor context, Font font,
                                  int screenWidth, int screenHeight) {
        if (flashStartedAt < 0L) {
            return;
        }
        long elapsed = System.currentTimeMillis() - flashStartedAt;
        if (elapsed < 0L) {
            // The wall clock moved backwards (NTP sync, manual change): drop the
            // flash instead of leaving it stuck on screen.
            flashStartedAt = -1L;
            return;
        }
        if (elapsed >= FLASH_DURATION_MS) {
            flashStartedAt = -1L;
            return;
        }

        float remaining = 1.0f - elapsed / (float) FLASH_DURATION_MS;
        int overlayAlpha = (int) (remaining * 100f);
        if (overlayAlpha > 0) {
            context.fill(0, 0, screenWidth, screenHeight, (overlayAlpha << 24) | COLOR_FLASH);
        }

        int textAlpha = (int) (remaining * 255f);
        String message = Component.translatable("remix_companion.hud.totem_used").getString();
        int width = font.width(message);
        context.text(font, message,
                (screenWidth - width) / 2, screenHeight / 2 - 30,
                (textAlpha << 24) | 0xFFFFFF, true);
    }

    private static String labelFor(EquipmentSlot slot) {
        String key = switch (slot) {
            case HEAD -> "remix_companion.slot.helmet";
            case CHEST -> "remix_companion.slot.chestplate";
            case LEGS -> "remix_companion.slot.leggings";
            case FEET -> "remix_companion.slot.boots";
            default -> "remix_companion.slot.armor";
        };
        return Component.translatable(key).getString();
    }

    private static boolean isTrackedWeapon(ItemStack stack) {
        if (stack.isEmpty()) {
            return false;
        }
        return stack.getItem() instanceof ShieldItem
                || stack.is(holder -> holder.is(ItemTags.SWORDS))
                || stack.is(holder -> holder.is(ItemTags.AXES));
    }

    /** Remaining durability as a whole percent, 0-100. */
    private static int durabilityPercent(ItemStack stack) {
        int max = stack.getMaxDamage();
        if (max <= 0) {
            return 100;
        }
        int remaining = max - stack.getDamageValue();
        if (remaining <= 0) {
            return 0;
        }
        if (remaining >= max) {
            return 100;
        }
        return (int) Math.floor(remaining * 100.0 / max);
    }

    private static int colorFor(int percent) {
        if (percent > 60) {
            return COLOR_GREEN;
        }
        if (percent >= 30) {
            return COLOR_YELLOW;
        }
        return COLOR_RED;
    }

    private static int countTotems(LocalPlayer player) {
        // Inventory is a Container in 26.3: slots 0..n cover main inventory plus equipment.
        Inventory inventory = player.getInventory();
        int total = 0;
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            total += countIn(inventory.getItem(slot));
        }
        return total;
    }

    private static int countIn(ItemStack stack) {
        if (!stack.isEmpty() && stack.getItem() == Items.TOTEM_OF_UNDYING) {
            return stack.getCount();
        }
        return 0;
    }

    private static String shortName(ItemStack stack) {
        String name = stack.getHoverName().getString();
        if (name.length() <= MAX_NAME_CHARS) {
            return name;
        }
        // Keep the result inside MAX_NAME_CHARS and never split a surrogate pair.
        int end = Math.min(name.length(), MAX_NAME_CHARS - 3);
        if (end > 0 && Character.isHighSurrogate(name.charAt(end - 1))) {
            end--;
        }
        return name.substring(0, end) + "...";
    }

    private record Line(String text, int color) {
    }
}
