# EffectCraft Mobile UI

The phone UI is a separate layer on top of the desktop engine and panels. It is switched on on
Android (or with `EFFECTCRAFT_MOBILE_UI=1` on a desktop, to try it in a narrow window) and never
changes how the desktop UI works.

## Screens

- **Home**: Welcome, New Project, Open Project, recent projects, a small Community & Social box
  (edit `COMMUNITY` in `mobile_home.rs`). New Project opens a full-screen composition settings page
  (name, presets, width/height, aspect ratio, frame rate, duration, background); OK opens the editor.
- **Editor** (everything opens closed): thin toolbar (toolbar toggle, Home, Import +, Options,
  Undo/Redo, Export), tool strip, preview with its scrolling control strip, comp navigation strip,
  timeline with a floating +. Options opens the File … Help strip; each menu is a list with
  expandable submenus built from the same definitions as the desktop menu bar.
- **+ panel** replaces the timeline: Assets, Effects & Presets, Effect Settings, Preview, Properties,
  Align, Audio, Info, Character, Paragraph, History, Markers. The ✕ returns to the timeline.
- **Layer sheet**: double-tap a layer. Create (Text, Solid, Light, Camera, Null, Shape, Adjustment),
  Pre-compose, Duplicate, Delete, Split, Arrange, Blending Mode, Time, Transform, Mask, Layer
  Settings and the layer switches (3D, Motion Blur, Adjustment, Shy, …).
- **Import** (the + button): in-app browser for videos, photos and audio, with "Import" and
  "Add to comp". **Export**: format, resolution, quality and destination from the real render
  queue, with progress, Stop, and the saved path or the error.
- **Tools**: Selection, Hand (+ Zoom), Rotation, Pan Behind (Anchor), Shape (Rectangle, Rounded
  Rectangle, Ellipse, Polygon, Star), Pen (+ Add/Delete/Convert Vertex, Mask Feather), Text
  (horizontal/vertical), Camera (Orbit/Pan/Dolly, only with a 3D layer). Tap a selected slot again to
  pick another tool in it.

## Where the code is

All new code is in `crates/ui-egui/src/mobile*.rs` (`mobile.rs` shell, `mobile_kit.rs` widgets,
`mobile_home.rs`, `mobile_menu.rs`, `mobile_viewbar.rs`, `mobile_import.rs`, `mobile_export.rs`),
plus `crates/ui-egui/tests/ui_mobile.rs`. Android-only host code is
`apps/effectcraft/src/storage_access.rs` (the Photos/Videos/Music permission dialog).

## Edits in shared files (re-apply after updating from the desktop repository)

Each is marked `MOBILE-HOOK` or is listed here; everything else in those files is the desktop code.

| File | Edit |
|------|------|
| `crates/ui-egui/src/lib.rs` | `pub mod mobile*;` lines; in `frame()` a `if mobile::active() { mobile::show(...) } else { …desktop… }` around the menu bar/header/dock; `mobile::toast_lift`; `Hooks.media_access` field |
| `crates/ui-egui/src/panels/viewer.rs` | three `MOBILE-HOOK`s: no breadcrumb row, no bottom bar when `mobile::active()` |
| `crates/ui-egui/src/panels/timeline.rs` | `mobile` flag: toolbar row + narrower layer controls, touch pan/pinch, wider scroll bar, double-tap hook (`MOBILE-HOOK`), footer text |
| `crates/ui-egui/src/icons.rs` | `Scissors`, `Duplicate`, `Undo`, `Redo` icons |
| `crates/engine/src/media_browser.rs` | Android shared-storage folders (Camera, Pictures, Movies, Downloads, Music) |
| `apps/effectcraft/src/lib.rs` | Android: no forced 1.5 scale, `set_data_dir`, `media_access` hook |
| `apps/effectcraft/Cargo.toml`, `Cargo.lock` | `jni`, `ndk-context` (already in the lock via android-activity) |
| `apps/effectcraft-android/.../AndroidManifest.xml` | `screenOrientation="portrait"` |

To merge a desktop update: take the new desktop files, then re-apply the table above (search for
`mobile::` and `MOBILE-HOOK`). The mobile modules only use stable crate-internal calls
(`menus::invoke`, `Session::execute`, `panels::show`, `viewer_tools` labels).

## Known limits

- Android's system media picker needs Java glue for the activity result, which this native-activity
  app does not have. Import uses the in-app browser plus the Photos/Videos/Music permission dialog.
- Exports and projects are saved in the app's own storage folder (shown on the Export page).
- The viewer layouts offered are 1, 2 and 4 views (the engine has no 3-view layout).
- Classic 3D / Active Camera / Extended Viewer are dimmed unless the composition has a 3D layer.
- Motion-tracking features are hidden from the menus (`mobile_menu::hidden`). The Effects & Presets
  panel still lists every effect the engine has.
- Desktop dialogs (Composition Settings, Preferences, …) are the desktop windows.
- Not tested on a physical Android device; layout and flows are tested headlessly
  (`cargo test -p effectcraft-ui-egui --test ui_mobile`).
