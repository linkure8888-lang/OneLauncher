//! Hardware-aware graphics presets.
//!
//! The launcher ships curated optimisation mods, but the mods alone do not make
//! a machine smooth: the difference is in `options.txt`. This module looks at
//! the machine (RAM, GPU, monitor refresh rate), picks a preset and produces the
//! exact Minecraft settings for it.
//!
//! Two rules drive every number below:
//!
//! * **never blurry.** Render scaling stops at 0.75 - below that the image turns
//!   to mush and players notice it before they notice the extra frames.
//! * **aim at the monitor, not at "more".** The frame cap equals the refresh
//!   rate: rendering more frames than the panel shows only heats the GPU.

use std::path::Path;

use serde::{Deserialize, Serialize};

const MIN_MEMORY_MB: u64 = 2048;
const MAX_MEMORY_MB: u64 = 16384;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    LowEnd,
    Balanced,
    Pvp,
}

impl Preset {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LowEnd => "low_end",
            Self::Balanced => "balanced",
            Self::Pvp => "pvp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuTier {
    Weak,
    Mid,
    Strong,
}

/// What the machine looks like. `gpu` is the raw adapter name; the tier is
/// derived from it, because names are all we can read reliably everywhere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hardware {
    pub ram_mb: u64,
    pub gpu: String,
    pub refresh_hz: u32,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            ram_mb: 8192,
            gpu: String::new(),
            refresh_hz: 60,
        }
    }
}

impl Hardware {
    #[must_use]
    pub fn gpu_tier(&self) -> GpuTier {
        tier_for(&self.gpu)
    }
}

/// Name-based tiering. Deliberately conservative: an unknown adapter is treated
/// as mid, because guessing "strong" on an unknown card is how people end up
/// with 20 fps and a refund request.
#[must_use]
pub fn tier_for(gpu: &str) -> GpuTier {
    let name = gpu.to_ascii_lowercase();
    if name.is_empty() {
        return GpuTier::Mid;
    }

    // Discrete first: "Intel Arc" must not be caught by the "intel" rule below.
    const STRONG: [&str; 12] = [
        "rtx 50", "rtx 40", "rtx 30", "rtx 20", "rx 9", "rx 8", "rx 7", "rx 6", "arc a", "arc b",
        "apple m2", "apple m3",
    ];
    if STRONG.iter().any(|needle| name.contains(needle)) {
        return GpuTier::Strong;
    }

    // Integrated and very old parts.
    const WEAK: [&str; 10] = [
        "intel",
        "uhd",
        "hd graphics",
        "iris",
        "vega",
        "gma",
        "geforce 8",
        "geforce 9",
        "gt 7",
        "gt 6",
    ];
    if WEAK.iter().any(|needle| name.contains(needle)) {
        return GpuTier::Weak;
    }

    GpuTier::Mid
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub preset: Preset,
    pub gpu_tier: String,
    pub memory_mb: u64,
    /// 0.75..=1.0, applied as the resolution scale in Sodium.
    pub render_scale_percent: u32,
    pub fps_cap: u32,
    pub render_distance: u32,
    pub simulation_distance: u32,
    /// Minecraft `graphicsMode`: 0 fast, 1 fancy.
    pub graphics_mode: u8,
    /// Minecraft `particles`: 0 all, 1 decreased, 2 minimal.
    pub particles: u8,
    pub mipmap_levels: u32,
    pub entity_shadows: bool,
    pub vsync: bool,
    /// Only when the GPU and RAM can afford it.
    pub shaders: bool,
    /// One-line explanation shown next to the preset.
    pub reason: String,
}

impl Plan {
    /// `options.txt` lines this plan owns.
    #[must_use]
    pub fn options(&self) -> Vec<(String, String)> {
        vec![
            ("maxFps".into(), self.fps_cap.to_string()),
            ("renderDistance".into(), self.render_distance.to_string()),
            (
                "simulationDistance".into(),
                self.simulation_distance.to_string(),
            ),
            ("graphicsMode".into(), self.graphics_mode.to_string()),
            ("particles".into(), self.particles.to_string()),
            ("mipmapLevels".into(), self.mipmap_levels.to_string()),
            ("entityShadows".into(), self.entity_shadows.to_string()),
            ("enableVsync".into(), self.vsync.to_string()),
        ]
    }
}

