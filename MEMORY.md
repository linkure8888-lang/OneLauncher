# RemixClient — project memory

This file is the single place for decisions, feature status, placeholders and the
next steps. Update it whenever a feature lands or a placeholder is replaced.

- Project: **RemixClient** — offline-first community fork of
  [OneLauncher](https://github.com/Polyfrost/OneLauncher) by Polyfrost.
- License: **GPL-3.0-only**. Keep `LICENSE`, `ATTRIBUTION.md` and upstream
  copyright notices. This is an unofficial fork, not affiliated with Polyfrost,
  Mojang or Microsoft.
- Scope decision: rebrand only the user-facing product (brand, UI, installer,
  data folders). Internal crate names stay `oneclient_*` on purpose.

## Current status

| Area | Status | Notes |
|---|---|---|
| Offline accounts | done | Nickname + local UUID (`MD5("OfflinePlayer:<name>")`), no Microsoft call, `--userType legacy`, `--accessToken 0` |
| Offline skins/capes | done (needs in-game test) | Local HTTP texture server + `--userProperties`; UI preview comes from the local server, no Mojang request |
| Microsoft sign-in | **removed (v2.12.0)** | No buttons, no flows, no browser opens — see session notes below |
| Portable mode | done | `--portable` / `-p`, `portable.txt` marker, `RemixClient-data` next to the exe, separate IPC |
| World backups | done | Auto before launch (changed worlds only, retention 5), manual backup + restore from Worlds → right-click → Backups… |
| Offline skins/capes | done (needs in-game test) | Local HTTP texture server + `--userProperties`, shared dialog in Accounts and Wardrobe |
| Branch / commits | done | Work lives on `remix/offline-and-rebrand`; `ce9849e7` features, `8e2a73d1` review fixes |
| Crash autopsy | done | Suspected mods get a one-click **Disable** button |
| Discord RPC | done | App ID `1569698419491217500`; Discord desktop must be running; large image optional |
| Russian | done | `lang/ru_RU.json`, 100% of keys; machine-translated + curated, proofreading welcome |
| Help tab | done | Settings → Help, feature guide + report/logs buttons |
| Report button | placeholder | `ISSUES_URL` in `packages/oneclient_app/src/view/app/settings/help.rs` points to `https://github.com/linkure8888-lang/OneLauncher/issues/new/choose`; replace when the real repo exists |
| MSI / release | **v2.12.0 published** | Tag pushed 2026-11-24 evening, workflow Release builds assets; old releases (v2.10.0…v2.11.5) deleted via API, only the actual one remains |
| Hardware auto-tune | done | First launch of a cluster writes the hardware preset into `options.txt` automatically (only while the file does not exist yet); button in settings stays |
| CI / GitHub Actions | todo | fmt, clippy, tests, MSI, release signing |
| Code signing (Authenticode) | skipped | User chose to skip; SmartScreen will warn |
| ToS / Privacy | done | Bundled `assets/legal/{terms,privacy}.md` compiled into the binary (`TERMS_VERSION`/`PRIVACY_VERSION`), no network, no Polyfrost text. Repo copies in `docs/` |
| Sentry | disabled | No DSN by default; `REMIXCLIENT_SENTRY_DSN` enables it |
| CurseForge | works | Embedded upstream key; override with `REMIXCLIENT_CURSEFORGE_API_KEY` |
| Updater | done (endpoint pending) | Endpoint `https://github.com/linkure8888-lang/OneLauncher/releases/latest/download/latest.json`; new minisign key pair generated |
| Linux/mac packaging | partial | IDs renamed to `org.remixclient.RemixClient`; macOS `OneClient.icon` assets remain; untested |
| App icon | done | Real design from the user's image (`design/icon-source.jpg`): transparent master `C:\Users\user\Desktop\remix-icon.png` → PNG set, `icon.ico` (16–256), `icon.icns` (16–1024), tray, StoreLogo. Generator: `%TEMP%\remix-icon-tool` (Rust + `image`) |
| HUD (axe/sword/totem + presets) | done in `companion/`, not shipped | Movable HUD modules + on-screen editor, see "Companion mod" below. Still missing the launcher-side install |
| QA sandbox | research | Subagent researching Windows Sandbox / separate desktop / UI automation |

## Session notes: v2.12.0 (2026-11-24 evening)

- **Microsoft login cut completely**: deleted `components/microsoft_login.rs`, buttons/popups in
  `view/app/settings/accounts.rs` + `view/onboarding/account.rs`, mutations Begin/Finish/Cancel/
  AddMicrosoftAccount in `hooks/queries/auth.rs` (+ `leave_party_before_sign_out` helper that used
  `oneclient_polyplus`, which is NOT a dependency of oneclient_app), re-exports in
  `hooks/queries/mod.rs` + `hooks/mod.rs`, 16 i18n keys from both langs. No site redirects remain.
- **Careful with `git show HEAD:hooks/queries/auth.rs`** — the HEAD version references
  `oneclient_polyplus` and does not compile; the working-tree version had it removed already.
- **Hardware auto-tune**: `oneclient_core/src/clusters/prepare.rs`, after stage=Ready, if
  `options.txt` does not exist yet → detect hardware, write the preset (FPS = refresh rate,
  render scale by GPU tier). After the game's first run the file exists → never overwritten.
- **Release v2.12.0**: version bumped in workspace `Cargo.toml`, commits pushed to main, tag
  `v2.12.0` pushed → workflow Release builds exe/dmg/AppImage + `latest.json` (signed).
  Old releases v2.10.0/v2.11.2/v2.11.3/v2.11.4/v2.11.5 deleted via GitHub API (bearer token from
  `git credential fill`), tags removed too. Only the latest release remains.
- Push works with the locally stored GitHub credentials (`git credential fill`), no `gh` CLI installed.

## Session notes: v2.12.1 (2026-11-25)

- **GPU detection fixed**: `query_adapter()` in `oneclient_core/src/presets.rs` now filters out
  virtual/emulator display adapters (MuMu, BlueStacks, LDPlayer, Nox, Hyper-V, Indirect, Basic,
  Remote) and sorts by `CurrentHorizontalResolution` desc — picks the card driving the biggest
  real screen. MuMu virtual adapter (16 Hz) was winning "First 1" before; now RTX 2060 @ 180 Hz.
  `hardware.json` cache in both profile data dirs was deleted so detection re-runs.
- **Bundle catalog retired for good** (user: card in mods picker, ugly Polyfrost design):
  `BundlesManager::sync()` is now a no-op that purges the whole catalog via
  `bundle_dao::purge_catalog` (plain non-macro sqlx queries: bundles,
  cluster_bundle_choices, cluster_bundle_overrides, cluster_bundle_type_opt_outs). Existing
  user clusters untouched. No more traffic to data-v2.polyfrost.org for bundles.
- **Cosmetics placeholder** mentions Remix Coin (future currency for cosmetics, fair play).
- **Crash reports**: `SENTRY_DSN` is compile-time `option_env!("REMIXCLIENT_SENTRY_DSN")`,
  empty by default → Sentry never initializes, the settings toggle does nothing, nothing
  leaves the PC. To actually receive reports user needs a sentry.io account + rebuild with DSN.
- **Auto-update**: wired and enabled in release builds (events.rs:402 → spawn_update_check).
  Debug builds: check runs but only notify-only toast. If user sees no prompt it means their
  version == latest.json version.
- **TG avatar**: use `C:/Users/user/Desktop/OneClient/avatar.png` (the launcher 512x512 icon
  with the white R) — NOT the tile-only OneConfig brand mark.
- After v2.12.1 release completes, delete the v2.12.0 release via API (keep only the actual one).

## Session notes: v2.12.5 + companion batch (2026-11-26)

- **Auto-update root cause fixed**: latest.json lacked the required `format` field per platform
  (cargo-packager-updater refuses it) → released v2.12.5 with the fix; macOS entries say `app`
  even for dmg. Player confirmed auto-update works. Only the latest release is kept on GitHub.
- Careful: tag v2.12.4 shipped without the Cargo.toml bump (interrupted turn) → immediately
  superseded by v2.12.5. Always verify `git show <tag>:Cargo.toml` has the right version.
- **Companion batch built** (Pulse-style, own code): hit sound (crit pitch higher, volume slider),
  saturation HUD (10 squares, exact saturation, air when < 100%), pickup logger (inventory
  diffed per tick, entries fade after ~3s). Defaults ON, deployed to both clusters.
- Totem tracker + CPS already existed (status module / keystrokes).
- Batch 2 candidates from the Pulse backlog: Freelook, No Render (rain/shake/fire),
  Time Changer (26.3 changed time to WorldClock/ClockManager — needs research before mixin),
  Streamer Mode, Hitbox Customizer, Block Overlay. ESP-like (Target ESP, China Hat) stay out
  by the fair-play rule.

### Session notes: OneConfig dup + poly-stack cleanup (2026-11-26, late)
- **OneConfig 1.9.4 came back** next to 1.6.2 in the MAIN cluster (content sync re-downloaded it);
  141 jars total after the user installed a poly* stack himself (freelook, polytime, polyweather,
  chatting, autotext, betterstats, Gnetum, NBTac, ...). The lag the user complained about was the
  duplicate OneConfig init.
- Removed from both clusters + DB rows purged so they do not re-download: OneConfig-1.9.4,
  evergreenhud, vanillahud, damagetint, polycrosshair, polyblur (all duplicate our own modules).
  33 mods in the main cluster still declare a OneConfig dependency; that is the user's own install
  and he decides whether to thin it out.
- OneConfig **module switches removed from the settings UI** (player asked repeatedly): the
  @Switch/@Slider annotations for showStatus/showArmor/showWeapon/showTotems/showKeystrokes/
  showKeybinds/showTargetHud/showPotions/showWatermark/showCoords/opacity were stripped;
  plain fields keep the values via the JSON file; HUD editor (Right Alt) toggles modules.
  Visuals (hit particles, jump circles, trails, crosshair) keep their switches.
- Hit sound + pickup logger removed at the player's request (he saw them in game and did not
  like them). Saturation HUD was removed even earlier. Lesson: propose the batch as a list,
  wait for his pick, only then build.
