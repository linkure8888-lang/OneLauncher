package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.List;

/**
 * Item pickup logger: a short fading feed of everything the player just picked up.
 *
 * <p>The inventory is diffed once per tick, so nothing depends on server packets or extra
 * Fabric events; entries live for a few seconds and fade out instead of scrolling forever.</p>
 */
final class RemixPickups implements RemixModule {

    static final RemixPickups INSTANCE = new RemixPickups();

    /** One entry stays fully visible for this long, then fades out over the same span. */
    private static final long ENTRY_LIFETIME_MS = 3000L;
    private static final int MAX_ENTRIES = 5;
    private static final int ROW_HEIGHT = 10;
    private static final int PADDING = 2;
    private static final int MIN_WIDTH = 64;
    private static final int MAX_NAME_CHARS = 24;

    private record Entry(String name, int count, long at) {
    }

    private final Deque<Entry> entries = new ArrayDeque<>();
    private ItemStack[] previous;
    private long lastTickAt;

    private RemixPickups() {
    }

    @Override
    public String id() {
        return "pickups";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.pickups");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showPickups;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showPickups = enabled;
    }

    /** Called once per client tick: diffs the combined inventory and logs every change. */
    void tick(Minecraft client) {
        try {
            LocalPlayer player = client.player;
            if (player == null) {
                previous = null;
                entries.clear();
                return;
            }
            long now = System.currentTimeMillis();
            if (lastTickAt == 0) {
                lastTickAt = now;
            }
            ItemStack[] current = snapshot(player);
            if (previous == null) {
                previous = current;
                return;
            }

            int shortest = Math.min(previous.length, current.length);
            for (int slot = 0; slot < shortest; slot++) {
                recordDifference(previous[slot], current[slot], now);
            }
            previous = current;

            // Drop entries that had their full life even when the panel is hidden.
            while (!entries.isEmpty() && now - entries.peekFirst().at() > ENTRY_LIFETIME_MS * 2) {
                entries.pollFirst();
            }
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error("[Remix Companion] Pickup logger failed.", t);
            previous = null;
        }
    }

    private void recordDifference(ItemStack was, ItemStack now, long at) {
        String name = shortName(was);
        String other = shortName(now);
        if (was.isEmpty() && now.isEmpty()) {
            return;
        }
        if (name.equals(other)) {
            int delta = now.getCount() - was.getCount();
            if (delta != 0) {
                push(name, delta, at);
            }
            return;
        }
        if (!was.isEmpty()) {
            // Slots that swap items outright are usually crafting or throwing, not picking
            // up, but the stack that arrived is still worth one line.
            if (!now.isEmpty()) {
                push(other, now.getCount(), at);
            }
        } else {
            push(other, now.getCount(), at);
        }
    }

    private void push(String name, int count, long at) {
        entries.addLast(new Entry(name, count, at));
        while (entries.size() > MAX_ENTRIES) {
            entries.pollFirst();
        }
    }

    private static ItemStack[] snapshot(LocalPlayer player) {
        Inventory inventory = player.getInventory();
        List<ItemStack> stacks = new ArrayList<>();
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            stacks.add(inventory.getItem(slot));
        }
        return stacks.toArray(new ItemStack[0]);
    }

    private static String shortName(ItemStack stack) {
        if (stack == null || stack.isEmpty()) {
            return "";
        }
        String name = stack.getHoverName().getString();
        if (name.length() > MAX_NAME_CHARS) {
            name = name.substring(0, MAX_NAME_CHARS - 1) + "…";
        }
        return name;
    }

    private List<Entry> live() {
        long now = System.currentTimeMillis();
        List<Entry> alive = new ArrayList<>();
        for (Entry entry : entries) {
            if (now - entry.at() <= ENTRY_LIFETIME_MS * 2) {
                alive.add(entry);
            }
        }
        return alive;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        Font font = client.font;
        int widest = MIN_WIDTH;
        for (Entry entry : live()) {
            widest = Math.max(widest, font.width(line(entry)) + PADDING * 2);
        }
        return widest;
    }

    private String line(Entry entry) {
        String amount = (entry.count() > 0 ? "+" : "") + entry.count();
        return amount + " " + entry.name();
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return Math.max(1, live().size()) * ROW_HEIGHT;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        long now = System.currentTimeMillis();
        Font font = client.font;
        int backgroundAlpha = config.backgroundAlpha();
        int index = 0;
        for (Entry entry : live()) {
            long age = now - entry.at();
            float life = 1f - age / (float) (ENTRY_LIFETIME_MS * 2);
            int alpha = Math.round(Math.max(0f, Math.min(1f, life)) * backgroundAlpha);
            String text = line(entry);
            int y = Math.round(index * ROW_HEIGHT);
            int color = entry.count() > 0 ? RemixDraw.GREEN : RemixDraw.TEXT_DIM;
            context.fill(0, y, font.width(text) + PADDING * 2, y + ROW_HEIGHT,
                    RemixDraw.panel(alpha));
            // Text keeps full opacity so it never disappears into bright biomes.
            context.text(font, text, PADDING, y, RemixDraw.alpha(color, 255), true);
            index++;
        }
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        int height = height(client, config);
        return new Origin(4, Math.max(4, client.getWindow().getGuiScaledHeight() / 3 - height));
    }
}