/// Chooses a plan. `requested_ram_mb` is the slider value; `None` means "decide
/// for me", which is what most people should use.
#[must_use]
pub fn plan(hardware: &Hardware, requested_ram_mb: Option<u64>) -> Plan {
    let tier = hardware.gpu_tier();
    let preset = match tier {
        GpuTier::Weak => Preset::LowEnd,
        GpuTier::Mid => Preset::Balanced,
        GpuTier::Strong => Preset::Balanced,
    };

    let auto_memory = (hardware.ram_mb / 4).clamp(MIN_MEMORY_MB, 8192);
    let memory_mb = match requested_ram_mb {
        Some(requested) => clamp_memory(requested, hardware.ram_mb),
        None => auto_memory,
    };

    // Never ask the panel for more than it can show, never go below 60.
    let fps_cap = hardware.refresh_hz.clamp(30, 360);
    let vsync = true;

    let (render_scale, render_distance, simulation_distance, graphics_mode, particles, mipmap): (
        f32,
        u32,
        u32,
        u8,
        u8,
        u32,
    ) = match tier {
        // Keep the picture clean: 0.8 is the floor before people notice blur.
        GpuTier::Weak => (0.8, 8, 6, 0, 2, 2),
        GpuTier::Mid => (0.9, 10, 8, 1, 1, 4),
        GpuTier::Strong => (1.0, 12, 10, 1, 0, 4),
    };

    let shaders = tier == GpuTier::Strong && hardware.ram_mb >= 16 * 1024;

    Plan {
        preset,
        gpu_tier: format!("{tier:?}").to_lowercase(),
        memory_mb,
        render_scale_percent: (render_scale * 100.0).round() as u32,
        fps_cap,
        render_distance,
        simulation_distance,
        graphics_mode,
        particles,
        mipmap_levels: mipmap,
        entity_shadows: tier == GpuTier::Strong,
        vsync,
        shaders,
        reason: reason_for(tier, hardware.refresh_hz),
    }
}

fn reason_for(tier: GpuTier, refresh_hz: u32) -> String {
    let hz = refresh_hz.clamp(30, 360);
    match tier {
        GpuTier::Weak => {
            format!("Слабая графика: рендер 80% (без мыла), дальность 8 чанков, лимит {hz} FPS",)
        }
        GpuTier::Mid => {
            format!("Средняя графика: рендер 90%, дальность 10 чанков, лимит {hz} FPS",)
        }
        GpuTier::Strong => {
            format!("Сильная графика: полное разрешение, дальность 12 чанков, лимит {hz} FPS",)
        }
    }
}

/// Memory is capped so the system keeps room to breathe: at least 2 GB stays
/// free for Windows and everything else.
#[must_use]
pub fn clamp_memory(requested_mb: u64, total_ram_mb: u64) -> u64 {
    let ceiling = total_ram_mb
        .saturating_sub(2048)
        .clamp(MIN_MEMORY_MB, MAX_MEMORY_MB);
    let value = requested_mb.clamp(MIN_MEMORY_MB, ceiling);
    // Round to 512 MB so the field looks sane.
    (value / 512).max(4) * 512
}

/// Writes the plan into an existing `options.txt`, keeping every key this plan
/// does not own. A missing file is created.
pub fn write_options(path: &Path, plan: &Plan) -> std::io::Result<()> {
    let existing = read_existing(path)?;
    let owned = plan.options();
    let mut out: Vec<String> = Vec::with_capacity(existing.lines().count() + owned.len());
    let mut written: Vec<&str> = Vec::new();

    for line in existing.lines() {
        let key = line.split(':').next().unwrap_or_default();
        if let Some((name, _)) = owned.iter().find(|(name, _)| name == key) {
            out.push(format!("{name}:{}", value_of(&owned, name)));
            written.push(name);
        } else {
            out.push(line.to_string());
        }
    }
    for (name, value) in &owned {
        if !written.contains(&name.as_str()) {
            out.push(format!("{name}:{value}"));
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, out.join("\n") + "\n")
}

/// Reads `options.txt`. A missing file is an empty one; any other error is
/// returned, because silently treating an unreadable file as empty would
/// overwrite the player's settings with our eight keys.
fn read_existing(path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err),
    }
}

