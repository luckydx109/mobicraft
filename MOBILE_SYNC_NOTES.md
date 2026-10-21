# EffectCraft Mobile: synced to the PC version (v0.13.0)

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
