package org.remixclient.companion;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;

import java.io.Reader;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Small JSON-backed settings object for the Remix Companion HUD.
 *
 * <p>The file lives at {@code config/remix-companion.json} inside the game directory and is
 * created on the very first launch. All reads are defensive: a missing, unreadable or corrupt
 * file simply falls back to the defaults, and out-of-range values are clamped instead of
 * crashing the HUD.</p>
 */
public final class RemixConfig {

    public static final float MIN_SCALE = 0.75f;
    public static final float MAX_SCALE = 1.5f;
    public static final float SCALE_STEP = 0.05f;

    public static final int MIN_OPACITY = 30;
    public static final int MAX_OPACITY = 100;

    private static final String FILE_NAME = "remix-companion.json";
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    public boolean showArmor = true;
    public boolean showWeapon = true;
    public boolean showTotems = true;
    public float scale = 1.0f;
    public int opacity = 100;

    /** Absolute path of the config file, e.g. {@code <gamedir>/config/remix-companion.json}. */
    public static Path file() {
        return FabricLoader.getInstance().getConfigDir().resolve(FILE_NAME);
    }

    /**
     * Loads the config, creating it with defaults when it does not exist yet. Never throws —
     * on any failure the defaults are returned so the mod keeps working.
     */
    public static RemixConfig load() {
        RemixConfig config = new RemixConfig();
        Path path = file();

        if (Files.exists(path)) {
            try (Reader reader = Files.newBufferedReader(path, StandardCharsets.UTF_8)) {
                RemixConfig parsed = GSON.fromJson(reader, RemixConfig.class);
                if (parsed != null) {
                    config = parsed;
                }
            } catch (Exception e) {
                RemixCompanionClient.LOGGER.warn(
                        "[Remix Companion] Could not read {}, falling back to defaults.", path, e);
            }
        }

        config.sanitize();
        // Only create the file when it is missing: rewriting it on every launch
        // would silently replace a hand-edited or half-written config.
        if (!Files.exists(path)) {
            config.save();
        }
        return config;
    }

    /** Persists the current values. Failures are logged, never propagated. */
    public void save() {
        sanitize();
        Path path = file();
        try {
            Path parent = path.getParent();
            if (parent != null) {
                Files.createDirectories(parent);
            }
            // Write to a temp file and move it into place, so a crash mid-write
            // cannot leave a truncated config behind.
            Path temp = path.resolveSibling(path.getFileName() + ".tmp");
            try (Writer writer = Files.newBufferedWriter(temp, StandardCharsets.UTF_8)) {
                GSON.toJson(this, writer);
            }
            Files.move(temp, path, StandardCopyOption.REPLACE_EXISTING);
        } catch (Exception e) {
            RemixCompanionClient.LOGGER.warn("[Remix Companion] Could not write {}.", path, e);
        }
    }

    /** Restores every option to its factory default. */
    public void reset() {
        showArmor = true;
        showWeapon = true;
        showTotems = true;
        scale = 1.0f;
        opacity = 100;
    }

    /** Clamps every value into its supported range and normalises NaN. */
    public void sanitize() {
        if (Float.isNaN(scale) || Float.isInfinite(scale)) {
            scale = 1.0f;
        }
        scale = Math.round(scale / SCALE_STEP) * SCALE_STEP;
        scale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));

        opacity = Math.max(MIN_OPACITY, Math.min(MAX_OPACITY, opacity));
    }

    /** Configured opacity as an 0-255 alpha channel. */
    public int backgroundAlpha() {
        int clamped = Math.max(MIN_OPACITY, Math.min(MAX_OPACITY, opacity));
        return clamped * 255 / 100;
    }
}