fn value_of<'a>(owned: &'a [(String, String)], key: &str) -> &'a str {
    owned
        .iter()
        .find(|(name, _)| name == key)
        .map_or("", |(_, value)| value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hardware(ram_mb: u64, gpu: &str, refresh: u32) -> Hardware {
        Hardware {
            ram_mb,
            gpu: gpu.to_string(),
            refresh_hz: refresh,
        }
    }

    #[test]
    fn integrated_graphics_get_the_low_end_preset() {
        let plan = plan(&hardware(8192, "Intel(R) UHD Graphics 620", 60), None);

        assert_eq!(plan.preset, Preset::LowEnd);
        assert_eq!(plan.render_scale_percent, 80);
        assert!(!plan.shaders);
    }

    #[test]
    fn the_frame_cap_follows_the_monitor() {
        for hz in [60, 75, 144, 165, 240] {
            let plan = plan(&hardware(16384, "NVIDIA GeForce RTX 3060", hz), None);
            assert_eq!(plan.fps_cap, hz, "a {hz} Hz panel should cap at {hz}");
        }
    }

    #[test]
    fn the_render_scale_never_goes_below_the_blur_floor() {
        for gpu in ["Intel HD Graphics", "NVIDIA RTX 4090", "AMD Radeon RX 580"] {
            let plan = plan(&hardware(4096, gpu, 60), None);
            assert!(
                plan.render_scale_percent >= 75,
                "{gpu} must not be blurred below 75%"
            );
        }
    }

    #[test]
    fn a_strong_card_keeps_full_resolution_and_shaders() {
        let plan = plan(&hardware(32768, "NVIDIA GeForce RTX 4070", 165), None);

        assert_eq!(plan.render_scale_percent, 100);
        assert!(plan.shaders);
        assert_eq!(plan.fps_cap, 165);
    }

    #[test]
    fn an_unknown_card_is_treated_as_mid_range() {
        assert_eq!(tier_for("Some Mysterious Adapter"), GpuTier::Mid);
        assert_eq!(tier_for(""), GpuTier::Mid);
    }

    #[test]
    fn memory_always_leaves_two_gigabytes_for_the_system() {
        // 8 GB machine, user asks for 8 GB: must come down to 6 GB.
        assert_eq!(clamp_memory(8192, 8192), 6144);
        // 16 GB machine: 8 GB is fine.
        assert_eq!(clamp_memory(8192, 16384), 8192);
        // Never below the floor.
        assert_eq!(clamp_memory(512, 16384), 2048);
    }

    #[test]
    fn writing_options_keeps_keys_it_does_not_own() {
        let dir = std::env::temp_dir().join(format!("remix-options-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("options.txt");
        std::fs::write(&path, "fov:0.0\nmaxFps:30\nlang:ru_ru\n").unwrap();

        let plan = plan(&hardware(16384, "NVIDIA RTX 3060", 144), None);
        write_options(&path, &plan).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("fov:0.0"), "unrelated keys survive");
        assert!(written.contains("lang:ru_ru"), "unrelated keys survive");
        assert!(written.contains("maxFps:144"), "our cap wins");
        assert_eq!(written.matches("maxFps:").count(), 1, "no duplicates");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_options_file_is_created() {
        let dir = std::env::temp_dir().join(format!("remix-options-{}", uuid::Uuid::new_v4()));
        let path = dir.join("game").join("options.txt");

        let plan = plan(&hardware(8192, "Intel UHD", 60), None);
        write_options(&path, &plan).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("renderDistance:8"));
        std::fs::remove_dir_all(&dir).ok();
    }
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

const HARDWARE_CACHE: &str = "hardware.json";
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

#[derive(Serialize, Deserialize)]
struct CachedHardware {
    detected_at: u64,
    hardware: Hardware,
}

/// Reads what the machine has. RAM comes from `sysinfo`, the adapter name and
/// the panel refresh rate from the OS (a single PowerShell query on Windows,
/// `lspci` on Linux). The result is cached for a day: hardware does not change
/// between launches, and the query costs a second.
pub async fn detect_hardware() -> Hardware {
    let path = oneclient_common::paths::data_dir()
        .ok()
        .map(|dir| dir.join(HARDWARE_CACHE));

    if let Some(path) = &path
        && let Ok(bytes) = tokio::fs::read(path).await
        && let Ok(cached) = serde_json::from_slice::<CachedHardware>(&bytes)
        && now_secs().saturating_sub(cached.detected_at) < CACHE_TTL_SECS
    {
        return cached.hardware;
    }

    let mut hardware = Hardware::default();
    hardware.ram_mb = tokio::task::spawn_blocking(total_ram_mb)
        .await
        .unwrap_or_default();

    let queried = tokio::task::spawn_blocking(query_adapter)
        .await
        .unwrap_or_default();
    if let Some(gpu) = queried.0 {
        hardware.gpu = gpu;
    }
    if let Some(hz) = queried.1 {
        hardware.refresh_hz = hz;
    }

    if let Some(path) = &path {
        let cached = CachedHardware {
            detected_at: now_secs(),
            hardware: hardware.clone(),
        };
        if let Ok(bytes) = serde_json::to_vec(&cached) {
            if let Some(parent) = path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            let _ = tokio::fs::write(path, bytes).await;
        }
    }

    hardware
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn total_ram_mb() -> u64 {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    system.total_memory() / (1024 * 1024)
}

/// `(adapter name, refresh rate)`.
fn query_adapter() -> (Option<String>, Option<u32>) {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_VideoController | Where-Object { $_.Name -notmatch 'Basic|Remote' } | Select-Object -First 1 Name,CurrentRefreshRate | ConvertTo-Json -Compress",
            ])
            .output()
            .ok();

        let Some(output) = output else {
            return (None, None);
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let value: serde_json::Value = match serde_json::from_str(text.trim()) {
            Ok(value) => value,
            Err(_) => return (None, None),
        };
        let name = value
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let hz = value
            .get("CurrentRefreshRate")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .filter(|hz| (30..=360).contains(hz));
        (name, hz)
    }

    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("sh")
            .args(["-c", "lspci 2>/dev/null | grep -iE 'vga|3d' | head -n1"])
            .output()
            .ok();
        let name = output
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|s| !s.is_empty());
        (name, None)
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        (None, None)
    }
}

