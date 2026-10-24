# Keeping the phone project in step with the PC version

This project is the **PC EffectCraft, unchanged, plus a phone layer**. The phone layer is small and
lives in files of its own, so a newer PC version can be brought in by a script instead of by hand.

## For you (the owner): updating
1. Send Claude **two zips**: the new PC version, and the latest zip of this phone project.
2. Say: "sync the phone project to the new PC version".
3. Claude runs `mobile-sync/sync.py`, fixes any conflict, checks the build and tests, and hands back a
   new phone zip plus a short report (what changed, what to look at, anything PC added that the touch
   UI cannot reach yet). Then run `build_desktop_android.ps1` for the APK.

To update the phone layer only (new touch features, fixes): send the phone zip alone.

## For Claude: the procedure
1. Unzip both. Read `mobile-sync/pc-base.json` (`pc_version` = the PC version this project is based on).
2. `python mobile-sync/sync.py <new_pc_dir> <phone_dir> --out <result_dir>` (needs `git` for `git merge-file`).
   Use `--dry-run` first to see the plan. Exit code 1 = conflicts or files "needing a look".
3. Fix every `<<<<<<< phone / ======= pc-old ... >>>>>>> pc-new` block: keep PC's new code **and** the phone hook
   (hook list below). Do not rewrite PC code to suit the phone; adapt the hook.
4. Build and test (below). Fix compile errors in the phone files (`mobile*.rs`, `lib.rs` of the app) to the new
   PC APIs. Typical breakages so far: a renamed field (`ui.viewer.res` became `viewer_res()`), a new `Icon`,
   a changed `Hooks` struct.
5. **Parity audit** (the owner's standing request: nothing the PC UI offers may be missing on the phone, except the
   heavy features removed on purpose): for what the new PC version added (new menu commands, panels, tools,
   timeline columns, viewer-bar items, context menus) check it can be reached by touch. Right-click menus open on
   a long press (egui turns a long touch into a secondary click); hover-only, keyboard-only and
   F-key-only controls need a touch button. Add what is missing to the phone layer, with a test.
6. Report: PC version before/after, what merged, conflicts resolved, tests run, anything left.

`sync.py` rewrites `pc-base.json` and `pc-base/` to the new PC version in the result, so the next sync starts there.

## Checks after a sync (a Linux sandbox has no Android NDK; the APK is built on the owner's PC)
```
cargo check -p effectcraft-ui-egui -p effectcraft -p effectcraft-export          # host
cargo check -p effectcraft-export --features extra-codecs                          # PC codecs still compile
cargo check --target aarch64-linux-android -p effectcraft --lib                    # Android-only code type-checks
cargo test  -p effectcraft-ui-egui --test ui_mobile                                # the phone layout / touch tests
cargo test  -p effectcraft-ui-egui --test ui_keyframes --test ui_layer_panel_zoom --test ui_scrolling --test ui_panels   # desktop unchanged
```
Android target std: `rust-std-aarch64-linux-android` (conda-forge). Tests need `libasound2-dev`; screenshots need a
software Vulkan driver (`mesa-vulkan-drivers`, `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`) and
`Harness::render()`. Rust 1.95+ is required by the workspace.

## What belongs to the phone layer
Only in the phone project (never in PC): `crates/ui-egui/src/mobile*.rs`, `crates/ui-egui/tests/ui_mobile.rs`,
`apps/effectcraft/src/lib.rs` + `storage_access.rs`, `apps/effectcraft-android/` (manifest, `MainActivity.java`),
`build_desktop_android.ps1`, `crates/mobile/` (unused, excluded from the workspace), `docs/MOBILE_UI.md`,
`docs/mobile_architecture/`, `docs/parity.md`, `MOBILE_SYNC_NOTES.md`, `README_MOBIEFFECTS.md`, `mobile-sync/`.

## The hook files (PC files the phone project edits; copies of the PC originals are in `pc-base/`)
| File | Phone edit |
| --- | --- |
| `Cargo.toml` | `default-members`, `exclude` (android app, crates/mobile), `[profile.android]` (quick builds) |
| `apps/effectcraft/Cargo.toml` | `[lib]` cdylib, Android deps, `unsafe` allowed for JNI, desktop-only deps gated |
| `crates/ui-egui/Cargo.toml` | `accesskit` only off Android |
| `crates/ui-egui/src/lib.rs` | `mod mobile*`; `Hooks` fields (`media_access`, `window_insets`, `pick_media`, `take_picked`); `if mobile::active() { mobile::show } else { menu bar + header + dock }`; `mobile::toast_lift` |
| `crates/ui-egui/src/panels/viewer.rs` | no navigator / bottom bar on the phone (the shell draws its own) |
| `crates/ui-egui/src/panels/timeline.rs` | phone toolbar strip (`mobile` flag), columns with the name last (`cols(.., name_last)`), whole timeline slides (`view` / `mobile-tl-view`), two-finger scroll/pan/pinch, double-tap layer sheet, search, `mobile_columns_w`, twirl |
| `crates/ui-egui/src/icons.rs` | `Scissors, Undo, Redo, Duplicate, Columns` |
| `crates/engine/src/media_browser.rs` | Android shared storage places |
| `crates/export/*` | HEVC / AV1 / VP9 encoders behind the `extra-codecs` feature (off here) |
Mobile-wins files (kept as they are): `README.md`, `.gitignore`, `.cargo/config.toml`. `Cargo.lock` comes from PC.

## Phone design rules
Touch only; nothing cut off, overlapping or compressed; everything inside the system bars (`safe_rect`); features
removed on purpose stay out (motion tracking, Warp Stabilizer, Roto Brush / Refine Edge / Object Matte;
see `mobile_menu::hidden`); heavy PC features are not added. Every new control gets an automation id
(`app.auto.add`) and a test in `ui_mobile.rs`.

## Things learned the hard way
* An earlier phone build **hid** the timeline columns on first run and said they stayed in the column menu: that menu
  is right-click only. Nothing may be left reachable only by right-click, hover or keyboard.
* egui scrolls lists by dragging only after it has seen a touch (`DragScroll::OnTouch`): tests call `enable_touch`.
* `NdkMediaCodecEncoder` in the very first phone build was a stub (no video packets); Android export uses the PC's
  software H.264. A real MediaCodec encoder / decoder is still to do (1080p smooth playback depends on it).
* `crates/mobile` (JNI bridge) never compiled against the engine; it is excluded.
* Edition 2024: `#[unsafe(no_mangle)]`. JNI calls on the render thread need `with_local_frame`.
* Android picker: `MainActivity.java` copies picked content to `<files>/imports` and writes `picked.txt`; Rust polls it.
* The build script compiles Java with `javac` + `d8` into `classes.dex`; the manifest needs `hasCode="true"`.
* Test with a finger, not a mouse: egui-winit sends Touch, press, release and then `PointerGone`. Code that reads the
  hover position at release, or state written during a frame and overwritten at its end (`open_page`), only fails
  there. `tests/ui_mobile_touch.rs` has the helpers; `tests/ui_mobile_audit.rs` is the dead-button detector.
* The name column is last on the phone: sub-row controls (values, add menus, enable toggles) use `aux_x`, not
  `cw.switches`. Any new PC row that places controls "to the right of the name" needs the same.
* The PC panels call host file-dialog hooks (`file_dialog_async`, `pick_save_file`, `pick_files`). A hook the
  Android host does not set makes its buttons do nothing: after a sync, grep `app.hooks.` for new ones.
