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

## 🚀 What is MobiEffects?

MobiEffects is a **touch-first motion graphics and visual effects app** for Android phones and tablets. It brings professional-grade compositing, animation and effects — the kind of work people reach for After Effects to do — natively to your mobile device.

Built on top of [EffectCraft](https://github.com/storytold/effectcraft)'s powerful open-source pure-Rust engine, MobiEffects delivers a purpose-built touch UI while retaining the full power of the desktop compositor underneath.

**This is not a dumbed-down mobile app.** It is the real After Effects-compatible engine with an interface designed for thumbs.

---

## 🎬 Included Features

| Category | Features |
|---|---|
| **Layers** | Solids, shapes, text, footage, nested compositions, nulls, adjustment layers, cameras, lights |
| **Animation & Easing** | Keyframes (linear, Bezier, hold, auto, Easy Ease F9, Ease In/Out), Graph Editor with value & speed graphs, roving keys |
| **Keyframe Navigation** | Quick jump to Prev Keyframe (J) and Next Keyframe (K), Add/Toggle Keyframe at current playhead time |
| **Parenting & Links** | Full parent/child hierarchy, pick-whip parent selector, unlink, link expressions |
| **Blend Modes** | All 38 blend modes grouped into Standard, Darken, Lighten, Contrast, Inversion, and Color/Matte |
| **Track Mattes** | Alpha Matte, Alpha Inverted, Luma Matte, Luma Inverted, Matte with Layer Above/Below |
| **Layer Switches** | Solo, lock, eye, shy, collapse transformations, motion blur, 3D layer toggle, adjustment layer |
| **Layer Styles** | Drop Shadow, Inner Shadow, Outer Glow, Inner Glow, Bevel & Emboss, Satin, Color/Gradient Overlay, Stroke |
| **Timing & Trim** | Trim In to Playhead (`[`), Trim Out to Playhead (`]`), Move In/Out, Work Area Start (`B`) / End (`N`) |
| **Timeline Columns** | Interactive column drawer to toggle Parent/Link, Switches, Modes, Keys, In/Out/Duration, Stretch, Numbers, Comments |
| **Effects** | 306 effects across all categories — blur, color correction, distort, generate, keying, simulation, stylize, time, audio, transitions |
| **Shapes & Masks** | Shape layers with trim paths, repeaters, round corners, offset, zig zag, twist, wiggle, merge paths; masks with pen tool & feathering |
| **Text** | Point and paragraph text, Character & Paragraph panels, text animators with range selectors |
| **Expressions** | JavaScript expressions with After Effects object model (`wiggle`, `loopOut`, `thisComp`, etc.) |
| **3D** | 3D layers, cameras (one/two-node) with depth of field, lights (parallel, spot, point, ambient) with shadows |
| **Puppet Tools** | Position, Bend, Starch, Overlap pins, real-time pin recording |
| **Export & Render Queue** | Render Queue panel, H.264 MP4, ProRes MOV, PNG/JPEG/TIFF/EXR sequences, animated GIF, WAV/AIFF, Lottie JSON |
| **Touch UX** | Pinch-to-zoom, two-finger pan, draggable floating panels, gesture-aware timeline, quick bottom action sheets |

---

## 📱 Mobile Touch Architecture

The mobile interface is designed to maximize viewport space while keeping controls within easy thumb reach:

```
┌──────────────────────────────────────────────┐
│ ◀ 🏠  ＋  ≡        [Undo] [Redo] [Export]    │ ← Top Toolbar
├──────────────────────────────────────────────┤
│ ▶ ✋ ↻ ⊕ □ ✎ T 📷                            │ ← Tool Strip (toggable)
├──────────────────────────────────────────────┤
│                                              │
│             Composition Preview              │ ← Pinch-to-zoom, 2-finger pan
│                (canvas area)                 │
│                                              │
│  ▶                                           │ ← Floating Play/Pause
├──────────────────────────────────────────────┤
│ Mag  Res  Grid  Channel  Fast  3D  Layout    │ ← Viewer Control Strip (scrollable)
├──────────────────────────────────────────────┤
│ Comp 1 │ Comp 2 │  ＋                         │ ← Composition Tabs
├═════════════════ drag splitter ══════════════┤ ← Draggable Viewport Splitter
│ [Switches (F4)] [Columns ▾] [📈 Graph] ◀ ◆ ▶ │ ← Timeline Quick Header
├──────────────────────────────────────────────┤
│                                              │
│                  Timeline                    │ ← Touch-friendly layers & curves
│             (slides sideways)                │    Double-tap layer → Action Sheet
│                                              │
│                                        (＋)  │ ← Floating Panel Switcher
│  ─────────────── pan bar ──────────────────  │
└──────────────────────────────────────────────┘
```

### 1. Timeline Quick Strip
Directly above the timeline, quick touch buttons provide instant control:
- **`[Switches / Modes (F4)]`**: Toggle between layer switches (shy, blur, 3D) and blending modes/track mattes.
- **`[Columns ▾]`**: Open the Column Configuration Sheet to toggle Parent, Keys, Switches, Modes, In/Out/Duration, Stretch, or Comments.
- **`[📈 Graph]`**: Toggle between standard timeline view and the Bezier Speed/Value Graph Editor.
- **`[◀ Key (J)]`, `[◆ Key]`, `[Key (K) ▶]`**: Instant keyframe navigation and keyframe toggling.
- **`[▾ Twirl]`**: Twirl selected layers open/closed.

### 2. Layer Action Sheet (Double-tap any layer)
Double-tapping a layer opens an action sheet with comprehensive controls:
- **Quick Toggles**: Eye (Visibility), Solo, Lock
- **Blending Mode**: 1-tap selector across all 38 blend modes grouped by Standard, Darken, Lighten, Contrast, Difference, and Matte
- **Track Matte**: Alpha, Alpha Inverted, Luma, Luma Inverted, Above, Below
- **Parent & Link**: Pick parent layer or unlink
- **Layer Styles**: 9 layer styles with live toggles + Show All / Remove All / Editable
- **Keyframe Animation**: Easy Ease (F9), Ease In, Ease Out, Hold, Linear, Prev/Next Keyframe jump
- **Timing & Playhead**: Trim In/Out to Playhead, Move In/Out, Work Area Start/End
- **Label Color**: AE's 16 color labels with preview chips
- **Create**: Add Text, Solid, Light, Camera, Null, Shape, or Adjustment layers
- **Operations**: Rename, Pre-compose, Duplicate, Delete, Split Layer, Add Expression

### 3. Floating Panel Switcher (＋ button)
Tap the floating **(＋)** button to swap the bottom area to any docked panel:
- **Assets** (Project footage & comps)
- **Effects & Presets** (306 built-in effects)
- **Effect Settings** (Parameters & controls)
- **Preview** (Playback settings)
- **Properties** (Transform & property editor)
- **Align** (Alignment & distribution)
- **Audio** (Audio meters & levels)
- **Render Queue** (Batch render & export queue)
- **Essential Graphics** (MOGRT controls)
- **Paint & Brushes** (Brush tools & stroke settings)
- **Character & Paragraph** (Typography settings)
- **History & Markers** (Undo stack & markers)
- **Flowchart** (Node composition graph)

---

## 🛠️ Build Instructions

### Prerequisites
- [Rust](https://rustup.rs/) 1.95+
- Android SDK (API 35+)
- Android NDK (r26b+)
- Java JDK 17+ (or Android Studio JBR)

### 1-Click APK Build
To compile the native Rust library, assemble Java pickers, package assets, and sign the APK:

```powershell
.\build_desktop_android.ps1
```

The script produces `EffectCraft-Mobile.apk` and installs it automatically if a phone or emulator is connected via ADB.

### 💻 Previewing on PC (No Phone Required)
You can test and run the mobile UI directly in a desktop window:

```powershell
# Force mobile UI on desktop:
$env:EFFECTCRAFT_MOBILE_UI = "1"
cargo run --release -p effectcraft
```

Or run the WebAssembly browser build:
```powershell
cargo xtask web --serve 8765
```
Then open `http://127.0.0.1:8765/` in your browser and resize the window to mobile proportions.

---

## 📄 License & Attribution

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

Built upon [EffectCraft](https://github.com/storytold/effectcraft) by the ArtCraft team. See [ATTRIBUTION.md](ATTRIBUTION.md) for third-party licenses.

<sub>Adobe and After Effects are trademarks of Adobe Inc. MobiEffects is an independent open-source project and is not affiliated with or endorsed by Adobe Inc.</sub>
