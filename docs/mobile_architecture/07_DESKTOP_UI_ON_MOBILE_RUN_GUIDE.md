# EffectCraft Desktop UI on Android Mobile — Complete Run Guide

## 1. UI Confirmation (No Changes Made)
> [!IMPORTANT]
> **Aapki desktop UI bilkul bhi change ya simplify nahi ki gayi hai!**
> `crates/ui-egui` ke sare panels (Timeline, Composition Viewer, Effects Inspector, Project Media Bin, Curve Editor, Tools) bilkul waise hi hain jaise PC version me the. Koi bhi panel delete ya chhota nahi kiya gaya hai.
> Egui ka native hardware-accelerated viewport direct Android NativeActivity ke screen par render hota hai.

---

## 2. Automated Setup (Jo Steps Humne Auto Complete Kar Diye)
Aapko manual configuration na karni pade, isliye ye sab steps automatically configure kar diye gaye hain:

1. **Rust & Android Target Installation:**
   - `Rustup` aur Cargo successfully install ho chuka hai.
   - `aarch64-linux-android` cross-compilation target install ho chuka hai.
2. **NDK Cross-Linker Configuration (`.cargo/config.toml`):**
   - Android NDK `28.2.13676358` ke Clang toolchain (`aarch64-linux-android26-clang.cmd`) aur `llvm-ar.exe` ko Cargo linker me configure kar diya gaya hai.
3. **Desktop App Android Bridge (`apps/effectcraft`):**
   - `apps/effectcraft/Cargo.toml` me `cdylib` target aur `android-activity` configure kiya gaya hai.
   - `apps/effectcraft/src/lib.rs` me `android_main` NativeActivity entry point add kiya gaya hai jo direct desktop UI ko mobile screen par start karta hai.
   - Mobile touch screens ke liye initial touch scale factor (`1.5x`) set kiya gaya hai.
4. **Ready-To-Run Android Wrapper Project (`apps/effectcraft-android`):**
   - Android NativeActivity `AndroidManifest.xml` (Full-screen, landscape sensor, hardware acceleration, audio/media permissions).
   - Android Studio project structure ready kar diya gaya hai (`settings.gradle.kts`, `build.gradle.kts`, `app/build.gradle.kts`).
   - High-resolution app icon copy kar diya gaya hai.
5. **1-Click Automation Script (`build_desktop_android.ps1`):**
   - Ek hi script me NDK paths, Cargo aarch64 build, `libeffectcraft.so` copy, aur ADB device connection check automated hai.

---

## 3. How to Run on Mobile (Kam se kam steps aapke liye)

Aapko sirf niche diye gaye **Option 1** ya **Option 2** me se koi ek simple tarika use karna hai:

### Option 1: 1-Click Automated Script (Sabse Simple)
Apne terminal ya PowerShell me sirf ye command chalayein:
```powershell
.\build_desktop_android.ps1
```
Yeh script automatically:
1. Environment variables (`ANDROID_HOME`, `JAVA_HOME`, `NDK`) load karega.
2. Desktop UI ko mobile library (`libeffectcraft.so`) me compile karega.
3. Android wrapper me place karega.
4. Connected phone detect karke status show karega.

---

### Option 2: Android Studio se Run Karna (Direct Visual Run)
1. **Android Studio** open karein (jo aapke system me `D:\android studio` me installed hai).
2. **Open Project** par click karein aur select karein:
   ```
   E:\effectcraft-mainformobile\apps\effectcraft-android
   ```
3. Apna phone USB cable se connect karein (USB Debugging ON honi chahiye).
4. Top toolbar me apna device select karein aur **Green Run (Play ▶)** button click karein.
5. EffectCraft ka complete desktop interface aapke mobile screen par start ho jayega!

---

## 4. Touch Navigation Controls (Mobile Par Desktop UI Kaise Chalayein)
- **Click / Select:** Single Tap.
- **Scroll Timeline / Panels:** Drag with one finger.
- **Scrub Time Indicator:** Tap and slide playhead on timeline.
- **Zoom In / Out:** Pinch with two fingers.
- **Context Menus / Right Click:** Long press.
