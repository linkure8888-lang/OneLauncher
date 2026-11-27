package org.remixclient.companion;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;
import org.polyfrost.oneconfig.api.config.v1.Config;
import org.polyfrost.oneconfig.api.config.v1.annotations.Slider;
import org.polyfrost.oneconfig.api.config.v1.annotations.Switch;

import java.io.Reader;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Settings for the Remix Companion HUD and visual modules, hosted as a OneConfig page.
 *
 * <p>The class extends OneConfig's {@link Config}, so the annotated fields below render as
 * switches and sliders in the OneConfig GUI (the page is titled "Remix Client"). At the same
 * time the values keep living in {@code config/remix-companion.json} inside the game
 * directory: the launcher writes the {@link #language} field there to keep its own language
 * in sync, so the JSON file stays the source of truth and OneConfig's own store mirrors it.</p>
 *
 * <p>Fields without an annotation (language, placements) are invisible to OneConfig and are
 * managed only through the JSON file, which is exactly what the launcher needs.</p>
 */
public class RemixConfig extends Config {

    public static final float MIN_MODULE_SCALE = 0.5f;
    public static final float MAX_MODULE_SCALE = 2.5f;
    public static final float MODULE_SCALE_STEP = 0.05f;

    public static final int MIN_OPACITY = 30;
    public static final int MAX_OPACITY = 100;

    public static final int MIN_GAP = 0;
    public static final int MAX_GAP = 10;
    public static final int MIN_LENGTH = 2;
    public static final int MAX_LENGTH = 10;
    public static final int MIN_THICKNESS = 1;
    public static final int MAX_THICKNESS = 3;

    public static final int MIN_HIT_PARTICLES = 4;
    public static final int MIN_HITSOUND_VOLUME = 10;
    public static final int MAX_HITSOUND_VOLUME = 100;
    public static final int MAX_HIT_PARTICLES = 16;

    private static final String FILE_NAME = "remix-companion.json";
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    /** Armour, weapon and totem rows, drawn as one movable block. */
    @Switch(title = "Armor & Totems", description = "Armour, held weapon and totem counts in one panel.",
            category = "HUD", subcategory = "Modules")
    public boolean showStatus = false;
    @Switch(title = "Armor row", description = "Worn armour pieces with durability.",
            category = "HUD", subcategory = "Modules")
    public boolean showArmor = false;
    @Switch(title = "Weapon display", description = "Show the held item with its stack count.",
            category = "HUD", subcategory = "Modules")
    public boolean showWeapon = false;
    @Switch(title = "Totem counter", description = "Count totems and flash when one pops.",
            category = "HUD", subcategory = "Modules")
    public boolean showTotems = false;

    /** Visual modules, all drawn as separate movable panels. */
    @Switch(title = "Keystrokes", description = "WASD, jump and mouse buttons with CPS counters.",
            category = "HUD", subcategory = "Modules")
    public boolean showKeystrokes = false;
    @Switch(title = "Action keys", description = "Shift, sprint and offhand indicator rows.",
            category = "HUD", subcategory = "Modules")
    public boolean showKeybinds = false;
    @Switch(title = "Target HUD", description = "Health and armour of whoever is under the crosshair.",
            category = "HUD", subcategory = "Modules")
    public boolean showTargetHud = false;
    @Switch(title = "Potion effects", description = "Active status effects with their remaining time.",
            category = "HUD", subcategory = "Modules")
    public boolean showPotions = false;
    @Switch(title = "Watermark", description = "Client name with FPS and ping.",
            category = "HUD", subcategory = "Modules")
    public boolean showWatermark = false;
    @Switch(title = "Coordinates", description = "Current XYZ position under the watermark.",
            category = "HUD", subcategory = "Modules")
    public boolean showCoords = false;
    @Switch(title = "Pickup logger", description = "A fading feed of the items you just picked up.",
            category = "HUD", subcategory = "Modules")
    public boolean showPickups = true;

    @Slider(title = "Panel opacity", description = "Transparency of every HUD panel background.",
            category = "HUD", subcategory = "Panels",
            min = MIN_OPACITY, max = MAX_OPACITY, step = 1)
    public int opacity = 100;

    /** Combat effects drawn as client-side particles, so servers see nothing extra. */
    @Switch(title = "Hit particles", description = "A burst of particles when you hit an entity.",
            category = "Visuals", subcategory = "Effects")
    public boolean hitEffects = true;
    @Switch(title = "Jump circles", description = "A particle ring around the feet on every jump.",
            category = "Visuals", subcategory = "Effects")
    public boolean jumpCircles = true;
    @Switch(title = "Player trail", description = "A soft particle trail behind you while walking.",
            category = "Visuals", subcategory = "Effects")
    public boolean trails = true;

    @Slider(title = "Hit particles count", description = "How many particles one hit produces.",
            category = "Visuals", subcategory = "Effects",
            min = MIN_HIT_PARTICLES, max = MAX_HIT_PARTICLES, step = 1)
    public int hitParticleCount = 8;
    /** Colour of the hit particles; packed 0xRRGGBB, the alpha channel is decided at draw time. */
    public int hitParticleColor = 0xFFB44C;

    @Switch(title = "Hit sound", description = "A short chime whenever a swing connects.",
            category = "Visuals", subcategory = "Effects")
    public boolean hitsound = true;
    @Slider(title = "Hit sound volume", description = "Loudness of the hit chime.",
            category = "Visuals", subcategory = "Effects",
            min = MIN_HITSOUND_VOLUME, max = MAX_HITSOUND_VOLUME, step = 1)
    public int hitsoundVolume = 70;

    /** Replaces the vanilla crosshair; needs the HUD mixin, so it is opt-in. */
    @Switch(title = "Custom crosshair", description = "Draw the companion crosshair instead of the vanilla one.",
            category = "Visuals", subcategory = "Crosshair")
    public boolean showCrosshair = false;
    @Switch(title = "Centre dot", description = "Show the dot in the middle of the crosshair.",
            category = "Visuals", subcategory = "Crosshair")
    public boolean crosshairDot = true;
    @Switch(title = "Crosshair outline", description = "Dark outline around the crosshair lines.",
            category = "Visuals", subcategory = "Crosshair")
    public boolean crosshairOutline = true;
    @Switch(title = "Hit highlight", description = "Tint the crosshair red while hitting something.",
            category = "Visuals", subcategory = "Crosshair")
    public boolean crosshairHitTint = true;

    /** Colours are packed 0xRRGGBB; the alpha channel is decided at draw time. */
    public int crosshairColor = 0xFFFFFF;
    public int crosshairHitColor = 0xFF4C4C;

    @Slider(title = "Crosshair gap", description = "Distance between the centre and the lines.",
            category = "Visuals", subcategory = "Crosshair",
            min = MIN_GAP, max = MAX_GAP, step = 1)
    public int crosshairGap = 4;
    @Slider(title = "Crosshair length", description = "Length of each crosshair line.",
            category = "Visuals", subcategory = "Crosshair",
            min = MIN_LENGTH, max = MAX_LENGTH, step = 1)
    public int crosshairLength = 6;
    @Slider(title = "Crosshair thickness", description = "Width of each crosshair line.",
            category = "Visuals", subcategory = "Crosshair",
            min = MIN_THICKNESS, max = MAX_THICKNESS, step = 1)
    public int crosshairThickness = 1;

    /**
     * Live mirror of the full-brightness state; OneConfig toggles the field and the tick
     * handler applies it, the B key updates the field back from the real state.
     */
    @Switch(title = "Full brightness", description = "Gamma boost, same as the B key.",
            category = "Visuals", subcategory = "Effects")
    public boolean fullBrightness = false;

    /**
     * UI language of the companion screens: {@code auto} follows the game language, while
     * {@code ru} and {@code en} pin it, so the launcher can keep its own language in sync.
     * Not shown in OneConfig: the launcher owns this value.
     */
    public String language = "auto";

    /** Per-module position and scale, keyed by module id. Missing entries use the default spot. */
    public Map<String, Placement> placements = new LinkedHashMap<>();

    public RemixConfig() {
        // Arg order is (id, iconPath, title, category) in OneConfig's Config.
        super("remix-companion.json", "/assets/remix_companion/icon.png", "Remix Client", Category.OTHER);
    }

    /** Absolute path of the config file, e.g. {@code <gamedir>/config/remix-companion.json}. */
    public static Path file() {
        return FabricLoader.getInstance().getConfigDir().resolve(FILE_NAME);
    }

    /**
     * Loads the config, creating it with defaults when it does not exist yet. Never throws —
     * on any failure the defaults are returned so the mod keeps working.
     *
     * <p>The parsed JSON lands in a throw-away instance (Gson builds those without running
     * the constructor, so their OneConfig identity is unusable) and every value is applied
     * onto the real singleton afterwards.</p>
     */
    public static RemixConfig load() {
        RemixConfig config = new RemixConfig();
        Path path = file();

        if (Files.exists(path)) {
            try (Reader reader = Files.newBufferedReader(path, StandardCharsets.UTF_8)) {
                RemixConfig parsed = GSON.fromJson(reader, RemixConfig.class);
                if (parsed != null) {
                    config.apply(parsed);
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
            config.writeJson();
        }
        return config;
    }

    /** Copies every persisted value from {@code other} onto this instance. */
    private void apply(RemixConfig other) {
        showStatus = other.showStatus;
        showArmor = other.showArmor;
        showWeapon = other.showWeapon;
        showTotems = other.showTotems;
        showKeystrokes = other.showKeystrokes;
        showKeybinds = other.showKeybinds;
        showTargetHud = other.showTargetHud;
        showPotions = other.showPotions;
        showWatermark = other.showWatermark;
        showCoords = other.showCoords;
        showPickups = other.showPickups;
        hitEffects = other.hitEffects;
        jumpCircles = other.jumpCircles;
        trails = other.trails;
        hitParticleColor = other.hitParticleColor;
        hitParticleCount = other.hitParticleCount;
        hitsound = other.hitsound;
        hitsoundVolume = other.hitsoundVolume;
        showCrosshair = other.showCrosshair;
        crosshairDot = other.crosshairDot;
        crosshairOutline = other.crosshairOutline;
        crosshairHitTint = other.crosshairHitTint;
        crosshairColor = other.crosshairColor;
        crosshairHitColor = other.crosshairHitColor;
        crosshairGap = other.crosshairGap;
        crosshairLength = other.crosshairLength;
        crosshairThickness = other.crosshairThickness;
        opacity = other.opacity;
        fullBrightness = other.fullBrightness;
        language = other.language;
        placements = other.placements == null ? new LinkedHashMap<>() : other.placements;
    }

    /** Persists the current values to both the JSON file and OneConfig's own store. */
    @Override
    public void save() {
        sanitize();
        writeJson();
        // The OneConfig tree only exists once its bootstrap ran; before that the JSON
        // file is the only store and super.save() would fail on the missing tree.
        if (tree != null) {
            super.save();
        }
    }

    /** Writes only the JSON file the launcher and the editor read. */
    private void writeJson() {
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

    /** Position and scale of one module. Negative coordinates mean "use the default spot". */
    public Placement placement(String id) {
        if (placements == null) {
            placements = new LinkedHashMap<>();
        }
        return placements.computeIfAbsent(id, key -> new Placement());
    }

    /** Restores every module to its default spot and scale. */
    public void resetLayout() {
        if (placements == null) {
            placements = new LinkedHashMap<>();
        }
        placements.clear();
    }

    /** Restores every option to its factory default. */
    public void reset() {
        showStatus = false;
        showArmor = false;
        showWeapon = false;
        showTotems = false;
        showKeystrokes = false;
        showKeybinds = false;
        showTargetHud = false;
        showPotions = false;
        showWatermark = false;
        showCoords = false;
        showPickups = true;
        hitEffects = true;
        jumpCircles = true;
        trails = true;
        hitParticleColor = 0xFFB44C;
        hitParticleCount = 8;
        hitsound = true;
        hitsoundVolume = 70;
        showCrosshair = false;
        crosshairDot = true;
        crosshairOutline = true;
        crosshairHitTint = true;
        crosshairColor = 0xFFFFFF;
        crosshairHitColor = 0xFF4C4C;
        crosshairGap = 4;
        crosshairLength = 6;
        crosshairThickness = 1;
        opacity = 100;
        fullBrightness = false;
        resetLayout();
    }

    /** Clamps every value into its supported range and normalises NaN. */
    public void sanitize() {
        opacity = Math.max(MIN_OPACITY, Math.min(MAX_OPACITY, opacity));
        if (!"auto".equals(language) && !"ru".equals(language) && !"en".equals(language)) {
            language = "auto";
        }

        crosshairColor &= 0xFFFFFF;
        crosshairHitColor &= 0xFFFFFF;
        crosshairGap = Math.max(MIN_GAP, Math.min(MAX_GAP, crosshairGap));
        crosshairLength = Math.max(MIN_LENGTH, Math.min(MAX_LENGTH, crosshairLength));
        crosshairThickness = Math.max(MIN_THICKNESS, Math.min(MAX_THICKNESS, crosshairThickness));

        hitParticleColor &= 0xFFFFFF;
        hitParticleCount = Math.max(MIN_HIT_PARTICLES, Math.min(MAX_HIT_PARTICLES, hitParticleCount));

        if (placements == null) {
            placements = new LinkedHashMap<>();
        }
        for (Placement placement : placements.values()) {
            if (placement == null) {
                continue;
            }
            if (Float.isNaN(placement.scale) || Float.isInfinite(placement.scale)) {
                placement.scale = 1.0f;
            }
            placement.scale = Math.max(MIN_MODULE_SCALE, Math.min(MAX_MODULE_SCALE, placement.scale));
            if (Float.isNaN(placement.x) || Float.isNaN(placement.y)) {
                placement.x = -1.0f;
                placement.y = -1.0f;
            }
        }
    }

    /** Configured opacity as an 0-255 alpha channel. */
    public int backgroundAlpha() {
        int clamped = Math.max(MIN_OPACITY, Math.min(MAX_OPACITY, opacity));
        return clamped * 255 / 100;
    }

    /** One module's layout; {@code x}/{@code y} are GUI-scaled pixels, {@code -1} means default. */
    public static final class Placement {
        public float x = -1.0f;
        public float y = -1.0f;
        public float scale = 1.0f;

        public boolean custom() {
            return x >= 0.0f && y >= 0.0f;
        }
    }
}
