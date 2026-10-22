# 01. Android Core Architecture

## 1. System Architecture Overview

EffectCraft Mobile adopts a decoupled architecture:
1. **Core Motion Graphics Engine (Pure Rust):**
   * Encapsulated into a shared dynamic library: `libeffectcraft_mobile.so`.
   * Houses the document model (`project`), keyframing (`keyframe`), text shaping (`text`), vector geometry (`path`), blend modes (`color`), compositor (`render`), GPU shaders (`gpu`), and command execution (`engine`).
   * Completely independent of any desktop GUI libraries (`egui`, `eframe`, `winit`).
2. **Platform & Rendering Bridge (Android NDK & JNI):**
   * Exposes C/JNI FFI bindings for session management, command dispatch, timeline scrubbing, surface attachment, and render callbacks.
   * Directly interfaces with Android NDK native APIs (`ANativeWindow`, `AMediaCodec`, `AMediaExtractor`).
3. **Frontend UI Layer (Touch-First Mobile App):**
   * Drives the user experience using modern mobile UI frameworks (Flutter via `dart:ffi` or Kotlin/Jetpack Compose via JNI).
   * Renders the preview viewport by passing an Android `SurfaceView` / `TextureView` to the native Rust engine.

```
+-------------------------------------------------------------------+
|               Mobile Frontend (Flutter or Jetpack Compose)         |
|   - Canvas Viewport       - Bottom Sheet Inspector                |
|   - Touch Scrub Timeline  - Curve Editor & Preset Picker          |
+---------------------------------+---------------------------------+
                                  | FFI / JNI Calls
+---------------------------------v---------------------------------+
|                   Platform Bridge (crates/mobile)                  |
|   - JNI Entrypoints             - ANativeWindow Surface Glue      |
|   - Memory Pressure Listeners   - NDK AMediaCodec Encoder/Decoder |
+---------------------------------+---------------------------------+
                                  | In-Process Rust API
+---------------------------------v---------------------------------+
|                 EffectCraft Engine & Compositor Core              |
|   - effectcraft-engine          - effectcraft-gpu (WGPU/Vulkan)   |
|   - effectcraft-render          - effectcraft-project             |
|   - effectcraft-keyframe        - effectcraft-text / path         |
+-------------------------------------------------------------------+
```

---

## 2. Toolchain, SDK, and Target ABIs

### 2.1 Android SDK & NDK Requirements
* **Minimum SDK:** `26` (Android 8.0 Oreo)
  * *Rationale:* Guarantees hardware Vulkan 1.0 support, NDK `AMediaCodec` buffer APIs, and 64-bit kernel memory management.
* **Target SDK:** `35` (Android 15)
* **NDK Version:** `r26d` or `r27` (LLVM 17+)
* **C++ STL:** `c++_shared`

### 2.2 Target Architectures
| Target Triple | Priority | Device Scope |
| :--- | :---: | :--- |
| `aarch64-linux-android` | **Tier 1 (Primary)** | All modern Android devices (Snapdragon, MediaTek Dimensity, Tensor, Exynos). |
| `x86_64-linux-android` | **Tier 1 (Development)** | Android Studio Emulator on Windows/macOS. |
| `armv7-linux-androideabi` | **Tier 2 (Fallback)** | Older 32-bit budget devices (Snapdragon 400 series, Unisoc). |

---

## 3. Surface Lifecycle & WGPU Vulkan Integration

### 3.1 Native Surface Handshake
The mobile UI passes an Android `android.view.Surface` to native Rust via JNI:
```rust
#[no_mangle]
pub extern "system" fn Java_com_storytold_effectcraft_NativeBridge_setSurface(
    mut env: JNIEnv,
    _class: JClass,
    surface: jobject,
    width: jint,
    height: jint,
) {
    let native_window = unsafe {
        ndk_sys::ANativeWindow_fromSurface(env.get_native_interface(), surface)
    };
    // Initialize or reconfigure WGPU surface
}
```

### 3.2 WGPU Surface Setup
WGPU natively supports `raw_window_handle` with Android NDK:
```rust
use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle, HasDisplayHandle, HasWindowHandle};

// Wrap ANativeWindow into WGPU SurfaceTarget
let surface = instance.create_surface(wgpu::SurfaceTarget::from_raw_window_handle(
    window_handle,
    display_handle,
))?;
```

### 3.3 Surface Destruction & App Backgrounding
When the Android Activity pauses or goes into the background (`SurfaceHolder.Callback.surfaceDestroyed`):
1. Immediately detach the WGPU swapchain.
2. Flush and release in-flight command buffers.
3. Keep the document `Session` and project model in RAM so user state is never lost.
4. If rendering export is running in the background, transition to headless rendering (offscreen texture rendering without presenting to the display surface).

---

## 4. JNI Command & State API Contract

All UI actions map 1:1 to `effectcraft_engine::Session` commands via JSON strings or binary buffers:

| JNI Method | Arguments | Purpose |
| :--- | :--- | :--- |
| `nativeInit` | `(cacheDir: String, internalDir: String)` | Initializes engine prefs, disk cache root, and system memory watchers. |
| `nativeExecuteCommand` | `(commandId: String, jsonParams: String) -> String` | Executes engine commands (e.g. `layer.select`, `property.set`, `comp.addLayer`). |
| `nativeSeekFrame` | `(comp: Long, tick: Long, scale: Float)` | Requests an urgent preview frame for the viewer canvas. |
| `nativeSetTrimMemory` | `(level: Int)` | Dispatches Android memory pressure trim signals directly to the cache manager. |
| `nativeStartExport` | `(jobJson: String)` | Spawns a background export worker using the hardware encoder. |
| `nativeCancelExport` | `()` | Atomically cancels active background export and triggers cleanup. |
