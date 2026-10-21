package com.effectcraft

import android.view.Surface

/**
 * JNI Bridge to EffectCraft Rust Native Engine (`libeffectcraft_mobile.so`).
 */
object EffectCraftBridge {

    init {
        System.loadLibrary("effectcraft_mobile")
    }

    /**
     * Initialize engine instance with cache directory path.
     */
    external fun nativeInit(cacheDir: String, internalDir: String)

    /**
     * Execute engine command with JSON parameters.
     * @return JSON string output
     */
    external fun nativeExecuteCommand(commandId: String, paramsJson: String): String

    /**
     * Attach Android Surface for native ANativeWindow hardware presentation.
     */
    external fun nativeSetSurface(surface: Surface, width: Int, height: Int)

    /**
     * Detach / destroy current surface.
     */
    external fun nativeDestroySurface()

    /**
     * Seek and composite a composition frame at specific tick with preview scale.
     */
    external fun nativeSeekFrame(compId: Long, tick: Long, scale: Float)

    /**
     * Notify engine of Android onTrimMemory level (e.g. 80 = COMPLETE, 15 = CRITICAL).
     */
    external fun nativeSetTrimMemory(level: Int)

    /**
     * Start background video export job with export JSON settings.
     */
    external fun nativeStartExport(exportJson: String): Boolean

    /**
     * Cancel currently running export job.
     */
    external fun nativeCancelExport()

    /**
     * Get current export progress (0.0 to 1.0).
     */
    external fun nativeGetExportProgress(): Float
}
