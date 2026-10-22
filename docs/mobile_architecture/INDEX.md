# EffectCraft Mobile — Architecture & Implementation Master Plan

> **Target Platform:** Android (Devices with 4 GB, 6 GB, 8 GB, and 12 GB+ RAM)  
> **Primary Goal:** Professional Motion Graphics & Visual Effects Compositor for Cinematic Social Media Reels  
> **Status:** All 5 Implementation Phases Complete (`crates/mobile`, Memory Budgeting, Hardware Video Pipeline, Adaptive LOD, Mobile UI Shell).

---

## Document Index

1. [01. Android Core Architecture](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/01_ANDROID_CORE_ARCHITECTURE.md)
   * NDK & JNI interop bridge architecture (`libeffectcraft_mobile.so`).
   * Surface lifecycle (`ANativeWindow`), WGPU Vulkan integration.
   * Target toolchains, Android SDK/NDK requirements (Min SDK 26, Target SDK 35).
   * Process lifecycle & background render handling.

2. [02. Memory Management & Low-RAM Adaptation](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/02_MEMORY_MANAGEMENT_AND_LOW_RAM.md)
   * Device RAM tier categorization (4 GB, 6 GB, 8 GB, 12 GB+).
   * Configurable and dynamic cache budgeting (`LayerCache`, `MediaPool`, `Frames`).
   * Texture memory optimization (`Rgba32Float` to `Rgba8Unorm` / `Rgba16Float`).
   * Android OS Low Memory Killer (LMK) protection via `ComponentCallbacks2.onTrimMemory`.
   * Rayon CPU thread-pool bounding per device tier.

3. [03. Hardware-Accelerated Video Pipeline](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/03_HARDWARE_VIDEO_PIPELINE.md)
   * Android NDK `AMediaCodec` hardware encoder integration.
   * Implementation of `VideoEncoder` trait for AVC/H.264 & HEVC.
   * Hardware video decoding via `AMediaExtractor` & `AMediaCodec`.
   * Sample-accurate audio multiplexing (`Tick` time with AAC).
   * Checkpoint journal & crash recovery for interrupted exports.

4. [04. Adaptive Preview & Effect Quality LOD](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/04_ADAPTIVE_PREVIEW_AND_EFFECT_LOD.md)
   * Selectable preview resolution scaling (360p, 480p, 720p, 1080p).
   * Independence of preview quality from final export settings.
   * Effect Level-of-Detail (LOD) parameter in `EffectEnv`.
   * Configurable shader complexity (blur pass downscaling, particle count clamping).
   * Interactive playback motion blur throttling.

5. [05. Touch-First Mobile UI Specification](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/05_TOUCH_FIRST_UI_SPECIFICATION.md)
   * Single-screen mobile layout (Canvas + Touch Scrub Timeline + Bottom Sheet Inspector).
   * Direct-manipulation gestures: Pinch-to-zoom, pan, rotation, transform bounding gizmo.
   * Mobile keyframe curve editor (Bezier handle touch controls).
   * Cinematic Reel presets: Motion titles, glow bursts, speed ramps, camera shakes.

6. [06. Step-by-Step Implementation Guide](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/06_IMPLEMENTATION_STEP_BY_STEP_GUIDE.md)
   * Phased implementation roadmap.
   * Exact file paths to create and modify.
   * JNI function signatures and Rust FFI contracts.
   * Real-device testing and performance validation criteria.

7. [07. Desktop UI on Mobile Run Guide](file:///E:/effectcraft-mainformobile/docs/mobile_architecture/07_DESKTOP_UI_ON_MOBILE_RUN_GUIDE.md)
   * Full unmodified desktop UI (`crates/ui-egui`) in Android `NativeActivity`.
   * Cross-compilation via `cargo build --target aarch64-linux-android`.
   * Ready-to-build Android wrapper (`apps/effectcraft-android`).
   * 1-click execution script (`build_desktop_android.ps1`).

---

## Summary of Completed Cleanup

The following desktop-only and software-heavy components were removed from the workspace:
* `crates/automation`: Desktop stdio MCP JSON-RPC server.
* `crates/av1enc`: Pure-Rust unaccelerated AV1 CPU software encoder.
* `crates/hevcenc`: Pure-Rust unaccelerated HEVC CPU software encoder.
* `crates/vp9enc`: Pure-Rust unaccelerated VP9 CPU software encoder.
* `apps/effectcraft-cli`: Desktop terminal CLI binary.
* `crates/export/src/hevc_av1.rs`, `crates/export/src/webm_av1.rs`, `crates/export/tests/*vp9*`, `crates/export/tests/*hevc*`: Software encoder glue and roundtrip tests.

**Workspace Status:** Cleaned, lightweight, and ready for mobile platform bindings.
