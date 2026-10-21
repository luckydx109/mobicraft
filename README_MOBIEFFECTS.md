<p align="center">
  <h1 align="center">📱 MobiEffects</h1>
</p>

<p align="center">
  <b>A free, open-source motion graphics & visual effects app for Android — After Effects in your pocket.</b>
</p>

<p align="center">
  <img alt="Status: Alpha" src="https://img.shields.io/badge/status-alpha-e0368f?style=flat-square">
  <img alt="Written in Rust" src="https://img.shields.io/badge/rust-1.95%2B-b0206c?style=flat-square&logo=rust&logoColor=white">
  <img alt="Platform: Android" src="https://img.shields.io/badge/platform-Android-3ddc84?style=flat-square&logo=android&logoColor=white">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-555?style=flat-square">
</p>

---

## What is MobiEffects?

MobiEffects is a **touch-first motion graphics and visual effects app** for Android phones and tablets. It brings professional-grade compositing, animation and effects — the kind of work people use After Effects for — to your mobile device.

Built on top of [EffectCraft](https://github.com/storytold/effectcraft)'s powerful open-source Rust engine, MobiEffects delivers a purpose-built touch UI while keeping the full power of the desktop compositor underneath.

**This is not a dumbed-down mobile app.** It's the real engine with a UI redesigned for thumbs.

---

## 🎬 What's Inside

### ✅ Included Features

| Category | Features |
|---|---|
| **Layers** | Solids, shapes, text, footage, nested compositions, nulls, adjustment layers, cameras, lights |
| **Animation** | Keyframes (linear, Bezier, hold, auto, Easy Ease), Graph Editor with value/speed graphs, motion paths, time remapping |
| **Parenting & Links** | Full parent/child hierarchy, pick-whip style parent selector, link expressions |
| **Blend Modes** | All 38 blend modes (Normal, Add, Multiply, Screen, Overlay, etc.) |
| **Layer Switches** | Solo, lock, shy, collapse transformations, motion blur, 3D layer toggle |
| **Effects** | 306 effects across all categories — blur, color correction, distort, generate, keying, simulation, stylize, time, audio, transitions |
| **Layer Styles** | Drop Shadow, Inner/Outer Glow, Bevel & Emboss, Satin, Color/Gradient/Pattern Overlay, Stroke |
| **Shapes & Masks** | Shape layers with trim paths, repeaters, round corners, offset, zig zag, twist, wiggle, merge paths; masks with pen tool, modes, feather |
| **Text** | Point and paragraph text, Character & Paragraph panels, text animators with range selectors |
| **Expressions** | JavaScript expressions with After Effects object model (wiggle, loopOut, thisComp, etc.) |
| **3D** | 3D layers, cameras (one/two-node) with depth of field, lights (parallel, spot, point, ambient) with shadows |
| **Puppet Tools** | Position, Bend, Starch, Overlap pins, real-time pin recording |
| **Export** | H.264 MP4, ProRes MOV, PNG/JPEG/TIFF/EXR sequences, animated GIF, WAV/AIFF, Lottie JSON |
| **Lottie** | Import and export Lottie animations (precomps, shapes, text, keyframes, masks, etc.) |
| **Audio** | Audio preview with sync, meters, waveforms, audio effects (Reverb, EQ, Delay, etc.) |
| **Touch UI** | Pinch-to-zoom, two-finger pan, draggable floating panels, gesture-aware timeline, bottom sheets |

### ❌ Excluded Features (Too Heavy for Mobile)

These are PC-only features that require significant CPU/GPU power and are intentionally not included:

| Feature | Reason |
|---|---|
| **Warp Stabilizer** | Requires multi-pass full-frame analysis — too CPU-intensive for mobile |
| **3D Camera Tracker** | Heavy computer vision + point cloud matching — not practical on phone hardware |
| **Face Track Points** | ML-based face landmark detection — high memory/compute cost |
| **Roto Brush & Refine Edge** | Real-time segmentation with ML models — very GPU-heavy |
| **Content-Aware Fill** | Multi-frame temporal analysis — desktop-only workload |
| **HEVC/AV1/VP9-WebM encoding** | Large pure-Rust encoders disabled to keep build fast; H.264 & ProRes are available |

### 🔧 Desktop-Only UI (Not Applicable to Mobile)

| Feature | Reason |
|---|---|
| Workspace save/reset/delete | Phone has a fixed touch layout |
| Script installation/running | Desktop scripting workflow |
| Watch folder | Desktop automation |
| Reveal in Finder | Desktop file manager integration |

---

## 📱 UI Design

The mobile UI follows a clean, space-efficient layout optimized for thumb reach:

```
┌─────────────────────────────────────┐
│ ◀ 🏠  ＋  ≡    [Undo] [Redo] [Export] │ ← Toolbar
├─────────────────────────────────────┤
│ ▶ ✋ ↻ ⊕ □ ✎ T 📷                    │ ← Tool Strip
├─────────────────────────────────────┤
│                                     │
│        Composition Preview          │ ← Pinch-to-zoom, 2-finger pan
│         (canvas area)               │
│                                     │
│  ▶                                  │ ← Play/Pause overlay
├─────────────────────────────────────┤
│ Mag  Res  Grid  Channel  Fast  3D   │ ← Viewer Controls (scrollable)
├─────────────────────────────────────┤
│ Comp 1 │ Comp 2 │  ＋                │ ← Composition Tabs
├═══════════ ═══════ drag ════════════┤ ← Resize Splitter
│                                     │
│           Timeline                  │ ← Layer names, keyframes, time graph
│      (slides sideways)              │    Double-tap layer → action sheet
│                                     │
│                               (＋)   │ ← Floating + for panel switching
│  ───────── pan bar ─────────        │
└─────────────────────────────────────┘
```

### Panel System (via ＋ button)
Tap the floating ＋ to switch the timeline area to any of these panels:
- **Assets** (Project panel)
- **Effects & Presets**
- **Effect Settings** (Effect Controls)
- **Preview**
- **Properties**
- **Align**
- **Audio**
- **Info**
- **Character** (text properties)
- **Paragraph**
- **History**
- **Markers**

### Layer Action Sheet (double-tap a layer)
- Parent picker & unlink
- Blend mode selector
- Layer switches (solo, lock, shy, 3D, motion blur, collapse)
- Timing controls (in/out/duration)
- Label color
- Pre-compose, split, duplicate, delete
- Add effects
- Add expressions

---

## 🛠️ Build Instructions

### Prerequisites
- [Rust](https://rustup.rs/) 1.95+
- Android SDK (API 35)
- Android NDK
- ADB (for device deployment)

### Build the APK

```powershell
# From the project root:
.\build_desktop_android.ps1
```

This script:
1. Compiles Rust for `aarch64-linux-android`
2. Packages resources with AAPT2
3. Builds, aligns and signs the APK
4. Installs to a connected device (if available)

### Preview on Desktop (with Mobile UI)

To test the mobile layout without an Android device:

```powershell
# Option 1: Run desktop app with mobile UI forced on
$env:EFFECTCRAFT_MOBILE_UI = "1"
cargo run --release -p effectcraft

# Option 2: Web build (runs in browser)
cargo xtask web --serve 8765
# Then open http://127.0.0.1:8765/ and resize to phone proportions
```

---

## 🏗️ Architecture

```
apps/
  effectcraft/         # Desktop + Android entry points (android_main)
  effectcraft-android/ # Android manifest, resources, launcher icons
  effectcraft-cli/     # Command-line tool
  effectcraft-web/     # WebAssembly build
crates/
  engine/              # Core engine: project, timeline, composition
  render/              # Rendering pipeline, compositing
  effects/             # 306 effects implementation
  gpu/                 # GPU acceleration (WGPU)
  export/              # Render queue, H.264, ProRes, Lottie
  keyframe/            # Keyframe interpolation, easing
  text/                # Text shaping, fonts
  ui-egui/             # All UI code (desktop + mobile)
    src/mobile*.rs     # Touch-first mobile UI modules
    src/panels/        # Desktop panels (embedded in mobile)
  model/               # Session, commands, undo/redo
  expr/                # JavaScript expression engine
  lottie/              # Lottie import/export
  time/                # Time model, frame rates
  ...
```

The mobile UI lives in `crates/ui-egui/src/mobile*.rs`. It wraps the same panels and commands as the desktop UI, with touch-sized widgets and a phone-friendly layout. The engine underneath is identical — every edit goes through the same command system and undo/redo stack.

---

## 📋 Based On

MobiEffects is built on [EffectCraft](https://github.com/storytold/effectcraft), an open-source After Effects alternative written in Rust by the ArtCraft team. We use their engine as-is and add a mobile-optimized touch UI on top.

- **Engine code**: Unchanged from EffectCraft (effects, rendering, export, etc.)
- **UI code**: Custom mobile modules + small hooks in the shared UI crate
- **New tools and bug fixes**: Ported from EffectCraft with minor changes for mobile

---

## 📄 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Based on [EffectCraft](https://github.com/storytold/effectcraft) by the ArtCraft Team. See [ATTRIBUTION.md](ATTRIBUTION.md) for bundled asset licenses.

<sub>Adobe and After Effects are trademarks of Adobe Inc. MobiEffects is independent and not affiliated with Adobe.</sub>