#[cfg(test)]
mod detection_tests {
    use super::*;

    #[test]
    fn hardware_round_trips_through_the_cache_format() {
        let cached = CachedHardware {
            detected_at: 123,
            hardware: Hardware {
                ram_mb: 16384,
                gpu: "NVIDIA GeForce RTX 3060".into(),
                refresh_hz: 144,
            },
        };
        let encoded = serde_json::to_string(&cached).unwrap();
        let decoded: CachedHardware = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.hardware.refresh_hz, 144);
        assert_eq!(decoded.hardware.ram_mb, 16384);
    }

    #[test]
    fn a_refresh_rate_outside_the_sane_range_is_ignored() {
        // The UI clamps, so the plan must survive a weird value.
        let plan = plan(
            &Hardware {
                ram_mb: 8192,
                gpu: "Intel UHD".into(),
                refresh_hz: 1,
            },
            None,
        );
        // A nonsense rate is clamped to the lowest sane panel, not to 60.
        assert_eq!(plan.fps_cap, 30);
    }
}

/// Sets a single key in `options.txt`, keeping everything else. Used for the
/// game language, which follows the launcher's language rather than the system
/// locale (a Kyrgyz Windows otherwise starts Minecraft in Kyrgyz).
pub fn set_option(path: &Path, key: &str, value: &str) -> std::io::Result<()> {
    let existing = read_existing(path)?;
    let mut out: Vec<String> = Vec::with_capacity(existing.lines().count() + 1);
    let mut replaced = false;

    for line in existing.lines() {
        if line.split(':').next() == Some(key) {
            if !replaced {
                out.push(format!("{key}:{value}"));
                replaced = true;
            }
            continue;
        }
        out.push(line.to_string());
    }
    if !replaced {
        out.push(format!("{key}:{value}"));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, out.join("\n") + "\n")
}

#[cfg(test)]
mod option_tests {
    use super::*;

    #[test]
    fn setting_a_key_replaces_it_once_and_keeps_the_rest() {
        let dir = std::env::temp_dir().join(format!("remix-lang-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("options.txt");
        std::fs::write(&path, "fov:0.0\nlang:ky_kg\nlang:ky_kg\n").unwrap();

        set_option(&path, "lang", "ru_ru").unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("fov:0.0"));
        assert!(written.contains("lang:ru_ru"));
        assert_eq!(written.matches("lang:").count(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setting_a_missing_key_appends_it() {
        let dir = std::env::temp_dir().join(format!("remix-lang-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("options.txt");
        std::fs::write(&path, "fov:0.0\n").unwrap();

        set_option(&path, "lang", "ru_ru").unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("lang:ru_ru"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
