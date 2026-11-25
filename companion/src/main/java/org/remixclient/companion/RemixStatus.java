package org.remixclient.companion;

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

import java.util.ArrayList;
import java.util.List;

/**
 * Armour and weapon durability plus the totem counter, drawn as one movable block, and the
 * full-screen flash whenever a totem pops.
 */
final class RemixStatus implements RemixModule {

    private static final List<EquipmentSlot> ARMOR_SLOTS = List.of(
            EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET);

    private static final long FLASH_DURATION_MS = 900L;
    private static final float ROW_HEIGHT = 10f;
    private static final int MAX_NAME_CHARS = 16;
    private static final int PADDING = 2;
    /** Minimum width so an empty block still has a grabbable frame in the HUD editor. */
    private static final int MIN_WIDTH = 64;

    static final RemixStatus INSTANCE = new RemixStatus();

    private int lastTotemCount = -1;
    private long flashStartedAt = -1L;

    private RemixStatus() {
    }

    @Override
    public String id() {
        return "status";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.status");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showStatus;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showStatus = enabled;
    }

    /** Called once per client tick: tracks the totem count so a drop can trigger the flash. */
    void tick(Minecraft client) {
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

    private List<Line> lines(Minecraft client, RemixConfig config, boolean preview) {
        List<Line> lines = new ArrayList<>(6);
        LocalPlayer player = client.player;
        if (player == null) {
            return lines;
        }

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
            lines.add(new Line(label + " " + totems, totems > 0 ? RemixDraw.GOLD : RemixDraw.RED));
        }

        // Keep the tick counter honest even on the very first frame after a world join.
        if (lastTotemCount < 0) {
            lastTotemCount = countTotems(player);
        }
        return lines;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        List<Line> lines = lines(client, config, false);
        Font font = client.font;
        int widest = 0;
        for (Line line : lines) {
            widest = Math.max(widest, font.width(line.text()));
        }
        return Math.max(MIN_WIDTH, widest + PADDING * 2);
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return Math.max(1, lines(client, config, false).size()) * (int) ROW_HEIGHT;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        List<Line> lines = lines(client, config, false);
        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();

        for (int i = 0; i < lines.size(); i++) {
            Line line = lines.get(i);
            int y = Math.round(i * ROW_HEIGHT);
            int textWidth = font.width(line.text());
            // Panel only — the text stays fully opaque so it survives bright biomes.
            context.fill(0, y, textWidth + PADDING * 2, y + (int) ROW_HEIGHT,
                    RemixDraw.panel(backgroundAlpha));
            context.text(font, line.text(), PADDING, y, line.color(), true);
        }
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int height = height(client, config);
        return new Origin(4, Math.max(4, client.getWindow().getGuiScaledHeight() - 58 - height));
    }

    /** Full-screen flash on a totem pop; drawn without translation, so it covers the screen. */
    void renderFlash(GuiGraphicsExtractor context, Minecraft client) {
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

        int screenWidth = client.getWindow().getGuiScaledWidth();
        int screenHeight = client.getWindow().getGuiScaledHeight();

        float remaining = 1.0f - elapsed / (float) FLASH_DURATION_MS;
        int overlayAlpha = (int) (remaining * 100f);
        if (overlayAlpha > 0) {
            context.fill(0, 0, screenWidth, screenHeight,
                    (overlayAlpha << 24) | RemixDraw.TOTEM_FLASH);
        }

        int textAlpha = (int) (remaining * 255f);
        String message = Component.translatable("remix_companion.hud.totem_used").getString();
        int width = client.font.width(message);
        context.text(client.font, message,
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
            return RemixDraw.GREEN;
        }
        if (percent >= 30) {
            return RemixDraw.YELLOW;
        }
        return RemixDraw.RED;
    }

    private static int countTotems(LocalPlayer player) {
        // Inventory is a Container in 26.3: slots 0..n cover main inventory plus equipment.
        Inventory inventory = player.getInventory();
        int total = 0;
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            ItemStack stack = inventory.getItem(slot);
            if (!stack.isEmpty() && stack.getItem() == Items.TOTEM_OF_UNDYING) {
                total += stack.getCount();
            }
        }
        return total;
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
