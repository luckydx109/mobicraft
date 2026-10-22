# 06. Step-by-Step Implementation Guide

This guide is the exact technical execution blueprint for building the mobile editor. Any developer or AI coding agent can follow these phases sequentially to complete the implementation.

---

## Phase 1: Mobile Platform Crate (`crates/mobile`)

### Goal
Establish the Android NDK / JNI compilation target and shared library output (`libeffectcraft_mobile.so`).

### Actions
1. Create `crates/mobile/Cargo.toml`:
   ```toml
   [package]
   name = "effectcraft-mobile"
   version.workspace = true
   edition.workspace = true
   license.workspace = true

   [lib]
   crate-type = ["cdylib", "rlib"]

   [dependencies]
   effectcraft-engine = { workspace = true }
   effectcraft-project = { workspace = true }
   effectcraft-render = { workspace = true }
   effectcraft-gpu = { workspace = true }
   effectcraft-media = { workspace = true }
   effectcraft-export = { workspace = true }
   effectcraft-keyframe = { workspace = true }
   effectcraft-time = { workspace = true }
   jni = "0.21"
   ndk = "0.9"
   ndk-sys = "0.6"
   raw-window-handle = "0.6"
   serde_json = { workspace = true }
   log = { workspace = true }
   ```
2. Implement JNI entrypoints in `crates/mobile/src/lib.rs`:
   * `nativeInit(cache_path, internal_path)`
   * `nativeExecuteCommand(command_id, json_params) -> String`
   * `nativeSetSurface(surface, width, height)`
   * `nativeSeekFrame(comp_id, tick, scale)`
   * `nativeSetTrimMemory(level)`
   * `nativeStartExport(export_settings_json)`
3. Add `effectcraft-mobile = { path = "crates/mobile" }` to root `Cargo.toml`.

---

## Phase 2: Memory Budgeting & Dynamic Caching

### Goal
Implement device RAM tier detection and configurable cache budgeting so the app never triggers Android LMK.

### Actions
1. In `crates/engine/src/prefs.rs`:
   * Add `DeviceTier` enum (`Tier4Gb`, `Tier6Gb`, `Tier8Gb`, `Tier12GbPlus`).
   * Update `Prefs::cache_budgets()` to scale floor sizes (16 MB for 4GB RAM devices up to 64 MB for 12GB+).
2. In `crates/gpu/src/context.rs`:
   * Make working texture format configurable (`Rgba8Unorm` on Tier 1 & 2; `Rgba16Float` on Tier 3; `Rgba32Float` on Tier 4).
   * Clamp `UPLOAD_BUDGET` and `POOL_BUDGET` according to device tier.
3. In `crates/engine/src/session.rs`:
   * Add `Session::on_trim_memory(level: i32)` which calls `FootageSource::purge()` and shrinks cache budgets by 50% under critical memory pressure.

---

## Phase 3: Android Hardware Video Encoder (`AMediaCodec`)

### Goal
Connect Android NDK hardware video encoding into EffectCraft's export pipeline.

### Actions
1. In `crates/export/src/encode.rs`:
   * Implement `NdkMediaCodecEncoder` conforming to the existing `VideoEncoder` trait:
     ```rust
     pub(crate) trait VideoEncoder {
         fn sample_entry(&self) -> SampleEntry;
         fn encode(&mut self, rgba: &[u8], index: u64) -> Result<Vec<Packet>>;
         fn flush(&mut self) -> Result<Vec<Packet>>;
         fn media_start(&self) -> Option<i64> { None }
     }
     ```
   * Wire `AMediaCodec_createEncoderByType("video/avc")` for H.264 export.
   * Add NV12 color conversion: convert incoming rendered RGBA8 buffers into semi-planar YUV420 buffers.
   * Extract Annex-B NAL units, convert to length-prefixed MP4 samples, and pipe into `filmcraft_isobmff::Mp4Writer`.
2. Add resumable export journaling:
   * Implement `.ecrender` sidecar file writer tracking completed frame chunks.

---

## Phase 4: Adaptive Preview & Effect Quality LOD

### Goal
Allow real-time 30–60 fps playback during editing on all devices by decoupling preview scale from export quality.

### Actions
1. In `crates/effects/src/lib.rs`:
   * Add `EffectQuality` (`Draft`, `Balanced`, `Full`) to `EffectEnv`.
2. In `crates/effects/src/blur2.rs` and `crates/gpu/src/fx_stylize.rs`:
   * Add branching for `EffectQuality::Draft`:
     * Blurs: use 2-pass approximation on downscaled buffer.
     * Glow: single 1/4 resolution mip pass.
     * Particles: clamp active particle count to 250 in draft mode.
3. In `crates/render/src/lib.rs`:
   * Ensure `RenderOpts::scale` accepts 0.333x (360p), 0.444x (480p), 0.667x (720p), and 1.0x (1080p).
   * Disable sub-frame motion blur during interactive timeline scrubbing.

---

## Phase 5: Mobile Touch-First UI Integration

### Goal
Connect the native core to an ergonomic mobile frontend (Flutter or Jetpack Compose).

### Actions
1. Setup Android frontend shell:
   * Main Viewport with `SurfaceView` displaying the native WGPU canvas.
   * Touch gestures: Pinch-to-zoom and pan matrix transformation.
2. Timeline widget:
   * Horizontal draggable strip synchronized to `Tick` time.
   * Add/Toggle keyframe diamond button.
   * In/Out trim handles for layer slices.
3. Bottom Sheet Inspector:
   * Transform controls (Position, Scale, Rotation, Opacity sliders).
   * Blend mode selector (38 blend modes).
   * Preset library for cinematic reel titles, speed ramps, and transitions.

---

## 6. Verification & Device Testing Matrix

To guarantee production readiness across the Android ecosystem, test on real devices representing each RAM tier:

| RAM Tier | Representative Device / Chipset | Verification Target |
| :---: | :--- | :--- |
| **4 GB RAM** | Redmi 10 / Samsung Galaxy A14 (Helio G85 / Exynos 850) | 360p/480p preview @ 30+ fps; Peak RAM < 350 MB; zero OOM crashes. |
| **6 GB RAM** | Poco M6 / Realme Narzo (Snapdragon 4 Gen 2 / Dimensity 6100+) | 480p/720p preview @ 45+ fps; Peak RAM < 600 MB; H.264 export @ 60 fps. |
| **8 GB RAM** | OnePlus Nord / Galaxy A54 (Snapdragon 7s Gen 2 / Exynos 1380) | 720p preview @ 60 fps; Peak RAM < 1.0 GB; smooth 5-layer compositing. |
| **12 GB+ RAM**| Galaxy S23 / Pixel 8 Pro / OnePlus 12 (Snapdragon 8 Gen 2/3) | 1080p full preview; high-fidelity particle systems; instant export. |
