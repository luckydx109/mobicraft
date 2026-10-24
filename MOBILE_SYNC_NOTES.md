# EffectCraft Mobile: synced to the PC version (v0.14.0)

The engine and every PC crate are the PC version's code, copied as it is. The phone parts were put
back on top of it. Search the code for `MOBILE-HOOK` and `crate::mobile::active()` to see where the
phone shell meets PC code.

## What is the PC version, what is the phone's

| Part | Source |
| --- | --- |
| `crates/*` (engine, render, effects, gpu, export ...) | PC, unchanged except the three edits below |
| `crates/ui-egui` | PC, plus `mobile*.rs` and small hooks in `lib.rs`, `panels/viewer.rs`, `panels/timeline.rs`, `icons.rs` |
| `apps/effectcraft/src/main.rs` and the other desktop files | PC, unchanged |
| `apps/effectcraft/src/lib.rs`, `storage_access.rs` | phone: `android_main`, permissions, insets, pickers |
| `apps/effectcraft-android` | phone: manifest, resources, `MainActivity.java` |

Edits to PC code (all small):
1. `crates/engine/src/media_browser.rs`: Android shared storage (Camera, Pictures, Movies, Downloads, Music).
2. `crates/export`: HEVC / AV1 / VP9-WebM encoders are behind the cargo feature `extra-codecs`
   (off here): they are large pure-Rust encoders and the main reason builds were slow.
   H.264, ProRes, image sequences, GIF and WAV/AIFF export are unchanged.
3. `crates/ui-egui/Cargo.toml`: `accesskit` is not built for Android.

## New in the phone shell
* Everything is laid out inside the system bars (status bar, navigation bar / gesture area, notch,
  keyboard): the host reads the window insets over JNI (`storage_access::window_insets`).
* Portrait and landscape (the manifest no longer locks portrait).
* The whole timeline (layer names, properties, time graph) slides sideways: the bar under the
  timeline, or two fingers over the names / over a graph that shows the whole comp.
* The floating + can be dragged; a tap still opens the panels.
* Import: "Photos & Videos" opens the system gallery, "Audio" the system file manager. Picked
  files are copied into the app's storage (`imports/`) and imported into the open composition.
  The in-app folder browser stays below as a fallback.

## Faster builds
`build_desktop_android.ps1` builds only the library, in the `android` profile (no LTO, 16 codegen
units, incremental). Keep the `target` folder between builds: a small UI change then rebuilds
quickly. A full `--release` build is still possible.

## Corrections to CLAUDE_MOBILE_PROJECT_GUIDE.md
* `NdkMediaCodecEncoder` in the old phone version was a stub (it produced no video packets), so it
  was not carried over: export on the phone uses the PC's software H.264 encoder.
* `crates/mobile` (JNI bridge) was never linked into the app and does not compile against the
  engine; it is excluded from the workspace.

## Building the APK
Run `build_desktop_android.ps1` (PowerShell) in this folder. It needs: Rust 1.95 or newer
(`rustup update stable`), git and internet for the first build (some crates come from GitHub), the
Android NDK 28 / SDK build-tools 35 / platform 36 at the paths at the top of the script, and a JDK
(Android Studio's `jbr`). The APK is written next to the script as `EffectCraft-Mobile.apk`.

## Update log
* PC 0.14.0 brought in (version, CI and packaging files only; the rest of 0.13 -> 0.14 was identical).
* From the MobiEffects ("mobicraft") copy of this project: the layer sheet is much fuller (quick Visible / Solo /
  Lock, Blending Mode, Track Matte, **Parent & Link picker**, Layer Styles, keyframe and timing actions, label
  colors, Create), the **Columns sheet**, and more + panel sections. Fixes made while merging: the sheet's third
  quick pill was cut off; creating a layer closes the sheet again; its "Twirl" button called a command that does
  not exist (now it really opens / closes the selected layers); Reset restores the PC columns.
* The old phone build hid the PC timeline columns (Switches, Parent & Link, ...) on first run and said they stayed in
  a right-click menu that touch cannot open. They are back, and the timeline slides sideways to reach them.
* New: one scrolling strip of timeline controls (Columns, Modes, Search, Graph, Prev Key, Key, Next Key, Twirl, Motion
  Blur, Frame Blend, Shy, Draft 3D, Flowchart); Paint and Puppet tool groups; Layer, Paint, Brushes, Wiggler,
  Smoother, Motion Sketch, Mask Interpolation, Essential Graphics, Footage, Flowchart, Create Nulls in the + panel.
* **Updating from now on: see `mobile-sync/MOBILE_SYNC_GUIDE.md`** (send a new PC zip + this project's zip).

## Fixes from the first phone test (buttons that did nothing)
* The **+ next to the comp tabs** did nothing: `open_page` wrote the page into the shell's state, and `show()` then
  stored its own older copy over it at the end of the frame. The page is now queued and taken by `show()`.
* **Property values were missing** from the timeline, and the add-menu / enable-toggle / expression controls of
  groups, effects and expressions were out of reach: the phone puts the name column last, but those rows put
  their controls at the Switches column. They now sit in the right part of the (wider) name column (`aux_x`).
* **Fill & Stroke** (the PC header's tool options) was not on the phone at all: new `mobile_style.rs`, a section of
  the + panel: Fill / Stroke None - Solid - Linear - Radial, swatch with hex and R G B, opacity, blend mode,
  Stroke Width, Tool Creates Shape / Mask, Snapping. Same engine commands as the PC.
* The PC panels' **file dialogs** (Essential Graphics, Export, Collect Files, Open Project ...) had no host hook on
  Android and silently did nothing: Save and Folder now answer with the app's own storage (`Saved/`), Open goes
  to the system file manager (`MainActivity.java`, kind "any").
* Camera view: it works (tested with a 3D layer) but is dimmed until the comp has a 3D layer, as on the PC.
* Tests now tap the way a finger does (`ui_mobile_touch.rs`); `ui_mobile_audit.rs` (run by hand with `--ignored`)
  taps every button in many states and lists the ones that change nothing. Its list still holds legitimate
  no-ops (already-selected tool, 100% opacity at +, Redo with nothing to redo, ...): read it, do not trust it blindly.
