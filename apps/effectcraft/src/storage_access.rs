//! Android: the Photos / Videos / Music permission the touch import screen needs.
//!
//! The system file dialog (`rfd`) does not exist on Android, so the app browses shared storage
//! itself, and Android only lets it read photos, videos and audio once the user has allowed it in
//! the system dialog this module opens. Compiled everywhere (so it is checked on the desktop) but
//! only called on Android, where `ndk-context` holds the activity.

use jni::JavaVM;
use jni::objects::{JObject, JValue};

const MEDIA: [&str; 4] = [
    "android.permission.READ_MEDIA_IMAGES",
    "android.permission.READ_MEDIA_VIDEO",
    "android.permission.READ_MEDIA_AUDIO",
    "android.permission.READ_EXTERNAL_STORAGE",
];

/// `ask == false`: whether any media permission is already granted. `ask == true`: open the
/// system permission dialog (when none is granted yet), then report what is granted so far.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn media_access(ask: bool) -> bool {
    match run(ask) {
        Ok(granted) => granted,
        Err(e) => {
            log::warn!("media permission: {e}");
            false
        }
    }
}

fn run(ask: bool) -> Result<bool, String> {
    let cx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(cx.context().cast()) };

    let mut granted = false;
    for p in MEDIA {
        let name = env.new_string(p).map_err(|e| e.to_string())?;
        let r = env.call_method(&activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&name)]);
        match r.and_then(|v| v.i()) {
            Ok(0) => granted = true,
            Ok(_) => {}
            Err(_) => {
                let _ = env.exception_clear();
            }
        }
    }
    if granted || !ask {
        return Ok(granted);
    }
    let string_class = env.find_class("java/lang/String").map_err(|e| e.to_string())?;
    let arr = env.new_object_array(MEDIA.len() as i32, string_class, JObject::null()).map_err(|e| e.to_string())?;
    for (i, p) in MEDIA.iter().enumerate() {
        let s = env.new_string(p).map_err(|e| e.to_string())?;
        env.set_object_array_element(&arr, i as i32, s).map_err(|e| e.to_string())?;
    }
    let r = env.call_method(&activity, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::Object(&arr), JValue::Int(4711)]);
    if r.is_err() {
        let _ = env.exception_clear();
    }
    r.map(|_| false).map_err(|e| e.to_string())
}

/// Android: how far the status bar, the navigation bar (or gesture area), a display cutout and the
/// on-screen keyboard reach into the window, in pixels: `[left, top, right, bottom]`. `None` when
/// the window has no insets yet (before its first layout) or the call fails. The app draws behind
/// the system bars, so the touch shell keeps its controls inside these.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn window_insets() -> Option<[f32; 4]> {
    match insets() {
        Ok(v) => v,
        Err(e) => {
            log::warn!("window insets: {e}");
            None
        }
    }
}

fn insets() -> Result<Option<[f32; 4]>, String> {
    let cx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(cx.context().cast()) };
    // The render thread stays attached for the whole run, so every local reference made here is
    // released with the frame, not at the end of the thread.
    let out = env.with_local_frame(16, |env| -> jni::errors::Result<Option<[f32; 4]>> {
        let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
        let decor = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
        let wi = env.call_method(&decor, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[])?.l()?;
        if wi.is_null() {
            return Ok(None);
        }
        let mut v = [0i32; 4];
        for (slot, name) in v.iter_mut().zip(["getSystemWindowInsetLeft", "getSystemWindowInsetTop", "getSystemWindowInsetRight", "getSystemWindowInsetBottom"]) {
            *slot = env.call_method(&wi, name, "()I", &[])?.i()?;
        }
        // A notch / camera hole: the cutout's safe insets (API 28+; absent on older systems).
        let cutout = env.call_method(&wi, "getDisplayCutout", "()Landroid/view/DisplayCutout;", &[]);
        match cutout.and_then(|c| c.l()) {
            Ok(c) if !c.is_null() => {
                for (slot, name) in v.iter_mut().zip(["getSafeInsetLeft", "getSafeInsetTop", "getSafeInsetRight", "getSafeInsetBottom"]) {
                    if let Ok(n) = env.call_method(&c, name, "()I", &[]).and_then(|r| r.i()) {
                        *slot = (*slot).max(n);
                    }
                }
            }
            _ => {
                let _ = env.exception_clear();
            }
        }
        Ok(Some(v.map(|n| n.max(0) as f32)))
    });
    match out {
        Ok(v) => Ok(v),
        Err(e) => {
            let _ = env.exception_clear();
            Err(e.to_string())
        }
    }
}

// ------------------------------------------------------------------------------------------------
// The system pickers (gallery for photos and videos, file manager for audio)
// ------------------------------------------------------------------------------------------------

static DATA_DIR: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

/// The app's own storage folder (`getExternalFilesDir`, else `getFilesDir`): `MainActivity.java`
/// copies what the user picks into `<dir>/imports` and lists the files in `<dir>/imports/picked.txt`.
pub fn set_data_dir(dir: &std::path::Path) {
    if let Ok(mut d) = DATA_DIR.lock() {
        *d = Some(dir.to_path_buf());
    }
}

fn result_file() -> Option<std::path::PathBuf> {
    DATA_DIR.lock().ok()?.as_ref().map(|d| d.join("imports").join("picked.txt"))
}

/// Open the system picker: `"media"` is the gallery (photos and videos), `"audio"` the file
/// manager. `false` when it could not be opened.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn pick_media(kind: &str) -> bool {
    // A result left over from an earlier pick must not be taken for this one.
    if let Some(f) = result_file() {
        let _ = std::fs::remove_file(f);
    }
    match open_picker(kind) {
        Ok(()) => true,
        Err(e) => {
            log::warn!("picker: {e}");
            false
        }
    }
}

fn open_picker(kind: &str) -> Result<(), String> {
    let cx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(cx.context().cast()) };
    let arg = env.new_string(kind).map_err(|e| e.to_string())?;
    let r = env.call_method(&activity, "pickMedia", "(Ljava/lang/String;)V", &[JValue::Object(&arg)]);
    if r.is_err() {
        let _ = env.exception_clear();
    }
    r.map(|_| ()).map_err(|e| e.to_string())
}

/// `None` while the user is still picking (or no picker is open); then, once, the files picked
/// (empty when it was cancelled).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn take_picked() -> Option<Vec<String>> {
    let file = result_file()?;
    let text = std::fs::read_to_string(&file).ok()?;
    let _ = std::fs::remove_file(&file);
    Some(text.lines().map(str::trim).filter(|l| !l.is_empty() && std::path::Path::new(l).is_file()).map(str::to_string).collect())
}