- STILL OPEN: the skins page (and other full-width ScrollArea pages) clips content flush at the
  right window edge (empty-state card glued to the border). Suspected torin fill+padding
  measurement; needs a live repro to fix properly.

### OneConfig 1.9.4 rebrand (2026-11-26, session 2)
- User's own poly-stack mods (autotext, chatting, polydium, polycompose, Custom Block Highlight,
  CrashPatch, Screencapper...) require OneConfig >=1.8/1.9; our branded fork was 1.6.2, so the
  game refused to start. Rebuilt the rebrand on **OneConfig 1.9.4** (source: dev-profile metadata
  cache metadata/packages/mods/modrinth/AibBIVmj/Mr9retFM/OneConfig-26.3-fabric-1.9.4.jar).
- Rebrand recipe (script at %TEMP%/oc9/brand.cjs): outer fabric.mod.json name/description/authors/
  contact + oneconfig-icon.png swapped; nested internal-1.9.4.jar gets the branded assets from
  %TEMP%/oc/patchstage/internal_x (brand/*.svg+png, ico/default_mod.svg, images/minecraft-*,
  polyglass-*) plus 1.9.4-only files pointed at our art: brand/oneconfig-white.svg -> oneconfig.svg,
  images/polyfrosted-dark|light.svg -> polyglass replacements; en_default.lang of 1.9.4 itself
  gets 'OneConfig'->'Remix Companion' + polyfrost.org/mods->t.me/remixclient_ru in VALUES only.
- Deployed as OneConfig-26.3-fabric-1.9.4.jar to both clusters (1.6.2 removed); DB
  cluster_artifacts.cluster_file_name renamed. Companion API classes all exist in 1.9.4
  (Config/Property/Tree/Slider/Switch/OneConfigUI verified by byte-scan of nested jars).
- Crash dialog Telegram card: body label had max_lines(1) -> RU text ran under the button;
  removed max_lines so it wraps (crash_dialog/mod.rs discord_card).
- Launcher-side theme images in OneConfig (polyfrosted-*) now show Remix art.

### Light profile + violet accent (2026-11-28, local only — NO release)
- User asked: de-lag OneConfig for weak PCs + gradient redesign, everything rollback-able per item,
  no release. Applied LOCALLY via config files (no jar change needed for the runtime look):
  .minecraft/config/themes.json + oneconfig.json:
  blur OFF (heaviest), all menu/page animations OFF, instantSearch OFF, marquee OFF, UI sounds
  OFF (all kinds), pageOpacity/sidebarOpacity 100, glowOpacity 12, accentColor [124,92,255] Remix
  violet, glassTint/pixelGrid 0, activeTheme stays Nord. theme SVGs in the jar are only picker
  thumbnails — real look = code-drawn glass, so config is the lever.
- Rollback kit: %TEMP%/oc9/rollback/ (oneconfig.json.orig, themes.json.orig,
  OneConfig-remix-1.9.4.v1.jar). Per-item revert = restore file or restore jar.
- Launcher launched via oneclient://launch/26.3%20Fabric URL arg did NOT trigger the game boot
  (no pending_launch log) — --launch "26.3 Fabric" got split at the space by arg parsing
  (folder="26.3" bug in cli.rs? worth checking later). Launcher left running + minimized.
- Static hard check: ALL PASSED (configs, single branded 1.9.4 in both clusters, companion
  deployed, rollback kit). Real game boot pending user's manual test.

## Keys, secrets and endpoints

- Discord Application ID: `1569698419491217500` (`oneclient_common/src/constants.rs`).
- Discord large image: optional `REMIXCLIENT_DISCORD_LARGE_IMAGE`; without it Discord
  shows the application icon uploaded in the Developer Portal.
- Updater private key: `C:\Users\user\remixclient-updater.key` (never commit, back it up).
  Public key is embedded in `packages/oneclient_app/src/constants.rs`.
- Updater releases URL: `https://github.com/linkure8888-lang/OneLauncher/releases/latest`.
- Microsoft app id override: `REMIXCLIENT_MICROSOFT_CLIENT_ID`.
- CurseForge key override: `REMIXCLIENT_CURSEFORGE_API_KEY`.
- Sentry: `REMIXCLIENT_SENTRY_DSN` (empty by default).
- Portable flag: `REMIXCLIENT_PORTABLE=1` also works.
- Discord RPC only works while the Discord desktop client is running and logged in.
  There is no server component.

## Companion mod (source ready, not shipped)

The launcher does **not** install `companion/`; no Rust code references it. It was only
copied by hand into the QA sandbox and has been removed from there. Decision 2026-11-15:
keep the source for the 3.0.0 redesign, but do not ship it — everything it showed then
already exists in the bundle (armor: `DetailArmorBarReconstructed`), and the extra Right Alt
hotkey confuses players. OneConfig and the eight `poly*` mods stay for now; `polydium` has
**no license field**, so it must never be rebuilt or rebranded.

2026-11-21: the mod grew into a module set modelled on the feature list of the paid
**Pulse Visuals** client — own code, own look, no assets or branding taken from it (and we
will never copy its paid-subscription model; the project stays free). What is in now:
keystrokes with CPS, keybinds panel (sneak/sprint/offhand), target HUD (name/HP/armor/
distance), potion timers, watermark (FPS/ping/coords), custom crosshair (gap/length/
thickness/dot/outline, tints on a living target), hit particles on the hit point (gold burst
on crit) via `ClientPreAttackCallback`, jump circles as a client-side particle ring, and a
full-brightness toggle on **B**. All of it is Fabric events only except the crosshair, which
needs `mixin/HudMixin.java` to cancel `Hud.extractCrosshair`; the mixin config is
`required: false` and the custom crosshair refuses to draw until the mixin has actually
hidden the vanilla one, so a failed mixin degrades to the vanilla crosshair.

Deliberately not copied from Pulse: **damage numbers** (the client never sees the damage a
hit dealt, so any number would be invented) and **AutoFish** (a client-side autoclicker on
the fishing bobber breaks server rules and gets accounts banned).

2026-11-21 (later): our own settings panel was thrown away — it was our design, it looked
bad, and the player asked for OneConfig-style layout editing instead. Nothing of it is
left: `RemixSettingsScreen` is deleted, and Right Alt now opens `RemixHudEditorScreen`.
Each module implements `RemixModule` (id, title, enable flag, unscaled size, local-origin
render, default origin) and the HUD applies `placements.<id>.{x,y,scale}` around it:
**right-drag** moves, **wheel** scales (0.5–2.5x), **left-click** toggles, **R** resets the
layout, **Esc** saves. The crosshair is `movable() == false` — it always sits on the
crosshair position, but it still scales with the wheel.

Why not the real OneConfig: 1.6.1 does ship a 26.3 Fabric build, but it drags in Compose
Multiplatform + Fabric Language Kotlin and the whole Polyfrost stack, which the 3.0 plan is
moving away from, and `CLAUDE.md` bans Polyfrost references in the product. So the editor
reproduces the interaction, not the dependency. If the real integration is ever wanted it
is a separate decision, not a drop-in.

Not done yet (needs world-render mixins, not HUD events): tracer lines, jump circles, hit
particles, hitbox customizer, custom hand, aspect ratio, scope overlay. Also `build.gradle`
now sets `useLegacyMixinAp = false`: the legacy mixin AP cannot map names in an
unobfuscated game and fails the build.

Still open: nothing installs the jar into instances. Next step is launcher-side install
(bundle entry, path `mods/remix-companion-<version>.jar`, Fabric loader required) plus an
in-game test — none of the above has been launched in a real client yet.

## Telegram

- Channel: `@remixclient_ru` (`RemixClient`, id `-1004487205418`). Bot: `@remix_client_bot`
  (id `9106394088`), administrator with post / edit / delete / pin / change-info.
- The bot token lives in `.env` in the repo root as `TG_TOKEN=...` (gitignored). Keys
  must be dotenv-valid: `sqlx` parses `.env` at compile time, so a hyphen in the key
  name (`tg-token`) breaks every `sqlx::query!` with "error reading dotenv file". The
  posting script reads `.env` first, then `REMIX_TG_TOKEN`, then
  `%USERPROFILE%\.remix\tg-token.txt`, and refuses to post anywhere except the
  channel unless `-AllowAnyChat` is passed.
- Channel avatar: `design/channel-avatar.png` — the white R from
  `packages/oneclient_app/icons/512x512.png` on a full-bleed violet→cyan gradient,
  so the circular crop has no dead margin. Generator lives in
  `%TEMP%\remix-avatar` (Rust + `image`); regenerate with
  `remix-avatar <icons/512x512.png> <out.png> 0.60`.
- Channel description is set through the Bot API, not by hand.

## Release procedure (verified 2026-11-15)

Pushing a tag does **not** start the Release workflow on this fork: commits and tags
since 2026-11-11 get zero check-runs. Dispatch it explicitly instead. The Windows
Credential Manager stores a PAT with `repo` + `workflow` scopes, so
`git credential fill` yields it without any manual step:

```bash
TOKEN=$(printf 'protocol=https\nhost=github.com\n\n' | git credential fill | sed -n 's/^password=//p')
curl -s -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Accept: application/vnd.github+json" \
  https://api.github.com/repos/linkure8888-lang/OneLauncher/actions/workflows/release.yml/dispatches \
  -d '{"ref":"main","inputs":{"tag":"v2.11.4"}}'
```

Then bump `version` in the root `Cargo.toml` (and `oneclient_social`), commit, push,
create the tag, push it, and dispatch. A full run takes ~25 min and now builds four
platforms: Windows NSIS, Linux tarball, macOS arm64 dmg, macOS Intel dmg.

Known gaps in the updater manifest (`latest.json`): Linux has no entry because the
tarball is not signed by cargo-packager, and macOS builds are not Apple-notarised,
so Gatekeeper warns on first launch.

## Build and test

Run commands through the MSVC wrapper so `link.exe` resolves to Visual Studio:

```powershell
C:\Users\user\msvc.cmd cargo check -p oneclient_app
C:\Users\user\msvc.cmd cargo test -p oneclient_app --lib
C:\Users\user\msvc.cmd cargo test -p oneclient_auth
C:\Users\user\msvc.cmd cargo test -p oneclient_mc -j 1
C:\Users\user\msvc.cmd cargo test -p oneclient_common
C:\Users\user\msvc.cmd cargo test -p oneclient_core --lib world_backup -j 1
C:\Users\user\msvc.cmd cargo test -p oneclient_core --lib offline_skin -j 1
```

- `SQLX_OFFLINE=true` is set by the wrapper.
- Full `cargo test -p oneclient_core` may fail on this machine because of a small
  Windows pagefile (`os error 1455`) while linking integration tests. Use `--lib`
  and `-j 1`.
- Release build: `C:\Users\user\msvc.cmd cargo build --release -p oneclient_app`.
  On this machine the default `lto = "fat"` profile crashes `link.exe`
  (`STATUS_STACK_BUFFER_OVERRUN`, 0xc0000409). Use:
  `set CARGO_PROFILE_RELEASE_LTO=thin&& set CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`
  before the build; that links fine (~8 min incremental).
- MSI (when ready): `cargo packager -p oneclient_app --release -f wix`.
  `cargo-packager` does not build the binary itself; build release first.

## Data folders

| Build | Folder |
|---|---|
| Release | `%APPDATA%\Remix\RemixClient` |
| Debug | `%APPDATA%\Remix\RemixClient-dev` |
| Portable | `<exe dir>\RemixClient-data` |

First launch migrates `%APPDATA%\Polyfrost\OneClient(-dev)` when the new folder is
missing. World backups live in `<instance>/backups/worlds/<world>/*.zip`. Offline
skins live in `<data>/skins/<account uuid>/`.

## Subagent policy (hard rule)

- **Never** launch a subagent on a paid model. Always use a free / flash-tier model.
- Default free picks: `deepai/qwen-3.8-flash`, `deepai/deepseek-v4.1-flash`,
  `deepai/glm-5.3-flash`, `deepai/mimo-2.6-flash`, `deepai/minimax-m3.1-flash`,
  `deepai/solar-mini-4`.
- Never delegate to Claude / Opus / GPT / "-max" / "-pro" / "prime" models.
- Paid-tier subagents burned the daily limit on 2026-11-10 and produced no reports.
  The main thread may use whatever model the user picked; only children are restricted.

## Decisions

- Keep Polyfrost **content** services (bundles, metadata, translations, Poly+) for
  compatibility, but self-updates point to the RemixClient GitHub repo so an update
  cannot turn the fork back into OneClient.
- Sentry is off by default; Discord RPC is opt-out via settings.
- Offline accounts never touch Microsoft. Microsoft sign-in is optional.
- World backups are per instance and skip unchanged worlds; restore always creates a
  safety backup first.
- Offline skin textures are served from a loopback HTTP server for the launcher
  lifetime; the game receives a standard `textures` profile property.
- HUD/cosmetics that draw over Minecraft are **not** a launcher feature; they need a
  mod. Do not add a fake launcher-only setting for them.

## Placeholders to replace

1. `ISSUES_URL` in Help → real GitHub repository URL.
2. Updater endpoint → real repository releases (`latest.json` + signed artifacts).
3. Discord large image key → upload `remixclient_logo` in the Developer Portal and set
   `REMIXCLIENT_DISCORD_LARGE_IMAGE`.
4. App icon — done. Regenerate with the Rust tool in `%TEMP%\remix-icon-tool`
   (`remix-icon-tool.exe design/icon-source.jpg`); it finds the square, makes it
   transparent and writes every size.
5. ToS/Privacy — done, bundled; bump the version constants when the text changes.
6. Microsoft/CurseForge keys → own values for public builds.
7. `org.polyfrost.*` identifiers in Linux/mac packaging — done, untested.
8. MSI + code signing decision.

## HUD research (free-tier subagent, Modrinth API, 2026-11-10)

- Existing mods cover armor bars and durability numbers, never the requested
  «axe + sword + totem + one-shot hit animation + presets» combo.
- Candidates (all: no preset system; launcher would have to write each mod's own
  `config/` file): `ukus-armor-hud` (MIT, 10M downloads, config file), `armor-hud`
  (MIT), `durability-viewer-updated` (MIT, thresholds), `detail-armor-bar-reconstructed`
  (MIT), `betterhudfabric` (AGPL, Cloth Config), `totem-counter-mx` and `inventorio`
  (All-Rights-Reserved — cannot be bundled), `durability101`, `show-durability`.
- Recommendation: write a small MIT companion mod for **Fabric 1.21.1** (widget +
  one frame animation + a preset JSON the launcher can generate). Bundling 2–3 MIT
  mods is only a stopgap: inconsistent settings, no hit animation.
- Still needed from the user: confirm 1.21.1 Fabric as the first target (or 1.8.9).

## QA run (2026-11-11, portable mode)

Harness: `%TEMP%\remix-qa` — `RemixClient.exe --portable`, window moved to
`-5000,-5000`, screenshots via `PrintWindow`, clicks via `PostMessage`
(`start.ps1`, `qa.ps1`). Synthetic typing does NOT reach the window (needs real
foreground focus), so the account was seeded by writing `auth.json`.

Verified end to end: portable folder + `portable.txt`, full onboarding,
`ru_RU`, bundled Terms/Privacy render, offline account shown on the summary,
Microsoft JDK 25 auto-download, Minecraft+Fabric 26.3 download (1.5 GB), game
process spawned with the offline account.

Findings:
- **polyplus-1.4.7** inside the default bundle keeps calling Mojang `joinServer`
  and retries the Poly+ websocket (401) on an offline account. The game still
  starts and plays; the log is noisy and P2P/invites do not work. Not fixed yet:
  the mods dir is manifest-managed, so simply moving the jar would be undone on
  the next launch.
- The updater logs `update endpoint did not respond with a successful status`
  on every start — expected until the GitHub repo exists.
- Launching the game makes real window + sound on the user's desktop; a real
  sandbox (Hyper-V/Windows Sandbox) is the only way to hide that.

## Known issues / reminders

- Discord RPC requires the Discord desktop app open; closing it removes the status.
- Offline skins still need a real-game verification on the user's target version.
- Verified by a free-tier review subagent (Qwen 3.8 Flash) on 2026-11-10 and fixed:
  `--userProperties` must be `{"textures":[{"name":"textures","value":<base64>}]}`
  (a bare base64 string is dropped by authlib); offline UI no longer calls
  `sessionserver.mojang.com`; 128x64 skins accepted; a failed texture-server bind
  is retried; offline launches send `--accessToken 0`.
- `CredentialsStore::add_offline_account` is memory-only (tests only); production
  must use `add_offline_account_and_save`.
- World backup zips currently read each file into memory; very large worlds can be slow
  and the 180 s timeout may skip them.
- Update check fails until the GitHub repository exists (expected).
- Russian is machine-translated in places; review the main screens.
- Report button is a placeholder until the repository is public.

## User decisions (2026-11-10)

- **GitHub / release infra**: the user will create the repository; MSI, CI and signing come after that.
- **ToS/Privacy**: not understood / postponed. Keep as backlog; the current onboarding still shows Polyfrost terms.
- **Keys**: Discord is enough for now. CurseForge users can attach their own key. Microsoft sign-in is optional.
- **LAN hosting, mod health, instance snapshots, low-end profile**: not priorities right now; keep in backlog. World backups already cover the critical data-loss case.
- **Report button**: placeholder GitHub Issues URL is acceptable until the repository is public; this file tracks the replacement.
- **Memory**: this `MEMORY.md` is the source of truth for features, placeholders and decisions.
- **QA sandbox**: the user wants a separate desktop/environment where the agent can run RemixClient, click through all settings and report broken/unneeded features. The agent must ask before disabling anything.
- **HUD**: the user wants small in-game axe/sword/totem indicators with one-shot hit animations, 3 popular presets and saved custom presets. This is a mod/client feature, not launcher-only; research in progress.
- **Visuals**: add later; do not fake a launcher-only HUD without a mod.

## Open questions

- Which Minecraft version and loader should the HUD mod target first (Fabric/Forge/NeoForge, 1.21.x / 1.8.9)?
- Does the user want a companion HUD mod written from scratch, or an existing mod bundled with launcher presets?
- Is a separate Windows Sandbox/VM acceptable for QA, or should QA use portable mode + UI automation in the current session?
- Should the fork keep Polyfrost content services long-term or move to self-hosted endpoints?

## Next steps

1. Collect the three research reports (HUD, QA sandbox, competitive features).
2. Integrate the user's AI icon and regenerate the launcher icon set.
3. Create the GitHub repository, CI, signed release, MSI.
4. Replace ToS/Privacy and the report URL.
5. Decide the HUD approach and implement the first preset (after user approval).

## RemixServer (self-hosted test cluster)

Lives outside the repository at `C:\Users\user\Desktop\RemixServer` (never commit
worlds or jars). Layout: `proxy/` Velocity 4.3.1 on 25565, `lobby/` Paper 26.3 on
25566, `duels/` Paper 26.3 on 25567, bundled JDK 25 in `java/`, `plugin*/` sources
for our own plugins, `START.bat` / `STOP.bat` / `BACKUP.bat`.

- Backends bind `127.0.0.1` only; players reach the proxy. Velocity uses modern
  forwarding with the secret in `proxy/forwarding.secret` (mirrored in each
  backend's `paper-global.yml`).
- `plugin/` = RemixLobby (hub island built from code, title, `/hub`, `/setspawn`,
  compass that hands the player to `duels` over `bungeecord:main`).
- `plugin-duels/` = RemixDuels (arena from code, kits, `/duel`, `/leave`, `/wins`,
  wins persisted in the plugin config).
- `plugin-auth/` = RemixAuth (nickname gate, see below).
- Backups: `BACKUP.bat` zips both worlds plus plugin data, prunes beyond 14 days.

### Nickname auth

Offline mode trusts the nickname, so a hub cannot tell players apart. The launcher
owns a per-install secret (`auth.key` in the data dir, created by
`packages/oneclient_core/src/auth_gate.rs`) and, only for loopback servers,
`POST`s `nick` + `token` to `http://127.0.0.1:8765/session` right before launch.
RemixAuth binds the nick to the secret on first join and refuses other launchers
afterwards; `/remixauth unbind <nick>` (op) releases it. The announcement is wired
in `game/launch.rs` through `server_argument`.

Publishing the server requires more before it is safe: the secret must never go
over plaintext to a remote host, so a public gate needs TLS, an explicit opt-in in
the server catalogue, and probably per-account identities instead of one install
secret (today two offline accounts on the same install share it).

## 2026-11-24 — OneConfig integration, new mods, launcher fixes

**This section supersedes the older "Companion mod" notes about avoiding OneConfig.**
The player asked for the real OneConfig look; it is now our settings host. Rule:
**always read this file before working on the project.**

### Companion mod (shipped into both clusters)
- `RemixConfig` extends OneConfig's `Config`: the annotated fields render as the
  "Remix Client" page (categories HUD / Visuals). Storage stays
  `<gamedir>/config/remix-companion.json` (the launcher writes `language` there);
  OneConfig's own store mirrors it via the overridden `save()`. Unannotated fields
  (`language`, `placements`) are invisible to OneConfig on purpose.
- Compile-only API jars: `companion/libs/{config,config-impl,ui}-1.6.2.jar`
  (extracted from the installed OneConfig jar; never shaded). `fabric.mod.json`
  now hard-depends on `oneconfig`.
- Keys: **Right Alt or Right Shift** open the OneConfig GUI (`OneConfigUI.INSTANCE.open()`),
  **B** full brightness (also a page switch), **G** zoom (hold; C is taken by Zoomify),
  **H** HUD editor. The old `RemixMenuScreen`/`RemixSettingsScreen` were deleted.
- `RemixDescriptions` rewrites our page's description metadata to Russian on the first
  tick when the launcher language is Russian (titles stay English by player decision).
  OneConfig resolves `titleKey`/`descriptionKey` through the vanilla language map, so
  metadata rewriting is the only way to follow the launcher language.
- Russian text of `RemixLang` pruned to `editor.*` + `module.*` only.

### Launcher
- Navbar logo is always visible now (was hidden below 1280px window width).
- Changelog fetches `linkure8888-lang/OneLauncher` releases; when unreachable it
  shows a bundled 2.11.5 entry, never upstream Polyfrost history.
- Translations: only English (source) + bundled Russian; the Polyfrost CDN
  (`translations.r2.polyfrost.org`) and the coverage ("26% localized") badge are gone.
- Donate row in Help shows a toast instead of opening a URL (payment methods pending).

### Mods installed into both clusters (Modrinth, fabric 26.3)
dynamic-fps, krypton, badoptimizations, reeses-sodium-options, notenoughanimations,
jade, mousetweaks, betterf3, nochatreports, xaeros-minimap, xaeros-world-map, jei.

### Deferred backlog
- PulseVisuals-style modules to add in batches: Cooldowns HUD, Saturation HUD,
  Totem Tracker, Item Pickup Logger, Streamer Mode, Hit Bubbles, Hitsound,
  Hitbox Customizer, Block Overlay, Custom Hand, Target ESP, China Hat/Nimbus,
  No Fluid, No Render (rain/shake/fire), Time Changer, World Customizer,
  Aspect Ratio, Freelook, Sound Controller, Lock Slot, PvP Safe, Waypoints +
  Death Marker, Fast XP, Fake Player, Config Sharing. Automation/server helpers
  (AutoEat/AutoLeave/Item Swap/Predictions/AutoRebuy) stay out by fair-play rule.
- OneConfig open-lag cleanup: candidate mods to disable are evergreenhud,
  ukus-armor-hud, damagetint, polycrosshair, vanillahud (duplicate our HUD),
  macos-input-fixes (useless on Windows), blur-fabric + polyblur (duplicates),
  zoomify (our zoom exists). Player approves each batch before disabling.
- FPS limit: vanilla already has Options → Video → Max Framerate +
  Inactivity FPS Limit — no work needed.

### 2026-11-24 (later) — OneConfig rebrand + HUD off + maps trimmed
- The installed OneConfig jar is rebranded in place (both clusters): fabric.mod.json
  name/icon → "Remix Companion" + our R icon, sidebar wordmark SVGs
  (`assets/oneconfig/brand/*`) replaced with a blocky "REMIX COMPANION" mark
  (violet→cyan gradient), `en_default.lang` says Remix Companion everywhere and the
  polyfrost.org link points to t.me/remixclient_ru, `by-polyfrost.svg`/`atmofrost.svg`
  show our mark. The OneConfig preferences page header still says "OneConfig" (class
  constant — not patchable without bytecode surgery). Launcher content sync may
  re-download the jar and revert the branding; re-run the patch script if it does.
- Old-style HUD panels are OFF by default (code defaults + both config stores): the
  player wants the PulseVisuals-style modules instead. Particles (trails, jump
  circles, hit effects) stay on.
- Xaero's World Map removed (kept Minimap only). Reese's Sodium Options removed
  (needs Sodium 0.9.4-beta; 0.9.2 stays). mezz_config 0.7.1 added for JEI.

### 2026-11-24 (night) — TikTok ad + OneConfig dup crash found
- TikTok promo video built with HyperFrames in `tikkok-ad/` (project committed to repo dir;
  render at `tikkok-ad/renders/*.mp4`, 1080x1920, 31s, silent — HeyGen not signed in, no BGM yet).
  Script: MC ЛАГАЕТ? -> 3 steps pain -> logo reveal -> 4 proof cards (12 mods, RU settings,
  keybinds G/B/RAlt, free forever) -> CTA end card (github + @remixclient.official).
  User's TikTok account exists: remixclient.official (RemixClient Official), avatar = avatar.png.
- `avatar.png` in repo root = real launcher icon (user's request), old logo.png/banner deleted.
- CRASH FIX 3 (static sweep): dev cluster had TWO OneConfig jars (1.6.2 patched + 1.9.4
  auto-downloaded) -> duplicate mod id `oneconfig` = crash. Removed 1.9.4. If launcher content
  sync re-downloads it, delete again or pin version.

### 2026-11-24 (late night) — changelog screen deleted, bundle content purged
- **"Список изменений" screen REMOVED entirely** (user: чужой контент rozwader/junapur):
  route SettingsChangelog, settings_shell tab, view file, hooks/queries/changelog.rs,
  core changelog.rs, set_seen_changelog_version, seen_changelog_version field (serde-safe,
  old settings files ignore unknown keys), 7 i18n keys x2 langs. cargo check + i18n tests OK.
- **Bundle content purge (fix for "Incompatible mods found" + slow load):** Polyfrost bundle
  mrpacks (hud/performance/qol/utility/skyblock from data-v2.polyfrost.org) had installed
  ~137 mods requiring OneConfig >=1.8, but fork ships rebranded 1.6.2 -> Fabric "Incompatible
  mods found!" modal. Sweep script (Temp/oc/sweep.cjs): removed 118/121 bundle jars from both
  26.3 Fabric clusters, kept whitelist: fabric-api, sodium, lithium, c2me, ferritecore,
  modernfix, entityculling, ImmediatelyFast, ScalableLux, servercore, debugify, BadOptimizations,
  krypton, dynamic-fps, modmenu, appleskin, our 12 (Jade/jei+mezz_config/xaero/BetterF3/
  MouseTweaks/NoChatReports/notenoughanimations), OneConfig 1.6.2 + remix-companion +
  compose-bundle (OneConfig hard dep!) + fabric-language-kotlin. 30 jars total.
- DB (user_data.db both profiles): cluster_bundle_choices accepted=0 for all 5 26.3 bundle
  names (cluster 7) -> install_cluster_bundles at launch installs nothing; deleted 137/138
  cluster_artifacts rows of removed jars. Re-added dynamic-fps/c2me/ferritecore from Modrinth
  (whitelist id mismatch: dynamic_fps, c2me, ferritecore).
- NOTE: nested jars inside OneConfig outer jar are its own libs (adventure/ui/config) — NOT
  content mods, keep. The bundle content lives as STANDALONE jars in mods/.
- Raw video cut: RemixServer/RemixClient-cut.mp4 (67s, from user's 172s recording; foreign
  changelog segment 108-128 and the crash 152+ cut out). In-game footage still missing —
  user should re-record now that the crash is fixed.

### 2026-11-25 (reverted same day)
- Cooldowns HUD (RemixCooldowns) was built and deployed, then REVERTED on user request
  ("не надо, откати все, погоди, завтра решим"): module file, config field showCooldowns,
  MODULES entry, lang keys removed, rebuilt + redeployed. Module batches are ON HOLD until
  the player decides tomorrow. Keep: changelog screen removal + bundle purge (separate task).

### 2026-11-25 (evening) — Appearance page + cosmetics placeholder
- The "Скины" navbar tab is now «Внешность» / "Appearance" (keys: navbar.skins, routes.skins,
  skins.title; route path unchanged /app/skins). Inside: two sections — "Скины" (existing
  library) and "Косметика" with two placeholder cards: PaintPour "Coming soon" card (capes,
  hats, wings in a future update) and a "Support the author 😅" card opening
  https://t.me/remixclient_ru via platform::open_url. i18n keys added symmetric, tests pass,
  cargo check OK. Reason: user asked where cosmetics went (fork never had Polyfrost cosmetics
  backend; placeholders make the roadmap visible). Future cosmetics = donation-allowed area.
