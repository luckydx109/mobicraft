# EffectCraft Mobile — Claude Project & UI Development Guide

Welcome! This repository is **EffectCraft Mobile**, an open-source, high-performance motion graphics and video animation suite (After Effects alternative) ported to run natively on **Android (ARM64 / aarch64-linux-android)**.

---

## 📌 Project Overview & Current State

1. **Native Engine & Android Compilation: 100% Working**
   - The entire Rust codebase compiles cleanly down to a single native shared library: `libeffectcraft.so`.
   - Android target: `aarch64-linux-android` (Min SDK 26, Target SDK 35).
   - Android wrapper: `apps/effectcraft-android` using `android.app.NativeActivity` with fullscreen immersive theme and custom launcher icons.
   - APK packaging script: `.\build_desktop_android.ps1` (compiles Rust, runs AAPT2 resource linking, zipalign, apksigner, and outputs `EffectCraft-Mobile.apk`).

2. **Core Mobile Systems Already Implemented (`crates/mobile` & `crates/export`)**:
   - `crates/mobile`: JNI FFI bindings, ANativeWindow session manager, 9:16 vertical reel presets (1080x1920 @ 30/60fps), and headless session runner.
   - `crates/mobile`: Dynamic cache budgeting (`Session::on_trim_memory`) based on Android OS low-memory callbacks.
   - `crates/export`: Hardware-accelerated `NdkMediaCodecEncoder` for fast H.264/MP4 export directly via Android NDK MediaCodec (`AMediaCodec`).
   - `crates/export`: Fast NV12 conversion and `.ecrender` crash journal tracker for resumable rendering.
   - Interactive motion blur throttling and draft effect quality switches (`EffectQuality::Draft`, `Balanced`, `Full`).

---

## 🎨 Your Mission: Mobile UI Refinement & Touch UX

Currently, the app runs the **Full Desktop UI** (`crates/ui-egui`) rendered via WGPU/Egui directly onto the Android screen with 1.5x scaling.
Your primary task is to adapt or build a dedicated **Touch-First Mobile UI** for phone screens (portrait 9:16 and landscape modes) without breaking or simplifying the underlying engine.

### Key UI Directories:
- **`crates/ui-egui/src/`**: All UI panels and rendering logic:
  - `lib.rs`: Main Egui entry point, frame loop, toolbar, window management.
  - `viewer.rs`: Canvas viewport, pan/zoom, transformation gizmos, canvas interactions.
  - `timeline.rs`: Composition timeline, playhead, time ruler, layer bars, keyframes.
  - `panels/`: Individual docked inspectors and panels:
    - `layers.rs`: Layer stack, parenting, blending modes, track mattes.
    - `inspector.rs`: Property editor, transforms, effect parameters.
    - `effects.rs`: Effect browser, search, presets.
    - `graph.rs`: Speed/value Bezier curve graph editor.
  - `theme.rs` & `style.rs`: Colors, spacing, typography, rounding, and widget sizes.
- **`apps/effectcraft/src/lib.rs`**: Android native entry point (`android_main`). Handles window creation, scale factor (`pixels_per_point`), and launches `EffectcraftApp`.

---

## 📐 Recommended Mobile UI Architecture

When designing the touch interface for mobile:
1. **Adaptive Layout (Mobile vs Desktop)**:
   - Detect mobile via `cfg(target_os = "android")` or runtime screen dimensions (`ctx.screen_rect().width() < 700.0`).
   - On mobile portrait:
     - **Top**: Compact header (Back, Project Name, Undo/Redo, Export button).
     - **Center**: Composition Preview (Pinch-to-zoom, two-finger pan, 9:16 canvas centered).
     - **Bottom Half**: Touch-friendly Tabbed Workspace (Timeline / Layer List / Effect Inspector / Color Picker).
     - **Bottom Bar**: Quick-action tool rail (Text, Shape, Media, Split, Keyframe, Play/Pause).
2. **Touch Target Sizing**:
   - Minimum touch target: 44x44 dp (ensure buttons, keyframe diamonds, and layer toggle switches are easy to hit with thumbs).
3. **Keyframe & Curve Editing on Mobile**:
   - Provide a bottom slide-up drawer for the Bezier Graph Editor (`panels/graph.rs`) so users can adjust easing curves with large touch handles.
4. **Never Break Engine Architecture**:
   - Maintain the command pattern (`effectcraft-model` commands). All mutations should continue to go through `Session::apply(cmd)`.
   - Adhere strictly to the `AGENTS.md` **never-crash** rule (zero `unwrap()`, `expect()`, `panic!` in non-test code).

---

## 🛠️ How to Rebuild and Test the Android APK

A 1-click PowerShell build script is provided in the repository root:

```powershell
# From the project root (E:\effectcraft-mainformobile):
.\build_desktop_android.ps1
```

This automated script will:
1. Compile the Rust library for ARM64: `cargo build --package effectcraft --target aarch64-linux-android --release`
2. Copy `libeffectcraft.so` to the Android wrapper.
3. Run `aapt2 compile` and `aapt2 link` with Android SDK 35.
4. Inject the `.so` binary into the APK.
5. Align with `zipalign -v -p 4`.
6. Sign with `apksigner`.
7. Output `EffectCraft-Mobile.apk` and automatically install it if an Android device is connected via USB!
