//! JNI C-FFI entrypoints for Java / Kotlin / Flutter host integration.
//!
//! Exposes `com.effectcraft.EffectCraftBridge` native methods for Android.

use std::ffi::c_void;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jfloat, jint, jlong, jstring};

use crate::session_holder::MobileEngine;

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    cache_dir: JString,
    internal_dir: JString,
) {
    let cache: String = env.get_string(&cache_dir).map(Into::into).unwrap_or_default();
    let internal: String = env.get_string(&internal_dir).map(Into::into).unwrap_or_default();

    let engine = MobileEngine::global();
    if let Err(e) = engine.init(&cache, &internal) {
        log::error!("nativeInit failed: {:?}", e);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeExecuteCommand(
    mut env: JNIEnv,
    _class: JClass,
    command_id: JString,
    params_json: JString,
) -> jstring {
    let cmd: String = env.get_string(&command_id).map(Into::into).unwrap_or_default();
    let params: String = env.get_string(&params_json).map(Into::into).unwrap_or_default();

    let engine = MobileEngine::global();
    let result_str = match engine.execute_command(&cmd, &params) {
        Ok(json) => json,
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    };

    env.new_string(result_str)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeSetSurface(
    _env: JNIEnv,
    _class: JClass,
    _surface: JObject,
    width: jint,
    height: jint,
) {
    let engine = MobileEngine::global();
    let Ok(mut surface_guard) = engine.surface.lock() else { return };

    #[cfg(target_os = "android")]
    {
        unsafe {
            let native_window_ptr = ndk_sys::ANativeWindow_fromSurface(_env.get_native_interface() as *mut _, _surface.as_raw() as *mut _);
            if !native_window_ptr.is_null() {
                let _ = surface_guard.set_android_surface(native_window_ptr, width.max(1) as u32, height.max(1) as u32);
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        surface_guard.set_mock_surface(width.max(1) as u32, height.max(1) as u32);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeDestroySurface(
    _env: JNIEnv,
    _class: JClass,
) {
    let engine = MobileEngine::global();
    if let Ok(mut surface_guard) = engine.surface.lock() {
        surface_guard.destroy();
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeSeekFrame(
    _env: JNIEnv,
    _class: JClass,
    comp_id: jlong,
    tick: jlong,
    scale: jfloat,
) {
    let engine = MobileEngine::global();
    let cid = if comp_id > 0 { Some(comp_id as u64) } else { None };
    let render_scale = if scale > 0.0 { Some(scale as f64) } else { None };

    if let Err(e) = engine.seek_frame(cid, tick, render_scale) {
        log::error!("nativeSeekFrame failed: {:?}", e);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeSetTrimMemory(
    _env: JNIEnv,
    _class: JClass,
    level: jint,
) {
    let engine = MobileEngine::global();
    if let Err(e) = engine.on_trim_memory(level) {
        log::error!("nativeSetTrimMemory failed: {:?}", e);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeStartExport(
    mut env: JNIEnv,
    _class: JClass,
    export_json: JString,
) -> jboolean {
    let params: String = env.get_string(&export_json).map(Into::into).unwrap_or_default();
    let engine = MobileEngine::global();

    // Kick off render queue render command
    match engine.execute_command("renderQueue.render", &params) {
        Ok(_) => 1,
        Err(e) => {
            log::error!("nativeStartExport failed: {:?}", e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeCancelExport(
    _env: JNIEnv,
    _class: JClass,
) {
    let engine = MobileEngine::global();
    engine.cancel_export();
    let _ = engine.execute_command("renderQueue.stop", "{}");
}

#[no_mangle]
pub extern "system" fn Java_com_effectcraft_EffectCraftBridge_nativeGetExportProgress(
    _env: JNIEnv,
    _class: JClass,
) -> jfloat {
    let engine = MobileEngine::global();
    engine.get_export_progress()
}
