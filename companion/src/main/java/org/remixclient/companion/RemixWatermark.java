package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.List;

/**
 * Small badge: build name, frame rate, latency and (optionally) the player's block position.
 */
final class RemixWatermark implements RemixModule {

    static final RemixWatermark INSTANCE = new RemixWatermark();

    private static final String NAME = "RemixClient";
    private static final int ROW = 11;

    private RemixWatermark() {
    }

    @Override
    public String id() {
        return "watermark";
    }

    @Override
    public Component title() {
        return Component.translatable("remix_companion.module.watermark");
    }

    @Override
    public boolean enabled(RemixConfig config) {
        return config.showWatermark;
    }

    @Override
    public void setEnabled(RemixConfig config, boolean enabled) {
        config.showWatermark = enabled;
    }

    private record Row(String text, int color) {
    }

    private static List<Row> rows(Minecraft client, RemixConfig config) {
        List<Row> rows = new ArrayList<>(3);
        rows.add(new Row(NAME, RemixDraw.ACCENT));

        int fps = client.getFps();
        int ping = ping(client);
        String stats = fps + " FPS" + (ping >= 0 ? "  " + ping + " ms" : "");
        rows.add(new Row(stats, fpsColor(fps)));

        if (config.showCoords && client.player != null) {
            String coords = client.player.getBlockX() + " "
                    + client.player.getBlockY() + " "
                    + client.player.getBlockZ();
            rows.add(new Row(coords, RemixDraw.TEXT_DIM));
        }
        return rows;
    }

    @Override
    public int width(Minecraft client, RemixConfig config) {
        Font font = client.font;
        int widest = 0;
        for (Row row : rows(client, config)) {
            widest = Math.max(widest, font.width(row.text()));
        }
        return widest + 4;
    }

    @Override
    public int height(Minecraft client, RemixConfig config) {
        return rows(client, config).size() * ROW;
    }

    @Override
    public void render(GuiGraphicsExtractor context, Minecraft client, RemixConfig config) {
        List<Row> rows = rows(client, config);
        for (int i = 0; i < rows.size(); i++) {
            Row row = rows.get(i);
            RemixDraw.line(context, client.font, row.text(), 2, i * ROW, row.color(),
                    config.backgroundAlpha());
        }
    }

    @Override
    public Origin defaultOrigin(Minecraft client, RemixConfig config) {
        return new Origin(4, 4);
    }

    /** Latency of the local player's tab entry, or -1 when there is none yet. */
    private static int ping(Minecraft client) {
        try {
            if (client.getConnection() == null || client.player == null) {
                return -1;
            }
            PlayerInfo info = client.getConnection().getPlayerInfo(client.player.getUUID());
            return info == null ? -1 : info.getLatency();
        } catch (Throwable ignored) {
            // A missing tab entry is not worth a log line per frame.
            return -1;
        }
    }

    private static int fpsColor(int fps) {
        if (fps >= 120) {
            return RemixDraw.GREEN;
        }
        return fps >= 60 ? RemixDraw.YELLOW : RemixDraw.RED;
    }
}
